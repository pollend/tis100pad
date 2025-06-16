// ISO 639 language codes
#[derive(Clone, Copy)]
pub enum Il8nLocale {
    Eng,
}

impl Il8nLocale {
    pub fn to_display_str(&self) -> &'static str {
        match self {
            Il8nLocale::Eng => "English",
        }
    }

    pub fn to_iso_639_3(&self) -> &'static str {
        match self {
            Il8nLocale::Eng => "eng",
        }
    }

    pub fn is_rtl(&self) -> bool {
        match self {
            Il8nLocale::Eng => false,
        }
    }

    pub fn from_iso_639_3_str(code: &str) -> Option<Il8nLocale> {
        match code {
            "eng" => Some(Il8nLocale::Eng),
            _ => None,
        }
    }
}

