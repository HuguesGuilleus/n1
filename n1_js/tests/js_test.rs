use n1_js::js;

#[test]
fn js_test() {
    assert_eq!(
        r#"console.log("Hello World");"#,
        js! {
            console.log("Hello World");
        }
    );
    assert_eq!(
        r#"const a=1+2;"#,
        js! {
            const _x = 1 + 2;
        }
    );
    assert_eq!(
        r#"const a=/[a-z]+/g;"#,
        js! {
            const _x ="regexp/[a-z]+/g";
        }
    );
    assert_eq!(
        r#"const a=/\w+/g;"#,
        js! {
            const _x =r#"regexp/\w+/g"#;
        }
    );
}

#[test]
fn test_time_js() {
    let raw = r#"document.querySelectorAll("time").forEach(a=>a.innerText=new Intl.DateTimeFormat(document.documentElement.lang,{dateStyle:"full",timeStyle:"long"}).format(new Date(parseInt(a.innerText)*1000)));"#;

    let with_js = js!(
        document.querySelectorAll("time").forEach(_t =>
            _t.innerText = new Intl.DateTimeFormat(document.documentElement.lang, {
                dateStyle: "full",
                timeStyle: "long"
            }).format(new Date(parseInt(_t.innerText) * 1000))
        );
    );

    assert_eq!(with_js, raw);
}
