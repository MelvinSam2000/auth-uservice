use derive_builder::Builder;
use serde::Deserialize;
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Builder, Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq, FromRow)]
#[builder(setter(into, strip_option), default)]
pub struct User {
    pub id: Option<Uuid>,
    pub username: String,
    pub password_hash: String,
    pub password_salt: String,
    pub email: Option<String>,
    pub created_at: Option<String>,
    pub last_login: Option<String>,
}

#[derive(Builder, Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq)]
#[builder(setter(into, strip_option), default)]
pub struct UserCreateReqDto {
    pub username: String,
    pub password_raw: String,
    pub email: Option<String>,
    pub created_at: Option<String>,
    pub last_login: Option<String>,
}

impl From<UserCreateReqDto> for User {
    fn from(other: UserCreateReqDto) -> Self {
        Self {
            id: None,
            username: other.username,
            password_hash: other.password_raw.clone(),
            password_salt: other.password_raw,
            email: other.email,
            created_at: other.created_at,
            last_login: other.last_login,
        }
    }
}
