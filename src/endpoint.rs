use std::collections::HashMap;

use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;

use crate::enums;
use crate::request_bodies;
use crate::responses;
use crate::traits::{self, ReqwestUtils};
use crate::CapitalDotComError;

#[derive(Debug)]
pub struct CapitalDotComApiEndpoints {
    base_url: String,

    x_cap_api_key: String,
    x_security_token: String, // Needs to be requested
    cst: String,              // Needs to be requested
    identifier: String,
    password: String,
    encryption_key: String, // TODO: Implement encryption.
    auth_header_map: HeaderMap,

    http_client: reqwest::Client,
}
impl CapitalDotComApiEndpoints {
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

    fn update_auth(&mut self, headers: HashMap<String, String>) {
        self.x_security_token = match headers.get("x-security-token") {
            Some(x_security_token) => x_security_token.to_owned(),
            None => String::new(),
        };

        self.cst = match headers.get("cst") {
            Some(cst) => cst.to_owned(),
            None => String::new(),
        };

        let mut header_map = HeaderMap::new();
        header_map.append(
            "x-security-token",
            HeaderValue::from_str(&self.x_security_token).expect("x_security_token too large!"),
        );
        header_map.append(
            "cst",
            HeaderValue::from_str(&self.cst).expect("cst too large!"),
        );

        self.auth_header_map = header_map;
    }
}

impl traits::CapitalDotComEndpoints for CapitalDotComApiEndpoints {
    async fn get_server_time(
        &mut self,
    ) -> Result<(HashMap<String, String>, responses::ServerTimeResponse), CapitalDotComError> {
        let request_builder = self.http_client.get(Self::get_url(&self, "/api/v1/time"));

        Self::request_data(request_builder).await
    }

    async fn ping(
        &mut self,
    ) -> Result<(HashMap<String, String>, responses::PingResponse), CapitalDotComError> {
        self.has_credentials()?;

        let request_builder = self
            .http_client
            .get(Self::get_url(&self, "/api/v1/ping"))
            .headers(self.auth_header_map.clone());

        Self::request_data(request_builder).await
    }

    async fn get_encryption_key(
        &mut self,
    ) -> Result<(HashMap<String, String>, responses::EncryptionKeyResponse), CapitalDotComError>
    {
        let request_builder = self
            .http_client
            .get(Self::get_url(&self, "/api/v1/session/encryptionKey"))
            .header("X-CAP-API-KEY", &self.x_cap_api_key);

        Self::request_data(request_builder).await
    }

    async fn get_session_details(
        &mut self,
    ) -> Result<(HashMap<String, String>, responses::SessionDetailsResponse), CapitalDotComError>
    {
        self.has_credentials()?;

        let request_builder = self
            .http_client
            .get(Self::get_url(&self, "/api/v1/session"))
            .headers(self.auth_header_map.clone());

        Self::request_data(request_builder).await
    }

    async fn create_new_session(
        &mut self,
    ) -> Result<(HashMap<String, String>, responses::CreateNewSessionResponse), CapitalDotComError>
    {
        let body = Self::get_json_from_value(request_bodies::CreateSessionBody::new(
            &self.identifier,
            &self.password,
        ))?;

        let request_builder = self
            .http_client
            .post(Self::get_url(&self, "/api/v1/session"))
            .header("X-CAP-API-KEY", &self.x_cap_api_key)
            .header("Content-Type", "application/json")
            .body(body);

        let (headers, body) = Self::request_data(request_builder).await?;

        // Update authorization values
        self.update_auth(headers.clone());

        Ok((headers, body))
    }

    async fn get_all_accounts(
        &mut self,
    ) -> Result<(HashMap<String, String>, responses::AllAccountsResponse), CapitalDotComError> {
        self.has_credentials()?;

        let request_builder = self
            .http_client
            .get(Self::get_url(&self, "/api/v1/accounts"))
            .headers(self.auth_header_map.clone());

        Self::request_data(request_builder).await
    }

