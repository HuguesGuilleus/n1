use n1_html::DirectHTML;

pub const HEAD: DirectHTML = DirectHTML(concat!(
    "<meta charset=utf-8>",
    "<meta name=viewport content='width=device-width,initial-scale=1'>",
    "<link rel=stylesheet href=/_style.css>",
    "<link rel=icon href=/_favicon.webp>",
));

pub const CSS: &[u8] = include_bytes!("style.css");
pub const FAVICON: &[u8] = include_bytes!("favicon.webp");

pub const ROBOTSTXT: &[u8] = b"User-agent: *
Allow: /
Disallow: /_*
Disallow: /.*
";
