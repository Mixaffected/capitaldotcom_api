use std::{collections::HashMap, string::FromUtf8Error, time};

use reqwest::header::{HeaderMap, HeaderValue};
use serde::Serialize;

mod responses;

mod enums;
mod request_bodies;
mod traits;

pub use traits::CapitalDotComInterface;

use traits::ReqwestUtils;

/// Limitations:
///  * Max of 10 requests per second
///  * Max of 1 request per 0.1 seconds else position/orders get rejected
///  * Max of 1 request per second for session creation
///
/// Explanation:
///  * x_cap_api_key: the api key from Settings > API Integrations
///  * x_security_token: the account token
///  * cst: the access token
///  * identifier: the email address you log in with
///  * password: the password you created for this API key
#[derive(Debug)]
struct CapitalDotComAPI {
    base_url: String,

    x_cap_api_key: String,
    x_security_token: String, // Needs to be requested
    cst: String,              // Needs to be requested
    identifier: String,
    password: String,
    encryption_key: String,
    auth_header_map: HeaderMap,

    last_request_timestamp: chrono::DateTime<chrono::Utc>, // timestamp of the last request (API times out after 10 mins)
    http_client: reqwest::Client,
}
impl CapitalDotComAPI {
    pub fn new(
        session_type: SessionType,
        x_cap_api_key: String,
        identifier: String,
        password: String,
    ) -> Self {
        Self {
            base_url: Self::get_session_url_from_sessiontype(session_type),

            x_cap_api_key,
            x_security_token: String::new(),
            cst: String::new(),
            identifier,
            password,
            encryption_key: String::new(),
            auth_header_map: HeaderMap::new(),

            last_request_timestamp: chrono::Utc::now() - time::Duration::from_secs(610), // Get time from the past so the session has timeouted
            http_client: reqwest::Client::new(),
        }
    }

    fn get_session_url_from_sessiontype(session_type: SessionType) -> String {
        match session_type {
            SessionType::Live => String::from("https://api-capital.backend-capital.com"),
            SessionType::Demo => String::from("https://demo-api-capital.backend-capital.com"),
        }
    }

    fn get_url(&self, path: &str) -> String {
        let mut endpoint = String::new();
        endpoint.push_str(&self.base_url);
        endpoint.push_str(path);

        endpoint
    }

    fn get_string_from_header(headers: &HeaderMap, key: &str) -> Option<String> {
        match headers.get(key) {
            Some(x_security_token) => {
                Some(String::from_utf8_lossy(x_security_token.as_bytes()).to_string())
            }
            None => None,
        }
    }

    fn update_last_request_timestamp(&mut self) {
        self.last_request_timestamp = chrono::Utc::now();
    }

    fn update_auth(&mut self, headers: &HashMap<String, String>) {
        self.x_security_token = headers
            .get("X-SECURITY-TOKEN")
            .unwrap_or(&String::new())
            .to_owned();
        self.cst = headers.get("CST").unwrap_or(&String::new()).to_owned();

        let mut header_map = HeaderMap::new();
        header_map.append(
            "X-SECURITY-TOKEN",
            HeaderValue::from_str(&self.x_security_token).expect("x_security_token too large!"),
        );
        header_map.append(
            "CST",
            HeaderValue::from_str(&self.cst).expect("cst too large!"),
        );

        self.auth_header_map = header_map;
    }
}

impl traits::ReqwestUtils for CapitalDotComAPI {}

impl traits::CapitalDotComEndpoints for CapitalDotComAPI {
    async fn get_server_time(
        &self,
    ) -> Result<
        (u16, HashMap<String, String>, responses::ServerTimeResponse),
        CapitalDotComError<responses::ServerTimeResponse>,
    > {
        let request_builder = self.http_client.get(Self::get_url(&self, "/api/v1/time"));

        Self::request_data(request_builder).await
    }

    async fn ping(
        &self,
    ) -> Result<
        (u16, HashMap<String, String>, responses::PingResponse),
        CapitalDotComError<responses::PingResponse>,
    > {
        self.has_credentials()?;

        let request_builder = self
            .http_client
            .get(Self::get_url(&self, "/api/v1/ping"))
            .headers(self.auth_header_map.clone());

        Self::request_data(request_builder).await
    }

    async fn get_encryption_key(
        &self,
    ) -> Result<
        (
            u16,
            HashMap<String, String>,
            responses::EncryptionKeyResponse,
        ),
        CapitalDotComError<responses::EncryptionKeyResponse>,
    > {
        let request_builder = self
            .http_client
            .get(Self::get_url(&self, "/api/v1/session/encryptionKey"))
            .header("X-CAP-API-KEY", &self.x_cap_api_key);

        Self::request_data(request_builder).await
    }

