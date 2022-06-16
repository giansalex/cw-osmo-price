use cosmwasm_std::{
    entry_point, from_binary, from_slice, DepsMut, Env, IbcBasicResponse, IbcChannelCloseMsg,
    IbcChannelConnectMsg, IbcChannelOpenMsg, IbcOrder, IbcPacketAckMsg, IbcPacketReceiveMsg,
    IbcPacketTimeoutMsg, IbcReceiveResponse, StdError, StdResult,
};

use crate::ibc_msg::{EstimateSwapAck, GammPacket, PacketAck, PacketMsg, SpotPriceAck};
use crate::state::{AccountData, ACCOUNTS_INFO};

pub const GAMM_VERSION: &str = "cw-query-1";
pub const GAMM_ORDERING: IbcOrder = IbcOrder::Unordered;

/// default one hour
pub const DEFAULT_PACKET_LIFETIME: u64 = 60 * 60;

#[entry_point]
/// enforces ordering and versioing constraints
pub fn ibc_channel_open(_deps: DepsMut, _env: Env, msg: IbcChannelOpenMsg) -> StdResult<()> {
    let channel = msg.channel();

    if channel.order != GAMM_ORDERING {
        return Err(StdError::generic_err("Only supports unordered channels"));
    }

    if channel.version.as_str() != GAMM_VERSION {
        return Err(StdError::generic_err(format!(
            "Must set version to `{}`",
            GAMM_VERSION
        )));
    }

    if let Some(version) = msg.counterparty_version() {
        if version != GAMM_VERSION {
            return Err(StdError::generic_err(format!(
                "Counterparty version must be `{}`",
                GAMM_VERSION
            )));
        }
    }

    Ok(())
}

#[entry_point]
/// once it's established, we send a WhoAmI message
pub fn ibc_channel_connect(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelConnectMsg,
) -> StdResult<IbcBasicResponse> {
    let channel = msg.channel();

    let channel_id = &channel.endpoint.channel_id;

    // create an account holder the channel exists (not found if not registered)
    let data = AccountData::default();
    ACCOUNTS_INFO.save(deps.storage, channel_id, &data)?;

    Ok(IbcBasicResponse::new()
        .add_attribute("action", "ibc_connect")
        .add_attribute("channel_id", channel_id))
}

#[entry_point]
/// On closed channel, simply delete the account from our local store
pub fn ibc_channel_close(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelCloseMsg,
) -> StdResult<IbcBasicResponse> {
    let channel = msg.channel();

    // remove the channel
    let channel_id = &channel.endpoint.channel_id;
    ACCOUNTS_INFO.remove(deps.storage, channel_id);

    Ok(IbcBasicResponse::new()
        .add_attribute("action", "ibc_close")
        .add_attribute("channel_id", channel_id))
}

#[entry_point]
/// never should be called as the other side never sends packets
pub fn ibc_packet_receive(
    _deps: DepsMut,
    _env: Env,
    _packet: IbcPacketReceiveMsg,
) -> StdResult<IbcReceiveResponse> {
    unimplemented!();
}

#[entry_point]
pub fn ibc_packet_ack(
    deps: DepsMut,
    env: Env,
    msg: IbcPacketAckMsg,
) -> StdResult<IbcBasicResponse> {
    // which local channel was this packet send from
    let caller = msg.original_packet.src.channel_id;
    // we need to parse the ack based on our request
    let packet: PacketMsg = from_slice(&msg.original_packet.data)?;
    let ack: PacketAck = from_binary(&msg.acknowledgement.data)?;
    match packet.query {
        GammPacket::SpotPrice(_) => acknowledge_spot_price_result(deps, env, caller, ack),
        GammPacket::EstimateSwap(_) => acknowledge_estimate_swap_result(deps, env, caller, ack),
    }
}

// receive PacketMsg::SpotPrice response
fn acknowledge_spot_price_result(
    deps: DepsMut,
    env: Env,
    caller: String,
    ack: PacketAck,
) -> StdResult<IbcBasicResponse> {
    let result: SpotPriceAck = match ack {
        PacketAck::Result(data) => from_binary(&data)?,
        PacketAck::Error(e) => {
            return Ok(IbcBasicResponse::new()
                .add_attribute("action", "receive_spot_price")
                .add_attribute("error", e))
        }
    };
    ACCOUNTS_INFO.update(deps.storage, &caller, |orig| -> StdResult<_> {
        let mut account = orig.ok_or_else(|| StdError::generic_err("no account to update"))?;
        account.last_update_time = env.block.time;
        account.remote_spot_price = result.price.to_string();
        Ok(account)
    })?;

    Ok(IbcBasicResponse::new()
        .add_attribute("action", "receive_spot_price")
        .add_attribute("amount", result.price.to_string()))
}

