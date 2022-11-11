use pwhash::bcrypt;
use rocket_db_pools::Connection;
use sqlx::{query, query_as};

use crate::{error::AbunchError, bunch_url::BunchURL, model::{Bunch, Entry}};
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

pub async fn get_bunch_password_by_url(bunch_url: BunchURL, mut conn: Connection<AbunchDB>) -> Result<Option<String>, AbunchError>{
    let bunch = query!("SELECT password FROM bunch WHERE uri = $1", bunch_url.to_string())
        .fetch_one(&mut *conn)
        .await?;

    Ok(bunch.password)
}

pub async fn get_bunch_by_url(bunch_url: BunchURL, mut conn: Connection<AbunchDB>) -> Result<Bunch, AbunchError>{
    let bunch = query!("SELECT id, title, description, date, open_graph, incognito, creator_id FROM bunch WHERE uri = $1", bunch_url.to_string())
        .fetch_one(&mut *conn)
        .await?;

    let username = if !bunch.incognito{
        let record = query!("SELECT username FROM creator WHERE id = $1", bunch.creator_id).fetch_one(&mut *conn).await?;
        Some(record.username)
    } else {
        None
    };

    let entries = query_as!(Entry, "SELECT id, title, description, url FROM entry WHERE bunch_id = $1", bunch.id).fetch_all(&mut *conn).await?;

    let bunch_nested = Bunch{
        id: bunch.id,
        title: bunch.title,
        description: bunch.description,
        date: bunch.date,
        open_graph: bunch.open_graph,
        username,
        entries
    };

    Ok(bunch_nested)
}

pub async fn clicked_url(bunch_url: BunchURL, entry_id: i32, mut conn: Connection<AbunchDB>) -> Result<(), AbunchError>{
    let bunch = query!("SELECT id FROM bunch WHERE uri = $1", bunch_url.to_string()).fetch_one(&mut *conn).await?;

    let res = query!("UPDATE entry SET clickcounter = clickcounter + 1 WHERE id = $1 AND bunch_id = $2;", entry_id, bunch.id).execute(&mut *conn).await?;

    if res.rows_affected() == 0{
        Err(AbunchError::StatusCode(403))
    } else {
        Ok(())
    }
}