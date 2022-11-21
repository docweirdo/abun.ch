use rocket::launch;
use rocket_db_pools::{sqlx, Database};
use crate::cors::Cors;

mod api;
mod db;
mod error;
mod bunch_url;
mod model;
mod cors;

#[derive(Database)]
#[database("abunch_db")]
pub struct AbunchDB(sqlx::PgPool);

#[launch]
fn rocket() -> _ {
    let mut rocket = rocket::build().attach(AbunchDB::init()).attach(Cors);
    rocket = api::mount_endpoints(rocket);
    rocket
}