fn acknowledge_estimate_swap_result(
    deps: DepsMut,
    env: Env,
    caller: String,
    ack: PacketAck,
) -> StdResult<IbcBasicResponse> {
    let result: EstimateSwapAck = match ack {
        PacketAck::Result(data) => from_binary(&data)?,
        PacketAck::Error(e) => {
            return Ok(IbcBasicResponse::new()
                .add_attribute("action", "receive_estimate_swap")
                .add_attribute("error", e))
        }
    };
    ACCOUNTS_INFO.update(deps.storage, &caller, |orig| -> StdResult<_> {
        let mut account = orig.ok_or_else(|| StdError::generic_err("no account to update"))?;
        account.last_update_time = env.block.time;
        account.remote_spot_price = result.amount.to_string();
        Ok(account)
    })?;

    Ok(IbcBasicResponse::new()
        .add_attribute("action", "receive_estimate_swap")
        .add_attribute("amount", result.amount))
}

#[entry_point]
pub fn ibc_packet_timeout(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketTimeoutMsg,
) -> StdResult<IbcBasicResponse> {
    Ok(IbcBasicResponse::new().add_attribute("action", "ibc_packet_timeout"))
}

pub fn proto_decode<M: prost::Message + std::default::Default>(data: &[u8]) -> StdResult<M> {
    prost::Message::decode(data).map_err(|_| StdError::generic_err("cannot decode proto"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::{instantiate, query};
    use crate::msg::{AccountResponse, InstantiateMsg, QueryMsg};

    use cosmwasm_std::testing::{
        mock_dependencies, mock_env, mock_ibc_channel_connect_ack, mock_ibc_channel_open_init,
        mock_ibc_channel_open_try, mock_info, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{IbcOrder, OwnedDeps};

    const CREATOR: &str = "creator";

    fn setup() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {};
        let info = mock_info(CREATOR, &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
        deps
    }

    // connect will run through the entire handshake to set up a proper connect and
    // save the account (tested in detail in `proper_handshake_flow`)
    fn connect(mut deps: DepsMut, channel_id: &str) {
        let handshake_open =
            mock_ibc_channel_open_init(channel_id, IbcOrder::Unordered, GAMM_VERSION);
        // first we try to open with a valid handshake
        ibc_channel_open(deps.branch(), mock_env(), handshake_open).unwrap();

        // then we connect (with counter-party version set)
        let handshake_connect =
            mock_ibc_channel_connect_ack(channel_id, IbcOrder::Ordered, GAMM_VERSION);
        let res = ibc_channel_connect(deps.branch(), mock_env(), handshake_connect).unwrap();

        // this should send a WhoAmI request, which is received some blocks later
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn enforce_version_in_handshake() {
        let mut deps = setup();

        let wrong_order = mock_ibc_channel_open_try("channel-12", IbcOrder::Ordered, GAMM_VERSION);
        ibc_channel_open(deps.as_mut(), mock_env(), wrong_order).unwrap_err();

        let wrong_version = mock_ibc_channel_open_try("channel-12", IbcOrder::Unordered, "reflect");
        ibc_channel_open(deps.as_mut(), mock_env(), wrong_version).unwrap_err();

        let valid_handshake =
            mock_ibc_channel_open_try("channel-12", IbcOrder::Unordered, GAMM_VERSION);
        ibc_channel_open(deps.as_mut(), mock_env(), valid_handshake).unwrap();
    }

    #[test]
    fn proper_handshake_flow() {
        // setup and connect handshake
        let mut deps = setup();
        let channel_id = "channel-1234";
        connect(deps.as_mut(), channel_id);

        // check for empty account
        let q = QueryMsg::Account {
            channel_id: channel_id.into(),
        };
        let r = query(deps.as_ref(), mock_env(), q).unwrap();
        let acct: AccountResponse = from_slice(&r).unwrap();
        assert!(acct.remote_spot_price.is_empty());
        assert_eq!(0, acct.last_update_time.nanos());

        // account should be set up
        let q = QueryMsg::Account {
            channel_id: channel_id.into(),
        };
        let r = query(deps.as_ref(), mock_env(), q).unwrap();
        let acct: AccountResponse = from_slice(&r).unwrap();
        assert!(acct.remote_spot_price.is_empty());
        assert_eq!(0, acct.last_update_time.nanos());
    }
}
