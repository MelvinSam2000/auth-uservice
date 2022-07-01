use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

use crate::models::user::User;

#[async_trait]
pub trait UserRepo: Send + Sync + 'static {
    async fn create_user(&self, user: &User) -> Result<Uuid>;
    async fn get_user_by_id(&self, user_id: &Uuid) -> Result<User>;
    async fn update_user_by_id(&self, user_id: &Uuid, new_user: &User) -> Result<()>;
    async fn delete_user_by_id(&self, user_id: &Uuid) -> Result<()>;
    async fn contains_user_with_username(&self, username: &str) -> Result<bool>;
}
