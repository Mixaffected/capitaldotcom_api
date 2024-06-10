use std::{collections::HashMap, os::unix::fs::chroot};

use chrono::{DateTime, Utc};
use reqwest::RequestBuilder;
use serde::Deserialize;

use crate::*;

pub trait ReqwestUtils {
    /// Return the body T. Checks the status code.
    async fn get_body<T: for<'a> Deserialize<'a>>(
        response: reqwest::Response,
    ) -> Result<T, CapitalDotComError> {
        let status_code = response.status().as_u16();

        // get body
        let body_raw = match response.text().await {
            Ok(body) => body,
            Err(e) => return Err(CapitalDotComError::ReqwestError(e)),
        };

        if status_code == 200 {
            // json to rust struct
            match serde_json::from_str(&body_raw) {
                Ok(body) => Ok(body),
                Err(e) => return Err(CapitalDotComError::JsonError(e)),
            }
        } else {
            return Err(CapitalDotComError::StatusCode(
                status_code,
                Self::get_value_from_json(&body_raw)?,
                body_raw,
            ));
        }
    }

    /// Serialize an object into a string
    fn get_json_from_value<T: Serialize>(value: T) -> Result<String, CapitalDotComError> {
        match serde_json::to_string(&value) {
            Ok(json) => Ok(json),
            Err(e) => Err(CapitalDotComError::JsonError(e)),
        }
    }

