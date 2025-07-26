use std::ops::Add;

use chrono::{Local, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStatistics {
    pub dollars: f32,   // $
    pub watthours: f32, // Wh
    pub co2: f32,       // grams
    pub water: f32,     // mL
}

impl Default for UsageStatistics {
    fn default() -> Self {
        UsageStatistics {
            watthours: 0.,
            co2: 0.,
            water: 0.,
            dollars: 0.,
        }
    }
}
impl UsageStatistics {
    pub fn new(dollars: f32, watthours: f32, co2: f32, water: f32) -> Self {
        UsageStatistics {
            dollars,
            watthours,
            co2,
            water,
        }
    }

    pub fn get_current_day() -> u32 {
        let today = Local::now().date_naive();
        let epoch_date = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
        today.signed_duration_since(epoch_date).num_days().abs() as u32
    }
}

impl Add for UsageStatistics {
    type Output = UsageStatistics;

    fn add(self, rhs: Self) -> Self::Output {
        UsageStatistics {
            watthours: self.watthours + rhs.watthours,
            co2: self.co2 + rhs.co2,
            water: self.water + rhs.water,
            dollars: self.dollars + rhs.dollars,
        }
    }
}
