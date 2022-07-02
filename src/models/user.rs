use chrono::serde::ts_seconds_option;
use chrono::DateTime;
use chrono::Utc;
use derive_builder::Builder;
use serde::Deserialize;
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Builder, Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq, FromRow)]
#[builder(setter(into, strip_option), default)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub email: Option<String>,
    #[serde(with = "ts_seconds_option")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(with = "ts_seconds_option")]
    pub last_login: Option<DateTime<Utc>>,
}

#[derive(Builder, Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq, Validate)]
#[builder(setter(into, strip_option), default)]
pub struct UserCreateReqDto {
    #[validate(length(min = 3, max = 30))]
    pub username: String,
    #[validate(length(min = 8, max = 30))]
    pub password_raw: String,
    #[validate(email)]
    pub email: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq)]
pub struct UserGetRespDto {
    pub id: Uuid,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(with = "ts_seconds_option", skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(with = "ts_seconds_option", skip_serializing_if = "Option::is_none")]
    pub last_login: Option<DateTime<Utc>>,
}

impl From<User> for UserGetRespDto {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            created_at: user.created_at,
            last_login: user.last_login,
        }
    }
}
