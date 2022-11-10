use rocket::launch;
use rocket_db_pools::{sqlx, Database};

mod api;
mod db;
mod error;

#[derive(Database)]
#[database("abunch_db")]
pub struct AbunchDB(sqlx::PgPool);

#[launch]
fn rocket() -> _ {
    let mut rocket = rocket::build().attach(AbunchDB::init());
    rocket = api::mount_endpoints(rocket);
    rocket
}
