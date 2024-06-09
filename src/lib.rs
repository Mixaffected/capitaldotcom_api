use std::{
    result,
    str::Utf8Error,
    string::FromUtf8Error,
    sync::{Arc, Mutex},
    thread, time,
};

use chrono::Utc;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Response, StatusCode,
};
use serde::{Deserialize, Serialize};

mod responses;

mod enums;
mod request_bodies;
mod traits;

pub use traits::CapitalDotComInterface;

use traits::Reqwest;

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

    fn get_header_value(headers: &HeaderMap, key: &str) -> String {
        match headers.get(key) {
            Some(x_security_token) => {
                String::from_utf8_lossy(x_security_token.as_bytes()).to_string()
            }
            None => String::new(),
        }
    }

    fn update_auth(&mut self, response: &Response) {
        let headers = response.headers();

        self.x_security_token = Self::get_header_value(headers, "X-SECURITY-TOKEN");
        self.cst = Self::get_header_value(headers, "CST");

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

impl traits::Reqwest for CapitalDotComAPI {}

impl traits::CapitalDotComEndpoints for CapitalDotComAPI {
    async fn get_server_time(&self) -> Result<Response, CapitalDotComError<Response>> {
        match self
            .http_client
            .get(Self::get_url(&self, "/api/v1/time"))
            .send()
            .await
        {
            Ok(response) => Ok(response),
            Err(e) => Err(CapitalDotComError::ReqwestError(e)),
        }
    }

    async fn ping(&self) -> Result<Response, CapitalDotComError<Response>> {
        self.has_credentials()?;

        match self
            .http_client
            .get(Self::get_url(&self, "/api/v1/ping"))
            .headers(self.auth_header_map.clone())
            .send()
            .await
        {
            Ok(response) => Ok(response),
            Err(e) => Err(CapitalDotComError::ReqwestError(e)),
        }
    }

    async fn get_encryption_key(&self) -> Result<Response, CapitalDotComError<Response>> {
        match self
            .http_client
            .get(Self::get_url(&self, "/api/v1/session/encryptionKey"))
            .header("X-CAP-API-KEY", &self.x_cap_api_key)
            .send()
            .await
        {
            Ok(response) => Ok(response),
            Err(e) => Err(CapitalDotComError::ReqwestError(e)),
        }
    }

    async fn get_session_details(&self) -> Result<Response, CapitalDotComError<Response>> {
        self.has_credentials()?;

        match self
            .http_client
            .get(Self::get_url(&self, "/api/v1/session"))
            .headers(self.auth_header_map.clone())
            .send()
            .await
        {
            Ok(response) => Ok(response),
            Err(e) => Err(CapitalDotComError::ReqwestError(e)),
        }
    }

    async fn create_new_session(&mut self) -> Result<Response, CapitalDotComError<Response>> {
        let response = match self
            .http_client
            .post(Self::get_url(&self, "/api/v1/session/encryptionKey"))
            .header("X-CAP-API-KEY", &self.x_cap_api_key)
            .send()
            .await
        {
            Ok(response) => response,
            Err(e) => return Err(CapitalDotComError::ReqwestError(e)),
        };

        // Update authorization values
        self.update_auth(&response);

        Ok(response)
    }

    async fn switch_active_account(
        &self,
        account_id: String,
    ) -> Result<Response, CapitalDotComError<Response>> {
        self.has_credentials()?;

        let body =
            Self::get_json_from_value(request_bodies::SwitchActiveAccountBody::new(account_id))?;

        match self
            .http_client
            .put(Self::get_url(&self, "/api/v1/session"))
            .headers(self.auth_header_map.clone())
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
        {
            Ok(response) => Ok(response),
            Err(e) => Err(CapitalDotComError::ReqwestError(e)),
        }
    }

    async fn session_log_out(&self) -> Result<Response, CapitalDotComError<Response>> {
        self.has_credentials()?;

        match self
            .http_client
            .delete(Self::get_url(&self, "/api/v1/session"))
            .headers(self.auth_header_map.clone())
            .send()
            .await
        {
            Ok(response) => Ok(response),
            Err(e) => Err(CapitalDotComError::ReqwestError(e)),
        }
    }

    async fn get_all_accounts(&self) -> Result<Response, CapitalDotComError<Response>> {
        self.has_credentials()?;

        match self
            .http_client
            .get(Self::get_url(&self, "/api/v1/accounts"))
            .headers(self.auth_header_map.clone())
            .send()
            .await
        {
            Ok(response) => Ok(response),
            Err(e) => Err(CapitalDotComError::ReqwestError(e)),
        }
    }

    async fn order_confirmation(
        &self,
        deal_reference: String,
    ) -> Result<Response, CapitalDotComError<Response>> {
        self.has_credentials()?;

        match self
            .http_client
            .get(Self::get_url(
                &self,
                &format!("/api/v1/confirms/{}", deal_reference),
            ))
            .headers(self.auth_header_map.clone())
            .send()
            .await
        {
            Ok(response) => Ok(response),
            Err(e) => Err(CapitalDotComError::ReqwestError(e)),
        }
    }

    async fn get_all_positions(&self) -> Result<Response, CapitalDotComError<Response>> {
        self.has_credentials()?;

        match self
            .http_client
            .get(Self::get_url(&self, "/api/v1/positions"))
            .headers(self.auth_header_map.clone())
            .send()
            .await
        {
            Ok(response) => Ok(response),
            Err(e) => Err(CapitalDotComError::ReqwestError(e)),
        }
    }

    async fn create_position(
        &self,
        position_data: request_bodies::CreatePositionBody,
    ) -> Result<Response, CapitalDotComError<Response>> {
        self.has_credentials()?;

        let body = Self::get_json_from_value(position_data)?;

        match self
            .http_client
            .post(Self::get_url(&self, "/api/v1/positions"))
            .headers(self.auth_header_map.clone())
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
        {
            Ok(response) => Ok(response),
            Err(e) => Err(CapitalDotComError::ReqwestError(e)),
        }
    }

    async fn get_position(
        &self,
        deal_id: String,
    ) -> Result<Response, CapitalDotComError<Response>> {
        self.has_credentials()?;

        match self
            .http_client
            .get(Self::get_url(
                &self,
                &format!("/api/v1/positions/{}", deal_id),
            ))
            .headers(self.auth_header_map.clone())
            .send()
            .await
        {
            Ok(response) => Ok(response),
            Err(e) => Err(CapitalDotComError::ReqwestError(e)),
        }
    }

    async fn update_position(
        &self,
        deal_id: String,
        position_update_data: request_bodies::PositionUpdateBody,
    ) -> Result<Response, CapitalDotComError<Response>> {
        self.has_credentials()?;

        let body = Self::get_json_from_value(position_update_data)?;

        match self
            .http_client
            .put(Self::get_url(
                &self,
                &format!("/api/v1/positions/{}", deal_id),
            ))
            .headers(self.auth_header_map.clone())
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
        {
            Ok(response) => Ok(response),
            Err(e) => Err(CapitalDotComError::ReqwestError(e)),
        }
    }

    async fn close_position(
        &self,
        deal_id: String,
    ) -> Result<Response, CapitalDotComError<Response>> {
        self.has_credentials()?;

        match self
            .http_client
            .delete(Self::get_url(
                &self,
                &format!("/api/v1/positions/{}", deal_id),
            ))
            .headers(self.auth_header_map.clone())
            .send()
            .await
        {
            Ok(response) => Ok(response),
            Err(e) => Err(CapitalDotComError::ReqwestError(e)),
        }
    }

    async fn get_market_details(
        &self,
        search_term: String,
        epics: Vec<String>,
    ) -> Result<Response, CapitalDotComError<Response>> {
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

        let mut request = self
            .http_client
            .get(Self::get_url(&self, "/api/v1/markets"))
            .headers(self.auth_header_map.clone());

        if epic_query.is_empty() {
            request = request.query(&[("searchTerm", search_term)]);
        } else {
            request = request.query(&[("searchTerm", search_term), ("epics", epic_query)]);
        }

        match request.send().await {
            Ok(response) => Ok(response),
            Err(e) => Err(CapitalDotComError::ReqwestError(e)),
        }
    }

    async fn single_market_details(
        &self,
        epic: String,
    ) -> Result<Response, CapitalDotComError<Response>> {
        self.has_credentials()?;

        match self
            .http_client
            .get(Self::get_url(&self, &format!("/api/v1/markets/{}", epic)))
            .headers(self.auth_header_map.clone())
            .send()
            .await
        {
            Ok(response) => Ok(response),
            Err(e) => Err(CapitalDotComError::ReqwestError(e)),
        }
    }

    async fn get_historical_prices(
        &self,
        epic: String,
        resolution: enums::Resolution,
        max: i32,
        from: String,
        to: String,
    ) -> Result<Response, CapitalDotComError<Response>> {
        self.has_credentials()?;

        match self
            .http_client
            .get(Self::get_url(&self, &format!("/api/v1/prices/{}", epic)))
            .query(&[
                ("resolution", Self::get_json_from_value(resolution)?),
                ("max", max.to_string()),
                ("from", from),
                ("to", to),
            ])
            .headers(self.auth_header_map.clone())
            .send()
            .await
        {
            Ok(response) => Ok(response),
            Err(e) => Err(CapitalDotComError::ReqwestError(e)),
        }
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
    StatusCode(StatusCode, ErrD),
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
