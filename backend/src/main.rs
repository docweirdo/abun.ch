use rocket::launch;
use rocket_db_pools::{sqlx, Database};
use crate::cors::Cors;

mod api;
mod db;
mod error;
mod identifier;
mod model;
mod cors;

#[derive(Database)]
#[database("abunch_db")]
pub struct AbunchDB(sqlx::PgPool);

#[launch]
fn rocket() -> _ {
    let mut rocket = rocket::build();
    let figment = rocket.figment();

    let cors_config: cors::Config = figment.extract_inner("cors").expect("custom");

    rocket = rocket.attach(AbunchDB::init()).attach(Cors(cors_config));

    rocket = api::mount_endpoints(rocket);
    rocket
}
