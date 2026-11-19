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
///         console.log("Hello World",);
///     })
/// );
///
/// assert_eq!(
///     r#"const x=3;console.log(x+1,{a:1});"#,
///     &render(js! {
///         const x = { extern 1 + 2 };
///         console.log(x + {extern 1}, { a: 1 ,});
///     })
/// );
///
/// assert_eq!(
///     r#"let r=[/(\w+)*/];"#,
///     &render(js! {
///         let r = [{ regexp r"(\w+)*" },];
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
///
/// Source: <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators>
#[macro_export]
macro_rules! js {
    [$($v:tt)*] => {
        $crate::HtmlFn {
            render_fn: |buf: &mut String| {
                $( $crate::js_render!(buf $v) ; )*
            },
            len: {
                0 $(+ $crate::js_size!($v) )*
            },
        }
    };
}

#[macro_export]
macro_rules! js_render {
    ($buf:ident _) => ( $buf.push_str("_") );
    ($buf:ident -) => ( $buf.push_str("-") );
    ($buf:ident ,) => ( $buf.push_str(",") );
    ($buf:ident ;) => ( $buf.push_str(";") );
    ($buf:ident :) => ( $buf.push_str(":") );
    ($buf:ident !) => ( $buf.push_str("!") );
    ($buf:ident ?) => ( $buf.push_str("?") );
    ($buf:ident .) => ( $buf.push_str(".") );
    ($buf:ident @) => ( $buf.push_str("@") );
    ($buf:ident *) => ( $buf.push_str("*") );
    ($buf:ident /) => ( $buf.push_str("/") );
    ($buf:ident &) => ( $buf.push_str("&") );
    ($buf:ident #) => ( $buf.push_str("#") );
    ($buf:ident %) => ( $buf.push_str("%") );
    ($buf:ident ^) => ( $buf.push_str("^") );
    ($buf:ident +) => ( $buf.push_str("+") );
    ($buf:ident <) => ( $buf.push_str("<") );
    ($buf:ident =) => ( $buf.push_str("=") );
    ($buf:ident >) => ( $buf.push_str(">") );
    ($buf:ident |) => ( $buf.push_str("|") );
    ($buf:ident ~) => ( $buf.push_str("~") );

    ($buf:ident --) => ( $buf.push_str("--") );
    ($buf:ident -=) => ( $buf.push_str("-=") );
    ($buf:ident !=) => ( $buf.push_str("!=") );
    ($buf:ident ??) => ( $buf.push_str("??") );
    ($buf:ident ?.) => ( $buf.push_str("?.") );
    ($buf:ident **) => ( $buf.push_str("**") );
    ($buf:ident *=) => ( $buf.push_str("*=") );
    ($buf:ident /=) => ( $buf.push_str("/=") );
    ($buf:ident &&) => ( $buf.push_str("&&") );
    ($buf:ident &=) => ( $buf.push_str("&=") );
    ($buf:ident ^=) => ( $buf.push_str("^=") );
    ($buf:ident ++) => ( $buf.push_str("++") );
    ($buf:ident +=) => ( $buf.push_str("+=") );
    ($buf:ident <<) => ( $buf.push_str("<<") );
    ($buf:ident ==) => ( $buf.push_str("==") );
    ($buf:ident =>) => ( $buf.push_str("=>") );
    ($buf:ident |=) => ( $buf.push_str("|=") );
    ($buf:ident ||) => ( $buf.push_str("||") );

    ($buf:ident !==) => ( $buf.push_str("!==") );
    ($buf:ident ??=) => ( $buf.push_str("??=") );
    ($buf:ident ...) => ( $buf.push_str("...") );
    ($buf:ident **=) => ( $buf.push_str("**=") );
    ($buf:ident &&=) => ( $buf.push_str("&&=") );
    ($buf:ident <<=) => ( $buf.push_str("<<=") );
    ($buf:ident ===) => ( $buf.push_str("===") );
    ($buf:ident >>=) => ( $buf.push_str(">>=") );
    ($buf:ident >>>) => ( $buf.push_str(">>>") );
    ($buf:ident >>>) => ( $buf.push_str(">>>") );
    ($buf:ident ||=) => ( $buf.push_str("||=") );

    ($buf:ident>>>=) => ( $buf.push_str(">>>=") );

    ($buf:ident async) => ( $crate::DirectJSWord("async").render($buf) );
    ($buf:ident await) => ( $crate::DirectJSWord("await").render($buf) );
    ($buf:ident break) => ( $crate::DirectJSWord("break").render($buf) );
    ($buf:ident class) => ( $crate::DirectJSWord("class").render($buf) );
    ($buf:ident const) => ( $crate::DirectJSWord("const").render($buf) );
    ($buf:ident continue) => ( $crate::DirectJSWord("continue").render($buf) );
    ($buf:ident default) => ( $crate::DirectJSWord("default").render($buf) );
    ($buf:ident delete) => ( $crate::DirectJSWord("delete").render($buf) );
    ($buf:ident else) => ( $crate::DirectJSWord("else").render($buf) );
    ($buf:ident function) => ( $crate::DirectJSWord("function").render($buf) );
    ($buf:ident in) => ( $crate::DirectJSWord("in").render($buf) );
    ($buf:ident instanceof) => ( $crate::DirectJSWord("instanceof").render($buf) );
    ($buf:ident let) => ( $crate::DirectJSWord("let").render($buf) );
    ($buf:ident new) => ( $crate::DirectJSWord("new").render($buf) );
    ($buf:ident of) => ( $crate::DirectJSWord("of").render($buf) );
    ($buf:ident return) => ( $crate::DirectJSWord("return").render($buf) );
    ($buf:ident throw) => ( $crate::DirectJSWord("throw").render($buf) );
    ($buf:ident typeof) => ( $crate::DirectJSWord("typeof").render($buf) );
    ($buf:ident using) => ( $crate::DirectJSWord("using").render($buf) );
    ($buf:ident void) => ( $crate::DirectJSWord("void").render($buf) );
    ($buf:ident yield) => ( $crate::DirectJSWord("yield").render($buf) );

    ($buf:ident $i:ident) => { $crate::DirectJSWord(stringify!($i)).render($buf) };
    ($buf:ident $l:literal) => { $crate::DirectJSWord(stringify!($l)).render($buf) };

    ($buf:ident {regexp $r:literal}) => {
        $buf.push_str(concat!("/", $r, "/"))
    };

    ($buf:ident {html $e:expr}) => {
        $crate::AssertHtml( $e ).render($buf)
    };

    ($buf:ident {extern $e:expr}) => {
        $crate::J( $e ).render($buf)
    };

    ($buf:ident ($($v:tt)*)) => {
        $buf.push('(');
        $( $crate::js_render!($buf $v) ; )*
        if $buf.as_bytes().last() == Some(&b',') {
            $buf.pop();
        }
        $buf.push(')');
    };

    ($buf:ident [$($v:tt)*]) => {
        $buf.push('[');
        $( $crate::js_render!($buf $v) ; )*
        if $buf.as_bytes().last() == Some(&b',') {
            $buf.pop();
        }
        $buf.push(']');
    };

    [$buf:ident {$($v:tt)*}] => {
        $buf.push('{');
        $( $crate::js_render!($buf $v) ; )*
        match $buf.as_bytes().last() {
            Some(&b',' | &b';') => { $buf.pop(); }
            _ => { }
        }
        if $buf.as_bytes().last() == Some(&b',') {
            $buf.pop();
        }
        $buf.push('}');
    };
}

#[macro_export]
macro_rules! js_size {
    (_) => ( 1 );
    (-) => ( 1 );
    (,) => ( 1 );
    (;) => ( 1 );
    (:) => ( 1 );
    (!) => ( 1 );
    (?) => ( 1 );
    (.) => ( 1 );
    (@) => ( 1 );
    (*) => ( 1 );
    (/) => ( 1 );
    (&) => ( 1 );
    (#) => ( 1 );
    (%) => ( 1 );
    (^) => ( 1 );
    (+) => ( 1 );
    (<) => ( 1 );
    (=) => ( 1 );
    (>) => ( 1 );
    (|) => ( 1 );
    (~) => ( 1 );

    (--) => ( 2 );
    (-=) => ( 2 );
    (!=) => ( 2 );
    (??) => ( 2 );
    (?.) => ( 2 );
    (**) => ( 2 );
    (*=) => ( 2 );
    (/=) => ( 2 );
    (&&) => ( 2 );
    (&=) => ( 2 );
    (^=) => ( 2 );
    (++) => ( 2 );
    (+=) => ( 2 );
    (<<) => ( 2 );
    (==) => ( 2 );
    (=>) => ( 2 );
    (|=) => ( 2 );
    (||) => ( 2 );

    (!==) => ( 3 );
    (??=) => ( 3 );
    (...) => ( 3 );
    (**=) => ( 3 );
    (&&=) => ( 3 );
    (<<=) => ( 3 );
    (===) => ( 3 );
    (>>=) => ( 3 );
    (>>>) => ( 3 );
    (>>>) => ( 3 );
    (||=) => ( 3 );

    (>>>=) => ( 4 );

    (async) => ( ( "async ".len() ) );
    (await) => ( ( "await ".len() ) );
    (break) => ( ( "break ".len() ) );
    (class) => ( ( "class ".len() ) );
    (const) => ( ( "const ".len() ) );
    (continue) => ( ( "continue ".len() ) );
    (default) => ( ( "default ".len() ) );
    (delete) => ( ( "delete ".len() ) );
    (else) => ( ( "else ".len() ) );
    (function) => ( ( "function ".len() ) );
    (in) => ( ( " in ".len() ) );
    (instanceof) => ( ( "instanceof ".len() ) );
    (let) => ( ( "let ".len() ) );
    (new) => ( ( "new ".len() ) );
    (of) => ( ( " of ".len() ) );
    (return) => ( ( "return ".len() ) );
    (throw) => ( ( "throw ".len() ) );
    (typeof) => ( ( "typeof ".len() ) );
    (using) => ( ( "using ".len() ) );
    (void) => ( ( "void ".len() ) );
    (yield) => ( ( "yield ".len() ) );

    ($i:ident) => { stringify!($l).len() + 1 }; // +1 for space
    ($l:literal) => { stringify!($l).len() + 1 }; // +1 for space

    ({regexp $r:literal}) => {
        2 + stringify!($r).len()
    };

    ({html $e:expr}) => {
        $crate::AssertHtml( $e ).size()
    };

    ({extern $e:expr}) => {
        $crate::J( $e ).size()
    };

    (($($v:tt)*)) => {
        2 $(+ $crate::js_size!($v) )*
    };

    ([$($v:tt)*]) => {
        2 $(+ $crate::js_size!($v) )*
    };

    ({$($v:tt)*}) => {
        2 $(+ $crate::js_size!($v) )*
    };
}
