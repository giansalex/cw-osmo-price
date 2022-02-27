use cosmwasm_std::{Binary, Uint64};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// This is the message we send over the IBC channel
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PacketMsg {
    SpotPrice(GammPricePacket),
    EstimateSwapAmountIn(EstimateSwapAmountInPacket),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GammPricePacket {
    pub pool_id: Uint64,
    pub token_in: String,
    pub token_out: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct EstimateSwapAmountInPacket {
    pub sender: String,
    pub pool_id: Uint64,
    pub token_in: String,
    pub routes: Vec<SwapAmountInRoute>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SwapAmountInRoute {
    pub pool_id: Uint64,
    pub token_out_denom: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum PacketAck {
    Result(Binary),
    Error(String),
}

/// This is the success response we send on ack for PacketMsg::Balance.
/// Just acknowledge success or error
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PriceResponse {
    pub price: String,
}
