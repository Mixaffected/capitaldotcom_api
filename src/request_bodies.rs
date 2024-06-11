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
    guaranteed_stop: Option<bool>,
    trailing_stop: Option<bool>,
    stop_level: Option<f32>,
    stop_distance: Option<f32>,
    stop_amount: Option<f32>,
    profit_level: Option<f32>,
    profit_distance: Option<f32>,
    profit_amount: Option<f32>,
}
impl CreatePositionBody {
    pub fn new(
        direction: enums::Direction,
        epic: &str,
        size: f32,
        guaranteed_stop: Option<bool>,
        trailing_stop: Option<bool>,
        stop_level: Option<f32>,
        stop_distance: Option<f32>,
        stop_amount: Option<f32>,
        profit_level: Option<f32>,
        profit_distance: Option<f32>,
        profit_amount: Option<f32>,
    ) -> Self {
        Self {
            direction,
            epic: epic.to_string(),
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
impl Clone for CreatePositionBody {
    fn clone(&self) -> Self {
        Self {
            direction: self.direction.clone(),
            epic: self.epic.clone(),
            size: self.size.clone(),
            guaranteed_stop: self.guaranteed_stop.clone(),
            trailing_stop: self.trailing_stop.clone(),
            stop_level: self.stop_level.clone(),
            stop_distance: self.stop_distance.clone(),
            stop_amount: self.stop_amount.clone(),
            profit_level: self.profit_level.clone(),
            profit_distance: self.profit_distance.clone(),
            profit_amount: self.profit_amount.clone(),
        }
    }
}
pub struct CreatePositionBodyBuilder {
    create_position_body: CreatePositionBody,
}
impl CreatePositionBodyBuilder {
    pub fn new(direction: enums::Direction, epic: &str, size: f32) -> Self {
        Self {
            create_position_body: CreatePositionBody::new(
                direction, epic, size, None, None, None, None, None, None, None, None,
            ),
        }
    }

    /// Needs stop_level, stop_distance or stop_amount set. Disables trailing_stop. Can not be set if hedging mode is enabled.
    pub fn guaranteed_stop(mut self, guaranteed_stop: bool) -> Self {
        self.create_position_body.guaranteed_stop = Some(guaranteed_stop);
        self.create_position_body.trailing_stop = None;

        self
    }

    /// Needs to have stop_distance set. If disabled stop_distance gets disabled. Disables guaranteed_stop.
    pub fn trailing_stop(mut self, trailing_stop: bool) -> Self {
        if !trailing_stop {
            self.create_position_body.stop_distance = None;
        };

        self.create_position_body.trailing_stop = Some(trailing_stop);
        self.create_position_body.guaranteed_stop = None;

        self
    }

    /// Price level when a stop loss will be triggered.
    pub fn stop_level(mut self, stop_level: f32) -> Self {
        self.create_position_body.stop_level = Some(stop_level);

        self
    }

    /// Distance between current and stop loss triggering price.
    pub fn stop_distance(mut self, stop_distance: f32) -> Self {
        self.create_position_body.stop_distance = Some(stop_distance);

        self
    }

    /// Loss amount when a stop loss will be triggered.
    pub fn stop_amount(mut self, stop_amount: f32) -> Self {
        self.create_position_body.stop_amount = Some(stop_amount);

        self
    }

    /// Price level when a take profit will be triggered.
    pub fn profit_level(mut self, profit_level: f32) -> Self {
        self.create_position_body.profit_level = Some(profit_level);

        self
    }

    /// Distance between current and take profit triggering price.
    pub fn profit_distance(mut self, profit_distance: f32) -> Self {
        self.create_position_body.profit_distance = Some(profit_distance);

        self
    }

    /// Profit amount when a take profit will be triggered
    pub fn profit_amount(mut self, profit_amount: f32) -> Self {
        self.create_position_body.profit_amount = Some(profit_amount);

        self
    }

    pub fn build(self) -> CreatePositionBody {
        self.create_position_body
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
