use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, FromForm, Debug, Clone)]
pub struct UserName {
    pub name: String,
    pub age: i32,
}
