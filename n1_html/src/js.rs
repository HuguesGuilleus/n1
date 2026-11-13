/// A macro to write js code and expand it into HTML string.
///
/// Use it like `js! { ... }` to write JavaScript code within Rust.
///
/// code example:
/// ```rust
/// use n1_html::{Html, js};
///
/// fn render(h: impl Html) -> String {
///     let mut buff = String::new();
///     h.render(&mut buff);
///     buff
/// }
///
/// assert_eq!("", &render(js! {}));
///
/// assert_eq!(
///     r#"console.log("Hello World");"#,
///     &render(js! {
///         console.log("Hello World");
///     })
/// );
///
/// assert_eq!(
///     r#"const x=3;console.log(x+1,{a:1});"#,
///     &render(js! {
///         const x = { extern 1 + 2 };
///         console.log(x + {extern 1}, { a: 1 });
///     })
/// );
///
/// assert_eq!(
///     r#"let r=[/(\w+)*/];"#,
///     &render(js! {
///         let r = [{ regexp r"(\w+)*" }];
///     })
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
macro_rules! js {
    (=>) => ( $crate::DirectHTML("=>") );
    (!) => ( $crate::DirectHTML("!") );
    (#) => ( $crate::DirectHTML("#") );
    (%) => ( $crate::DirectHTML("%") );
    (&) => ( $crate::DirectHTML("&") );
    (*) => ( $crate::DirectHTML("*") );
    (+) => ( $crate::DirectHTML("+") );
    (,) => ( $crate::DirectHTML(",") );
    (-) => ( $crate::DirectHTML("-") );
    (.) => ( $crate::DirectHTML(".") );
    (/) => ( $crate::DirectHTML("/") );
    (:) => ( $crate::DirectHTML(":") );
    (;) => ( $crate::DirectHTML(";") );
    (<) => ( $crate::DirectHTML("<") );
    (=) => ( $crate::DirectHTML("=") );
    (>) => ( $crate::DirectHTML(">") );
    (?) => ( $crate::DirectHTML("?") );
    (@) => ( $crate::DirectHTML("@") );
    (^) => ( $crate::DirectHTML("^") );
    (_) => ( $crate::DirectHTML("_") );
    (|) => ( $crate::DirectHTML("|") );
    (~) => ( $crate::DirectHTML("~") );

    (||) => ( $crate::DirectHTML("||") );
    (&&) => ( $crate::DirectHTML("&&") );

    (async) => ( $crate::DirectHTML("async ") );
    (await) => ( $crate::DirectHTML("await ") );
    (break) => ( $crate::DirectHTML("break ") );
    (class) => ( $crate::DirectHTML("class ") );
    (const) => ( $crate::DirectHTML("const ") );
    (continue) => ( $crate::DirectHTML("continue ") );
    (default) => ( $crate::DirectHTML("default ") );
    (else) => ( $crate::DirectHTML("else ") );
    (function) => ( $crate::DirectHTML("function ") );
    (let) => ( $crate::DirectHTML("let ") );
    (new) => ( $crate::DirectHTML("new ") );
    (return) => ( $crate::DirectHTML("return ") );
    (throw) => ( $crate::DirectHTML("throw ") );
    (using) => ( $crate::DirectHTML("using ") );

    ($i:ident) => {
        $crate::DirectHTML(stringify!($i))
    };
    ($l:literal) => {
        $crate::DirectHTML(stringify!($l))
    };

    (($($v:tt)*)) => {
        $crate::H + $crate::DirectHTML("(") $(+ js!($v))* + $crate::DirectHTML(")")
    };

    ([$($v:tt)*]) => {
        $crate::H + $crate::DirectHTML("[") $(+ js!($v))* + $crate::DirectHTML("]")
    };

    ({regexp $r:literal}) => {
        $crate::DirectHTML(concat!("/", $r, "/"))
    };

    ({html $e:expr}) => {
        $crate::AssertHtml( $e )
    };

    ({extern $e:expr}) => {
        $crate::J( $e )
    };

    ({$($x:tt)*}) => {
        $crate::H + $crate::DirectHTML("{") $(+ js!($x) )* + $crate::DirectHTML("}")
    };

    ($($token:tt)*) => {
        [ $crate::H $( + js!($token) )* ]
    };
}
