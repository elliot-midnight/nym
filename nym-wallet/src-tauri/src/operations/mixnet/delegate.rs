use std::collections::HashMap;
use std::sync::Arc;

use cosmwasm_std::{Coin as CosmWasmCoin, Uint128};
use tokio::sync::RwLock;

use mixnet_contract_common::IdentityKey;
use nym_types::currency::{CurrencyDenom, MajorCurrencyAmount};
use nym_types::delegation::{
  from_contract_delegation_events, Delegation, DelegationEvent, DelegationRecord, DelegationResult,
  DelegationWithEverything, DelegationsSummaryResponse,
};
use nym_types::error::TypesError;

use crate::error::BackendError;
use crate::state::State;
use crate::{api_client, nymd_client};

#[tauri::command]
pub async fn get_pending_delegation_events(
  state: tauri::State<'_, Arc<RwLock<State>>>,
) -> Result<Vec<DelegationEvent>, BackendError> {
  let events = nymd_client!(state)
    .get_pending_delegation_events(nymd_client!(state).address().to_string(), None)
    .await?;

  match from_contract_delegation_events(events) {
    Ok(res) => Ok(res),
    Err(e) => Err(e.into()),
  }
}

#[tauri::command]
pub async fn delegate_to_mixnode(
  identity: &str,
  amount: MajorCurrencyAmount,
  state: tauri::State<'_, Arc<RwLock<State>>>,
) -> Result<DelegationResult, BackendError> {
  let delegation: CosmWasmCoin = amount.clone().into_minor_cosmwasm_coin()?;
  nymd_client!(state)
    .delegate_to_mixnode(identity, &delegation)
    .await?;
  Ok(DelegationResult::new(
    nymd_client!(state).address().as_ref(),
    identity,
    Some(amount),
  ))
}

#[tauri::command]
pub async fn undelegate_from_mixnode(
  identity: &str,
  state: tauri::State<'_, Arc<RwLock<State>>>,
) -> Result<DelegationResult, BackendError> {
  nymd_client!(state)
    .remove_mixnode_delegation(identity)
    .await?;
  Ok(DelegationResult::new(
    nymd_client!(state).address().as_ref(),
    identity,
    None,
  ))
}

struct DelegationWithHistory {
  pub delegation: Delegation,
  pub amount_sum: MajorCurrencyAmount,
  pub history: Vec<DelegationRecord>,
}

