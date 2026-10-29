use proc_macro::{Delimiter, Literal, TokenStream, TokenTree};
use std::collections::BTreeMap;

/// Converts Resut expressions into JavaScript code string.
///
/// # Example
///
/// ```rust
/// use n1_js::js;
///
/// let js_code = js!(let x = 42;);
/// assert_eq!("let x=42;", js_code);
/// ```
#[proc_macro]
pub fn js(input: TokenStream) -> TokenStream {
    let mut out = String::new();
    let mut names: BTreeMap<String, usize> = BTreeMap::new();
    add(&mut out, &mut names, input);

    let mut ts = TokenStream::new();
    ts.extend([Literal::string(&out)]);
    ts
}

fn add(out: &mut String, names: &mut BTreeMap<String, usize>, stream: TokenStream) {
    for tree in stream {
        match tree {
            TokenTree::Group(group) => {
                let (begin, end) = match group.delimiter() {
                    Delimiter::Parenthesis => ("(", ")"),
                    Delimiter::Brace => ("{", "}"),
                    Delimiter::Bracket => ("[", "]"),
                    Delimiter::None => ("", ""),
                };
                out.push_str(begin);
                add(out, names, group.stream());
                if out.ends_with(',') || out.ends_with(';') {
                    out.pop();
                }
                out.push_str(end);
            }
            TokenTree::Ident(ident) => match ident.to_string().as_str() {
                s @ ("async" | "await" | "const" | "let" | "var" | "new" | "in" | "of"
                | "return" | "continue" | "break" | "throw") => {
                    if out.ends_with([
                        'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o',
                        'p', 'q', 'r', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J',
                        'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'Y', 'Z', '0', '1', '2', '3', '4',
                        '5', '6', '7', '8', '9',
                    ]) {
                        out.push(' ');
                    }
                    out.push_str(s);
                    out.push(' ');
                }
                s if names.contains_key(s) => {
                    print_name(out, *names.get(s).unwrap());
                }
                s if s.starts_with("_") => {
                    names.insert(s.to_string(), names.len());
                    print_name(out, *names.get(s).unwrap());
                }
                s => out.push_str(s),
            },
            TokenTree::Punct(p) => out.push(p.as_char()),
            TokenTree::Literal(v) => {
                let s = v.to_string();
                if let Some(s) = s.strip_prefix("\"regexp/") {
                    out.push('/');
                    let s = s.trim_end_matches("\"");
                    out.push_str(&s);
                } else if let Some(s) = s.strip_prefix("r#\"regexp/") {
                    out.push('/');
                    let s = s.trim_end_matches("\"#");
                    out.push_str(&s);
                } else {
                    out.push_str(&s);
                }
            }
        }
    }
}

fn print_name(out: &mut String, id: usize) {
    match id % (26 * 2) {
        0..26 => out.push((b'a' + id as u8) as char),
        _ => out.push((b'A' + (id as u8 - 26)) as char),
    }
    let id = id / (26 * 2);
    if id != 0 {
        print_name(out, id);
    }
}
