use reqwest::header::HeaderMap;
use serde::Deserialize;

use crate::*;

pub trait Reqwest {
    fn get_status_code(data: &reqwest::Response) -> StatusCode {
        data.status()
    }

    fn get_headers(data: &reqwest::Response) -> HeaderMap {
        data.headers().to_owned()
    }

    async fn get_body<T: for<'a> Deserialize<'a>>(
        data: reqwest::Response,
    ) -> Result<T, CapitalDotComError<T>> {
        let status_code = data.status();

        // get body
        let body_raw = match data.text().await {
            Ok(body) => body,
            Err(e) => return Err(CapitalDotComError::ReqwestError(e)),
        };

        // json to rust struct
        let body: T = match serde_json::from_str(&body_raw) {
            Ok(body) => body,
            Err(e) => return Err(CapitalDotComError::JsonError(e)),
        };

        // check status
        if status_code != StatusCode::OK {
            return Err(CapitalDotComError::StatusCode(status_code, body));
        }

        Ok(body)
    }

    /// Serialize an object into a string
    fn get_json_from_value<T: Serialize, E>(value: T) -> Result<String, CapitalDotComError<E>> {
        match serde_json::to_string(&value) {
            Ok(json) => Ok(json),
            Err(e) => Err(CapitalDotComError::JsonError(e)),
        }
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

pub trait CapitalDotComEndpoints {
    async fn get_server_time(&self) -> Result<Response, CapitalDotComError<Response>>;

    async fn ping(&self) -> Result<Response, CapitalDotComError<Response>>;

    async fn get_encryption_key(&self) -> Result<Response, CapitalDotComError<Response>>;

    async fn get_session_details(&self) -> Result<Response, CapitalDotComError<Response>>;

    async fn create_new_session(&mut self) -> Result<Response, CapitalDotComError<Response>>;

    async fn switch_active_account(
        &self,
        account_id: String,
    ) -> Result<Response, CapitalDotComError<Response>>;

    async fn session_log_out(&self) -> Result<Response, CapitalDotComError<Response>>;

    async fn get_all_accounts(&self) -> Result<Response, CapitalDotComError<Response>>;

    /// Check if order was accepted
    async fn order_confirmation(
        &self,
        deal_reference: String,
    ) -> Result<Response, CapitalDotComError<Response>>;

    async fn get_all_positions(&self) -> Result<Response, CapitalDotComError<Response>>;

    async fn create_position(
        &self,
        position_data: request_bodies::CreatePositionBody,
    ) -> Result<Response, CapitalDotComError<Response>>;

    async fn get_position(&self, deal_id: String)
        -> Result<Response, CapitalDotComError<Response>>;

    async fn update_position(
        &self,
        deal_id: String,
        position_update_data: request_bodies::PositionUpdateBody,
    ) -> Result<Response, CapitalDotComError<Response>>;

    async fn close_position(
        &self,
        deal_id: String,
    ) -> Result<Response, CapitalDotComError<Response>>;

    async fn get_market_details(
        &self,
        search_term: String,
        epics: Vec<String>,
    ) -> Result<Response, CapitalDotComError<Response>>;

    /// Get detail from one market (Tesla for example)
    async fn single_market_details(
        &self,
        epic: String,
    ) -> Result<Response, CapitalDotComError<Response>>;

    /// from is the Start date. Date format: YYYY-MM-DDTHH:MM:SS (e.g. 2022-04-01T01:01:00). Filtration by date based on snapshotTimeUTC parameter.
    /// to is the End date. Date format: YYYY-MM-DDTHH:MM:SS (e.g. 2022-04-01T01:01:00). Filtration by date based on snapshotTimeUTC parameter.
    async fn get_historical_prices(
        &self,
        epic: String,
        resolution: enums::Resolution,
        max: i32,
        from: String,
        to: String,
    ) -> Result<Response, CapitalDotComError<Response>>;
}
