use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum InstrumentType {
    COMMODITIES,
    SHARES,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AccountType {
    CFD,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Currency {
    EUR,
    EURd,
    USD,
    AUD,
    PLN,
    AED,
    GBP,
    CHF,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Locale {
    EN,
    DE,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AccountStatus {
    ENABLED,
    DISABLED,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Direction {
    BUY,
    SELL,
}
impl Clone for Direction {
    fn clone(&self) -> Self {
        match self {
            Self::BUY => Self::BUY,
            Self::SELL => Self::SELL,
        }
    }
}
impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Self::BUY => "BUY",
            Self::SELL => "SELL",
        };
        
        write!(f, "{}", string)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TimeZone {
    UTC,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Unit {
    PERCENTAGE,
    POINTS,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Resolution {
    MINUTE,
    Minute5,
    Minute15,
    Minute30,
    HOUR,
    Hour4,
    DAY,
    WEEK,
}
impl ToString for Resolution {
    fn to_string(&self) -> String {
        match self {
            Self::MINUTE => String::from("MINUTE"),
            Self::Minute5 => String::from("MINUTE_5"),
            Self::Minute15 => String::from("MINUTE_15"),
            Self::Minute30 => String::from("MINUTE_30"),
            Self::HOUR => String::from("HOUR"),
            Self::Hour4 => String::from("HOUR_4"),
            Self::DAY => String::from("DAY"),
            Self::WEEK => String::from("WEEK"),
        }
    }
}
