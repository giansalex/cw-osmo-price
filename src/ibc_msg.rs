use cosmwasm_std::Binary;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// This is the message we send over the IBC channel
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct PacketMsg {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    pub path: String,
    pub data: Binary,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum PacketAck {
    Result(Binary),
    Error(String),
}
