use libs::pulldown_cmark::{html::ToClass, AdmonitionTagCallback};

/// Performs case-insensitive byte string comparison using bitwise operation
/// | 32 to lowercase the ASCII characters
pub const fn case_insensitive_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut i = 0;
    while i < a.len() {
        if a[i] | 32 != b[i] | 32 {
            return false;
        }
        i += 1;
    }
    true
}

#[derive(Clone, Debug, Copy, Eq, PartialEq)]
pub enum ObsidianCalloutType {
    Note,
    Tip,
    Warning,
    Important,
    Info,
    Question,
    Error,
}

impl ObsidianCalloutType {
    const VARIANTS: &'static [(&'static str, Self)] = &[
        ("note", Self::Note),
        ("tip", Self::Tip),
        ("warning", Self::Warning),
        ("important", Self::Important),
        ("info", Self::Info),
        ("question", Self::Question),
        ("error", Self::Error),
    ];

    /// Computes the maximum length of variant strings at compile-time
    const MAX_VARIANT_LENGTH: usize = {
        let mut max_len = 0;
        let mut i = 0;
        while i < Self::VARIANTS.len() {
            if Self::VARIANTS[i].0.len() > max_len {
                max_len = Self::VARIANTS[i].0.len();
            }
            i += 1;
        }
        max_len
    };

    pub fn as_str(&self) -> &'static str {
        Self::VARIANTS
            .iter()
            .find(|(_, variant)| variant == self)
            .map(|(name, _)| *name)
            .expect("Variant should always have a string representation") // Safety: variant is always valid
    }

    pub fn from_str(s: &str) -> Option<Self> {
        // Early return for strings that are too long
        if s.len() > Self::MAX_VARIANT_LENGTH {
            return None;
        }

        let input_bytes = s.as_bytes();
        Self::VARIANTS
            .iter()
            .find(|(name, _)| case_insensitive_eq(input_bytes, name.as_bytes()))
            .map(|(_, variant)| *variant)
    }
}

impl<'input> AdmonitionTagCallback<'input> for ObsidianCalloutsHandler {
    type DataKind = ObsidianCalloutType;

    fn handle_admonition_tag(&mut self, tag: &'input str) -> Option<Self::DataKind> {
        ObsidianCalloutType::from_str(tag)
    }
}

impl<'a> ToClass<'a> for ObsidianCalloutType {
    fn to_class(&self) -> &'a str {
        self.as_str()
    }
}

pub struct ObsidianCalloutsHandler;
