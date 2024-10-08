use crate::models::user_model::User;
use crate::repositories::error::DbRepoError;
use crate::repositories::user_repo::{UserRepo, UserRepoImpl};
use sqlx::postgres::PgConnection;

pub async fn create_user(db_con: &mut PgConnection) -> Result<User, DbRepoError> {
    let user_repo = UserRepoImpl::new();
    let name = "fer".to_string();
    let age = 31;
    user_repo.create(&mut *db_con, &name, age).await
}
