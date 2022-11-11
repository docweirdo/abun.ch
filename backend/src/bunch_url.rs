use nanoid::nanoid;
use nanoid::alphabet::SAFE;
use rocket::request::FromParam;
use std::fmt;

use crate::error::AbunchError;

#[derive(Debug)]
pub struct BunchURL(String);


impl BunchURL {
    pub fn new() -> Self{
        Self(nanoid!(6))
    }
}

impl TryFrom<String> for BunchURL{
    type Error = AbunchError;

    fn try_from(url: String) -> Result<Self, Self::Error>{
        if url.len() == 6 && url.chars().all(|c|{SAFE.contains(&c)}) {
            Ok(Self(url))
        } else{
            Err(AbunchError::BunchURL(url))
        }
    }
}

impl fmt::Display for BunchURL{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0,)
    }
}

impl<'r> FromParam<'r> for BunchURL{
    type Error = AbunchError;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        BunchURL::try_from(param.to_owned())
    }
    
}