    fn get_value_from_json<T: for<'a> Deserialize<'a>>(
        json: &str,
    ) -> Result<T, CapitalDotComError> {
        match serde_json::from_str(&json) {
            Ok(api_error) => Ok(api_error),
            Err(e) => return Err(CapitalDotComError::JsonError(e)),
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
    fn open_session(&mut self) -> Result<responses::CreateNewSessionResponse, CapitalDotComError>;

    /// Get informations about the current account
    fn get_session_details(&self) -> Result<responses::SessionDetailsResponse, CapitalDotComError>;

    fn get_all_accounts(&self) -> Result<responses::AllAccountsResponse, CapitalDotComError>;

    /// Switch the trading account
    fn switch_account(
        &mut self,
        account_id: String,
    ) -> Result<responses::SwitchAccountResponse, CapitalDotComError>;

    /// Log out of the session
    fn close_session(&self) -> Result<responses::SessionLogOutResponse, CapitalDotComError>;

    fn search_market(
        &self,
        search_term: String,
        epic: Vec<String>,
    ) -> Result<responses::MarketDetailsResponse, CapitalDotComError>;

    /// Get current bid and ask prices and other market data
    fn get_market_data(
        &self,
        epic: String,
    ) -> Result<responses::SingleMarketDetailsResponse, CapitalDotComError>;

    fn get_all_positions(&self) -> Result<responses::AllPositionsResponse, CapitalDotComError>;

    fn open_position(
        &self,
        position_data: request_bodies::CreatePositionBody,
    ) -> Result<responses::DealReferenceResponse, CapitalDotComError>;

    fn position_data(
        &self,
        deal_id: String,
    ) -> Result<responses::PositionResponse, CapitalDotComError>;

    fn close_position(
        &self,
        deal_id: String,
    ) -> Result<responses::DealReferenceResponse, CapitalDotComError>;

    fn get_historical_prices(
        &self,
        epic: String,
        resolution: enums::Resolution,
        max: i32,
        from: chrono::DateTime<chrono::Utc>,
        to: chrono::DateTime<chrono::Utc>,
    ) -> Result<responses::HistoricalPricesResponse, CapitalDotComError>;
}

pub trait CapitalDotComEndpoints: ReqwestUtils {
    async fn get_server_time(
        &mut self,
    ) -> Result<(HashMap<String, String>, responses::ServerTimeResponse), CapitalDotComError>;

    async fn ping(
        &mut self,
    ) -> Result<(HashMap<String, String>, responses::PingResponse), CapitalDotComError>;

    async fn get_encryption_key(
        &mut self,
    ) -> Result<(HashMap<String, String>, responses::EncryptionKeyResponse), CapitalDotComError>;

    async fn get_session_details(
        &mut self,
    ) -> Result<(HashMap<String, String>, responses::SessionDetailsResponse), CapitalDotComError>;

    async fn create_new_session(
        &mut self,
    ) -> Result<(HashMap<String, String>, responses::CreateNewSessionResponse), CapitalDotComError>;

    async fn get_all_accounts(
        &mut self,
    ) -> Result<(HashMap<String, String>, responses::AllAccountsResponse), CapitalDotComError>;

    async fn switch_active_account(
        &mut self,
        account_id: String,
    ) -> Result<(HashMap<String, String>, responses::SwitchAccountResponse), CapitalDotComError>;

    async fn session_log_out(
        &mut self,
    ) -> Result<(HashMap<String, String>, responses::SessionLogOutResponse), CapitalDotComError>;

    /// Check if order was accepted
    async fn order_confirmation(
        &mut self,
        deal_reference: String,
    ) -> Result<
        (
            HashMap<String, String>,
            responses::OrderConfirmationResponse,
        ),
        CapitalDotComError,
    >;

    async fn get_all_positions(
        &mut self,
    ) -> Result<(HashMap<String, String>, responses::AllPositionsResponse), CapitalDotComError>;

    async fn open_position(
        &mut self,
        position_data: request_bodies::CreatePositionBody,
    ) -> Result<(HashMap<String, String>, responses::DealReferenceResponse), CapitalDotComError>;

    async fn get_position(
        &mut self,
        deal_id: String,
    ) -> Result<(HashMap<String, String>, responses::PositionResponse), CapitalDotComError>;

    async fn update_position(
        &mut self,
        deal_id: String,
        position_update_data: request_bodies::PositionUpdateBody,
    ) -> Result<(HashMap<String, String>, responses::DealReferenceResponse), CapitalDotComError>;

    async fn close_position(
        &mut self,
        deal_id: String,
    ) -> Result<(HashMap<String, String>, responses::DealReferenceResponse), CapitalDotComError>;

    async fn get_market_details(
        &mut self,
        search_term: String,
        epics: Vec<String>,
    ) -> Result<(HashMap<String, String>, responses::MarketDetailsResponse), CapitalDotComError>;

    /// Get detail from one market (Tesla for example)
    async fn get_single_market_details(
        &mut self,
        epic: String,
    ) -> Result<
        (
            HashMap<String, String>,
            responses::SingleMarketDetailsResponse,
        ),
        CapitalDotComError,
    >;

    /// from is the Start date. Date format: YYYY-MM-DDTHH:MM:SS (e.g. 2022-04-01T01:01:00). Filtration by date based on snapshotTimeUTC parameter.
    /// to is the End date. Date format: YYYY-MM-DDTHH:MM:SS (e.g. 2022-04-01T01:01:00). Filtration by date based on snapshotTimeUTC parameter.
    async fn get_historical_prices(
        &mut self,
        epic: String,
        resolution: enums::Resolution,
        max: i32,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<(HashMap<String, String>, responses::HistoricalPricesResponse), CapitalDotComError>;

    fn has_credentials(&self) -> Result<(), CapitalDotComError>;

    /// Unwrap the response of the API to the status code, headers and the body that will be casted into the fitting response struct.
    async fn request_data<T: for<'a> Deserialize<'a>>(
        request_builder: RequestBuilder,
    ) -> Result<(HashMap<String, String>, T), CapitalDotComError> {
        let response = match request_builder.send().await {
            Ok(response) => response,
            Err(e) => return Err(CapitalDotComError::ReqwestError(e)),
        };

        let headers = Self::headers_to_hashmap(response.headers().to_owned());
        let body = Self::get_body(response).await?;

        Ok((headers.await, body))
    }

    fn get_readable_from_datetime(datetime: DateTime<Utc>) -> String {
        datetime.format("%Y-%m-%dT%H:%M:%S").to_string()
    }
}
