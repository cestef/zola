use libs::pulldown_cmark::{html::ToClass, AdmonitionTagCallback};
pub struct ObsidianCalloutsHandler;

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
    pub fn as_str(&self) -> &'static str {
        match self {
            ObsidianCalloutType::Note => "note",
            ObsidianCalloutType::Tip => "tip",
            ObsidianCalloutType::Warning => "warning",
            ObsidianCalloutType::Important => "important",
            ObsidianCalloutType::Info => "info",
            ObsidianCalloutType::Question => "question",
            ObsidianCalloutType::Error => "error",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "note" => Some(ObsidianCalloutType::Note),
            "tip" => Some(ObsidianCalloutType::Tip),
            "warning" => Some(ObsidianCalloutType::Warning),
            "important" => Some(ObsidianCalloutType::Important),
            "info" => Some(ObsidianCalloutType::Info),
            "question" => Some(ObsidianCalloutType::Question),
            "error" => Some(ObsidianCalloutType::Error),
            _ => None,
        }
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
