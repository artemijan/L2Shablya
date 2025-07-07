use anyhow::anyhow;
use macro_common::config_file;
use serde::Deserialize;
use crate as l2_core;

#[allow(unused)]
#[derive(Clone, Debug)]
pub enum CreatureStat {
    Con,
    Dex,
    Wit,
    Men,
    Str,
    Int,
}

#[derive(Clone, Debug)]
pub enum CreatureParameter {
    HP,
    MP,
    CP,
}

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
pub struct StatData {
    value: u8,
    bonus: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
#[config_file(path = "config/data/stats/stat_bonus.yaml", msg = "Base stats loaded")]
pub struct BaseStat {
    pub con: Vec<StatData>,
    pub dex: Vec<StatData>,
    pub wit: Vec<StatData>,
    pub men: Vec<StatData>,
    pub str: Vec<StatData>,
    pub int: Vec<StatData>,
}

impl BaseStat {
    /// # Errors
    /// - when con lvl is too big
    pub fn con_bonus(&self, con_lvl: u8) -> anyhow::Result<f64> {
        let con_stat = &self
            .con
            .get(con_lvl as usize)
            .ok_or(anyhow!("Con lvl is out of static data range {con_lvl}"))?;
        Ok(con_stat.bonus)
    }
}
