use crate::game_objects::creature::buff::BuffInfo;
use dashmap::DashSet;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, RwLock};
use crate::game_objects::player::Player;

#[derive(Debug, Clone)]
pub struct EffectList {
    actives: Arc<Mutex<VecDeque<BuffInfo>>>,
    passive: DashSet<BuffInfo>,
    options: DashSet<BuffInfo>,
    owner: Arc<RwLock<Player>>,
}
