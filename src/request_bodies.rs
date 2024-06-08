use serde::Serialize;

use crate::enums;

#[derive(Debug, Serialize)]
pub struct CreateSessionBody {
    identifier: String,
    password: String,
}
impl CreateSessionBody {
    pub fn new(identifier: &str, password: &str) -> Self {
        Self {
            identifier: identifier.to_string(),
            password: password.to_string(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePositionBody {
    direction: enums::Direction, // Long or Short position.
    epic: String,                // Instrument epic identifier.
    size: f32,
    guaranteed_stop: bool,
    trailing_stop: bool,
    stop_level: f32,
    stop_distance: f32,
    stop_amount: f32,
    profit_level: f32,
    profit_distance: f32,
    profit_amount: f32,
}
impl CreatePositionBody {
    pub fn new(
        direction: enums::Direction,
        epic: String,
        size: f32,
        guaranteed_stop: bool,
        trailing_stop: bool,
        stop_level: f32,
        stop_distance: f32,
        stop_amount: f32,
        profit_level: f32,
        profit_distance: f32,
        profit_amount: f32,
    ) -> Self {
        Self {
            direction,
            epic,
            size,
            guaranteed_stop,
            trailing_stop,
            stop_level,
            stop_distance,
            stop_amount,
            profit_level,
            profit_distance,
            profit_amount,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
/// # ***Field explanation:***
/// NAME                    DESCRIPTION
/// guaranteedStop          Must be true if a guaranteed stop is required.
pub struct PositionUpdateBody {
    guaranteed_stop: bool,
    trailing_stop: bool,
    stop_level: f32,
    stop_distance: f32,
    stop_amount: f32,
    profit_level: f32,
    profit_distance: f32,
    profit_amount: f32,
}
impl PositionUpdateBody {
    pub fn new(
        guaranteed_stop: bool,
        trailing_stop: bool,
        stop_level: f32,
        stop_distance: f32,
        stop_amount: f32,
        profit_level: f32,
        profit_distance: f32,
        profit_amount: f32,
    ) -> Self {
        Self {
            guaranteed_stop,
            trailing_stop,
            stop_level,
            stop_distance,
            stop_amount,
            profit_level,
            profit_distance,
            profit_amount,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SwitchActiveAccountBody {
    account_id: String,
}
impl SwitchActiveAccountBody {
    pub fn new(account_id: String) -> Self {
        Self { account_id }
    }
}

/*
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SwitchActiveAccountBody {
    account_id: String,
}
impl SwitchActiveAccountBody {
    pub fn new(account_id: String) -> Self {
        Self { account_id }
    }
}
 */
