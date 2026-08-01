pub trait Html {
    fn render(&self, buf: &mut String);

    fn render_page(&self) -> String {
        const DOCTYPE: &str = "<!DOCTYPE html>";
        let mut buf = String::with_capacity(DOCTYPE.len() + self.size());
        buf.push_str(DOCTYPE);
        self.render(&mut buf);
        buf
    }

    /// Evaluate the size of an render element.
    /// Can be oversize.
    fn size(&self) -> usize {
        100
    }
}

impl Html for &String {
    fn render(&self, buf: &mut String) {
        self.as_str().render(buf);
    }
    fn size(&self) -> usize {
        self.as_str().size()
    }
}
impl Html for &str {
    fn render(&self, buf: &mut String) {
        for c in self.chars() {
            match c {
                '<' => buf.push_str("&lt;"),
                '>' => buf.push_str("&gt;"),
                '&' => buf.push_str("&amp;"),
                '"' => buf.push_str("&#34;"),
                '\'' => buf.push_str("&#39;"),
                _ => buf.push(c),
            }
        }
    }

    fn size(&self) -> usize {
        self.as_bytes()
            .iter()
            .map(|b| match b {
                b'<' => 5,
                b'>' => 5,
                b'&' => 5,
                b'\"' => 5,
                b'\'' => 5,
                _ => 1,
            })
            .sum()
    }
}

/// HTML block direct injected in the output without escape.
pub struct DirectHTML<'a>(pub &'a str);
impl<'a> Html for DirectHTML<'a> {
    fn render(&self, buf: &mut String) {
        buf.push_str(self.0);
    }
    fn size(&self) -> usize {
        self.0.len()
    }
}

impl<T: Html> Html for Option<T> {
    fn render(&self, buf: &mut String) {
        if let Some(sub) = self {
            sub.render(buf);
        }
    }
    fn size(&self) -> usize {
        match self {
            Some(sub) => sub.size(),
            None => 0,
        }
    }
}

impl<T: Html> Html for &[T] {
    fn render(&self, buf: &mut String) {
        for item in *self {
            item.render(buf);
        }
    }
    fn size(&self) -> usize {
        self.iter().map(|item| item.size()).sum()
    }
}
impl<T: Html> Html for [T; 1] {
    fn render(&self, buf: &mut String) {
        self[0].render(buf);
    }
    fn size(&self) -> usize {
        self[0].size()
    }
}

impl Html for () {
    fn render(&self, _: &mut String) {}
    fn size(&self) -> usize {
        0
    }
}
impl<T0: Html, T1: Html> Html for (T0, T1) {
    fn render(&self, buf: &mut String) {
        self.0.render(buf);
        self.1.render(buf);
    }
    fn size(&self) -> usize {
        self.0.size() + self.1.size()
    }
}

macro_rules! impl_nb {
    ($t:ty) => {
        impl Html for $t {
            fn render(&self, buf: &mut String) {
                use std::fmt::Write;
                write!(buf, "{}", self).unwrap();
            }
        }
    };
}
impl_nb!(u8);
impl_nb!(u16);
impl_nb!(u32);
impl_nb!(u64);
impl_nb!(usize);
impl_nb!(i8);
impl_nb!(i16);
impl_nb!(i32);
impl_nb!(i64);
impl_nb!(isize);

impl<F: Fn() -> I, I: Iterator<Item = U>, U: Html> Html for F {
    fn render(&self, buf: &mut String) {
        for item in (self)() {
            item.render(buf);
        }
    }
    fn size(&self) -> usize {
        (self)().map(|item| item.size()).sum()
    }
}

/* HTML Node */

pub struct HtmlZero {}

pub const H: HtmlZero = HtmlZero {};

impl std::ops::Sub<&'static str> for HtmlZero {
    type Output = HtmlNode<(), ()>;

    fn sub(self, tag: &'static str) -> Self::Output {
        Self::Output {
            tag,
            attr: (),
            child: (),
        }
    }
}

