use rocket::http::Header;
use rocket::{Request, Response};
use rocket::fairing::{Fairing, Info, Kind};

use serde::Deserialize;

pub struct Cors(pub Config);

#[derive(Deserialize, Debug)]
pub struct Config{
    origin: Vec<String>
}

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Methods", "POST, GET, OPTIONS"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
        response.set_header(Header::new("Access-Control-Allow-Headers", "Authorization"));

        let headers = _request.headers();

        let Some(origin) = headers.get_one("origin") else {
            return;
        };

        if self.0.origin.iter().any(|o| o == origin){
            response.set_header(Header::new("Access-Control-Allow-Origin", origin));
        }
    }
}