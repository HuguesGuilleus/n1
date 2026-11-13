/// A macro to write js code and expand it into HTML string.
///
/// Use it like `js! { ... }` to write JavaScript code within Rust.
///
/// code example:
/// ```rust
/// use n1_html::{DirectHTML, Html, js, js_const};
///
/// assert_eq!(DirectHTML(""), js_const! {});
///
/// assert_eq!(
///     DirectHTML(r#"console.log("Hello World");"#),
///     js_const! {
///         console.log("Hello World");
///     }
/// );
///
/// assert_eq!(
///     DirectHTML(r#"const x=3;console.log(x+1,{a:1});"#),
///     js_const! {
///         const x = 3;
///         console.log(x + 1, { a: 1 });
///     }
/// );
///
/// assert_eq!(
///     DirectHTML(r#"let r=[/(\w+)*/];"#),
///     js_const! {
///         let r = [{ regexp r"(\w+)*" }];
///     }
/// );
/// ```
///
/// A more realistic example:
/// ```rust
/// use n1_html::{Html, js};
///
/// let raw = r#"document.querySelectorAll("time").forEach(t=>t.innerText=new Intl.DateTimeFormat(document.documentElement.lang,{dateStyle:"full",timeStyle:"long"}).format(new Date(parseInt(t.innerText)*1000)));"#;
/// let with_js = js!(
///     document.querySelectorAll("time").forEach(t =>
///         t.innerText = new Intl.DateTimeFormat(document.documentElement.lang, {
///             dateStyle: "full",
///             timeStyle: "long"
///         }).format(new Date(parseInt(t.innerText) * 1000))
///     );
/// );
///
/// let mut buff = String::new();
/// with_js.render(&mut buff);
/// assert_eq!(raw, &buff);
/// ```
#[macro_export]
macro_rules! js_const {

    (=>) => { "=>" };
    (!) => { "!" };
    (#) => { "#" };
    (%) => { "%" };
    (&) => { "&" };
    (*) => { "*" };
    (+) => { "+" };
    (,) => { "," };
    (-) => { "-" };
    (.) => { "." };
    (/) => { "/" };
    (:) => { ":" };
    (;) => { ";" };
    (<) => { "<" };
    (=) => { "=" };
    (>) => { ">" };
    (?) => { "?" };
    (@) => { "@" };
    (^) => { "^" };
    (_) => { "_" };
    (|) => { "|" };
    (~) => { "~" };

    (||) => { "||" };
    (&&) => { "&&" };

    (async) => ( "async " );
    (await) => ( "await " );
    (break) => ( "break " );
    (class) => ( "class " );
    (const) => ( "const " );
    (continue) => ( "continue " );
    (default) => ( "default " );
    (else) => ( "else " );
    (function) => ( "function " );
    (let) => ( "let " );
    (new) => ( "new " );
    (return) => ( "return " );
    (throw) => ( "throw " );
    (using) => ( "using " );

    ($i:ident) => {
        stringify!($i)
    };
    ($l:literal) => {
        stringify!($l)
    };

    ({regexp $r:literal}) => ( concat!("/", $r, "/") );
    ({$($x:tt)*}) => ( concat!["{", $( js_const!($x), )* "}"] );
    (($($v:tt)*)) => ( concat!["(", $( js_const!($v), )* ")"] );
    ([$($v:tt)*]) => ( concat!["[", $( js_const!($v), )* "]"] );

    ($($token:tt)*) => {
        $crate::DirectHTML(concat![$(js_const!($token),)*])
    };
}
