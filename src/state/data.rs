use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum ItemData {
    File(String),
    Dir(String),
}

impl ItemData {
    pub fn mime(&self) -> &'static str {
        match self {
            ItemData::File(_) | ItemData::Dir(_) => "text/uri-list",
        }
    }

    pub fn uri(&self) -> &str {
        match self {
            ItemData::File(u) | ItemData::Dir(u) => u,
        }
    }

    pub fn is_dir(&self) -> bool {
        matches!(self, ItemData::Dir(_))
    }
}
