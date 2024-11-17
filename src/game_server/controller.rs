use std::sync::Arc;
use crate::game_server::dto::config::GSServer;

#[derive(Clone, Debug)]
pub struct Controller {
    cfg: Arc<GSServer>,
}

impl Controller {
    pub fn new(cfg: Arc<GSServer>) -> Self {
        Controller {
            cfg
        }
    }
}