use macro_common::config_file;
use serde::Deserialize;

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
    pub fn con_bonus(&self, con_lvl: u8) -> f64 {
        let con_stat = &self.con[con_lvl as usize];
        assert_eq!(con_stat.value, con_lvl);
        con_stat.bonus
    }
}
