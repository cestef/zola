use std::slice::Iter;

use typst::{
    foundations::{
        Chainable, Content, SequenceElem, Smart, StyleChain, StyledElem, Styles, SymbolElem,
    },
    layout::{FixedAlignment, HElem, HideElem, Spacing},
    math::{
        AccentElem, AttachElem, BinomElem, CancelElem, CasesElem, ClassElem, EquationElem,
        FracElem, LimitsElem, LrElem, MatElem, MidElem, OpElem, OverbraceElem, OverbracketElem,
        OverlineElem, OverparenElem, OvershellElem, PrimesElem, RootElem, ScriptsElem, StretchElem,
        UnderbraceElem, UnderbracketElem, UnderlineElem, UnderparenElem, UndershellElem, VecElem,
    },
    text::TextElem,
};
use unicode_math_class::MathClass;

use super::{
    eval::eval,
    style::{self, characters},
};

pub fn convert_to_mathml(content: &str, add_annotation: bool) -> Result<String, String> {
    let (equation, styles) = eval(&content).map_err(|e| e.to_string())?;
    // println!("{:#?}", equation);
    let styles = styles.unwrap_or(Styles::new());
    let style_chain = StyleChain::new(&styles);

    let is_block = equation.block(style_chain);
    let attrs = if is_block {
        r#" display="block" class="katex katex-display""#
    } else {
        r#" class="katex""#
    };
    let annotation = if add_annotation {
        format!(r#"<annotation encoding="application/x-typst">{}</annotation>"#, content)
    } else {
        String::with_capacity(0)
    };

    Ok(format!(
        // r#"<math xmlns="http://www.w3.org/1998/Math/MathML"{}><semantics>{}{}</semantics></math>"#,
        r#"<math{}><semantics>{}{}</semantics></math>"#,
        attrs,
        convert_to_mathml_impl(&equation.body, style_chain),
        annotation
    ))
}

fn convert_to_mathml_impl(content: &Content, style_chain: StyleChain) -> String {
    let elem_type = content.elem().name();
    match elem_type {
        "frac" => {
            let coerced = content
                .to_packed::<FracElem>()
                .expect("Type conversion to `FracElem` must be successful.");
            let numerator: String = convert_to_mathml_impl(&coerced.num, style_chain);
            let denomenator: String = convert_to_mathml_impl(&coerced.denom, style_chain);
            format!("<mfrac>{}{}</mfrac>", numerator, denomenator)
        }
        "accent" => {
            let coerced = content
                .to_packed::<AccentElem>()
                .expect("Type conversion to `AccentElem` must be successful.");
            format!(
                r#"<mover accent="true">{}<mo>{}</mo></mover>"#,
                convert_to_mathml_impl(&coerced.base, style_chain),
                coerced.accent.0,
            )
        }
        "limits" => {
            let coerced = content
                .to_packed::<LimitsElem>()
                .expect("Type conversion to `LimitsElem` must be successful.");
            convert_to_mathml_impl(&coerced.body, style_chain)
        }
        "scripts" => {
            let coerced = content
                .to_packed::<ScriptsElem>()
                .expect("Type conversion to `ScriptsElem` must be successful.");
            format!("<mrow>{}</mrow>", convert_to_mathml_impl(&coerced.body, style_chain))
        }
        "sequence" => process_sequence(content, style_chain),
        "lr" => process_lr(content, style_chain),
        "attach" => process_attach(content, style_chain),
        "text" => process_text(content, style_chain),
        "root" => process_root(content, style_chain),
        "binom" => process_binom(content, style_chain),
        "cancel" => process_cancel(content, style_chain),
        "op" => process_op(content, style_chain),
        "cases" => process_cases(content, style_chain),
        "mat" => process_mat(content, style_chain),
        "vec" => process_vec(content, style_chain),
        "class" => process_class(content, style_chain),
        "equation" => process_equation(content, style_chain),
        "primes" => process_primes(content),
        "styled" => process_styled(content, style_chain),
        "h" => process_h(content),
        "hide" => process_hide(content, style_chain),
        "stretch" => process_stretch(content, style_chain),
        "mid" => process_mid(content, style_chain),
        "underline" => process_underline(content, style_chain),
        "overline" => process_overline(content, style_chain),
        "underbrace" => process_underbrace(content, style_chain),
        "overbrace" => process_overbrace(content, style_chain),
        "underbracket" => process_underbracket(content, style_chain),
        "overbracket" => process_overbracket(content, style_chain),
        "underparen" => process_underparen(content, style_chain),
        "overparen" => process_overparen(content, style_chain),
        "undershell" => process_undershell(content, style_chain),
        "overshell" => process_overshell(content, style_chain),
        // FIXME: align-point should be processed in `Sequence`.
        "space" | "align-point" => "".to_string(),
        "symbol" => {
            let text = content
                .to_packed::<SymbolElem>()
                .expect("Type conversion to `TextElem` must be successful.");
            format!("<mo>{}</mo>", escape_helper(&text.text.to_string()))
        }
        _ => format!("<merror>`{}` Not Implemented Yet</merror>", elem_type),
    }
}

fn process_attach(content: &Content, style_chain: StyleChain) -> String {
    let coerced = content
        .to_packed::<AttachElem>()
        .expect("Type conversion to `AttachElem` must be successful.");
    let merged = coerced.merge_base();
    let elem = merged.as_ref().unwrap_or(coerced);

    let base = &elem.base;
    let mut top = elem.t(style_chain);
    let mut bottom = elem.b(style_chain);
    let top_left = elem.tl(style_chain);
    let bottom_left = elem.bl(style_chain);
    let mut top_right = elem.tr(style_chain);
    let mut bottom_right = elem.br(style_chain);

    let base_type = base.elem().name();
    let base_str = convert_to_mathml_impl(base, style_chain);
    let is_limits = base_type == "limits"
        || base_str.contains("∑")
        || (base_type == "op"
            && base
                .to_packed::<OpElem>()
                .expect("Type conversion to `OpElem` must be successful.")
                .limits(style_chain));
    let is_stretch = base_type == "stretch";

    if !is_limits && !is_stretch {
        if top.is_some() && top_right.is_none() {
            [top, top_right] = [top_right, top];
        }
        if bottom.is_some() && bottom_right.is_none() {
            [bottom, bottom_right] = [bottom_right, bottom];
        }
    }

    match (top, bottom, top_left, bottom_left, top_right, bottom_right) {
        (None, None, Some(tl), Some(bl), Some(tr), Some(br)) => {
            format!(
                "<mmultiscripts>{}{}{}<mprescripts />{}{}</mmultiscripts>",
                convert_to_mathml_impl(base, style_chain),
                convert_to_mathml_impl(&br, style_chain),
                convert_to_mathml_impl(&tr, style_chain),
                convert_to_mathml_impl(&bl, style_chain),
                convert_to_mathml_impl(&tl, style_chain),
            )
        }
        (Some(t), Some(b), None, None, None, None) => {
            format!(
                "<munderover>{}{}{}</munderover>",
                convert_to_mathml_impl(base, style_chain),
                convert_to_mathml_impl(&b, style_chain),
                convert_to_mathml_impl(&t, style_chain)
            )
        }
        (Some(t), None, None, None, None, None) => {
            format!(
                "<mover>{}{}</mover>",
                convert_to_mathml_impl(base, style_chain),
                convert_to_mathml_impl(&t, style_chain)
            )
        }
        (None, Some(b), None, None, None, None) => {
            format!(
                "<munder>{}{}</munder>",
                convert_to_mathml_impl(base, style_chain),
                convert_to_mathml_impl(&b, style_chain)
            )
        }
        (None, None, None, None, Some(tr), Some(br)) => {
            format!(
                "<msubsup>{}{}{}</msubsup>",
                convert_to_mathml_impl(base, style_chain),
                convert_to_mathml_impl(&br, style_chain),
                convert_to_mathml_impl(&tr, style_chain),
            )
        }
        (None, None, None, None, Some(tr), None) => {
            format!(
                "<msup>{}{}</msup>",
                convert_to_mathml_impl(base, style_chain),
                convert_to_mathml_impl(&tr, style_chain),
            )
        }
        (None, None, None, None, None, Some(br)) => {
            format!(
                "<msub>{}{}</msub>",
                convert_to_mathml_impl(base, style_chain),
                convert_to_mathml_impl(&br, style_chain),
            )
        }
        (None, None, None, None, None, None) => convert_to_mathml_impl(base, style_chain),
        (t, b, tl, bl, tr, br) => {
            let row_or_attach = |attach| {
                if let Some(a) = attach {
                    convert_to_mathml_impl(&a, style_chain)
                } else {
                    "<mrow></mrow>".to_string()
                }
            };
            format!(
                    "<munderover><mmultiscripts>{}{}{}<mprescripts />{}{}</mmultiscripts>{}{}</munderover>",
                        convert_to_mathml_impl(base, style_chain),
                        row_or_attach(br),
                        row_or_attach(tr),
                        row_or_attach(bl),
                        row_or_attach(tl),
                        row_or_attach(b),
                        row_or_attach(t),
                    )
        }
    }
}

fn process_text(content: &Content, style_chain: StyleChain) -> String {
    let coerced =
        content.to_packed::<TextElem>().expect("Type conversion to `TextElem` must be successful.");
    let is_italic = EquationElem::italic_in(style_chain);

    let original_text = coerced.text.to_string();
    let text = original_text
        .chars()
        .map(|each| {
            characters::styled_char(style_chain, each, matches!(is_italic, Smart::Custom(true)))
        })
        .collect::<String>();

    let identifier_attr =
        if matches!(is_italic, Smart::Custom(false)) { r#" mathvariant="normal""# } else { "" };

    if original_text.parse::<f64>().is_ok()
        || original_text.parse::<i64>().is_ok()
        || original_text == "∞"
    {
        return format!("<mn>{}</mn>", original_text);
    }

    let mut chars = text.chars();
    let first_char = match chars.next() {
        Some(it) => it,
        None => return String::with_capacity(0),
    };
    if chars.next().is_none() {
        let char_class = match unicode_math_class::class(first_char) {
            Some(it) => it,
            None => return format!("<mi{}>{}</mi>", identifier_attr, first_char),
        };

        math_class_helper(
            first_char.to_string().as_str(),
            Some(&original_text),
            &char_class,
            style_chain,
        )
    } else {
        let text = escape_helper(&text);
        if text.find(" ").is_some() {
            // FIXME: it should add begging and endding space according to pre and post elements.
            format!("<mtext>&nbsp;{}&nbsp;</mtext>", text)
        } else {
            format!("<mi{}>{}</mi>", identifier_attr, text)
        }
    }
}

fn process_root(content: &Content, style_chain: StyleChain) -> String {
    let coerced =
        content.to_packed::<RootElem>().expect("Type conversion to `RootElem` must be successful.");
    let index = coerced.index(style_chain);
    let radicand = &coerced.radicand;

    if let Some(index) = index {
        format!(
            "<mroot>{}{}</mroot>",
            convert_to_mathml_impl(radicand, style_chain),
            convert_to_mathml_impl(&index, style_chain)
        )
    } else {
        format!("<msqrt>{}</msqrt>", convert_to_mathml_impl(radicand, style_chain))
    }
}

/// MDN Reference: https://developer.mozilla.org/en-US/docs/Web/MathML/Element/mfrac#fraction_without_bar
fn process_binom(content: &Content, style_chain: StyleChain) -> String {
    let coerced = content
        .to_packed::<BinomElem>()
        .expect("Type conversion to `BinomElem` must be successful.");

    format!(
        r#"<mrow><mo>(</mo><mfrac linethickness="0">{}{}</mfrac><mo>)</mo></mrow>"#,
        convert_to_mathml_impl(&coerced.upper, style_chain),
        format!(
            "<mrow>{}</mrow>",
            coerced
                .lower
                .iter()
                .map(|child| convert_to_mathml_impl(child, style_chain))
                .collect::<Vec<_>>()
                .join("<mo>,</mo>")
        ),
    )
}

fn process_cancel(content: &Content, style_chain: StyleChain) -> String {
    let coerced = content
        .to_packed::<CancelElem>()
        .expect("Type conversion to `CancelElem` must be successful.");

    // `menclose` is not supported in MathML, but FireFox support it.
    // For other browses, a polyfill is necessary:
    //
    // <style>
    // .equation-typst-cancel {
    //   position: relative;
    //   padding: 0.5ex 0ex;
    // }

    // .equation-typst-cancel-wrapper-placeholder {
    //   display: inline-block;
    //   position: absolute;
    //   left: 0.5px;
    //   bottom: 0;
    //   width: 100%;
    //   height: 100%;
    //   background-color: black;
    //   clip-path: polygon(0.05em 100%, 0em calc(100% - 0.05em), calc(100% - 0.05em) 0em, 100% 0.05em);
    // }
    // </style>
    format!(
        r#"<menclose class="equation-typst-cancel" notation="updiagonalstrike">{}<mrow class="equation-typst-cancel-wrapper-placeholder"></mrow></menclose>"#,
        convert_to_mathml_impl(&coerced.body, style_chain)
    )
}

fn process_op(content: &Content, style_chain: StyleChain) -> String {
    let coerced =
        content.to_packed::<OpElem>().expect("Type conversion to `OpElem` must be successful.");

    let text = &coerced.text;

    // FIXME: MathML Core recommends to use <mi> to describe functions rather than <mo>.
    if text.elem().name() == "text" {
        format!("<mo>{}</mo>", escape_helper(&text.plain_text()))
    } else {
        format!("<mo>{}</mo>", convert_to_mathml_impl(text, style_chain))
    }
}

fn process_lr(content: &Content, style_chain: StyleChain) -> String {
    let coerced =
        content.to_packed::<LrElem>().expect("Type conversion to `LrElem` must be successful.");

    let _size = coerced.size(style_chain);

    format!("<mrow>{}</mrow>", convert_to_mathml_impl(&coerced.body, style_chain))
}

fn process_equation(content: &Content, style_chain: StyleChain) -> String {
    let coerced = content
        .to_packed::<EquationElem>()
        .expect("Type conversion to `EquationElem` must be successful.");

    // FIXME: need more processing?
    convert_to_mathml_impl(&coerced.body, style_chain)
}

/// MDN Reference: https://developer.mozilla.org/en-US/docs/Web/MathML/Guides/Tables#usage_for_advanced_layout
fn process_cases(content: &Content, style_chain: StyleChain) -> String {
    let coerced = content
        .to_packed::<CasesElem>()
        .expect("Type conversion to `CasesElem` must be successful.");

    let delim = coerced.delim(style_chain);
    let reverse = coerced.reverse(style_chain);
    let _gap = coerced.gap(style_chain);
    let children = &coerced.children;

    let mtd_left_str = r#"<mtd style="text-align: left">"#;

    let mut res = format!("<mrow>");
    if !reverse && delim.open().is_some() {
        res.push_str("<mo>");
        res.push(delim.open().unwrap());
        res.push_str("</mo>");
    }
    res.push_str("<mtable>");

    for child in children {
        let child_type = child.elem().name();

        res.push_str("<mtr>");
        if child_type == "sequence" {
            let child_coerced = child
                .to_packed::<SequenceElem>()
                .expect("Type conversion to `SequenceElem` must be successful.");
            let mut tmp = child_coerced.children.iter();
            let first = tmp.next().and_then(|it| Some(convert_to_mathml_impl(it, style_chain)));
            if let Some(first) = first {
                res.push_str(mtd_left_str);
                res.push_str(&first);
                res.push_str("</mtd>");
            }
            let rest: String = tmp
                .map(|each| {
                    if each.elem().name() == "space" {
                        "<mtext>&nbsp;</mtext>".to_string()
                    } else {
                        convert_to_mathml_impl(each, style_chain)
                    }
                })
                .collect();
            if rest.len() > 0 {
                res.push_str(mtd_left_str);
                res.push_str(&rest);
                res.push_str("</mtd>");
            }
        } else {
            res.push_str(mtd_left_str);
            res.push_str(&convert_to_mathml_impl(&child, style_chain));
            res.push_str("</mtd>");
        }
        res.push_str("</mtr>");
    }

    res.push_str("</mtable>");
    if reverse && delim.close().is_some() {
        res.push_str("<mo>");
        res.push(delim.close().unwrap());
        res.push_str("</mo>");
    }
    res.push_str("</mrow>");

    res
}

/// MDN Reference: https://developer.mozilla.org/en-US/docs/Web/MathML/Element/mtd#matrix_using_mtable_mrow_mtr_and_mtd
fn process_mat(content: &Content, style_chain: StyleChain) -> String {
    let coerced =
        content.to_packed::<MatElem>().expect("Type conversion to `MatElem` must be successful.");

    let delim = coerced.delim(style_chain);
    let _augment = coerced.augment(style_chain);
    let align = coerced.align(style_chain);
    let _row_gap = coerced.row_gap(style_chain);
    let _column_gap = coerced.column_gap(style_chain);
    let rows = &coerced.rows;

    // FIXME: It seems that CSS align does not work in Chrome.
    let mtd_left_str = match align {
        FixedAlignment::Start => r#"<mtd style="text-align: left">"#,
        FixedAlignment::Center => r#"<mtd style="text-align: center">"#,
        FixedAlignment::End => r#"<mtd style="text-align: right">"#,
    };

    let mut res = format!("<mrow>");
    if delim.open().is_some() {
        res.push_str(r#"<mo form="prefix">"#);
        res.push(delim.open().unwrap());
        res.push_str("</mo>");
    }
    res.push_str("<mtable>");

    for row in rows {
        res.push_str("<mtr>");

        for item in row {
            res.push_str(mtd_left_str);
            res.push_str(&convert_to_mathml_impl(item, style_chain));
            res.push_str("</mtd>");
        }

        res.push_str("</mtr>");
    }

    res.push_str("</mtable>");
    if delim.close().is_some() {
        res.push_str(r#"<mo form="postfix">"#);
        res.push(delim.close().unwrap());
        res.push_str("</mo>");
    }
    res.push_str("</mrow>");

    res
}

fn process_vec(content: &Content, style_chain: StyleChain) -> String {
    let coerced =
        content.to_packed::<VecElem>().expect("Type conversion to `VecElem` must be successful.");

    let delim = coerced.delim(style_chain);
    let align = coerced.align(style_chain);
    let _gap = coerced.gap(style_chain);
    let children = &coerced.children;

    // FIXME: It seems that CSS align does not work in Chrome.
    let mtd_left_str = match align {
        FixedAlignment::Start => r#"<mtd style="text-align: left">"#,
        FixedAlignment::Center => r#"<mtd style="text-align: center">"#,
        FixedAlignment::End => r#"<mtd style="text-align: right">"#,
    };

    let mut res = format!("<mrow>");
    if delim.open().is_some() {
        res.push_str(r#"<mo form="prefix">"#);
        res.push(delim.open().unwrap());
        res.push_str("</mo>");
    }
    res.push_str("<mtable>");

    for child in children {
        res.push_str("<mtr>");
        res.push_str(mtd_left_str);
        res.push_str(&convert_to_mathml_impl(&child, style_chain));
        res.push_str("</mtd></mtr>");
    }

    res.push_str("</mtable>");
    if delim.close().is_some() {
        res.push_str(r#"<mo form="postfix">"#);
        res.push(delim.close().unwrap());
        res.push_str("</mo>");
    }
    res.push_str("</mrow>");

    res
}

fn process_class(content: &Content, style_chain: StyleChain) -> String {
    let coerced = content
        .to_packed::<ClassElem>()
        .expect("Type conversion to `ClassElem` must be successful.");

    let math_class = &coerced.class;
    let text = if coerced.body.elem().name() == "text" {
        coerced.body.plain_text().to_string()
    } else {
        convert_to_mathml_impl(&coerced.body, style_chain)
    };

    math_class_helper(&text, None, math_class, style_chain)
}

fn process_primes(content: &Content) -> String {
    let coerced = content
        .to_packed::<PrimesElem>()
        .expect("Type conversion to `PrimesElem` must be successful.");

    let count = coerced.count;

    format!("<mo>{}</mo>", "&#x2032;".repeat(count))
}

fn process_styled(content: &Content, style_chain: StyleChain) -> String {
    let coerced = content
        .to_packed::<StyledElem>()
        .expect("Type conversion to `StyledElem` must be successful.");

    let style_chain = coerced.styles.chain(&style_chain);
    let child = &coerced.child;

    convert_to_mathml_impl(child, style_chain)
}

fn process_h(content: &Content) -> String {
    let coerced =
        content.to_packed::<HElem>().expect("Type conversion to `HElem` must be successful.");

    let spacing = &coerced.amount;
    let spacing_str =
        if let Spacing::Rel(rel) = spacing { format!("{:?}", rel) } else { "".to_string() };

    if spacing_str.ends_with("em") {
        format!(r#"<mspace width="{}"></mspace>"#, spacing_str)
    } else {
        format!(r#"<mspace width="0.333em"></mspace>"#)
    }
}

fn process_hide(content: &Content, style_chain: StyleChain) -> String {
    let coerced =
        content.to_packed::<HideElem>().expect("Type conversion to `HideElem` must be successful.");

    let body = &coerced.body;

    format!(
        r#"<mrow style="visibility: hidden;">{}</mrow>"#,
        convert_to_mathml_impl(body, style_chain)
    )
}

fn process_stretch(content: &Content, style_chain: StyleChain) -> String {
    let coerced = content
        .to_packed::<StretchElem>()
        .expect("Type conversion to `StretchElem` must be successful.");

    // FIXME: Sadly, it seems that setting horizontal size for stretchy math operator
    // does not work in browsers now, maybe i can revisit it in the future.
    let _size = coerced.size(style_chain);
    let body = &coerced.body;

    if body.elem().name() == "text" {
        format!(
            r#"<mo stretchy="true">{}</mo>"#,
            escape_helper(body.plain_text().to_string().as_str())
        )
    } else {
        convert_to_mathml_impl(body, style_chain)
    }
}

fn process_sequence(content: &Content, style_chain: StyleChain) -> String {
    let coerced = content
        .to_packed::<SequenceElem>()
        .expect("Type conversion to `SequenceElem` must be successful.");
    let children = &coerced.children;

    let mtd_left_align_begin = r#"<mtd style="text-align: left">"#;
    let mtd_right_align_begin = r#"<mtd style="text-align: right">"#;
    let mtd_center_align_begin = r#"<mtd style="text-align: center">"#;

    let children_split: Vec<_> = children
        .split(|each| each.elem().name() == "linebreak")
        .map(|each| each.split(|it| it.elem().name() == "align-point").collect::<Vec<_>>())
        .collect();

    let combine_str = |it: Iter<Content>| {
        it.map(|child| convert_to_mathml_impl(child, style_chain)).collect::<String>()
    };

    let is_no_align_point = children_split.iter().map(|each| each.len()).all(|num| num == 1);

    if children_split.len() > 1 {
        let mut res = "<mtable>".to_string();

        for row in children_split {
            res.push_str("<mtr>");

            for (idx, item) in row.iter().enumerate() {
                if is_no_align_point {
                    res.push_str(mtd_center_align_begin);
                } else if idx % 2 == 0 {
                    res.push_str(mtd_right_align_begin);
                } else {
                    res.push_str(mtd_left_align_begin);
                }

                res.push_str(&combine_str(item.iter()));

                res.push_str("</mtd>");
            }

            res.push_str("</mtr>");
        }

        res.push_str("</mtable>");

        res
    } else {
        format!("<mrow>{}</mrow>", combine_str(children.iter()))
    }
}

fn process_underline(content: &Content, style_chain: StyleChain) -> String {
    let coerced = content
        .to_packed::<UnderlineElem>()
        .expect("Type conversion to `UnderlineElem` must be successful.");

    let body = &coerced.body;

    // format!(
    //     r#"<munder>{}<mo stretchy="true">&#x332;</mo></munder>"#,
    //     convert_to_mathml_impl(body, style_chain)
    // )

    // FIXME: Now Chrome does not respect <munder> with underline strechy operater. I opened a
    // bug report https://issues.chromium.org/issues/386610915. When the bug is resolved, this
    // should switch to <munder>.
    format!(
        r#"<mrow style="border-bottom: 1px solid currentColor; display: inline-block;">{}</mrow>"#,
        convert_to_mathml_impl(&body, style_chain)
    )
}

fn process_overline(content: &Content, style_chain: StyleChain) -> String {
    let coerced = content
        .to_packed::<OverlineElem>()
        .expect("Type conversion to `OverlineElem` must be successful.");

    let body = &coerced.body;

    // FIXME: Now Chrome does not respect <mover> with overline strechy operater. I opened a
    // bug report https://issues.chromium.org/issues/386610915. When the bug is resolved, this
    // should switch to <mover>.
    format!(
        r#"<mrow style="border-top: 1px solid currentColor; display: inline-block;">{}</mrow>"#,
        convert_to_mathml_impl(&body, style_chain)
    )
}

fn process_underbrace(content: &Content, style_chain: StyleChain) -> String {
    let coerced = content
        .to_packed::<UnderbraceElem>()
        .expect("Type conversion to `UnderbraceElem` must be successful.");

    let body = &coerced.body;
    let annotation = coerced.annotation(style_chain);

    let underbrace = if let Some(annotation) = annotation {
        format!(
            r#"<munder><mo stretchy="true">&#x23DF;</mo><mtext>{}</mtext></munder>"#,
            convert_to_mathml_impl(&annotation, style_chain)
        )
    } else {
        r#"<mo stretchy="true">&#x23DF;</mo>"#.to_string()
    };

    format!(r#"<munder>{}{}</munder>"#, convert_to_mathml_impl(&body, style_chain), underbrace)
}

fn process_overbrace(content: &Content, style_chain: StyleChain) -> String {
    let coerced = content
        .to_packed::<OverbraceElem>()
        .expect("Type conversion to `OverbraceElem` must be successful.");

    let body = &coerced.body;
    let annotation = coerced.annotation(style_chain);

    let overbrace = if let Some(annotation) = annotation {
        format!(
            r#"<mover><mo stretchy="true">&#x23DE;</mo><mtext>{}</mtext></mover>"#,
            convert_to_mathml_impl(&annotation, style_chain)
        )
    } else {
        r#"<mo stretchy="true">&#x23DE;</mo>"#.to_string()
    };

    format!(r#"<mover>{}{}</mover>"#, convert_to_mathml_impl(&body, style_chain), overbrace)
}

fn process_underbracket(content: &Content, style_chain: StyleChain) -> String {
    let coerced = content
        .to_packed::<UnderbracketElem>()
        .expect("Type conversion to `UnderbracketElem` must be successful.");

    let body = &coerced.body;
    let annotation = coerced.annotation(style_chain);

    let underbracket = if let Some(annotation) = annotation {
        format!(
            r#"<munder><mo stretchy="true">&#x23B5;</mo><mtext>{}</mtext></munder>"#,
            convert_to_mathml_impl(&annotation, style_chain)
        )
    } else {
        r#"<mo stretchy="true">&#x23B5;</mo>"#.to_string()
    };

    format!(r#"<munder>{}{}</munder>"#, convert_to_mathml_impl(&body, style_chain), underbracket)
}

fn process_overbracket(content: &Content, style_chain: StyleChain) -> String {
    let coerced = content
        .to_packed::<OverbracketElem>()
        .expect("Type conversion to `OverbracketElem` must be successful.");

    let body = &coerced.body;
    let annotation = coerced.annotation(style_chain);

    let overbracket = if let Some(annotation) = annotation {
        format!(
            r#"<mover><mo stretchy="true">&#x23B4;</mo><mtext>{}</mtext></mover>"#,
            convert_to_mathml_impl(&annotation, style_chain)
        )
    } else {
        r#"<mo stretchy="true">&#x23B4;</mo>"#.to_string()
    };

    format!(r#"<mover>{}{}</mover>"#, convert_to_mathml_impl(&body, style_chain), overbracket)
}

fn process_underparen(content: &Content, style_chain: StyleChain) -> String {
    let coerced = content
        .to_packed::<UnderparenElem>()
        .expect("Type conversion to `UnderparenElem` must be successful.");

    let body = &coerced.body;
    let annotation = coerced.annotation(style_chain);

    let underparen = if let Some(annotation) = annotation {
        format!(
            r#"<munder><mo stretchy="true">&#x2323;</mo><mtext>{}</mtext></munder>"#,
            convert_to_mathml_impl(&annotation, style_chain)
        )
    } else {
        r#"<mo stretchy="true">&#x2323;</mo>"#.to_string()
    };

    // FIXME: Both Firefox and Chrome does not render properly. However, I do not have
    // any idea how to implement it using other methods.
    // MathJax processes it correctly.
    format!(r#"<munder>{}{}</munder>"#, convert_to_mathml_impl(&body, style_chain), underparen)
}

fn process_overparen(content: &Content, style_chain: StyleChain) -> String {
    let coerced = content
        .to_packed::<OverparenElem>()
        .expect("Type conversion to `OverparenElem` must be successful.");

    let body = &coerced.body;
    let annotation = coerced.annotation(style_chain);

    let overparen = if let Some(annotation) = annotation {
        format!(
            r#"<mover><mo stretchy="true">&#x2322;</mo><mtext>{}</mtext></mover>"#,
            convert_to_mathml_impl(&annotation, style_chain)
        )
    } else {
        r#"<mo stretchy="true">&#x2322;</mo>"#.to_string()
    };

    // FIXME: Both Firefox and Chrome does not render properly. However, I do not have
    // any idea how to implement it using other methods.
    // MathJax processes it correctly.
    format!(r#"<mover>{}{}</mover>"#, convert_to_mathml_impl(&body, style_chain), overparen)
}

fn process_undershell(content: &Content, style_chain: StyleChain) -> String {
    let coerced = content
        .to_packed::<UndershellElem>()
        .expect("Type conversion to `UndershellElem` must be successful.");

    let body = &coerced.body;
    let annotation = coerced.annotation(style_chain);

    let undershell = if let Some(annotation) = annotation {
        format!(
            r#"<munder><mo stretchy="true">&#x23E1;</mo><mtext>{}</mtext></munder>"#,
            convert_to_mathml_impl(&annotation, style_chain)
        )
    } else {
        r#"<mo stretchy="true">&#x23E1;</mo>"#.to_string()
    };

    // FIXME: Both Firefox, Chrome and MathJax does not render properly. However, I do not have
    // any idea how to implement it using other methods.
    format!(r#"<munder>{}{}</munder>"#, convert_to_mathml_impl(&body, style_chain), undershell)
}

fn process_overshell(content: &Content, style_chain: StyleChain) -> String {
    let coerced = content
        .to_packed::<OvershellElem>()
        .expect("Type conversion to `OvershellElem` must be successful.");

    let body = &coerced.body;
    let annotation = coerced.annotation(style_chain);

    let overshell = if let Some(annotation) = annotation {
        format!(
            r#"<mover><mo stretchy="true">&#x23E0;</mo><mtext>{}</mtext></mover>"#,
            convert_to_mathml_impl(&annotation, style_chain)
        )
    } else {
        r#"<mo stretchy="true">&#x23E0;</mo>"#.to_string()
    };

    // FIXME: Both Firefox, Chrome and MathJax does not render properly. However, I do not have
    // any idea how to implement it using other methods.
    format!(r#"<mover>{}{}</mover>"#, convert_to_mathml_impl(&body, style_chain), overshell)
}

fn process_mid(content: &Content, style_chain: StyleChain) -> String {
    let coerced = content
        .to_packed::<MidElem>()
        .expect("Type conversion to `OvershellElem` must be successful.");

    let body = &coerced.body;
    let text = if body.elem().name() == "text" {
        body.plain_text().to_string()
    } else {
        convert_to_mathml_impl(body, style_chain)
    };

    format!(r#"<mo fence="true" form="infix" stretchy="true">{}</mo>"#, text)
}

fn math_class_helper(
    text: &str,
    original_text: Option<&str>,
    class: &MathClass,
    style_chain: StyleChain,
) -> String {
    let is_italic = EquationElem::italic_in(style_chain);

    let identifier_attr = if matches!(is_italic, Smart::Custom(false)) {
        r#" mathvariant="normal""#
    } else {
        let text = if let Some(original_text) = original_text { original_text } else { text };
        let char_vec: Vec<_> = text.chars().collect();
        if char_vec.len() == 1 && style::italic::is_normal(char_vec[0]) {
            r#" mathvariant="normal""#
        } else {
            ""
        }
    };

    let text = escape_helper(text);
    match class {
        MathClass::Normal => format!("<mi{}>{}</mi>", identifier_attr, text),
        MathClass::Alphabetic => format!("<mi{}>{}</mi>", identifier_attr, text),
        MathClass::Binary => format!(r#"<mo form="infix">{}</mo>"#, text),
        MathClass::Closing => format!(r#"<mo fence="true" form="postfix">{}</mo>"#, text),
        MathClass::Diacritic => format!("<mi{}>{}</mi>", identifier_attr, text),
        MathClass::Fence => format!(r#"<mo fence="true">{}</mo>"#, text),
        MathClass::GlyphPart => format!("<mo>{}</mo>", text),
        MathClass::Large => format!(r#"<mo largeop="true">{}</mo>"#, text),
        MathClass::Opening => format!(r#"<mo fence="true" form="prefix">{}</mo>"#, text),
        MathClass::Punctuation => format!(r#"<mo separator="true">{}</mo>"#, text),
        MathClass::Relation => format!("<mo>{}</mo>", text),
        MathClass::Space => format!(r#"<mspace width="0.333em"></mspace>"#),
        MathClass::Unary => format!(r#"<mo form="prefix">{}</mo>"#, text),
        // FIXME: need further processing
        MathClass::Vary => format!("<mo>{}</mo>", text),
        MathClass::Special => format!("<mo>{}</mo>", text),
    }
}

fn escape_helper(text: &str) -> String {
    // See <https://html.spec.whatwg.org/multipage/syntax.html#syntax-charref>
    let mut res = String::new();
    for c in text.chars() {
        match c {
            '&' => res.push_str("&amp;"),
            '<' => res.push_str("&lt;"),
            '>' => res.push_str("&gt;"),
            '"' => res.push_str("&quot;"),
            '\'' => res.push_str("&apos;"),
            // c if charsets::is_w3c_text_char(c) && c != '\r' => {
            //     res.push_str(&format!("&#x{:x};", c as u32))
            // }
            _ => res.push(c),
        }
    }

    res
}
