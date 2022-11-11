use crate::db;
use crate::db::AbunchDB;
use rocket::{
    http::{Cookie, CookieJar},
    post, routes, get,
    serde::json::Json,
    Build, Rocket,
};
use rocket_db_pools::Connection;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

use crate::error::AbunchError;
use crate::bunch_url::BunchURL;
use crate::model::Bunch;

const COOKIE_DURATION: u64 = 20 * 60; // 20 mins

pub fn mount_endpoints(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket.mount("/", routes![login, set_password, bunch])
}

#[get("/<bunch_url>")]
pub async fn bunch(bunch_url: BunchURL, conn: Connection<AbunchDB>) -> Result<Json<Bunch>, AbunchError>{
    let bunch: Bunch = db::get_bunch_by_url(bunch_url, conn).await?;

    Ok(Json(bunch))
}

#[derive(Serialize, Deserialize)]
pub struct Credentials {
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct JWTClaims {
    exp: u64,
    iat: u64,
    id: i32,
}

#[get("/test/<password>")]
pub async fn set_password(password: String, conn: Connection<AbunchDB>) -> Result<(), AbunchError> {
    db::set_password(conn, 1, &password)
        .await
        .map_err(|_| AbunchError::StatusCode(500))
}

#[post("/login", data = "<credentials>")]
pub async fn login(
    cookie_jar: &CookieJar<'_>,
    credentials: Json<Credentials>,
    conn: Connection<AbunchDB>,
) -> Result<(), AbunchError> {
    let username = credentials.username.trim();
    let password = credentials.password.trim();

    let id = db::verify_user(conn, username, password).await?;

    // Create JWT
    let now: u64 = SystemTime::UNIX_EPOCH.elapsed().unwrap().as_secs();
    let expiration_time: u64 = now + COOKIE_DURATION;
    let my_claims = JWTClaims {
        exp: expiration_time,
        iat: now,
        id,
    };

    cookie_jar.add_private(
        Cookie::build("logged_in", serde_json::to_string(&my_claims)?)
            .http_only(true)
            .permanent()
            .path("/")
            .secure(!cfg!(debug_assertions))
            .finish(),
    );

    Ok(())
}
