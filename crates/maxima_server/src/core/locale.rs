use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub enum Locale {
    #[default]
    EnUs,
}

impl Locale {
    pub fn short_str(&self) -> &'static str {
        match self {
            Locale::EnUs => "en",
        }
    }

    pub fn full_str(&self) -> &'static str {
        match self {
            Locale::EnUs => "en_US",
        }
    }
}