    async fn get_session_details(
        &self,
    ) -> Result<
        (
            u16,
            HashMap<String, String>,
            responses::SessionDetailsResponse,
        ),
        CapitalDotComError<responses::SessionDetailsResponse>,
    > {
        self.has_credentials()?;

        let request_builder = self
            .http_client
            .get(Self::get_url(&self, "/api/v1/session"))
            .headers(self.auth_header_map.clone());

        Self::request_data(request_builder).await
    }

    async fn create_new_session(
        &mut self,
    ) -> Result<
        (
            u16,
            HashMap<String, String>,
            responses::CreateNewSessionResponse,
        ),
        CapitalDotComError<responses::CreateNewSessionResponse>,
    > {
        let request_builder = self
            .http_client
            .post(Self::get_url(&self, "/api/v1/session/encryptionKey"))
            .header("X-CAP-API-KEY", &self.x_cap_api_key);

        let (status, headers, body) = Self::request_data(request_builder).await?;

        // Update authorization values
        self.update_auth(&headers);

        Ok((status, headers, body))
    }

    async fn switch_active_account(
        &self,
        account_id: String,
    ) -> Result<
        (
            u16,
            HashMap<String, String>,
            responses::SwitchAccountResponse,
        ),
        CapitalDotComError<responses::SwitchAccountResponse>,
    > {
        self.has_credentials()?;

        let body =
            Self::get_json_from_value(request_bodies::SwitchActiveAccountBody::new(account_id))?;

        let request_builder = self
            .http_client
            .put(Self::get_url(&self, "/api/v1/session"))
            .headers(self.auth_header_map.clone())
            .header("Content-Type", "application/json")
            .body(body);

        Self::request_data(request_builder).await
    }

    async fn session_log_out(
        &self,
    ) -> Result<
        (
            u16,
            HashMap<String, String>,
            responses::SessionLogOutResponse,
        ),
        CapitalDotComError<responses::SessionLogOutResponse>,
    > {
        self.has_credentials()?;

        let request_builder = self
            .http_client
            .delete(Self::get_url(&self, "/api/v1/session"))
            .headers(self.auth_header_map.clone());

        Self::request_data(request_builder).await
    }

    async fn get_all_accounts(
        &self,
    ) -> Result<
        (u16, HashMap<String, String>, responses::AllAccountsResponse),
        CapitalDotComError<responses::AllAccountsResponse>,
    > {
        self.has_credentials()?;

        let request_builder = self
            .http_client
            .get(Self::get_url(&self, "/api/v1/accounts"))
            .headers(self.auth_header_map.clone());

        Self::request_data(request_builder).await
    }

    async fn order_confirmation(
        &self,
        deal_reference: String,
    ) -> Result<
        (
            u16,
            HashMap<String, String>,
            responses::OrderConfirmationResponse,
        ),
        CapitalDotComError<responses::OrderConfirmationResponse>,
    > {
        self.has_credentials()?;

        let request_builder = self
            .http_client
            .get(Self::get_url(
                &self,
                &format!("/api/v1/confirms/{}", deal_reference),
            ))
            .headers(self.auth_header_map.clone());

        Self::request_data(request_builder).await
    }

    async fn get_all_positions(
        &self,
    ) -> Result<
        (
            u16,
            HashMap<String, String>,
            responses::AllPositionsResponse,
        ),
        CapitalDotComError<responses::AllPositionsResponse>,
    > {
        self.has_credentials()?;

        let request_builder = self
            .http_client
            .get(Self::get_url(&self, "/api/v1/positions"))
            .headers(self.auth_header_map.clone());

        Self::request_data(request_builder).await
    }

    async fn create_position(
        &self,
        position_data: request_bodies::CreatePositionBody,
    ) -> Result<
        (
            u16,
            HashMap<String, String>,
            responses::DealReferenceResponse,
        ),
        CapitalDotComError<responses::DealReferenceResponse>,
    > {
        self.has_credentials()?;

        let body = Self::get_json_from_value(position_data)?;

        let request_builder = self
            .http_client
            .post(Self::get_url(&self, "/api/v1/positions"))
            .headers(self.auth_header_map.clone())
            .header("Content-Type", "application/json")
            .body(body);

        Self::request_data(request_builder).await
    }

    async fn get_position(
        &self,
        deal_id: String,
    ) -> Result<
        (u16, HashMap<String, String>, responses::PositionResponse),
        CapitalDotComError<responses::PositionResponse>,
    > {
        self.has_credentials()?;

        let request_builder = self
            .http_client
            .get(Self::get_url(
                &self,
                &format!("/api/v1/positions/{}", deal_id),
            ))
            .headers(self.auth_header_map.clone());

        Self::request_data(request_builder).await
    }

