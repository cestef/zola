pub fn display_math(code: &str, addon: Option<&str>) -> String {
    let addon = addon.unwrap_or("");
    format!(
        r#"
#set page(height: auto, width: auto, margin: 0pt, fill: none)
#set text(14pt)
{addon}
$ {code} $
"#,
    )
}

pub fn inline_math(code: &str, addon: Option<&str>) -> String {
    let addon = addon.unwrap_or("");
    format!(
        r#"
#set page(height: auto, width: auto, margin: 0pt, fill: none)
#set text(13pt)
{addon}
#let s = state("t", (:))

#let pin(t) = context {{
    let computed = measure(
        line(length: here().position().y)
    )
    s.update(it => it.insert(t, computed.width) + it)
    }}

#show math.equation: it => {{
    box(it, inset: (top: 0.5em, bottom: 0.5em))
    }}
$pin("l1"){code}$

#context [
    #metadata(s.final().at("l1")) <label>
]
"#,
    )
}

pub fn raw(code: &str, addon: Option<&str>) -> String {
    let addon = addon.unwrap_or("");
    format!(
        r#"
#set page(height: auto, width: auto, margin: 0pt, fill: none)
#set text(16pt)
{addon}
{code}
"#,
    )
}
