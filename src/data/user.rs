use std::borrow::Cow;

use argon2::Argon2;
use base64::{prelude::BASE64_STANDARD, Engine};
use serde::{ser, Deserialize, Serialize};
use sqlx::{query, query_as, types::time::PrimitiveDateTime, Postgres, Transaction};
use time::OffsetDateTime;

use super::repository::Repository;

enum DecodeError {
    Sqlx(sqlx::Error),
    Serde(rmp_serde::decode::Error),
}

#[derive(Clone, Copy, Debug)]
pub struct UserID(pub i32);

impl ser::Serialize for UserID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_i32(self.0)
    }
}

impl<'de> serde::Deserialize<'de> for UserID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let id = i32::deserialize(deserializer)?;
        Ok(UserID(id))
    }
}

impl From<i32> for UserID {
    fn from(value: i32) -> Self {
        UserID(value)
    }
}

impl From<UserID> for i32 {
    fn from(value: UserID) -> Self {
        value.0
    }
}

#[derive(Deserialize, Serialize)]
pub struct UserSession {
    pub user_id: UserID,
}

#[derive(sqlx::FromRow)]
pub struct User {
    pub user_id: UserID,
    pub user_iid: Cow<'static, str>,
    pub username: String,
    pub password_hash: String,
    pub updated_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
}
pub type UserHashBuf = [u8; 32];

#[derive(sqlx::FromRow)]
pub struct UserSetting {
    pub user_id: UserID,
    pub language: Cow<'static, str>,
    pub is_dark_mode: bool,
    pub updated_at: Option<OffsetDateTime>,
    pub created_at: Option<OffsetDateTime>,
}

impl User {
    pub fn check_password(
        &self,
        salt: &str,
        password: &str,
    ) -> Result<bool, crate::error::UserAuth> {
        let argon2 = Argon2::default();
        let mut test_buf: [u8; 32] = [0; 32];
        let user_hashed_password = BASE64_STANDARD
            .decode::<&[u8]>(self.password_hash.as_ref())
            .unwrap();
        argon2.hash_password_into(password.as_bytes(), salt.as_bytes(), &mut test_buf)?;
        Ok(test_buf.eq(user_hashed_password.as_slice()))
    }
}

impl<'a> Repository<'a, Postgres> {
    pub async fn get_user_by_id<'b>(
        &self,
        tx: &mut Transaction<'b, Postgres>,
        id: i32,
    ) -> Result<User, sqlx::Error> {
        Ok(
            sqlx::query_as!(User, "SELECT * FROM users WHERE user_id = $1", id)
                .fetch_one(&mut **tx)
                .await?,
        )
    }

    pub async fn update_user_settings<'b>(
        &self,
        tx: &mut Transaction<'b, Postgres>,
        user_id: &UserID,
        setting: &UserSetting,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"INSERT INTO user_settings (user_id, language, is_dark_mode) 
            VALUES ($1, $2, $3) 
            ON CONFLICT (user_id) DO UPDATE SET 
                language = EXCLUDED.language,
                is_dark_mode = EXCLUDED.is_dark_mode"#,
            user_id.0,
            &setting.language,
            setting.is_dark_mode
        )
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    pub async fn get_user_settings<'b>(
        &self,
        tx: &mut Transaction<'b, Postgres>,
        user_id: UserID,
    ) -> Result<UserSetting, sqlx::Error> {
        match sqlx::query_as!(
            UserSetting,
            "SELECT * FROM user_settings WHERE user_id = $1",
            user_id.0
        )
        .fetch_one(&mut **tx)
        .await
        {
            Ok(settings) => Ok(settings),
            Err(sqlx::Error::RowNotFound) => {
                // If no settings found, return default settings
                Ok(UserSetting {
                    user_id,
                    language: Cow::Borrowed(crate::tr::DEFAULT_IL8N.to_iso_639_3()),
                    is_dark_mode: false,
                    updated_at: None,
                    created_at: None,
                })
            }
            Err(e) => Err(e),
        }
    }

    pub async fn get_user_by_username<'b>(
        &self,
        tx: &mut Transaction<'b, Postgres>,
        username: &str,
    ) -> Result<User, sqlx::Error> {
        sqlx::query_as!(User, "SELECT * FROM users WHERE username = $1", username)
            .fetch_one(&mut **tx)
            .await
    }

    pub async fn update_password<'b>(
        &self,
        tx: &mut Transaction<'b, Postgres>,
        id: i32,
        salt: &str,
        password: &str,
    ) -> Result<(), crate::error::UserAuth> {
        let argon2 = Argon2::default();
        let mut hashed_password: [u8; 32] = [0; 32];
        argon2.hash_password_into(password.as_bytes(), salt.as_bytes(), &mut hashed_password)?;

        let encode_str = BASE64_STANDARD.encode(hashed_password.as_slice());
        query!(
            r#"UPDATE users SET password_hash = $1 WHERE user_id = $2"#,
            encode_str.as_str(),
            id
        )
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    pub async fn create_user<'b>(
        &self,
        tx: &mut Transaction<'b, Postgres>,
        salt: &str,
        name: &str,
        password: &str,
    ) -> Result<User, crate::error::UserAuth> {
        let argon2 = Argon2::default();
        let mut hashed_password: [u8; 32] = [0; 32];
        if let Err(error) =
            argon2.hash_password_into(password.as_bytes(), salt.as_bytes(), &mut hashed_password)
        {
            return Err(crate::error::UserAuth::Authentication(error));
        }
        let encode_str = BASE64_STANDARD.encode(hashed_password.as_slice());
        Ok(query_as!(
            User,
            r#"INSERT INTO users(username, password_hash) VALUES ($1, $2) RETURNING *"#,
            name,
            encode_str.as_str()
        )
        .fetch_one(&mut **tx)
        .await?)
    }
}