    async fn update_position(
        &self,
        deal_id: String,
        position_update_data: request_bodies::PositionUpdateBody,
    ) -> Result<
        (
            u16,
            HashMap<String, String>,
            responses::DealReferenceResponse,
        ),
        CapitalDotComError<responses::DealReferenceResponse>,
    > {
        self.has_credentials()?;

        let body = Self::get_json_from_value(position_update_data)?;

        let request_builder = self
            .http_client
            .put(Self::get_url(
                &self,
                &format!("/api/v1/positions/{}", deal_id),
            ))
            .headers(self.auth_header_map.clone())
            .header("Content-Type", "application/json")
            .body(body);

        Self::request_data(request_builder).await
    }

    async fn close_position(
        &self,
        deal_id: String,
    ) -> Result<
        (
            u16,
            HashMap<String, String>,
            responses::DealReferenceResponse,
        ),
        CapitalDotComError<responses::DealReferenceResponse>,
    > {
        self.has_credentials()?;

        let request_builder = self
            .http_client
            .delete(Self::get_url(
                &self,
                &format!("/api/v1/positions/{}", deal_id),
            ))
            .headers(self.auth_header_map.clone());

        Self::request_data(request_builder).await
    }

    async fn get_market_details(
        &self,
        search_term: String,
        epics: Vec<String>,
    ) -> Result<
        (
            u16,
            HashMap<String, String>,
            responses::MarketDetailsResponse,
        ),
        CapitalDotComError<responses::MarketDetailsResponse>,
    > {
        self.has_credentials()?;

        if epics.len() > 50 {
            return Err(CapitalDotComError::TooManyParameters);
        }

        let mut epic_query = String::new();
        for (i, epic) in epics.iter().enumerate() {
            epic_query.push_str(&epic);

            if i <= epics.len() - 1 {
                epic_query.push(',');
            }
        }

        let mut request_builder = self
            .http_client
            .get(Self::get_url(&self, "/api/v1/markets"))
            .headers(self.auth_header_map.clone());

        if epic_query.is_empty() {
            request_builder = request_builder.query(&[("searchTerm", search_term)]);
        } else {
            request_builder =
                request_builder.query(&[("searchTerm", search_term), ("epics", epic_query)]);
        }

        Self::request_data(request_builder).await
    }

    async fn single_market_details(
        &self,
        epic: String,
    ) -> Result<
        (
            u16,
            HashMap<String, String>,
            responses::SingleMarketDetailsResponse,
        ),
        CapitalDotComError<responses::SingleMarketDetailsResponse>,
    > {
        self.has_credentials()?;

        let request_builder = self
            .http_client
            .get(Self::get_url(&self, &format!("/api/v1/markets/{}", epic)))
            .headers(self.auth_header_map.clone());

        Self::request_data(request_builder).await
    }

    async fn get_historical_prices(
        &self,
        epic: String,
        resolution: enums::Resolution,
        max: i32,
        from: String,
        to: String,
    ) -> Result<
        (
            u16,
            HashMap<String, String>,
            responses::HistoricalPricesResponse,
        ),
        CapitalDotComError<responses::HistoricalPricesResponse>,
    > {
        self.has_credentials()?;

        let request_builder = self
            .http_client
            .get(Self::get_url(&self, &format!("/api/v1/prices/{}", epic)))
            .query(&[
                ("resolution", Self::get_json_from_value(resolution)?),
                ("max", max.to_string()),
                ("from", from),
                ("to", to),
            ])
            .headers(self.auth_header_map.clone());

        Self::request_data(request_builder).await
    }

    fn has_credentials<T>(&self) -> Result<(), CapitalDotComError<T>> {
        if !self.x_security_token.is_empty() || !self.cst.is_empty() {
            Ok(())
        } else {
            Err(CapitalDotComError::MissingAuthorization)
        }
    }
}

#[derive(Debug)]
pub enum SessionType {
    Live,
    Demo,
}

#[derive(Debug)]
pub enum CapitalDotComError<ErrD> {
    ReqwestError(reqwest::Error),
    JsonError(serde_json::Error),
    StatusCode(u16, ErrD),
    HeaderNotFound,
    FromUtf8Error(FromUtf8Error),
    TooManyParameters,
    Unauthorized,
    MissingAuthorization,
}

#[cfg(test)]
mod tests {
    use std::fs;

    use serde::{Deserialize, Serialize};
    use tokio::runtime::Runtime;
    use traits::CapitalDotComEndpoints;

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
            SessionType::Demo,
            credentials.api_key,
            credentials.identifier,
            credentials.api_password,
        );

        let rt = Runtime::new().unwrap();
        rt.block_on(capital_api.create_new_session()).unwrap();
        println!("Created new Session...");

        rt.block_on(capital_api.ping()).unwrap();
        println!("Ping done...");
    }
}
