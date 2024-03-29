use std::path::PathBuf;

use crate::db;
use crate::db::AbunchDB;
use pwhash::bcrypt;
use rocket::{
    data::{self, Data, FromData, ToByteUnit},
    get,
    http::{Cookie, CookieJar, Status},
    options, post,
    request::{self, FromParam, FromRequest, Request},
    routes,
    serde::json::Json,
    time::Duration,
    Build, Rocket,
};
use rocket_db_pools::Connection;
use serde::{Deserialize, Serialize};

use crate::error::AbunchError;
use crate::identifier::AccountToken;
use crate::identifier::BunchURL;
use crate::model::Bunch;
use crate::model::NewAccount;
use crate::model::NewBunch;

// TODO: Put into config
const COOKIE_DURATION: i64 = 20 * 60; // 20 mins
const COOKIE_DOMAIN: &str = "abun.ch";

pub fn mount_endpoints(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket.mount(
        "/",
        routes![
            login,
            logout,
            bunch,
            clicked,
            new_bunch,
            create_account,
            cors_preflight
        ],
    )
}

#[get("/<bunch_url>")]
pub async fn bunch(
    _auth_header: AuthorizationGuard,
    bunch_url: BunchURL,
    conn: Connection<AbunchDB>,
) -> Result<Json<Bunch>, AbunchError> {
    let bunch: Bunch = db::get_bunch_by_url(bunch_url, conn).await?;

    Ok(Json(bunch))
}

#[post("/<bunch_url>/clicked/<entry_id>")]
pub async fn clicked(
    _auth_header: AuthorizationGuard,
    bunch_url: BunchURL,
    entry_id: i32,
    conn: Connection<AbunchDB>,
) -> Result<(), AbunchError> {
    db::clicked_url(bunch_url, entry_id, conn).await
}

#[post("/new", data = "<new_bunch>")]
pub async fn new_bunch(
    creator: CreatorGuard,
    new_bunch: NewBunch,
    conn: Connection<AbunchDB>,
) -> Result<Json<String>, AbunchError> {
    let uri = db::new_bunch(new_bunch, creator.0, conn).await?;
    Ok(Json(uri.to_string()))
}

#[post("/create_account", data = "<new_account>")]
pub async fn create_account(
    new_account: NewAccount,
    mut conn: Connection<AbunchDB>,
) -> Result<(), AbunchError> {
    db::invalidate_token(&new_account.token, &mut conn).await?;

    db::new_account(new_account, conn).await?;

    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct Credentials {
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct JWTClaims {
    exp: i64,
    iat: i64,
    id: i32,
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
    let now: i64 = time::OffsetDateTime::now_utc().unix_timestamp();
    let expiration_time: i64 = now + COOKIE_DURATION;
    let my_claims = JWTClaims {
        exp: expiration_time,
        iat: now,
        id,
    };

    cookie_jar.add_private(
        Cookie::build("logged_in_info", serde_json::to_string(&my_claims)?)
            .http_only(true)
            .max_age(Duration::seconds(COOKIE_DURATION - 1))
            .path("/")
            .secure(!cfg!(debug_assertions))
            .domain(COOKIE_DOMAIN)
            .finish(),
    );
    cookie_jar.add(
        Cookie::build("logged_in", "true")
            .path("/")
            .max_age(Duration::seconds(COOKIE_DURATION - 1))
            .domain(COOKIE_DOMAIN)
            .finish(),
    );

    Ok(())
}

#[post("/logout")]
pub async fn logout(cookie_jar: &CookieJar<'_>) {
    cookie_jar.remove(Cookie::build("logged_in", "").domain(COOKIE_DOMAIN).finish());
    cookie_jar.remove_private(Cookie::build("logged_in_info", "").domain(COOKIE_DOMAIN).finish());
}

#[options("/<_path..>")]
pub fn cors_preflight(_path: PathBuf) -> Status {
    Status::Ok
}

pub struct CreatorGuard(i32);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for CreatorGuard {
    type Error = AbunchError;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let cookie_jar: &CookieJar<'r> = req.cookies();

        let Some(cookie) = cookie_jar.get_private("logged_in_info") else {
            return request::Outcome::Failure((Status::Unauthorized, AbunchError::StatusCode(401)));
        };

        let Ok(claims) = serde_json::from_str::<JWTClaims>(cookie.value()) else {
            return request::Outcome::Failure((Status::Unauthorized, AbunchError::StatusCode(401)));
        };

        if time::OffsetDateTime::from_unix_timestamp(claims.exp) > time::OffsetDateTime::now_utc() {
            return request::Outcome::Success(Self(claims.id));
        } else {
            return request::Outcome::Failure((Status::Unauthorized, AbunchError::StatusCode(401)));
        }
    }
}

pub struct AuthorizationGuard;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthorizationGuard {
    type Error = AbunchError;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let Some(Ok(param)) = req.param(0) else {
            return request::Outcome::Failure((Status::NotFound, AbunchError::StatusCode(404)));
        };

        let bunch_url = match BunchURL::from_param(param) {
            Ok(bunch_url) => bunch_url,
            Err(e) => return request::Outcome::Failure((Status::NotFound, e)),
        };

        let request::Outcome::Success(conn) = req.guard::<Connection<AbunchDB>>().await else{
            return request::Outcome::Failure((Status::InternalServerError, AbunchError::StatusCode(500)));
        };

        let pw_hash = match db::get_bunch_password_by_url(bunch_url, conn).await {
            Ok(Some(pw_hash)) => pw_hash,
            Err(AbunchError::DatabaseError(sqlx::Error::RowNotFound)) => {
                return request::Outcome::Failure((
                    Status::NotFound,
                    AbunchError::DatabaseError(sqlx::Error::RowNotFound),
                ))
            }
            Err(e) => return request::Outcome::Failure((Status::InternalServerError, e)),
            Ok(None) => return request::Outcome::Success(AuthorizationGuard),
        };

        let Some(password) = req.headers().get_one("Authorization") else {
            return request::Outcome::Failure((Status::Unauthorized, AbunchError::WrongPassword(param.to_owned())))
        };

        if bcrypt::verify(password, &pw_hash) {
            return request::Outcome::Success(AuthorizationGuard);
        } else {
            return request::Outcome::Failure((
                Status::Unauthorized,
                AbunchError::WrongPassword(param.to_owned()),
            ));
        }
    }
}

