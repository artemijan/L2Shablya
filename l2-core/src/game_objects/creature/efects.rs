use crate::game_objects::creature::buff::BuffInfo;
use crate::game_objects::player::Player;
use dashmap::DashSet;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, RwLock};

#[derive(Debug, Clone)]
pub struct EffectList {
    actives: Arc<Mutex<VecDeque<BuffInfo>>>,
    passive: DashSet<BuffInfo>,
    options: DashSet<BuffInfo>,
    owner: Arc<RwLock<Player>>,
}
