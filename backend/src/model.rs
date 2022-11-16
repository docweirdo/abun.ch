
use serde::{Serialize, Deserialize};
use time::Date;

#[derive(Serialize, Deserialize)]
pub struct Bunch {
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

#[derive(Serialize, Deserialize)]
pub struct NewBunch{
    pub title: String,
    pub description: Option<String>,
    pub expiration: Option<Date>,
    pub password: Option<String>,
    pub open_graph: bool,
    pub incognito: bool,
    pub entries: Vec<NewEntry>
}


#[derive(Serialize, Deserialize)]
pub struct NewEntry{
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>
}