    async fn switch_active_account(
        &mut self,
        account_id: &str,
    ) -> Result<(HashMap<String, String>, responses::SwitchAccountResponse), CapitalDotComError>
    {
        self.has_credentials()?;

        let body = Self::get_json_from_value(request_bodies::SwitchActiveAccountBody::new(
            account_id.to_string(),
        ))?;

        let request_builder = self
            .http_client
            .put(Self::get_url(&self, "/api/v1/session"))
            .headers(self.auth_header_map.clone())
            .header("Content-Type", "application/json")
            .body(body);

        Self::request_data(request_builder).await
    }

    async fn session_log_out(
        &mut self,
    ) -> Result<(HashMap<String, String>, responses::SessionLogOutResponse), CapitalDotComError>
    {
        self.has_credentials()?;

        let request_builder = self
            .http_client
            .delete(Self::get_url(&self, "/api/v1/session"))
            .headers(self.auth_header_map.clone());

        Self::request_data(request_builder).await
    }

    async fn order_confirmation(
        &mut self,
        deal_reference: &str,
    ) -> Result<
        (
            HashMap<String, String>,
            responses::OrderConfirmationResponse,
        ),
        CapitalDotComError,
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
        &mut self,
    ) -> Result<(HashMap<String, String>, responses::AllPositionsResponse), CapitalDotComError>
    {
        self.has_credentials()?;

        let request_builder = self
            .http_client
            .get(Self::get_url(&self, "/api/v1/positions"))
            .headers(self.auth_header_map.clone());

        Self::request_data(request_builder).await
    }

    async fn open_position(
        &mut self,
        position_data: request_bodies::CreatePositionBody,
    ) -> Result<(HashMap<String, String>, responses::DealReferenceResponse), CapitalDotComError>
    {
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
        &mut self,
        deal_id: String,
    ) -> Result<(HashMap<String, String>, responses::PositionResponse), CapitalDotComError> {
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
        &mut self,
        deal_id: String,
        position_update_data: request_bodies::PositionUpdateBody,
    ) -> Result<(HashMap<String, String>, responses::DealReferenceResponse), CapitalDotComError>
    {
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
        &mut self,
        deal_id: String,
    ) -> Result<(HashMap<String, String>, responses::DealReferenceResponse), CapitalDotComError>
    {
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

    /// Search market from search term.
    async fn get_market_details(
        &mut self,
        search_term: &str,
        epics: Vec<String>,
    ) -> Result<(HashMap<String, String>, responses::MarketDetailsResponse), CapitalDotComError>
    {
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
                request_builder.query(&[("searchTerm", search_term), ("epics", &epic_query)]);
        }

        Self::request_data(request_builder).await
    }

    async fn get_single_market_details(
        &mut self,
        epic: String,
    ) -> Result<
        (
            HashMap<String, String>,
            responses::SingleMarketDetailsResponse,
        ),
        CapitalDotComError,
    > {
        self.has_credentials()?;

        let request_builder = self
            .http_client
            .get(Self::get_url(&self, &format!("/api/v1/markets/{}", epic)))
            .headers(self.auth_header_map.clone());

        Self::request_data(request_builder).await
    }

    async fn get_historical_prices(
        &mut self,
        epic: String,
        resolution: enums::Resolution,
        max: Option<i32>,
        from: chrono::DateTime<chrono::Utc>,
        to: chrono::DateTime<chrono::Utc>,
    ) -> Result<(HashMap<String, String>, responses::HistoricalPricesResponse), CapitalDotComError>
    {
        self.has_credentials()?;

        let mut request_builder = self
            .http_client
            .get(Self::get_url(&self, &format!("/api/v1/prices/{}", epic)))
            .query(&[
                ("resolution", resolution.to_string()),
                ("from", Self::get_readable_from_datetime(from)),
                ("to", Self::get_readable_from_datetime(to)),
            ])
            .headers(self.auth_header_map.clone());

        request_builder = match max {
            Some(max) => request_builder.query(&[("max", max.to_string())]),
            None => request_builder,
        };

        Self::request_data(request_builder).await
    }

    fn has_credentials(&self) -> Result<(), CapitalDotComError> {
        if !self.x_security_token.is_empty() || !self.cst.is_empty() {
            Ok(())
        } else {
            Err(CapitalDotComError::MissingAuthorization)
        }
    }
}

impl ReqwestUtils for CapitalDotComApiEndpoints {}

#[derive(Debug)]
pub enum SessionType {
    Live,
    Demo,
}
