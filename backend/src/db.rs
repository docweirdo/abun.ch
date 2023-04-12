use pwhash::bcrypt;
use rocket_db_pools::Connection;
use sqlx::{query, query_as, pool::PoolConnection, Postgres};

use crate::{error::AbunchError, identifier::{BunchURL, AccountToken}, model::{Bunch, Entry, NewBunch, NewAccount}};
pub use crate::AbunchDB;

pub async fn verify_user(
    mut conn: Connection<AbunchDB>,
    username: &str,
    password: &str,
) -> Result<i32, AbunchError> {
    let row= query!("SELECT id, password FROM creator WHERE LOWER(username) = LOWER($1)", username)
        .fetch_one(&mut *conn)
        .await?;

    if bcrypt::verify(password, &row.password) {
        Ok(row.id)
    } else {
        Err(AbunchError::WrongPassword(username.to_owned()))
    }
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
        title: bunch.title,
        description: bunch.description,
        date: bunch.date,
        open_graph: bunch.open_graph,
        username,
        entries
    };

    query!("UPDATE bunch SET clickcounter = clickcounter + 1 WHERE id = $1", bunch.id).execute(&mut *conn).await?;

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

pub async fn new_bunch(new_bunch: NewBunch, creator_id : i32, mut conn: Connection<AbunchDB>) -> Result<BunchURL, AbunchError>{
    
    let mut uri = BunchURL::new();

    loop {
        let result = query!("SELECT COUNT(1) FROM bunch WHERE uri = $1", uri.to_string()).fetch_one(&mut *conn).await?;

        if result.count == Some(0) {break;}

        uri = BunchURL::new();
    }
    
    let bunch = query!("
        INSERT INTO bunch(title, description, date, expiration, clickcounter, uri, password, open_graph, incognito, creator_id) 
        VALUES($1, $2, CURRENT_DATE, $3, 0, $4, $5, $6, $7, $8) RETURNING id;",
        new_bunch.title,
        new_bunch.description,
        new_bunch.expiration,
        uri.to_string(),
        new_bunch.password,
        new_bunch.open_graph,
        new_bunch.incognito,
        creator_id
    ).fetch_one(&mut *conn).await?;

    for entry in new_bunch.entries {

        query!("
            INSERT INTO entry(title, description, url, clickcounter, bunch_id) 
            VALUES($1, $2, $3, 0, $4);",
            entry.title,
            entry.description,
            entry.url,
            bunch.id
        ).execute(&mut *conn).await?;
    }
    

    Ok(uri)
}

pub async fn get_token_validity(token: AccountToken, mut conn: Connection<AbunchDB>) -> Result<(bool, bool), AbunchError>{

    let token_record = query!("SELECT valid, admin FROM token WHERE id = $1;", token.to_string()).fetch_one(&mut *conn).await?;

    Ok((token_record.valid, token_record.admin))
}

pub async fn invalidate_token(token: &str, conn: &mut PoolConnection<Postgres>) -> Result<(), AbunchError>{

    query!("UPDATE token SET valid = FALSE WHERE id = $1;", token).execute(conn).await?;

    Ok(())
}

pub async fn new_account(new_account: NewAccount, mut conn: Connection<AbunchDB>) -> Result<i32, AbunchError>{

    let pw_hash = bcrypt::hash(new_account.password)?;

    let result = query!("
                INSERT INTO creator(username, password, admin) VALUES ($1, $2, $3) RETURNING ID;", 
                new_account.username, pw_hash, new_account.admin.unwrap())
                .fetch_one(&mut *conn).await?;

    Ok(result.id)
}