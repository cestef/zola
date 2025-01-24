use bincode;
use errors::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CacheEntry {
    pub content: String,
    pub align: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TypstCache {
    cache_dir: PathBuf,
    entries: HashMap<String, CacheEntry>,
}

impl TypstCache {
    pub fn new() -> Result<Self> {
        let cache_dir = Path::new(".cache").to_path_buf();
        fs::create_dir_all(&cache_dir)?;

        let cache_path = cache_dir.join("typst.bin");
        if cache_path.exists() {
            let data = fs::read(&cache_path)?;
            Ok(bincode::deserialize(&data)?)
        } else {
            Ok(Self { cache_dir, entries: HashMap::new() })
        }
    }

    pub fn get(&self, key: &str) -> Option<CacheEntry> {
        self.entries.get(key).cloned()
    }

    pub fn insert(&mut self, key: String, content: String, align: Option<f64>) -> Result<()> {
        self.entries.insert(key, CacheEntry { content, align });
        self.save()
    }

    fn save(&self) -> Result<()> {
        let data = bincode::serialize(self)?;
        fs::write(self.cache_dir.join("typst.bin"), data)?;
        Ok(())
    }
}
