mod variations;
pub mod creature;
pub mod stat_enum;
pub mod calculator;
pub mod formulas;
#[cfg(test)]
mod tests;

pub use variations::*;
pub use stat_enum::Stat;
pub use calculator::{StatCalculator, Modifier};
pub use formulas::Formulas;