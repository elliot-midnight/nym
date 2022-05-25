use crate::error::BackendError;
use crate::state::State;
use crate::{api_client, nymd_client};
use cosmwasm_std::{Coin as CosmWasmCoin, Uint128};
use mixnet_contract_common::{IdentityKey, MixNodeBond};
use nym_types::currency::{CurrencyDenom, MajorCurrencyAmount};
use nym_types::delegation::{
  from_contract_delegation_events, Delegation, DelegationEvent, DelegationRecord, DelegationResult,
  DelegationWithEverything, DelegationsSummaryResponse,
};
use nym_types::error::TypesError;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

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

pub struct MixNodeExtras {
  pub total_delegation: MajorCurrencyAmount,
  pub pledge_amount: MajorCurrencyAmount,
  pub profit_margin_percent: u8,
  pub accumulated_rewards: Option<MajorCurrencyAmount>,
}

impl MixNodeExtras {
  pub fn from_mixnode_bond(mixnode: &MixNodeBond) -> Result<MixNodeExtras, TypesError> {
    let total_delegation: MajorCurrencyAmount = mixnode.total_delegation.clone().try_into()?;
    let pledge_amount = mixnode.pledge_amount.clone().try_into()?;
    let profit_margin_percent = mixnode.mix_node.profit_margin_percent;
    let accumulated_rewards = mixnode.accumulated_rewards.and_then(|r| {
      MajorCurrencyAmount::from_minor_uint128_and_denom(r, &mixnode.total_delegation.denom).ok()
    });

    Ok(MixNodeExtras {
      total_delegation,
      pledge_amount,
      profit_margin_percent,
      accumulated_rewards,
    })
  }
}

impl TryInto<MixNodeExtras> for MixNodeBond {
  type Error = TypesError;

  fn try_into(self) -> Result<MixNodeExtras, Self::Error> {
    MixNodeExtras::from_mixnode_bond(&self)
  }
}

struct DelegationWithHistory {
  pub delegation: Delegation,
  pub amount_sum: Uint128,
  pub denom: String,
  pub history: Vec<DelegationRecord>,
}

#[tauri::command]
pub async fn get_all_mix_delegations(
  state: tauri::State<'_, Arc<RwLock<State>>>,
) -> Result<Vec<DelegationWithEverything>, BackendError> {
  let delegations = nymd_client!(state)
    .get_delegator_delegations_paged(nymd_client!(state).address().to_string(), None, None) // get all delegations, ignoring paging
    .await?
    .delegations;

  let mut map: HashMap<String, DelegationWithHistory> = HashMap::new();

  for d in delegations {
    // create record of delegation
    let delegated_on_iso_datetime = nymd_client!(state)
      .get_block_timestamp(Some(d.block_height as u32))
      .await?
      .to_rfc3339();
    let amount: MajorCurrencyAmount = d.amount.clone().try_into()?;
    let record = DelegationRecord {
      amount,
      block_height: d.block_height,
      delegated_on_iso_datetime,
    };

    let amount = d.amount.amount;
    let denom = d.amount.denom.to_string();
    let entry = map
      .entry(d.node_identity.clone())
      .or_insert(DelegationWithHistory {
        delegation: d.try_into()?,
        history: vec![],
        denom,
        amount_sum: Uint128::zero(),
      });

    entry.history.push(record);
    entry.amount_sum += amount;
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

    let pending_events = nymd_client!(state)
      .get_pending_delegation_events(owner.clone(), proxy.clone())
      .await?;
    let pending_events = pending_events
      .into_iter()
      .map(|e| e.try_into())
      .collect::<Result<Vec<DelegationEvent>, TypesError>>()?;

    let mixnode_response = nymd_client!(state)
      .get_mixnodes_paged(Some(node_identity.to_string()), Some(1))
      .await?;

    let mixnode = mixnode_response.nodes.first();

    let extras: Option<MixNodeExtras> = match mixnode {
      Some(m) => MixNodeExtras::from_mixnode_bond(m).ok(),
      None => None,
    };

    let stake_saturation = match mixnode {
      Some(m) => Some(
        api_client!(state)
          .get_mixnode_stake_saturation(&m.mix_node.identity_key)
          .await?
          .saturation,
      ),
      None => None,
    };

    let avg_uptime_percent = match mixnode {
      Some(m) => Some(
        api_client!(state)
          .get_mixnode_avg_uptime(&m.mix_node.identity_key)
          .await?
          .avg_uptime,
      ),
      None => None,
    };

    let timestamp = nymd_client!(state)
      .get_block_timestamp(Some(d.block_height as u32))
      .await?;
    let delegated_on_iso_datetime = timestamp.to_rfc3339();

    with_everything.push(DelegationWithEverything {
      owner: owner.to_string(),
      node_identity: node_identity.to_string(),
      amount: MajorCurrencyAmount::from_minor_uint128_and_denom(item.1.amount_sum, &item.1.denom)?,
      block_height,
      proxy: proxy.clone(),
      delegated_on_iso_datetime,
      stake_saturation,
      accumulated_rewards: extras.as_ref().and_then(|e| e.accumulated_rewards.clone()),
      profit_margin_percent: extras.as_ref().map(|e| e.profit_margin_percent),
      total_delegation: extras.as_ref().map(|e| e.total_delegation.clone()),
      pledge_amount: extras.as_ref().map(|e| e.pledge_amount.clone()),
      avg_uptime_percent,
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
  let address = nymd_client!(state).address().to_string();
  let vesting_contract_address: Option<String> = nymd_client!(state)
    .vesting_contract_address()
    .ok()
    .map(|v| v.to_string());

  let denom_minor = state.read().await.current_network().denom();
  let denom: CurrencyDenom = denom_minor.clone().try_into()?;

  let delegations = get_all_mix_delegations(state.clone()).await?;
  let mut total_delegations = MajorCurrencyAmount::zero(&denom);
  let mut total_rewards = MajorCurrencyAmount::zero(&denom);

  for d in delegations.clone() {
    total_delegations = total_delegations + d.amount;
    let reward = get_delegator_rewards(
      address.to_string(),
      d.node_identity,
      vesting_contract_address.clone(),
      state.clone(),
    )
    .await?;
    total_rewards = total_rewards
      + MajorCurrencyAmount::from_minor_uint128_and_denom(reward, denom_minor.as_ref())?;
  }

  Ok(DelegationsSummaryResponse {
    delegations,
    total_delegations,
    total_rewards,
  })
}
