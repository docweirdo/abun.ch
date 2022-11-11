
use serde::{Serialize, Deserialize};
use time::Date;

#[derive(Serialize, Deserialize)]
pub struct Bunch {
    pub id : i32,
    pub title : String,
    pub description : Option<String>,
    pub date : Date,
    pub open_graph : bool,
    pub username : Option<String>,
    pub entries : Vec<Entry>
}

#[derive(Serialize, Deserialize)]
pub struct Entry{
    pub id : i32,
    pub title : Option<String>,
    pub url : String,
    pub description : Option<String>
}