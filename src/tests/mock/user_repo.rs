use std::collections::HashMap;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::models::user::User;
use crate::repositories::user::UserRepo;

pub struct MockUserRepo(pub Mutex<HashMap<Uuid, User>>);

impl Default for MockUserRepo {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl From<HashMap<Uuid, User>> for MockUserRepo {
    fn from(map: HashMap<Uuid, User>) -> Self {
        Self(Mutex::new(map))
    }
}

#[async_trait]
impl UserRepo for MockUserRepo {
    async fn create_user(&self, user: &User) -> Result<()> {
        if self.0.lock().await.insert(user.id, user.clone()).is_none() {
            Ok(())
        } else {
            Err(anyhow!("User creation failed!"))
        }
    }

    async fn get_user_by_id(&self, user_id: &Uuid) -> Result<User> {
        self.0
            .lock()
            .await
            .get(user_id)
            .cloned()
            .context("No user with given ID")
    }

    async fn update_user_by_id(&self, user_id: &Uuid, new_user: &User) -> Result<()> {
        *self
            .0
            .lock()
            .await
            .get_mut(user_id)
            .context("No user with given ID")? = new_user.clone();
        Ok(())
    }

    async fn delete_user_by_id(&self, user_id: &Uuid) -> Result<()> {
        self.0
            .lock()
            .await
            .remove(user_id)
            .map(|_| ())
            .context("Failed to delete user with given ID")
    }

    async fn contains_user_with_username(&self, username: &str) -> Result<bool> {
        Ok(self
            .0
            .lock()
            .await
            .values()
            .map(|user| &user.username)
            .any(|other_username| other_username == username))
    }

    async fn get_password_by_id(&self, user_id: &Uuid) -> Result<String> {
        self.0
            .lock()
            .await
            .get(user_id)
            .map(|user| user.password_hash.clone())
            .context("No user with given ID")
    }
}
