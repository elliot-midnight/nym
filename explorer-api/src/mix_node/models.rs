// Copyright 2022 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use crate::cache::Cache;
use crate::mix_nodes::location::Location;
use mixnet_contract_common::{Addr, Coin, Layer, MixNode};
use serde::Deserialize;
use serde::Serialize;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;

#[derive(Clone, Debug, Serialize, JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum MixnodeStatus {
    Active,   // in both the active set and the rewarded set
    Standby,  // only in the rewarded set
    Inactive, // in neither the rewarded set nor the active set
}

#[derive(Clone, Debug, Serialize, JsonSchema)]
pub(crate) struct PrettyDetailedMixNodeBond {
    pub location: Option<Location>,
    pub status: MixnodeStatus,
    pub pledge_amount: Coin,
    pub total_delegation: Coin,
    pub owner: Addr,
    pub layer: Layer,
    pub mix_node: MixNode,
}

pub(crate) struct MixNodeCache {
    pub(crate) descriptions: Cache<NodeDescription>,
    pub(crate) node_stats: Cache<NodeStats>,
}

#[derive(Clone)]
pub(crate) struct ThreadsafeMixNodeCache {
    inner: Arc<RwLock<MixNodeCache>>,
}

impl ThreadsafeMixNodeCache {
    pub(crate) fn new() -> Self {
        ThreadsafeMixNodeCache {
            inner: Arc::new(RwLock::new(MixNodeCache {
                descriptions: Cache::new(),
                node_stats: Cache::new(),
            })),
        }
    }

    pub(crate) async fn get_description(&self, identity_key: &str) -> Option<NodeDescription> {
        self.inner.read().await.descriptions.get(identity_key)
    }

    pub(crate) async fn get_node_stats(&self, identity_key: &str) -> Option<NodeStats> {
        self.inner.read().await.node_stats.get(identity_key)
    }

    pub(crate) async fn set_description(&self, identity_key: &str, description: NodeDescription) {
        self.inner
            .write()
            .await
            .descriptions
            .set(identity_key, description);
    }

    pub(crate) async fn set_node_stats(&self, identity_key: &str, node_stats: NodeStats) {
        self.inner
            .write()
            .await
            .node_stats
            .set(identity_key, node_stats);
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, JsonSchema)]
pub(crate) struct NodeDescription {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) link: String,
    pub(crate) location: String,
}

#[derive(Serialize, Clone, Deserialize, JsonSchema)]
pub(crate) struct NodeStats {
    #[serde(
        serialize_with = "humantime_serde::serialize",
        deserialize_with = "humantime_serde::deserialize"
    )]
    update_time: SystemTime,

    #[serde(
        serialize_with = "humantime_serde::serialize",
        deserialize_with = "humantime_serde::deserialize"
    )]
    previous_update_time: SystemTime,

    packets_received_since_startup: u64,
    packets_sent_since_startup: u64,
    packets_explicitly_dropped_since_startup: u64,
    packets_received_since_last_update: u64,
    packets_sent_since_last_update: u64,
    packets_explicitly_dropped_since_last_update: u64,
}

#[derive(Serialize, Clone, Deserialize, JsonSchema)]
pub(crate) struct EconomicDynamicsStats {
    pub(crate) stake_saturation: f32,

    pub(crate) active_set_inclusion_probability: f32,
    pub(crate) reserve_set_inclusion_probability: f32,

    pub(crate) estimated_total_node_reward: u64,
    pub(crate) estimated_operator_reward: u64,
    pub(crate) estimated_delegators_reward: u64,

    pub(crate) current_interval_uptime: u8,
}

impl EconomicDynamicsStats {
    pub(crate) fn dummy_fixture() -> Self {
        EconomicDynamicsStats {
            stake_saturation: 12.3,
            active_set_inclusion_probability: 4.56,
            reserve_set_inclusion_probability: 7.89,
            estimated_total_node_reward: 100000,
            estimated_operator_reward: 80000,
            estimated_delegators_reward: 20000,
            current_interval_uptime: 80,
        }
    }
}
