
use serde::{Serialize, Deserialize};
use time::Date;

#[derive(Serialize, Deserialize)]
pub struct Bunch {
    pub id : i32,
    pub title : String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description : Option<String>,
    pub date : Date,
    pub open_graph : bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username : Option<String>,
    pub entries : Vec<Entry>
}

#[derive(Serialize, Deserialize)]
pub struct Entry{
    pub id : i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title : Option<String>,
    pub url : String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description : Option<String>
}