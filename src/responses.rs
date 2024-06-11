use serde::Deserialize;

use crate::enums;

type Timestamp = i64;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct APIError {
    pub error_code: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerTimeResponse {
    pub server_time: Timestamp,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PingResponse {
    pub status: Status,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncryptionKeyResponse {
    pub encryption_key: String,
    pub time_stamp: Timestamp,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateNewSessionResponse {
    pub account_type: enums::AccountType,
    pub account_info: BalanceAccountInfo,
    pub currency_iso_code: enums::Currency,
    pub currency_symbol: char,
    pub current_account_id: String,
    pub streaming_host: String,
    pub accounts: Vec<Account>,
    pub client_id: String,
    pub timezone_offset: i8,
    pub has_active_demo_accounts: bool,
    pub has_active_live_accounts: bool,
    pub trailing_stops_enabled: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceAccountInfo {
    pub balance: f32,
    pub deposit: f32,
    pub profit_loss: f32,
    pub available: f32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub account_id: String,
    pub account_name: String,
    pub preferred: bool,
    pub account_type: enums::AccountType,
    pub currency: enums::Currency,
    pub symbol: char,
    pub balance: BalanceAccountInfo,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionDetailsResponse {
    pub client_id: String,
    pub account_id: String,
    pub timezone_offset: i8,
    pub locale: enums::Locale,
    pub currency: enums::Currency,
    pub stream_endpoint: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwitchAccountResponse {
    pub trailing_stops_enabled: bool,
    pub dealing_enabled: bool,
    pub has_active_demo_accounts: bool,
    pub has_active_live_accounts: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionLogOutResponse {
    pub status: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllAccountsResponse {
    pub accounts: Vec<StatusAccount>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusAccount {
    pub account_id: String,
    pub account_name: String,
    pub status: enums::AccountStatus,
    pub account_type: enums::AccountType,
    pub preferred: bool,
    pub balance: BalanceAccountInfo,
    pub currency: enums::Currency,
    pub symbol: char,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderConfirmationResponse {
    pub date: String,
    pub status: Status,
    pub deal_status: DealStatus,
    pub epic: String,
    pub deal_reference: String,
    pub deal_id: String,
    pub affected_deals: Vec<AffectedDeal>,
    pub level: f64,
    pub size: f64,
    pub direction: enums::Direction,
    pub guaranteed_stop: bool,
    pub trailing_stop: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AffectedDeal {
    pub deal_id: String,
    pub status: Status,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllPositionsResponse {
    pub positions: Vec<PositionResponse>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionResponse {
    pub position: PositionData,
    pub market: MarketPosition,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionData {
    pub contract_size: i32,
    pub created_date: String,
    pub created_date_UTC: String,
    pub deal_id: String,
    pub deal_reference: String,
    pub working_order_id: String,
    pub size: f32,
    pub leverage: i8,
    pub upl: f32,
    pub direction: enums::Direction,
    pub level: f32,
    pub currency: enums::Currency,
    pub guaranteed_stop: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketPosition {
    pub instrument_name: String,
    pub expiry: String,
    pub market_status: MarketStatus,
    pub epic: String,
    pub symbol: String,
    pub instrument_type: enums::InstrumentType,
    pub lot_size: i32,
    pub high: f32,
    pub low: f32,
    pub percentage_change: f32,
    pub net_change: f32,
    pub bid: f32,
    pub offer: f32,
    pub update_time: String,
    pub update_time_UTC: String,
    pub delay_time: f32,
    pub streaming_prices_available: bool,
    pub scaling_factor: f32,
    pub market_modes: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Market {
    pub delay_time: f64,
    pub epic: String,
    pub symbol: String,
    pub net_change: f64,
    pub lot_size: i32,
    pub expiry: String,
    pub instrument_type: enums::InstrumentType,
    pub instrument_name: String,
    pub high: f64,
    pub low: f64,
    pub percentage_change: f64,
    pub update_time: String,
    pub update_time_UTC: String,
    pub bid: f64,
    pub offer: f64,
    pub streaming_prices_available: bool,
    pub market_status: MarketStatus,
    pub scaling_factor: i32,
    pub market_modes: Vec<String>,
    pub pip_position: i32,
    pub tick_size: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DealReferenceResponse {
    pub deal_reference: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketDetailsResponse {
    pub markets: Vec<Market>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SingleMarketDetailsResponse {
    pub instrument: Instrument,
    pub dealing_rules: DealingRules,
    pub snapshot: Snapshot,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Instrument {
    pub epic: String,
    pub symbol: String,
    pub expiry: String,
    pub name: String,
    pub lot_size: i32,
    // pub type: enums::InstrumentType,
    pub guaranteed_stop_allowed: bool,
    pub streaming_prices_available: bool,
    pub currency: enums::Currency,
    pub margin_factor: i32,
    pub margin_factor_unit: enums::Unit,
    pub opening_hours: OpeningHours,
    pub overnight_fee: OvernightFee,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DealingRules {
    pub min_step_distance: UnitValue,
    pub min_deal_size: UnitValue,
    pub max_deal_size: UnitValue,
    pub min_size_increment: UnitValue,
    pub min_guaranteed_stop_distance: UnitValue,
    pub min_stop_or_profit_distance: UnitValue,
    pub max_stop_or_profit_distance: UnitValue,
    pub market_order_preference: String,
    pub trailing_stops_preference: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub market_status: MarketStatus,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpeningHours {
    pub mon: Vec<String>,
    pub tue: Vec<String>,
    pub wed: Vec<String>,
    pub thu: Vec<String>,
    pub fri: Vec<String>,
    pub sat: Vec<String>,
    pub sun: Vec<String>,
    pub zone: enums::TimeZone,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OvernightFee {
    pub long_rate: f32,
    pub short_rate: f32,
    pub swap_charge_timestamp: Timestamp,
    pub swap_charge_interval: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnitValue {
    pub unit: enums::Unit,
    pub value: f32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoricalPricesResponse {
    pub prices: Vec<Prices>,
    pub instrument_type: enums::InstrumentType,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Prices {
    pub snapshot_time: String,
    pub snapshot_time_UTC: String,
    pub open_price: Price,
    pub close_price: Price,
    pub high_price: Price,
    pub low_price: Price,
    pub last_traded_volume: f32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Price {
    pub bid: f32,
    pub ask: f32,
}

#[derive(Debug, Deserialize)]
pub enum MarketStatus {
    TRADEABLE,
    CLOSED,
}

#[derive(Debug, Deserialize)]
pub enum Status {
    OPEN,
    OPENED,
    PENDING,
}

#[derive(Debug, Deserialize)]
pub enum DealStatus {
    ACCEPTED,
}
