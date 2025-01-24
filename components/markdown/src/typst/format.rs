use super::RenderMode;

use libs::once_cell::sync::Lazy;
use libs::regex;

static HEIGHT_RE: Lazy<regex::Regex> =
    Lazy::new(|| regex::Regex::new(r#"height="(.*)pt""#).unwrap());

const EM_PER_PT: f64 = 11.0;
const DEFAULT_LIGHT_STYLES: &str = include_str!("styles/light.css");
const DEFAULT_DARK_STYLES: &str = include_str!("styles/dark.css");

pub fn format_svg(
    svg: &str,
    align: Option<f64>,
    render_mode: RenderMode,
    dark_styles: Option<&str>,
    light_styles: Option<&str>,
    dark_mode: bool,
) -> String {
    let height =
        HEIGHT_RE.captures(svg).and_then(|caps| caps[1].parse::<f64>().ok()).unwrap_or(0.0);

    let mut svg = svg.to_string();

    if render_mode == RenderMode::Raw {
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
            ">\n",
            &format!(">\n<style>{}</style>\n", light_styles.unwrap_or(DEFAULT_LIGHT_STYLES)),
            1,
        ),
        "light",
    )];

    if dark_mode {
        imgs.push((
            svg.replacen(
                ">\n",
                &format!(">\n<style>{}</style>\n", dark_styles.unwrap_or(DEFAULT_DARK_STYLES)),
                1,
            ),
            "dark",
        ));
    }

    let mut out = String::new();

    for (svg, theme) in imgs {
        let url_encoded = urlencoding::encode(&svg);
        out.push_str(&format!(
            "<img src=\"data:image/svg+xml,{url_encoded}\" class=\"{} typst-doc typst-{theme}\" style=\"{} height:{}pt\" loading=\"lazy\" decoding=\"async\" alt=\"\" />",
            match render_mode {
                RenderMode::Display | RenderMode::Raw => "typst-display",
                RenderMode::Inline => "typst-inline",
            },
            if let Some(shift_em) = shift_em {
                format!("vertical-align: -{}em;", shift_em)
            } else {
                String::new()
            },
            height,
        ));
    }

    out
}
