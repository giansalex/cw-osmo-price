use cosmwasm_std::{Timestamp, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::AccountData;

/// This needs no info. Owner of the contract is whoever signed the InstantiateMsg.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    SpotPrice(SpotPriceMsg),
}

/// This is the message we accept via Receive
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SpotPriceMsg {
    pub channel: String,
    pub pool: Uint128,
    pub token_in: String,
    pub token_out: String,
    /// How long the packet lives in seconds. If not specified, use default_timeout
    pub timeout: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // Shows all open accounts (incl. remote info)
    ListAccounts {},
    // Get account for one channel
    Account { channel_id: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ListAccountsResponse {
    pub accounts: Vec<AccountInfo>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AccountInfo {
    pub channel_id: String,
    pub last_update_time: Timestamp,
    pub remote_spot_price: String,
}

impl AccountInfo {
    pub fn convert(channel_id: String, input: AccountData) -> Self {
        AccountInfo {
            channel_id,
            last_update_time: input.last_update_time,
            remote_spot_price: input.remote_spot_price,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AccountResponse {
    pub last_update_time: Timestamp,
    pub remote_spot_price: String,
}

impl From<AccountData> for AccountResponse {
    fn from(input: AccountData) -> Self {
        AccountResponse {
            last_update_time: input.last_update_time,
            remote_spot_price: input.remote_spot_price,
        }
    }
}
