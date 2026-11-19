use std::collections::BTreeMap;

use tokio::{io::AsyncReadExt, net::tcp::ReadHalf};

pub struct HTTPParser {
    pub buffer: [u8; 8 * 1024],
    pub cursor_begin: usize,
    pub cursor_end: usize,
}

#[derive(Debug)]
/// Meta information about the HTTP request.
pub struct HTTPRequest {
    pub method: Method,
    /// Decoded path
    pub path: String,
    /// Undecoded query without `?`
    pub query: String,
    pub headers: BTreeMap<String, String>,
}

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Error {
    ContentLength,
    IO,
    NeedMore,
    NotUtf8,
    NotVersion1,
    WrongSyntax,
    WrongURLEncode,
    TooLong,
}

pub async fn next_req<'a>(
    reader: &mut ReadHalf<'_>,
    parser: &'a mut HTTPParser,
) -> Option<Result<(HTTPRequest, &'a [u8], u64), Error>> {
    parser.reinit();

    while !parser.contain_full_headers() {
        if parser.cursor_end == parser.buffer.len() {
            return Some(Err(Error::TooLong));
        }
        if let Err(_) = reader.readable().await {
            return Some(Err(Error::IO));
        }
        match reader.read(&mut parser.buffer[parser.cursor_end..]).await {
            Ok(0) => return None,
            Ok(read_len) => parser.cursor_end += read_len,
            Err(_) => return Some(Err(Error::IO)),
        };
    }

    Some(parser.parse())
}

impl HTTPParser {
    pub fn new() -> Self {
        Self {
            buffer: [0u8; 8 * 1024],
            cursor_begin: 0,
            cursor_end: 0,
        }
    }

    pub fn reinit(&mut self) {
        let range = self.cursor_begin..self.cursor_end;
        self.buffer.copy_within(range, 0);
        self.cursor_end -= self.cursor_begin;
        self.buffer[self.cursor_end..].fill(0);
    }

    fn contain_full_headers(&self) -> bool {
        for i in 0..self.cursor_end {
            if self.buffer[i..].starts_with(b"\r\n\r\n") {
                return true;
            }
        }
        return false;
    }

    fn parse<'a>(&'a mut self) -> Result<(HTTPRequest, &'a [u8], u64), Error> {
        let mut readed_len = 0;

        // First line
        let first = self.split_line(&mut readed_len)?;
        let first = first.strip_suffix(" HTTP/1.1").ok_or(Error::NotVersion1)?;
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
        let body_bytes_len = body_len.min(self.cursor_end - header_len);
        let body_reader_len = body_len - body_bytes_len;
        self.cursor_begin = header_len + body_bytes_len;

        Ok((
            HTTPRequest {
                method,
                path,
                query,
                headers,
            },
            &self.buffer[header_len..][..body_bytes_len],
            body_reader_len as u64,
        ))
    }

    fn split_line(&self, readed_len: &mut usize) -> Result<&str, Error> {
        let begin = *readed_len;
        for p in begin..self.cursor_end {
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
    let mut parser = HTTPParser::new();
    parser.buffer[0..data.len()].copy_from_slice(data.as_bytes());
    parser.cursor_end = data.len();

    // Request 1
    let (request, buffer, rest) = parser.parse().unwrap();
    assert_eq!(request.method, Method::PUT);
    assert_eq!(request.path, String::from("/abc"));
    assert_eq!(request.query, String::from("v=1"));
    assert_eq!(
        request.headers,
        BTreeMap::from_iter([(String::from("Content-Length"), String::from("5"))])
    );
    assert_eq!(buffer, &b"01234"[..]);
    assert_eq!(rest, 0);

    // Request 2
    parser.reinit();

    let (request, buffer, rest) = parser.parse().unwrap();
    assert_eq!(request.method, Method::GET);
    assert_eq!(request.path, String::from("/ABC"));
    assert_eq!(request.query, String::from("v=1"));
    assert_eq!(
        request.headers,
        BTreeMap::from_iter([(String::from("foo-bar"), String::from("hello"))])
    );
    assert_eq!(buffer, &b""[..]);
    assert_eq!(rest, 0);
}
