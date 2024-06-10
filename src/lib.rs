use std::{
    fmt::Display,
    string::FromUtf8Error,
    sync::{Arc, Mutex},
    thread, time,
};

use reqwest::header::HeaderMap;
use serde::Serialize;

mod responses;

mod endpoint;
mod enums;
mod request_bodies;
mod traits;

pub use endpoint::SessionType;
pub use traits::CapitalDotComInterface;

use endpoint::CapitalDotComApiEndpoints;
use traits::CapitalDotComEndpoints;

const TIME_BEFORE_LOGOUT: u32 = 600_000;

/// Limitations:
///  * Max of 10 requests per second
///  * Max of 1 request per 0.1 seconds (100 ms) else position/orders get rejected
///  * Max of 1 request per second for session creation
///
/// Explanation:
///  * x_cap_api_key: the api key from Settings > API Integrations
///  * x_security_token: the account token
///  * cst: the access token
///  * identifier: the email address you log in with
///  * password: the password you created for this API key
#[derive(Debug)]
pub struct CapitalDotComAPI {
    is_logged_in: Arc<Mutex<bool>>,
    capital_dot_com_endpoints: Arc<Mutex<endpoint::CapitalDotComApiEndpoints>>,
    runtime: tokio::runtime::Runtime,
}
impl CapitalDotComAPI {
    pub fn new(
        session_type: SessionType,
        x_cap_api_key: String,
        identifier: String,
        password: String,
    ) -> Self {
        let runtime = match tokio::runtime::Runtime::new() {
            Ok(runtime) => runtime,
            Err(e) => panic!("Could not initialize runtime! Error: {}", e),
        };

        Self {
            is_logged_in: Arc::new(Mutex::new(false)),
            capital_dot_com_endpoints: Arc::new(Mutex::new(CapitalDotComApiEndpoints::new(
                session_type,
                x_cap_api_key,
                identifier,
                password,
            ))),
            runtime,
        }
    }

    fn keep_session_alive_service(&self) {
        let is_logged_in_c = self.is_logged_in.clone();
        let capital_dot_com_endpoints_c = self.capital_dot_com_endpoints.clone();
        let runtime = tokio::runtime::Runtime::new().expect("Could not create Tokio runtime!");

        thread::spawn(move || loop {
            thread::sleep(time::Duration::from_millis(200));

            {
                let is_logged_in_lock = is_logged_in_c.lock().unwrap_or_else(|p| p.into_inner());
                if !*is_logged_in_lock {
                    return;
                };
            }

            {
                let mut capital_dot_com_endpoints_lock = capital_dot_com_endpoints_c
                    .lock()
                    .unwrap_or_else(|p| p.into_inner());

                let time_since_last_request =
                    capital_dot_com_endpoints_lock.get_time_since_last_request();

                if time_since_last_request
                    > chrono::TimeDelta::milliseconds((TIME_BEFORE_LOGOUT as f32 * 0.9) as i64)
                {
                    match runtime.block_on(capital_dot_com_endpoints_lock.ping()) {
                        Ok(_) => (),
                        Err(e) => println!("Could not ping server! Error: {}", e),
                    };
                };
            }
        });
    }
}

impl traits::CapitalDotComInterface for CapitalDotComAPI {
    fn open_session(&mut self) -> Result<responses::CreateNewSessionResponse, CapitalDotComError> {
        let mut capital_dot_com_endpoints_lock = self
            .capital_dot_com_endpoints
            .lock()
            .unwrap_or_else(|p| p.into_inner());

        let (_, body) = self
            .runtime
            .block_on(capital_dot_com_endpoints_lock.create_new_session())?;

        self.keep_session_alive_service();

        Ok(body)
    }

    fn get_session_details(&self) -> Result<responses::SessionDetailsResponse, CapitalDotComError> {
        let mut is_logged_in_lock = self.is_logged_in.lock().unwrap_or_else(|p| p.into_inner());
        *is_logged_in_lock = true;

        let mut capital_dot_com_endpoints_lock = self
            .capital_dot_com_endpoints
            .lock()
            .unwrap_or_else(|p| p.into_inner());

        let (_, body) = self
            .runtime
            .block_on(capital_dot_com_endpoints_lock.get_session_details())?;

        Ok(body)
    }

    fn get_all_accounts(&self) -> Result<responses::AllAccountsResponse, CapitalDotComError> {
        let mut capital_dot_com_endpoints_lock = self
            .capital_dot_com_endpoints
            .lock()
            .unwrap_or_else(|p| p.into_inner());

        let (_, body) = self
            .runtime
            .block_on(capital_dot_com_endpoints_lock.get_all_accounts())?;

        Ok(body)
    }

    fn switch_account(
        &mut self,
        account_id: String,
    ) -> Result<responses::SwitchAccountResponse, CapitalDotComError> {
        let mut capital_dot_com_endpoints_lock = self
            .capital_dot_com_endpoints
            .lock()
            .unwrap_or_else(|p| p.into_inner());

        let (_, body) = self
            .runtime
            .block_on(capital_dot_com_endpoints_lock.switch_active_account(account_id))?;

        Ok(body)
    }

