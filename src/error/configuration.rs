use std::borrow::Cow;

#[derive(Debug)]
pub enum Configuration {
    MissingField { path: Cow<'static, str> },
}

impl std::fmt::Display for Configuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Configuration::MissingField { path } => write!(f, "Field Missing: {}", path),
        }
    }
}

impl std::error::Error for Configuration {}

