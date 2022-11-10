use pwhash::bcrypt;
use rocket_db_pools::Connection;
use sqlx::query;

use crate::error::AbunchError;
pub use crate::AbunchDB;

pub async fn verify_user(
    mut conn: Connection<AbunchDB>,
    username: &str,
    password: &str,
) -> Result<i32, AbunchError> {
    let row= query!("SELECT id, password FROM creator WHERE username = $1", username)
        .fetch_one(&mut *conn)
        .await?;

    if bcrypt::verify(password, &row.password) {
        Ok(row.id)
    } else {
        Err(AbunchError::WrongPassword(username.to_owned()))
    }
}

pub async fn set_password(
    mut conn: Connection<AbunchDB>,
    user_id: i32,
    password: &str,
) -> Result<(), AbunchError> {
    let hash: String = bcrypt::hash(password).unwrap();

    query!("UPDATE creator SET password = $1 WHERE id = $2", hash, user_id).execute(&mut*conn).await?;

    Ok(())
}