    fn close_session(&self) -> Result<responses::SessionLogOutResponse, CapitalDotComError> {
        let mut is_logged_in_lock = self.is_logged_in.lock().unwrap_or_else(|p| p.into_inner());
        *is_logged_in_lock = false;

        let mut capital_dot_com_endpoints_lock = self
            .capital_dot_com_endpoints
            .lock()
            .unwrap_or_else(|p| p.into_inner());

        let (_, body) = self
            .runtime
            .block_on(capital_dot_com_endpoints_lock.session_log_out())?;

        Ok(body)
    }

    fn search_market(
        &self,
        search_term: String,
        epic: Vec<String>,
    ) -> Result<responses::MarketDetailsResponse, CapitalDotComError> {
        let mut capital_dot_com_endpoints_lock = self
            .capital_dot_com_endpoints
            .lock()
            .unwrap_or_else(|p| p.into_inner());

        let (_, body) = self
            .runtime
            .block_on(capital_dot_com_endpoints_lock.get_market_details(search_term, epic))?;

        Ok(body)
    }

    fn get_market_data(
        &self,
        epic: String,
    ) -> Result<responses::SingleMarketDetailsResponse, CapitalDotComError> {
        let mut capital_dot_com_endpoints_lock = self
            .capital_dot_com_endpoints
            .lock()
            .unwrap_or_else(|p| p.into_inner());

        let (_, body) = self
            .runtime
            .block_on(capital_dot_com_endpoints_lock.get_single_market_details(epic))?;

        Ok(body)
    }

    fn get_all_positions(&self) -> Result<responses::AllPositionsResponse, CapitalDotComError> {
        let mut capital_dot_com_endpoints_lock = self
            .capital_dot_com_endpoints
            .lock()
            .unwrap_or_else(|p| p.into_inner());

        let (_, body) = self
            .runtime
            .block_on(capital_dot_com_endpoints_lock.get_all_positions())?;

        Ok(body)
    }

    fn open_position(
        &self,
        position_data: request_bodies::CreatePositionBody,
    ) -> Result<responses::DealReferenceResponse, CapitalDotComError> {
        let mut capital_dot_com_endpoints_lock = self
            .capital_dot_com_endpoints
            .lock()
            .unwrap_or_else(|p| p.into_inner());

        let (_, body) = self
            .runtime
            .block_on(capital_dot_com_endpoints_lock.open_position(position_data))?;

        Ok(body)
    }

    fn position_data(
        &self,
        deal_id: String,
    ) -> Result<responses::PositionResponse, CapitalDotComError> {
        let mut capital_dot_com_endpoints_lock = self
            .capital_dot_com_endpoints
            .lock()
            .unwrap_or_else(|p| p.into_inner());

        let (_, body) = self
            .runtime
            .block_on(capital_dot_com_endpoints_lock.get_position(deal_id))?;

        Ok(body)
    }

    fn close_position(
        &self,
        deal_id: String,
    ) -> Result<responses::DealReferenceResponse, CapitalDotComError> {
        let mut capital_dot_com_endpoints_lock = self
            .capital_dot_com_endpoints
            .lock()
            .unwrap_or_else(|p| p.into_inner());

        let (_, body) = self
            .runtime
            .block_on(capital_dot_com_endpoints_lock.close_position(deal_id))?;

        Ok(body)
    }

    fn get_historical_prices(
        &self,
        epic: String,
        resolution: enums::Resolution,
        max: i32,
        from: chrono::DateTime<chrono::Utc>,
        to: chrono::DateTime<chrono::Utc>,
    ) -> Result<responses::HistoricalPricesResponse, CapitalDotComError> {
        let mut capital_dot_com_endpoints_lock = self
            .capital_dot_com_endpoints
            .lock()
            .unwrap_or_else(|p| p.into_inner());

        let (_, body) = self.runtime.block_on(
            capital_dot_com_endpoints_lock.get_historical_prices(epic, resolution, max, from, to),
        )?;

        Ok(body)
    }
}

#[derive(Debug)]
pub enum CapitalDotComError {
    ReqwestError(reqwest::Error),
    JsonError(serde_json::Error),
    StatusCode(u16, responses::APIError, String),
    HeaderNotFound,
    FromUtf8Error(FromUtf8Error),
    TooManyParameters,
    Unauthorized,
    MissingAuthorization,
    RequestingTooFast(chrono::TimeDelta),
}
impl Display for CapitalDotComError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use serde::{Deserialize, Serialize};
    use tokio::runtime::Runtime;

    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    struct Credentials {
        pub identifier: String,
        pub api_key: String,
        pub api_password: String,
    }

    fn get_api_credentials() -> Credentials {
        let credentials_file = fs::File::open("test_api_credentials.json").unwrap();
        let credentials: Credentials = serde_json::from_reader(credentials_file).unwrap();

        credentials
    }

    #[test]
    fn full_test() {
        let credentials = get_api_credentials();
        let mut capital_api = CapitalDotComAPI::new(
            SessionType::Demo, // For the sake of god, dont change this to live.
            credentials.api_key,
            credentials.identifier,
            credentials.api_password,
        );

        let session_details = capital_api.open_session().unwrap();
    }
}
