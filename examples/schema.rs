use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use ibc_gamm::ibc_msg::PacketMsg;
use ibc_gamm::msg::{
    AccountResponse, EstimateSwapMsg, ExecuteMsg, InstantiateMsg, ListAccountsResponse, QueryMsg,
    SpotPriceMsg,
};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(PacketMsg), &out_dir);
    export_schema(&schema_for!(EstimateSwapMsg), &out_dir);
    export_schema(&schema_for!(SpotPriceMsg), &out_dir);
    export_schema(&schema_for!(AccountResponse), &out_dir);
    export_schema(&schema_for!(ListAccountsResponse), &out_dir);
}