impl<T: Html> std::ops::Add<T> for HtmlZero {
    type Output = HtmlNode<(), T>;

    fn add(self, rhs: T) -> Self::Output {
        Self::Output {
            tag: "",
            attr: (),
            child: rhs,
        }
    }
}

pub struct HtmlNode<A: Html, T: Html> {
    tag: &'static str,
    attr: A,
    child: T,
}

impl<A: Html, T: Html, U: Html> std::ops::Sub<U> for HtmlNode<A, T> {
    type Output = HtmlNode<(A, U), T>;

    fn sub(self, rhs: U) -> Self::Output {
        Self::Output {
            tag: self.tag,
            attr: (self.attr, rhs),
            child: self.child,
        }
    }
}

impl<A: Html, T: Html, U: Html> std::ops::Add<U> for HtmlNode<A, T> {
    type Output = HtmlNode<A, (T, U)>;

    fn add(self, rhs: U) -> Self::Output {
        Self::Output {
            tag: self.tag,
            attr: self.attr,
            child: (self.child, rhs),
        }
    }
}

impl<A: Html, T: Html> Html for HtmlNode<A, T> {
    fn render(&self, buf: &mut String) {
        if self.tag == "" {
            return self.child.render(buf);
        }

        let (tags, attr) = self
            .tag
            .split_at(self.tag.find(" ").unwrap_or(self.tag.len()));
        let mut tags_iter = tags.split('.').peekable();
        let tag = tags_iter.next().unwrap();

        buf.push('<');
        buf.push_str(tag);
        if let Some(first_class) = tags_iter.next() {
            if tags_iter.peek().is_some() {
                buf.push_str(" class=\"");
                buf.push_str(first_class);
                for class in tags_iter {
                    buf.push(' ');
                    buf.push_str(class);
                }
                buf.push('"');
            } else {
                buf.push_str(" class=");
                buf.push_str(first_class);
            }
        }
        buf.push_str(attr);
        if self.attr.size() > 0 {
            self.attr.render(buf);
        }
        buf.push('>');

        self.child.render(buf);

        match tag {
            // Source: https://html.spec.whatwg.org/multipage/syntax.html#elements-2
            "area" | "base" | "br" | "col" | "embed" | "hr" | "img" | "input" | "link" | "meta"
            | "source" | "track" | "wbr" => {}
            "html" | "body" => {} // we chose to not close these tag
            _ => {
                buf.push_str("</");
                buf.push_str(tag);
                buf.push('>');
            }
        }
    }
    fn size(&self) -> usize {
        self.tag.len() + "< class=\"\"></>".len() + self.attr.size() + self.child.size()
    }
}

pub struct FnIterator<F, I, T>
where
    F: Fn() -> I,
    I: Iterator<Item = T>,
    T: Html,
{
    pub f: F,
}
impl<F, I, T> Html for FnIterator<F, I, T>
where
    F: Fn() -> I,
    I: Iterator<Item = T>,
    T: Html,
{
    fn render(&self, buf: &mut String) {
        for v in (self.f)() {
            v.render(buf);
        }
    }
    fn size(&self) -> usize {
        (self.f)().map(|v| v.size()).sum()
    }
}

#[test]
fn html_test() {
    assert_eq!(
        [H - "html" + [H - "title.c" + r#""Fran & Freddie's Diner" <tasty@example.com>"#]]
            .render_page(),
        "<!DOCTYPE html><html><title class=c>&#34;Fran &amp; Freddie&#39;s Diner&#34; &lt;tasty@example.com&gt;</title>"
    )
}

/// Add quote and escape content.
/// Use it for markup attribute.
pub struct Q<'a>(pub &'a str);

impl<'a> Html for Q<'a> {
    fn render(&self, buf: &mut String) {
        buf.push('"');
        self.0.render(buf);
        buf.push('"');
    }

    fn size(&self) -> usize {
        2 + self.0.size()
    }
}
