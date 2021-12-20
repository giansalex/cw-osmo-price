use serde::{Deserialize, Serialize};

use cosmwasm_std::{Storage, Timestamp};
use cosmwasm_storage::{bucket, bucket_read, Bucket, ReadonlyBucket};

pub const PREFIX_ACCOUNTS: &[u8] = b"accounts";

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
pub struct AccountData {
    pub last_update_time: Timestamp,
    pub remote_spot_price: String,
}

pub fn accounts(storage: &mut dyn Storage) -> Bucket<AccountData> {
    bucket(storage, PREFIX_ACCOUNTS)
}

pub fn accounts_read(storage: &dyn Storage) -> ReadonlyBucket<AccountData> {
    bucket_read(storage, PREFIX_ACCOUNTS)
}
