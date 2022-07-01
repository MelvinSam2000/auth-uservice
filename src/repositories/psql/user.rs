#![allow(unused_variables)]

use anyhow::anyhow;
use anyhow::Result;
use async_trait::async_trait;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::user::User;
use crate::repositories::user::UserRepo;

pub struct UserRepoDb(PgPool);

impl UserRepoDb {
    pub async fn init(db_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(db_url)
            .await?;
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id UUID PRIMARY KEY,
                username VARCHAR NOT NULL,
                password_hash VARCHAR NOT NULL,
                password_salt VARCHAR NOT NULL,
                email VARCHAR,
                created_at VARCHAR,
                last_login VARCHAR
            )"#,
        )
        .execute(&pool)
        .await?;
        Ok(Self(pool))
    }
}

#[async_trait]
impl UserRepo for UserRepoDb {
    async fn create_user(&self, user: &User) -> Result<Uuid> {
        let id = Uuid::new_v4();
        let mut user = user.clone();
        user.id = Some(id);
        sqlx::query(
            r#"
            INSERT INTO users 
            (id, username, password_hash, password_salt, email, created_at, last_login) 
            VALUES 
            ($1, $2, $3, $4, $5, $6, $7)"#,
        )
        .bind(&user.id)
        .bind(&user.username)
        .bind(&user.password_hash)
        .bind(&user.password_salt)
        .bind(&user.email)
        .bind(&user.created_at)
        .bind(&user.last_login)
        .execute(&self.0)
        .await?;
        Ok(id)
    }

    async fn get_user_by_id(&self, user_id: &Uuid) -> Result<User> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&self.0)
            .await?;
        Ok(user)
    }

    async fn update_user_by_id(&self, user_id: &Uuid, new_user: &User) -> Result<()> {
        Err(anyhow!("todo!"))
    }

    async fn delete_user_by_id(&self, user_id: &Uuid) -> Result<()> {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user_id)
            .execute(&self.0)
            .await?;
        Ok(())
    }

    async fn contains_user_with_username(&self, username: &str) -> Result<bool> {
        let user = sqlx::query("SELECT (id) FROM users WHERE username = $1")
            .bind(username)
            .fetch_optional(&self.0)
            .await?;
        Ok(user.is_some())
    }
}
