use std::collections::HashMap;

use reqwest::RequestBuilder;
use serde::Deserialize;

use crate::*;

pub trait ReqwestUtils {
    async fn get_body<T: for<'a> Deserialize<'a>>(
        response: reqwest::Response,
    ) -> Result<T, CapitalDotComError<T>> {
        // get body
        let body_raw = match response.text().await {
            Ok(body) => body,
            Err(e) => return Err(CapitalDotComError::ReqwestError(e)),
        };

        // json to rust struct
        let body: T = match serde_json::from_str(&body_raw) {
            Ok(body) => body,
            Err(e) => return Err(CapitalDotComError::JsonError(e)),
        };

        Ok(body)
    }

    /// Serialize an object into a string
    fn get_json_from_value<T: Serialize, E>(value: T) -> Result<String, CapitalDotComError<E>> {
        match serde_json::to_string(&value) {
            Ok(json) => Ok(json),
            Err(e) => Err(CapitalDotComError::JsonError(e)),
        }
    }

    async fn headers_to_hashmap(headers: HeaderMap) -> HashMap<String, String> {
        let mut hash_map = HashMap::new();
        for (header_name, header_value) in headers {
            let header_name = match header_name {
                Some(header_name) => header_name,
                None => continue,
            };

            hash_map.insert(
                header_name.to_string(),
                String::from_utf8_lossy(header_value.as_bytes()).to_string(),
            );
        }

        hash_map
    }
}

pub trait CapitalDotComInterface {
    /// Start a new session and connect to the Capital.com API
    fn login_into_session(&self);

    /// Get informations about the current account
    fn get_session_details(&self);

    /// Switch the trading account
    fn switch_account(&mut self, account_id: String);

    /// Log out of the session
    fn log_out_session(&self);

    fn search_market(&self, search_term: String, epic: Vec<String>);

    /// Get current bid and ask prices and other market data
    fn get_market_data(&self, epic: String);

    fn open_position(&self, position_data: request_bodies::CreatePositionBody);

    fn position_data(&self, deal_id: String);

    fn close_position(&self, deal_id: String);

    fn get_historical_prices(&self);
}

pub trait CapitalDotComEndpoints: ReqwestUtils {
    async fn get_server_time(
        &self,
    ) -> Result<
        (u16, HashMap<String, String>, responses::ServerTimeResponse),
        CapitalDotComError<responses::ServerTimeResponse>,
    >;

    async fn ping(
        &self,
    ) -> Result<
        (u16, HashMap<String, String>, responses::PingResponse),
        CapitalDotComError<responses::PingResponse>,
    >;

    async fn get_encryption_key(
        &self,
    ) -> Result<
        (
            u16,
            HashMap<String, String>,
            responses::EncryptionKeyResponse,
        ),
        CapitalDotComError<responses::EncryptionKeyResponse>,
    >;

    async fn get_session_details(
        &self,
    ) -> Result<
        (
            u16,
            HashMap<String, String>,
            responses::SessionDetailsResponse,
        ),
        CapitalDotComError<responses::SessionDetailsResponse>,
    >;

    async fn create_new_session(
        &mut self,
    ) -> Result<
        (
            u16,
            HashMap<String, String>,
            responses::CreateNewSessionResponse,
        ),
        CapitalDotComError<responses::CreateNewSessionResponse>,
    >;

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
    >;

    async fn session_log_out(
        &self,
    ) -> Result<
        (
            u16,
            HashMap<String, String>,
            responses::SessionLogOutResponse,
        ),
        CapitalDotComError<responses::SessionLogOutResponse>,
    >;

    async fn get_all_accounts(
        &self,
    ) -> Result<
        (u16, HashMap<String, String>, responses::AllAccountsResponse),
        CapitalDotComError<responses::AllAccountsResponse>,
    >;

    /// Check if order was accepted
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
    >;

    async fn get_all_positions(
        &self,
    ) -> Result<
        (
            u16,
            HashMap<String, String>,
            responses::AllPositionsResponse,
        ),
        CapitalDotComError<responses::AllPositionsResponse>,
    >;

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
    >;

    async fn get_position(
        &self,
        deal_id: String,
    ) -> Result<
        (u16, HashMap<String, String>, responses::PositionResponse),
        CapitalDotComError<responses::PositionResponse>,
    >;

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
    >;

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
    >;

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
    >;

    /// Get detail from one market (Tesla for example)
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
    >;

    /// from is the Start date. Date format: YYYY-MM-DDTHH:MM:SS (e.g. 2022-04-01T01:01:00). Filtration by date based on snapshotTimeUTC parameter.
    /// to is the End date. Date format: YYYY-MM-DDTHH:MM:SS (e.g. 2022-04-01T01:01:00). Filtration by date based on snapshotTimeUTC parameter.
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
    >;

    fn has_credentials<T>(&self) -> Result<(), CapitalDotComError<T>>;

    /// Unwrap the response of the API to the status code, headers and the body that will be casted into the fitting response struct.
    async fn request_data<T: for<'a> Deserialize<'a>>(
        request_builder: RequestBuilder,
    ) -> Result<(u16, HashMap<String, String>, T), CapitalDotComError<T>> {
        let response = match request_builder.send().await {
            Ok(response) => response,
            Err(e) => return Err(CapitalDotComError::ReqwestError(e)),
        };

        let headers = Self::headers_to_hashmap(response.headers().to_owned());
        let status = response.status().as_u16();
        let body = Self::get_body(response).await?;

        if status == 200 {
            Ok((status, headers.await, body))
        } else {
            return Err(CapitalDotComError::StatusCode(status, body));
        }
    }
}
