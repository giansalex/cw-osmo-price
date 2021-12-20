use serde::{Deserialize, Serialize};

use cosmwasm_std::Timestamp;
use cw_storage_plus::Map;

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
pub struct AccountData {
    pub last_update_time: Timestamp,
    pub remote_spot_price: String,
}

pub const ACCOUNTS_INFO: Map<&str, AccountData> = Map::new("accounts");
