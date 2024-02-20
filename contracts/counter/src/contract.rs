use cosmwasm_std::{
    coin, entry_point, to_binary, to_json_binary, Binary, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Response, StdError, StdResult, Uint128, WasmMsg,
};
use cw721;
use osmosis_std::types::osmosis::tokenfactory::v1beta1::MsgMint;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{Constants, CONSTANTS};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = Constants {
        token_denom: msg.token_denom.clone(),
        nft_contract: msg.nft_contract.clone(),
    };
    CONSTANTS.save(deps.storage, &state)?;
    Ok(Response::new()
        .add_attribute("action", "initialisation")
        .add_attribute("nft_contract", msg.nft_contract)
        .add_attribute("token_denom", msg.token_denom))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ReceiveNft {
            sender,
            token_id,
            msg: _,
        } => receive_nft(deps, env, info, sender, token_id),
        ExecuteMsg::RedeemNft {
            token_id,
            recipient,
        } => redeem_nft(deps, env, info, token_id, recipient),
    }
}

fn receive_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender: String,
    token_id: String,
) -> Result<Response, ContractError> {
    // Check if the sender is the NFT contract, specified in the constants
    if info.sender != &CONSTANTS.load(deps.storage)?.nft_contract {
        return Err(ContractError::Unauthorized {});
    }

    Ok(Response::new()
        .add_message(MsgMint {
            sender: env.contract.address.to_string(),
            mint_to_address: sender.to_string(),
            amount: Some(coin(1, "TOKEN_DENOM").into()),
        })
        .add_attribute("action", "mint")
        .add_attribute("recipient", sender)
        .add_attribute("token_id", token_id))
}

fn redeem_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
    recipient: String,
) -> Result<Response, ContractError> {
    // Check if the funds attached are in the correct denom and amount
    if info.funds.len() != 1 {
        return Err(ContractError::Std(StdError::generic_err(
            "Must send exactly one coin",
        )));
    }

    let amount = info.funds[0].amount;
    if info.funds[0].denom != CONSTANTS.load(deps.storage)?.token_denom {
        return Err(ContractError::Std(StdError::generic_err(
            "Must send the correct token",
        )));
    };

    // Send the nft with the token id to the recipient
    Ok(
        Response::new().add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: CONSTANTS.load(deps.storage)?.nft_contract.to_string(),
            msg: to_json_binary(&cw721::Cw721ExecuteMsg::TransferNft {
                recipient,
                token_id,
            })?,
            funds: vec![],
        })),
    )
}

// #[cfg_attr(not(feature = "library"), entry_point)]
// pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
//     match msg {
//         QueryMsg::GetCount {} => query_count(deps),
//     }
// }

// pub fn query_count(_deps: Deps) -> StdResult<Binary> {
//     let constant = CONSTANTS.load(_deps.storage)?;
//     to_json_binary()
// }
