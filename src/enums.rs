use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum InstrumentType {
    COMMODITIES,
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
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
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