#[tauri::command]
pub async fn get_all_mix_delegations(
  state: tauri::State<'_, Arc<RwLock<State>>>,
) -> Result<Vec<DelegationWithEverything>, BackendError> {
  // TODO: add endpoint to validator API to get a single mix node bond
  let mixnodes = api_client!(state).get_mixnodes().await?;

  let address = nymd_client!(state).address().to_string();

  let denom_minor = state.read().await.current_network().denom();

  let delegations = nymd_client!(state)
    .get_delegator_delegations_paged(address.clone(), None, None) // get all delegations, ignoring paging
    .await?
    .delegations;

  let pending_events_for_account = nymd_client!(state)
    .get_pending_delegation_events(address.clone(), None)
    .await?
    .into_iter()
    .map(|e| e.try_into())
    .collect::<Result<Vec<DelegationEvent>, TypesError>>()?;

  let mut map: HashMap<String, DelegationWithHistory> = HashMap::new();

  for d in delegations {
    // create record of delegation
    let delegated_on_iso_datetime = nymd_client!(state)
      .get_block_timestamp(Some(d.block_height as u32))
      .await?
      .to_rfc3339();
    let amount: MajorCurrencyAmount = d.amount.clone().try_into()?;
    let record = DelegationRecord {
      amount: amount.clone(),
      block_height: d.block_height,
      delegated_on_iso_datetime,
    };

    let entry = map
      .entry(d.node_identity.clone())
      .or_insert(DelegationWithHistory {
        delegation: d.try_into()?,
        history: vec![],
        amount_sum: MajorCurrencyAmount::zero(&amount.denom),
      });

    entry.history.push(record);
    entry.amount_sum = entry.amount_sum.clone() + amount;
  }

  let mut with_everything: Vec<DelegationWithEverything> = vec![];

  for item in map {
    let d = item.1.delegation;
    let history = item.1.history;
    let Delegation {
      owner,
      node_identity,
      amount: _,
      block_height,
      proxy,
    } = d;

    let mixnode = mixnodes
      .iter()
      .find(|m| m.mix_node.identity_key == node_identity);

    let pledge_amount: Option<MajorCurrencyAmount> =
      mixnode.and_then(|m| m.pledge_amount.clone().try_into().ok());

    let total_delegation: Option<MajorCurrencyAmount> =
      mixnode.and_then(|m| m.total_delegation.clone().try_into().ok());

    let profit_margin_percent: Option<u8> = mixnode.map(|m| m.mix_node.profit_margin_percent);

    let accumulated_rewards = match nymd_client!(state)
      .get_delegator_rewards(address.clone(), node_identity.clone(), proxy.clone())
      .await
    {
      Ok(rewards) => {
        let amount =
          MajorCurrencyAmount::from_minor_uint128_and_denom(rewards, denom_minor.as_ref())?;
        Some(amount)
      }
      Err(_) => None,
    };

    let pending_events = pending_events_for_account
      .iter()
      .filter(|e| e.node_identity == node_identity)
      .cloned()
      .collect::<Vec<DelegationEvent>>();

    let stake_saturation = api_client!(state)
      .get_mixnode_stake_saturation(&node_identity)
      .await
      .ok()
      .map(|r| r.saturation);

    let avg_uptime_percent = api_client!(state)
      .get_mixnode_avg_uptime(&node_identity)
      .await
      .ok()
      .map(|r| r.avg_uptime);

    let timestamp = nymd_client!(state)
      .get_block_timestamp(Some(d.block_height as u32))
      .await?;
    let delegated_on_iso_datetime = timestamp.to_rfc3339();

    with_everything.push(DelegationWithEverything {
      owner: owner.to_string(),
      node_identity: node_identity.to_string(),
      amount: item.1.amount_sum,
      block_height,
      proxy: proxy.clone(),
      delegated_on_iso_datetime,
      stake_saturation,
      accumulated_rewards,
      profit_margin_percent,
      pledge_amount,
      avg_uptime_percent,
      total_delegation,
      pending_events,
      history,
    })
  }

  Ok(with_everything)
}

#[tauri::command]
pub async fn get_delegator_rewards(
  address: String,
  mix_identity: IdentityKey,
  proxy: Option<String>,
  state: tauri::State<'_, Arc<RwLock<State>>>,
) -> Result<Uint128, BackendError> {
  // TODO: convert Uint128 in MajorCurrencyAmount
  Ok(
    nymd_client!(state)
      .get_delegator_rewards(address, mix_identity, proxy)
      .await?,
  )
}

#[tauri::command]
pub async fn get_delegation_summary(
  state: tauri::State<'_, Arc<RwLock<State>>>,
) -> Result<DelegationsSummaryResponse, BackendError> {
  let denom_minor = state.read().await.current_network().denom();
  let denom: CurrencyDenom = denom_minor.clone().try_into()?;

  let delegations = get_all_mix_delegations(state.clone()).await?;
  let mut total_delegations = MajorCurrencyAmount::zero(&denom);
  let mut total_rewards = MajorCurrencyAmount::zero(&denom);

  for d in delegations.clone() {
    total_delegations = total_delegations + d.amount;
    if let Some(rewards) = d.accumulated_rewards {
      total_rewards = total_rewards + rewards;
    }
  }

  Ok(DelegationsSummaryResponse {
    delegations,
    total_delegations,
    total_rewards,
  })
}