#[rocket::async_trait]
impl<'r> FromData<'r> for NewBunch {
    type Error = AbunchError;

    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> data::Outcome<'r, Self> {
        let limit = req
            .limits()
            .get("new_bunch")
            .unwrap_or_else(|| 2048_i32.bytes());

        let string = match data.open(limit).into_string().await {
            Ok(string) if string.is_complete() => string.into_inner(),
            Ok(_) => {
                return data::Outcome::Failure((
                    Status::PayloadTooLarge,
                    AbunchError::StatusCode(413),
                ))
            }
            Err(_) => {
                return data::Outcome::Failure((
                    Status::InternalServerError,
                    AbunchError::StatusCode(500),
                ))
            }
        };

        let new_bunch = match serde_json::from_str::<NewBunch>(&string) {
            Ok(n) => n,
            Err(e) => {
                return data::Outcome::Failure((Status::BadRequest, AbunchError::SerdeError(e)))
            }
        };

        if let Some(exp) = new_bunch.expiration {
            if time::OffsetDateTime::now_utc().date() >= exp {
                return data::Outcome::Failure((Status::BadRequest, AbunchError::StatusCode(400)));
            }
        }

        if let Some(ref password) = new_bunch.password {
            if password.chars().count() > 20 {
                return data::Outcome::Failure((Status::BadRequest, AbunchError::StatusCode(400)));
            }
        }

        if new_bunch.entries.is_empty() || new_bunch.entries.len() > 100 {
            return data::Outcome::Failure((Status::BadRequest, AbunchError::StatusCode(400)));
        }

        data::Outcome::Success(new_bunch)
    }
}

#[rocket::async_trait]
impl<'r> FromData<'r> for NewAccount {
    type Error = AbunchError;

    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> data::Outcome<'r, Self> {
        let request::Outcome::Success(conn) = req.guard::<Connection<AbunchDB>>().await else{
            return data::Outcome::Failure((Status::InternalServerError, AbunchError::StatusCode(500)));
        };

        let limit = req
            .limits()
            .get("new_account")
            .unwrap_or_else(|| 512_i32.bytes());

        let string = match data.open(limit).into_string().await {
            Ok(string) if string.is_complete() => string.into_inner(),
            Ok(_) => {
                return data::Outcome::Failure((
                    Status::PayloadTooLarge,
                    AbunchError::StatusCode(413),
                ))
            }
            Err(_) => {
                return data::Outcome::Failure((
                    Status::InternalServerError,
                    AbunchError::StatusCode(500),
                ))
            }
        };

        let mut new_account = match serde_json::from_str::<NewAccount>(&string) {
            Ok(n) => n,
            Err(e) => {
                return data::Outcome::Failure((Status::BadRequest, AbunchError::SerdeError(e)))
            }
        };

        if new_account.username.chars().count() > 15 {
            return data::Outcome::Failure((Status::BadRequest, AbunchError::StatusCode(400)));
        }

        if new_account.password.chars().count() > 20 {
            return data::Outcome::Failure((Status::BadRequest, AbunchError::StatusCode(400)));
        }

        let token = match AccountToken::try_from(new_account.token.to_owned()) {
            Ok(t) => t,
            Err(e) => return data::Outcome::Failure((Status::BadRequest, e)),
        };

        match db::get_token_validity(token, conn).await {
            Ok((true, admin)) => {
                new_account.admin = Some(admin);
                data::Outcome::Success(new_account)
            }
            Ok((false, _)) => data::Outcome::Failure((
                Status::Unauthorized,
                AbunchError::InvalidToken(new_account.token),
            )),
            Err(e) => data::Outcome::Failure((Status::InternalServerError, e)),
        }
    }
}
