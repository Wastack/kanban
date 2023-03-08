use crate::model::issue::Issue;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Board {
    pub issues: Vec<Issue>,
}

