use crate::cache::GenericCache;
use convert::convert_to_mathml;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use twox_hash::XxHash64;

use super::{Compiler, ShouldMinify, TypstRenderMode};

mod convert;
mod eval;
mod style;
pub type TypstMathMLCacheEntry = String;
pub type TypstMathMLCache = GenericCache<String, TypstMathMLCacheEntry>;
pub struct TypstMathMLCompiler {
    pub cache: Option<Arc<TypstMathMLCache>>,
}

impl TypstMathMLCompiler {
    pub fn new() -> Self {
        Self { cache: None }
    }
}

impl Compiler<TypstRenderMode, String> for TypstMathMLCompiler {
    fn set_cache(&mut self, cache: Arc<TypstMathMLCache>) {
        self.cache = Some(cache.clone());
    }

    fn write_cache(&self) -> Result<(), String> {
        if let Some(cache) = self.cache.as_ref() {
            cache.write().map_err(|e| e.to_string())?
        }
        Ok(())
    }

    fn compile(
        &self,
        input: &str,
        mode: TypstRenderMode,
        minify: &ShouldMinify,
    ) -> Result<String, String> {
        let input = match mode {
            TypstRenderMode::Display => format!("$ {} $", input),
            TypstRenderMode::Inline => format!("${}$", input),
            _ => return Err("Raw mode is not supported for Typst mathml".to_string()),
        };

        // Generate cache key
        let key = {
            let mut hasher = XxHash64::with_seed(42);
            input.hash(&mut hasher);
            mode.hash(&mut hasher);
            minify.hash(&mut hasher);
            format!("{:x}", hasher.finish())
        };

        if let Some(entry) = self.cache.as_ref().and_then(|e| e.get(&key)) {
            return Ok(entry.clone());
        }

        let rendered = convert_to_mathml(&input, false)?;

        if let Some(cache) = self.cache.as_ref() {
            cache.insert(key, rendered.clone());
        }

        Ok(rendered)
    }
}
