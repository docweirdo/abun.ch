use crate::db;
use crate::db::AbunchDB;
use pwhash::bcrypt;
use rocket::{
    http::{Cookie, CookieJar, Status},
    data::{self, Data, FromData, ToByteUnit},
    request::{self, Request, FromRequest, FromParam},
    post, routes, get,
    serde::json::Json,
    Build, Rocket,
    time::Duration
};
use rocket_db_pools::Connection;
use serde::{Deserialize, Serialize};


use crate::error::AbunchError;
use crate::bunch_url::BunchURL;
use crate::model::Bunch;
use crate::model::NewBunch;

const COOKIE_DURATION: i64 = 20 * 60; // 20 mins

pub fn mount_endpoints(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket.mount("/", routes![
        login, 
        bunch,
        clicked,
        new_bunch
        ])
}

#[get("/<bunch_url>")]
pub async fn bunch(_auth_header: AuthorizationGuard, bunch_url: BunchURL, conn: Connection<AbunchDB>) -> Result<Json<Bunch>, AbunchError>{
    
    let bunch: Bunch = db::get_bunch_by_url(bunch_url, conn).await?;

    Ok(Json(bunch))
}

#[post("/<bunch_url>/clicked/<entry_id>")]
pub async fn clicked(_auth_header: AuthorizationGuard, bunch_url: BunchURL, entry_id: i32, conn: Connection<AbunchDB>) -> Result<(), AbunchError>{
    db::clicked_url(bunch_url, entry_id, conn).await
}

#[post("/new", data = "<new_bunch>")]
pub async fn new_bunch(creator: CreatorGuard, new_bunch: NewBunch, conn: Connection<AbunchDB>) -> Result<Json<String>, AbunchError>{
    let uri = db::new_bunch(new_bunch, creator.0, conn).await?;
    Ok(Json(uri.to_string()))
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
            .max_age(Duration::seconds(COOKIE_DURATION-1))
            .path("/")
            .secure(!cfg!(debug_assertions))
            .finish(),
    );
    cookie_jar.add(Cookie::build("logged_in", "true").path("/").max_age(Duration::seconds(COOKIE_DURATION-1)).finish());

    Ok(())
}

pub struct CreatorGuard(i32);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for CreatorGuard{
    type Error = AbunchError;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let cookie_jar: &CookieJar<'r> = req.cookies();

        let Some(cookie) = cookie_jar.get_private("logged_in_info") else {
            return request::Outcome::Failure((Status::Unauthorized, AbunchError::StatusCode(401)));
        };

        let Ok(claims) = serde_json::from_str::<JWTClaims>(cookie.value()) else {
            return request::Outcome::Failure((Status::Unauthorized, AbunchError::StatusCode(401)));
        };

        if time::OffsetDateTime::from_unix_timestamp(claims.exp) > time::OffsetDateTime::now_utc(){
            return request::Outcome::Success(Self(claims.id));
        } else {
            return request::Outcome::Failure((Status::Unauthorized, AbunchError::StatusCode(401)));
        }
        
    }
}

pub struct AuthorizationGuard;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthorizationGuard{
    type Error = AbunchError;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {

        let Some(Ok(param)) = req.param(0) else {
            return request::Outcome::Failure((Status::NotFound, AbunchError::StatusCode(404)));
        };

        let bunch_url = match BunchURL::from_param(param){
            Ok(bunch_url) => bunch_url,
            Err(e) => return request::Outcome::Failure((Status::NotFound, e)),
        };

        let request::Outcome::Success(conn) = req.guard::<Connection<AbunchDB>>().await else{
            return request::Outcome::Failure((Status::InternalServerError, AbunchError::StatusCode(500)));
        };
        
        let pw_hash = match db::get_bunch_password_by_url(bunch_url, conn).await{
            Ok(Some(pw_hash)) => pw_hash,
            Err(AbunchError::DatabaseError(sqlx::Error::RowNotFound)) => return request::Outcome::Failure((Status::NotFound, AbunchError::DatabaseError(sqlx::Error::RowNotFound))),
            Err(e) => return request::Outcome::Failure((Status::InternalServerError, e)),
            Ok(None) => return request::Outcome::Success(AuthorizationGuard)
        };
        
        let Some(password) = req.headers().get_one("Authorization") else {
            return request::Outcome::Failure((Status::Unauthorized, AbunchError::WrongPassword(param.to_owned())))
        };
        
        
        if bcrypt::verify(password, &pw_hash){
            return request::Outcome::Success(AuthorizationGuard);
        } else{
            return request::Outcome::Failure((Status::Unauthorized, AbunchError::WrongPassword(param.to_owned())))
        }
        
    }
}


#[rocket::async_trait]
impl<'r> FromData<'r> for NewBunch {
    type Error = AbunchError;

    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> data::Outcome<'r, Self> {

        let limit = req.limits().get("new_bunch").unwrap_or_else(|| 2048_i32.bytes());

        let string = match data.open(limit).into_string().await {
            Ok(string) if string.is_complete() => string.into_inner(),
            Ok(_) => return data::Outcome::Failure((Status::PayloadTooLarge, AbunchError::StatusCode(413))),
            Err(_) => return data::Outcome::Failure((Status::InternalServerError, AbunchError::StatusCode(500))),
        };

        let new_bunch = match serde_json::from_str::<NewBunch>(&string){
            Ok(n) => n,
            Err(e) => return data::Outcome::Failure((Status::BadRequest, AbunchError::SerdeError(e)))
        };

        if let Some(exp) = new_bunch.expiration{
            if time::OffsetDateTime::now_utc().date() >= exp {
                return data::Outcome::Failure((Status::BadRequest, AbunchError::StatusCode(400)))
            }
        }

        data::Outcome::Success(new_bunch)

    }
}
