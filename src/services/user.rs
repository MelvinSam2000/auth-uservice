use actix_web::web::Data;
use actix_web::web::Json;
use actix_web::web::Path;
use chrono::Utc;
use uuid::Uuid;
use validator::Validate;

use crate::crypto::PasswordHasher;
use crate::errors::user::log_err;
use crate::errors::user::UserServiceError;
use crate::errors::user::UserServiceResult;
use crate::models::user::UserBuilder;
use crate::models::user::UserCreateReqDto;
use crate::models::user::UserGetRespDto;
use crate::repositories::user::UserRepo;

pub async fn get_user_by_id<T>(
    user_repo: Data<T>,
    user_id: Path<String>,
) -> UserServiceResult<UserGetRespDto>
where
    T: UserRepo,
{
    let user_id_str = user_id.into_inner();
    let user_id = Uuid::try_parse(&user_id_str)
        .map_err(|_| UserServiceError::InvalidId(user_id_str.clone()))?;
    let user = user_repo
        .get_user_by_id(&user_id)
        .await
        .map_err(|_| UserServiceError::NoUserForId(user_id_str))?;
    let user = UserGetRespDto::from(user);
    Ok(Json(user))
}

pub async fn post_user<T>(
    user_repo: Data<T>,
    passwd_hasher: Data<PasswordHasher>,
    user: Json<UserCreateReqDto>,
) -> UserServiceResult<Uuid>
where
    T: UserRepo,
{
    user.0
        .validate()
        .map_err(UserServiceError::InvalidUserFields)?;

    let UserCreateReqDto {
        username,
        password_raw,
        email,
    } = user.0;

    if user_repo
        .contains_user_with_username(&username)
        .await
        .map_err(log_err)
        .map_err(|_| UserServiceError::UnknownInternal)?
    {
        return Err(UserServiceError::UsernameTaken);
    }

    let password_hash = passwd_hasher
        .hash_password(&password_raw)
        .map_err(log_err)
        .map_err(|_| UserServiceError::UnknownInternal)?;

    let user_id = Uuid::new_v4();
    let mut user = UserBuilder::default()
        .id(user_id)
        .username(username)
        .password_hash(password_hash)
        .created_at(Utc::now())
        .build()
        .map_err(log_err)
        .map_err(|_| UserServiceError::UnknownInternal)?;
    user.email = email;

    user_repo
        .create_user(&user)
        .await
        .map_err(log_err)
        .map_err(|_| UserServiceError::UnknownInternal)?;
    Ok(Json(user_id))
}

pub async fn delete_user<T>(user_repo: Data<T>, user_id: Path<String>) -> UserServiceResult<()>
where
    T: UserRepo,
{
    let user_id_str = user_id.into_inner();
    let user_id = Uuid::try_parse(&user_id_str)
        .map_err(|_| UserServiceError::InvalidId(user_id_str.clone()))?;
    user_repo
        .delete_user_by_id(&user_id)
        .await
        .map(Json)
        .map_err(|_| UserServiceError::NoUserForId(user_id_str))
}
