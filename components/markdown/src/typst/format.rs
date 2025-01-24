use lazy_static::lazy_static;

use super::RenderMode;

lazy_static! {
    static ref HEIGHT_RE: regex::Regex = regex::Regex::new(r#"height="(.*)pt""#).unwrap();
}

const EM_PER_PT: f64 = 11.0;
const LIGHT_STYLES: &str = include_str!("styles/light.css");
const DARK_STYLES: &str = include_str!("styles/dark.css");

pub fn format_svg(svg: &str, align: f64, render_mode: RenderMode) -> String {
    let height =
        HEIGHT_RE.captures(svg).and_then(|caps| caps[1].parse::<f64>().ok()).unwrap_or(0.0);

    let shift = height - align;
    let shift_em = shift / EM_PER_PT;

    // Inject the styles into the SVG
    let light_svg = svg.replacen(">\n", &format!(">\n<style>{}</style>\n", LIGHT_STYLES), 1);
    let dark_svg = svg.replacen(">\n", &format!(">\n<style>{}</style>\n", DARK_STYLES), 1);
    let imgs = vec![(light_svg, "light"), (dark_svg, "dark")];
    let mut out = String::new();

    for (svg, theme) in imgs {
        let url_encoded = urlencoding::encode(&svg);
        out.push_str(&format!(
            "<img src=\"data:image/svg+xml,{url_encoded}\" class=\"{} typst-doc typst-{theme}\" style=\"vertical-align: -{shift_em}em;\" loading=\"lazy\" decoding=\"async\" alt=\"\" />",
            match render_mode {
                RenderMode::Display => "typst-display",
                RenderMode::Inline => "typst-inline",
            },
        ));
    }

    out
}
