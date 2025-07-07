use macro_common::config_file;
use serde::Deserialize;
use crate as l2_core;

#[derive(Debug, Clone, Deserialize)]
#[config_file(path = "config/data/action_list.yaml")]
pub struct ActionList {
    pub actions: Vec<Action>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Action {
    pub id: u32,
    pub handler: String,
    pub option: Option<u32>,
}
