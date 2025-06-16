#[derive(Debug)]
pub enum UserAuth {
    Authentication(argon2::Error),
    Sql(sqlx::Error),
}

impl std::fmt::Display for UserAuth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserAuth::Authentication(e) => write!(f, "{}", e),
            UserAuth::Sql(e) => write!(f, "{}", e),
        }
    }
}

impl core::error::Error for UserAuth {}

impl From<argon2::Error> for UserAuth {
    fn from(error: argon2::Error) -> Self {
        UserAuth::Authentication(error)
    }
}

impl From<sqlx::Error> for UserAuth {
    fn from(error: sqlx::Error) -> Self {
        UserAuth::Sql(error)
    }
}

