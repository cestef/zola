use std::path::Path;

use super::TypstRenderMode;

use libs::once_cell::sync::Lazy;
use libs::regex;
use utils::fs::read_file;

static HEIGHT_RE: Lazy<regex::Regex> =
    Lazy::new(|| regex::Regex::new(r#"height="((?:\d|\.)+)(?:pt)?""#).unwrap());
static WIDTH_RE: Lazy<regex::Regex> =
    Lazy::new(|| regex::Regex::new(r#"width="((?:\d|\.)+)(?:pt)?""#).unwrap());

const EM_PER_PT: f64 = 11.0;
const DEFAULT_LIGHT_STYLES: &str = include_str!("styles/light.css");
const DEFAULT_DARK_STYLES: &str = include_str!("styles/dark.css");

pub fn format_svg(
    svg: &str,
    align: Option<f64>,
    render_mode: TypstRenderMode,
    dark_styles: Option<&str>,
    light_styles: Option<&str>,
    dark_mode: bool,
) -> String {
    let dark_styles = dark_styles.map(|path| read_file(Path::new(path)).ok()).flatten();
    let light_styles = light_styles.map(|path| read_file(Path::new(path)).ok()).flatten();

    let height =
        HEIGHT_RE.captures(svg).and_then(|caps| caps[1].parse::<f64>().ok()).unwrap_or(0.0);

    let width = WIDTH_RE.captures(svg).and_then(|caps| caps[1].parse::<f64>().ok()).unwrap_or(0.0);
    let mut svg = svg.to_string();

    if render_mode == TypstRenderMode::Raw {
        // Add 10pt to the height to account for the padding
        svg = svg.replacen(
            &format!("height=\"{}pt\"", height),
            &format!("height=\"{}pt\"", height + 10.0),
            1,
        );
    }

    let shift = align.map(|align| height - align);
    let shift_em = shift.map(|shift| shift / EM_PER_PT);

    let mut imgs = vec![(
        svg.replacen(
            ">",
            &format!(
                "><style>{}</style>",
                light_styles.unwrap_or(DEFAULT_LIGHT_STYLES.to_string())
            ),
            1,
        ),
        "light",
    )];

    if dark_mode {
        imgs.push((
            svg.replacen(
                ">",
                &format!(
                    "><style>{}</style>",
                    dark_styles.unwrap_or(DEFAULT_DARK_STYLES.to_string())
                ),
                1,
            ),
            "dark",
        ));
    }

    let mut out = String::new();

    for (svg, theme) in imgs {
        let url_encoded = urlencoding::encode(&svg);
        out.push_str(&format!(
            "<img src=\"data:image/svg+xml,{url_encoded}\" class=\"{} typst-doc typst-{theme}\" style=\"{} width: {}em\" loading=\"lazy\" decoding=\"async\" alt=\"\" />",
            match render_mode {
                TypstRenderMode::Display | TypstRenderMode::Raw => "typst-display",
                TypstRenderMode::Inline => "typst-inline",
            },
            if let Some(shift_em) = shift_em {
                format!("vertical-align: -{}em;", shift_em)
            } else {
                String::new()
            },
            width / EM_PER_PT
        ));
    }

    out
}
