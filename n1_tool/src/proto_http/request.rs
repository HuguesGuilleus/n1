use std::collections::BTreeMap;

use tokio::io::{AsyncRead, AsyncReadExt, Chain, Take};

pub type Incoming<'a, R> = HTTPRequest<Chain<&'a [u8], Take<&'a mut R>>>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Method {
    GET,
    HEAD,
    POST,
    PUT,
    DELETE,
    TRACE,
    OPTION,
    X,
}

pub struct HTTPRequest<B> {
    pub method: Method,
    /// Decoded path
    pub path: String,
    /// Un decoded query without `?`
    pub query: String,
    pub headers: BTreeMap<String, String>,
    pub body: B,
}

pub struct HTTPParser<R> {
    r: R,
    buffer: [u8; 8 * 1024],
    cursor_begin: usize,
    cursor_end: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Error {
    ContentLength,
    IO,
    NeedMore,
    NotUtf8,
    NotVersion1,
    WrongSyntax,
    WrongURLEncode,
}

impl<R: AsyncRead + Unpin> HTTPParser<R> {
    pub fn new(r: R) -> Self {
        Self {
            r,
            buffer: [0u8; 8 * 1024],
            cursor_begin: 0,
            cursor_end: 0,
        }
    }

    pub async fn next<'a>(
        &'a mut self,
    ) -> Result<HTTPRequest<Chain<&'a [u8], Take<&'a mut R>>>, Error> {
        let range = self.cursor_begin..self.cursor_end;
        self.buffer.copy_within(range, 0);
        self.cursor_end -= self.cursor_begin;
        self.buffer[self.cursor_end..].fill(0);

        while !self.contain_full_headers() {
            match (self.r).read(&mut self.buffer[self.cursor_end..]).await {
                Ok(read_len) => self.cursor_end += read_len,
                Err(_) => return Err(Error::IO),
            };
        }

        self.parse()
    }

    fn contain_full_headers(&self) -> bool {
        for i in 0..self.cursor_end {
            if self.buffer[i..].starts_with(b"\r\n\r\n") {
                return true;
            }
        }
        return false;
    }

    fn parse(&mut self) -> Result<Incoming<'_, R>, Error> {
        let mut readed_len = 0;

        // First line
        let first = self.split_line(&mut readed_len)?;
        if !first.ends_with(" HTTP/1.1") && !first.ends_with(" HTTP/1.0") {
            return Err(Error::NotVersion1);
        }
        let first = &first[..first.len() - " HTTP/1.1".len()];
        let (method, uri) = first.split_once(' ').ok_or(Error::WrongSyntax)?;

        let method = match method {
            "GET" => Method::GET,
            "HEAD" => Method::HEAD,
            "POST" => Method::POST,
            "PUT" => Method::PUT,
            "DELETE" => Method::DELETE,
            "TRACE" => Method::TRACE,
            "OPTION" => Method::OPTION,
            _ => Method::X,
        };
        let (path, query) = Self::parse_uri(uri)?;

        // Headers
        let mut headers: BTreeMap<String, String> = BTreeMap::new();
        loop {
            let line = self.split_line(&mut readed_len)?;
            if line.is_empty() {
                break;
            }
            let (name, value) = line.split_once(':').ok_or(Error::WrongSyntax)?;
            headers.insert(name.trim().to_string(), value.trim().to_string());
        }
        let header_len = readed_len;

        let body_len = Self::get_content_len(&headers)?.unwrap_or(0);
        let body_bytes_len = body_len.min(self.buffer.len() - header_len);
        let body_reader_len = body_len - body_bytes_len;
        let body = (&self.buffer[header_len..header_len + body_bytes_len])
            .chain((&mut self.r).take(body_reader_len as u64));
        self.cursor_begin = header_len + body_bytes_len;

        Ok(HTTPRequest {
            method,
            path,
            query,
            headers,
            body,
        })
    }

    fn split_line(&self, readed_len: &mut usize) -> Result<&str, Error> {
        let begin = *readed_len;
        for p in *readed_len..self.cursor_end {
            if self.buffer[p..].starts_with(b"\r\n") {
                *readed_len = p + 2;
                return std::str::from_utf8(&self.buffer[begin..p]).map_err(|_| Error::NotUtf8);
            }
        }
        Err(Error::NeedMore)
    }

    fn parse_uri(uri: &str) -> Result<(String, String), Error> {
        let (raw_path, query) = uri.split_once('?').unwrap_or((uri, ""));

        let path = urlencoding::decode(raw_path)
            .map(|cow| cow.to_string())
            .map_err(|_| Error::WrongURLEncode)?;

        Ok((path, query.to_string()))
    }

    fn get_content_len(headers: &BTreeMap<String, String>) -> Result<Option<usize>, Error> {
        for (name, value) in headers {
            if str::eq_ignore_ascii_case("content-length", name) {
                return value
                    .parse::<usize>()
                    .map(|size| Some(size))
                    .map_err(|_| Error::ContentLength);
            }
        }
        Ok(None)
    }
}

#[tokio::test]
async fn next() {
    let data = concat!(
        "PUT /%61bc?v=1 HTTP/1.1\r\n",
        "Content-Length: 5\r\n",
        "\r\n",
        "01234",
        "GET /%41BC?v=1 HTTP/1.1\r\n",
        "foo-bar: hello\r\n",
        "\r\n",
    );
    let mut parser = HTTPParser::new(data.as_bytes());

    // Request 1
    let mut next = parser.next().await.unwrap();
    assert_eq!(next.method, Method::PUT);
    assert_eq!(next.path, String::from("/abc"));
    assert_eq!(next.query, String::from("v=1"));
    assert_eq!(
        next.headers,
        BTreeMap::from_iter([(String::from("Content-Length"), String::from("5"))])
    );
    let mut buff = String::new();
    next.body.read_to_string(&mut buff).await.unwrap();
    assert_eq!("01234", buff.as_str());

    // Request 2
    let mut next = parser.next().await.unwrap();
    assert_eq!(next.method, Method::GET);
    assert_eq!(next.path, String::from("/ABC"));
    assert_eq!(next.query, String::from("v=1"));
    assert_eq!(
        next.headers,
        BTreeMap::from_iter([(String::from("foo-bar"), String::from("hello"))])
    );
    let mut buff = String::new();
    next.body.read_to_string(&mut buff).await.unwrap();
    assert_eq!("", buff.as_str());
}
