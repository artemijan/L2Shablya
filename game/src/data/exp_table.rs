use macro_common::config_file;
use serde::{de, Deserialize, Deserializer};
use std::collections::HashMap;
use tracing::info;

#[derive(Debug, Clone)]
#[config_file(path="config/data/stats/exp_table.yaml")]
pub struct ExpTable {
    pub max_level: u8,
    exp_data: HashMap<u8, i64>,
    training_data: HashMap<u8, f64>,
}

#[derive(Debug, Clone, Deserialize)]
struct LevelData {
    level: u8,
    exp: i64,
    training_rate: f64,
}

impl<'de> Deserialize<'de> for ExpTable {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawExpTable {
            max_level: u8,
            data: Vec<LevelData>,
        }

        let raw: RawExpTable = RawExpTable::deserialize(deserializer)?;

        let mut exp_data = HashMap::new();
        let mut training_data = HashMap::new();

        for entry in raw.data {
            exp_data.insert(entry.level, entry.exp);
            training_data.insert(entry.level, entry.training_rate);
        }

        // Validation: Ensure max_level is valid
        if raw.max_level != *exp_data.keys().max().unwrap_or(&0) {
            return Err(de::Error::custom(format!(
                "max_level ({}) does not match the highest level ({}) in the data",
                raw.max_level,
                exp_data.keys().max().unwrap_or(&0)
            )));
        }

        let exp_table = ExpTable {
            max_level: raw.max_level,
            exp_data,
            training_data,
        };

        // Log a message
        info!(
            "ExpTable loaded with max_level: {} and {} levels of data",
            exp_table.max_level,
            exp_table.exp_data.len()
        );

        Ok(exp_table)
    }
}

impl ExpTable {
    #[must_use]
    pub fn get_exp(&self, level: u8) -> i64 {
        if level > self.max_level {
            return *self.exp_data.get(&self.max_level).unwrap_or(&0);
        }
        *self.exp_data.get(&level).unwrap_or(&0)
    }
    #[must_use]
    pub fn get_exp_for_next_lvl(&self, level: u8) -> i64 {
        if level == u8::MAX {
            return self.get_exp(level);
        }
        self.get_exp(level + 1)
    }
    #[must_use]
    pub fn get_training_exp(&self, level: u8) -> f64 {
        if level > self.max_level {
            return *self
                .training_data
                .get(&self.max_level)
                .unwrap_or(&f64::from(0));
        }
        *self.training_data.get(&level).unwrap_or(&f64::from(0))
    }
}
