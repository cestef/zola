use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

use pikchr::{Pikchr, PikchrFlags};
use twox_hash::XxHash64;

use crate::cache::GenericCache;

pub type PikchrCache = GenericCache<String, String>;

pub struct PikchrCompiler {
    pub cache: Arc<PikchrCache>,
}

impl PikchrCompiler {
    pub fn new(cache: Arc<PikchrCache>) -> Self {
        Self { cache }
    }

    pub fn render(&self, input: &str, auto_dark_mode: bool) -> crate::Result<String> {
        let mut svgs = vec![(self._render(input, false)?, false)];
        if auto_dark_mode {
            svgs.push((self._render(input, true)?, true));
        }

        Ok(svgs
            .into_iter()
            .map(|(svg, dark_mode)| {
                let mode = if dark_mode { "dark" } else { "light" };
                let url_encoded = urlencoding::encode(&svg);
                format!(
                    r#"<img src="data:image/svg+xml;charset=utf-8,{url}" alt="{mode} mode" class="pickhr-svg pikchr-{mode}">"#,
                    url = url_encoded,
                    mode = mode
                )
            })
            .collect())
    }

    fn _render(&self, input: &str, dark_mode: bool) -> crate::Result<String> {
        let key = {
            let mut hasher = XxHash64::with_seed(42);
            input.hash(&mut hasher);
            dark_mode.hash(&mut hasher);
            format!("{:x}", hasher.finish())
        };

        if let Some(cached) = self.cache.get(&key) {
            return Ok(cached.clone());
        }

        let mut flags = PikchrFlags::default();
        if dark_mode {
            flags.use_dark_mode();
        }
        Ok(Pikchr::render(input, Some("pikchr"), flags)
            .map_err(|e| errors::Error::msg(format!("Failed to render pikchr: {}", e)))?
            .to_string())
    }
}
