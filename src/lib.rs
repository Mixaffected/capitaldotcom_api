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
pub use enums::{Direction, Resolution};
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

    current_account_id: String,
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

            current_account_id: String::new(),
        }
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

        self.current_account_id = body.current_account_id.clone();

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

    fn get_balance(&self) -> Result<responses::BalanceAccountInfo, CapitalDotComError> {
        let mut capital_dot_com_endpoints_lock = self
            .capital_dot_com_endpoints
            .lock()
            .unwrap_or_else(|p| p.into_inner());

        let (_, body) = self
            .runtime
            .block_on(capital_dot_com_endpoints_lock.get_all_accounts())?;

        for account in body.accounts {
            if account.account_id == self.current_account_id {
                return Ok(account.balance);
            }
        }

        Err(CapitalDotComError::CurrentAccountNotFound)
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
        account_id: &str,
    ) -> Result<responses::SwitchAccountResponse, CapitalDotComError> {
        if account_id == self.current_account_id {
            return Err(CapitalDotComError::NotDifferentAccountId);
        }

        let mut capital_dot_com_endpoints_lock = self
            .capital_dot_com_endpoints
            .lock()
            .unwrap_or_else(|p| p.into_inner());

        let (_, body) = self
            .runtime
            .block_on(capital_dot_com_endpoints_lock.switch_active_account(account_id))?;

        self.current_account_id = account_id.to_string();

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
        search_term: &str,
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
        epic: &str,
    ) -> Result<responses::SingleMarketDetailsResponse, CapitalDotComError> {
        let mut capital_dot_com_endpoints_lock = self
            .capital_dot_com_endpoints
            .lock()
            .unwrap_or_else(|p| p.into_inner());

        let (_, body) = self
            .runtime
            .block_on(capital_dot_com_endpoints_lock.get_single_market_details(epic.to_string()))?;

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
    ) -> Result<responses::OrderConfirmationResponse, CapitalDotComError> {
        let mut capital_dot_com_endpoints_lock = self
            .capital_dot_com_endpoints
            .lock()
            .unwrap_or_else(|p| p.into_inner());

        let (_, body) = self
            .runtime
            .block_on(capital_dot_com_endpoints_lock.open_position(position_data))?;

        let (_, body) = self
            .runtime
            .block_on(capital_dot_com_endpoints_lock.order_confirmation(&body.deal_reference))?;

        Ok(body)
    }

    fn position_data(
        &self,
        deal_id: &str,
    ) -> Result<responses::PositionResponse, CapitalDotComError> {
        let mut capital_dot_com_endpoints_lock = self
            .capital_dot_com_endpoints
            .lock()
            .unwrap_or_else(|p| p.into_inner());

        let (_, body) = self
            .runtime
            .block_on(capital_dot_com_endpoints_lock.get_position(deal_id.to_string()))?;

        Ok(body)
    }

    fn close_position(
        &self,
        deal_id: &str,
    ) -> Result<responses::DealReferenceResponse, CapitalDotComError> {
        let mut capital_dot_com_endpoints_lock = self
            .capital_dot_com_endpoints
            .lock()
            .unwrap_or_else(|p| p.into_inner());

        let (_, body) = self
            .runtime
            .block_on(capital_dot_com_endpoints_lock.close_position(deal_id.to_string()))?;

        Ok(body)
    }

    /// * max: The maximum number of the values in answer. Default = 10, max = 1000
    fn get_historical_prices(
        &self,
        epic: &str,
        resolution: enums::Resolution,
        max: Option<i32>,
        from: chrono::DateTime<chrono::Utc>,
        to: chrono::DateTime<chrono::Utc>,
    ) -> Result<responses::HistoricalPricesResponse, CapitalDotComError> {
        let mut capital_dot_com_endpoints_lock = self
            .capital_dot_com_endpoints
            .lock()
            .unwrap_or_else(|p| p.into_inner());

        let (_, body) =
            self.runtime
                .block_on(capital_dot_com_endpoints_lock.get_historical_prices(
                    epic.to_string(),
                    resolution,
                    max,
                    from,
                    to,
                ))?;

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
    CurrentAccountNotFound,
    NotDifferentAccountId,
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

    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    struct Credentials {
        pub identifier: String,
        pub api_key: String,
        pub api_password: String,
        pub test_account_name: String,
    }

    fn get_api_credentials() -> Credentials {
        let credentials_file = fs::File::open("test_api_credentials.json").unwrap();
        let credentials: Credentials = serde_json::from_reader(credentials_file).unwrap();

        credentials
    }

    #[test]
    fn full_test() {
        println!("\n\n\n");

        let credentials = get_api_credentials();
        let mut capital_api = CapitalDotComAPI::new(
            SessionType::Demo, // For the sake of god, dont change this to live.
            credentials.api_key,
            credentials.identifier,
            credentials.api_password,
        );

        let session_details = capital_api.open_session().unwrap();

        // Select right account
        let mut account_id = String::new();
        for account in session_details.accounts {
            if account.account_name == credentials.test_account_name {
                account_id = account.account_id;
            };
        }
        if account_id != session_details.current_account_id {
            capital_api.switch_account(&account_id).unwrap();
        }

        let balance = capital_api.get_balance().unwrap();
        println!(
            "Balance:\n  Balance: {},\n  Deposit: {},\n  P/L: {},\n  Available: {}",
            balance.balance, balance.deposit, balance.profit_loss, balance.available
        );

        let markets = capital_api.search_market("Tesla", Vec::new()).unwrap();
        let mut epic = String::new();
        for market in markets.markets {
            if market.instrument_name.contains("Tesla") {
                epic = market.epic;
            };
        }
        println!("Epic: {}", epic);

        let market = capital_api.get_market_data(&epic).unwrap();
        println!("{:?}", market);

        let position_data = request_bodies::CreatePositionBodyBuilder::new(
            Direction::SELL,
            &epic,
            market.dealing_rules.min_deal_size.value * 10.,
        )
        .build();
        let deal_reference = capital_api.open_position(position_data).unwrap();
        println!("Order: {:?}", deal_reference);

        let all_positions = capital_api.get_all_positions().unwrap();
        println!("{:?}", all_positions);

        let session_details = capital_api.get_session_details().unwrap();
        println!("{:?}", session_details);

        for position in all_positions.positions {
            let deal_reference = capital_api
                .close_position(&position.position.deal_id)
                .unwrap();
            println!("{:?}", deal_reference);
        }

        let history = capital_api
            .get_historical_prices(
                &epic,
                Resolution::HOUR,
                Some(2),
                chrono::DateTime::from_timestamp_millis(1718109976000).unwrap(),
                chrono::DateTime::from_timestamp_millis(1718117176000).unwrap(),
            )
            .unwrap();
        println!("{:?}", history);

        let session_logout = capital_api.close_session().unwrap();
        println!("{:?}", session_logout);

        println!("\n\n\n");
    }
}
