use cosmwasm_std::{
    entry_point, to_binary, Deps, DepsMut, Env, IbcMsg, MessageInfo, Order, QueryResponse,
    Response, StdError, StdResult,
};

use crate::ibc::DEFAULT_PACKET_LIFETIME;
use crate::ibc_msg::{EstimateSwapPacket, GammPacket, PacketMsg, SpotPricePacket};
use crate::msg::{
    AccountInfo, AccountResponse, EstimateSwapMsg, ExecuteMsg, InstantiateMsg,
    ListAccountsResponse, QueryMsg, SpotPriceMsg,
};
use crate::state::ACCOUNTS_INFO;

#[entry_point]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::SpotPrice(msg) => handle_spot_price(deps, env, msg),
        ExecuteMsg::EstimateSwap(msg) => handle_estimate_swap(deps, env, msg),
    }
}

pub fn handle_spot_price(deps: DepsMut, env: Env, msg: SpotPriceMsg) -> StdResult<Response> {
    // ensure the channel exists (not found if not registered)
    if !ACCOUNTS_INFO.has(deps.storage, &msg.channel) {
        return Err(StdError::generic_err("Channel not found"));
    }

    // delta from user is in seconds
    let timeout_delta = match msg.timeout {
        Some(t) => t,
        None => DEFAULT_PACKET_LIFETIME,
    };
    // timeout is in nanoseconds
    let timeout = env.block.time.plus_seconds(timeout_delta);

    // construct a packet to send
    let packet = PacketMsg {
        client_id: None,
        query: GammPacket::SpotPrice(SpotPricePacket {
            pool: msg.pool,
            token_in: msg.token_in,
            token_out: msg.token_out,
        }),
    };

    let msg = IbcMsg::SendPacket {
        channel_id: msg.channel,
        data: to_binary(&packet)?,
        timeout: timeout.into(),
    };

    let res = Response::new()
        .add_message(msg)
        .add_attribute("action", "spot_price");
    Ok(res)
}

pub fn handle_estimate_swap(deps: DepsMut, env: Env, msg: EstimateSwapMsg) -> StdResult<Response> {
    // ensure the channel exists (not found if not registered)
    if !ACCOUNTS_INFO.has(deps.storage, &msg.channel) {
        return Err(StdError::generic_err("Channel not found"));
    }

    // delta from user is in seconds
    let timeout_delta = match msg.timeout {
        Some(t) => t,
        None => DEFAULT_PACKET_LIFETIME,
    };
    // timeout is in nanoseconds
    let timeout = env.block.time.plus_seconds(timeout_delta);

    // construct a packet to send
    let packet = PacketMsg {
        client_id: None,
        query: GammPacket::EstimateSwap(EstimateSwapPacket {
            pool: msg.pool,
            sender: msg.sender,
            token_in: msg.token_in,
            amount: msg.amount,
            token_out: msg.token_out,
        }),
    };

    let msg = IbcMsg::SendPacket {
        channel_id: msg.channel,
        data: to_binary(&packet)?,
        timeout: timeout.into(),
    };

    let res = Response::new()
        .add_message(msg)
        .add_attribute("action", "estimate_swap");
    Ok(res)
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::Account { channel_id } => to_binary(&query_account(deps, channel_id)?),
        QueryMsg::ListAccounts {} => to_binary(&query_list_accounts(deps)?),
    }
}

fn query_account(deps: Deps, channel_id: String) -> StdResult<AccountResponse> {
    let account = ACCOUNTS_INFO.load(deps.storage, &channel_id)?;
    Ok(account.into())
}

fn query_list_accounts(deps: Deps) -> StdResult<ListAccountsResponse> {
    let accounts: StdResult<Vec<_>> = ACCOUNTS_INFO
        .range(deps.storage, None, None, Order::Ascending)
        .map(|r| {
            let (channel_id, account) = r?;
            Ok(AccountInfo::convert(channel_id, account))
        })
        .collect();
    Ok(ListAccountsResponse {
        accounts: accounts?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    const CREATOR: &str = "creator";

    #[test]
    fn instantiate_works() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {};
        let info = mock_info(CREATOR, &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }
}
