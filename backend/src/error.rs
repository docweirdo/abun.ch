use rocket::http::Status;
use rocket::request::Request;
use rocket::response;
use rocket::response::Responder;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AbunchError {
    #[error(transparent)]
    DatabaseError(#[from] sqlx::Error),

    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),

    #[error("status code: {0}")]
    #[allow(dead_code)]
    StatusCode(u16),

    #[error("wrong password for user {0}")]
    WrongPassword(String),
}

impl<'r, 'o: 'r> Responder<'r, 'o> for AbunchError {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'o> {
        match self {
            AbunchError::DatabaseError(_) => Status::InternalServerError.respond_to(req),
            AbunchError::SerdeError(_) => Status::InternalServerError.respond_to(req),
            AbunchError::StatusCode(code) => Status::from_code(code).respond_to(req),
            AbunchError::WrongPassword(_) => Status::Unauthorized.respond_to(req)
        }
    }
}
