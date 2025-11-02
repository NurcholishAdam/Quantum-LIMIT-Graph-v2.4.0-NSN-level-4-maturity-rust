// -*- coding: utf-8 -*-
//! Cross-Agent Collaboration
//! 
//! Agents form dynamic coalitions to solve tasks and share memory shards
//! via quantum teleportation protocols.

use crate::error::Result;
use crate::types::{QuantumState, MemoryShard, AgentId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::{mpsc, RwLock};
use std::sync::Arc;

/// Agent specialization domains
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AgentSpecialization {
    Translation,
    CodeGeneration,
    SemanticAnalysis,
    FactChecking,
    StyleAlignment,
    DomainExpert(String),
}

/// Agent profile with capabilities and reliability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProfile {
    pub agent_id: AgentId,
    pub specializations: Vec<AgentSpecialization>,
    pub domain_reliability: HashMap<String, f64>,
    pub current_load: usize,
    pub max_capacity: usize,
}

/// Quantum teleportation handshake for shard exchange
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeleportationHandshake {
    pub sender_id: AgentId,
    pub receiver_id: AgentId,
    pub shard_id: String,
    pub entangled_state: QuantumState,
    pub classical_bits: Vec<u8>,
}

/// Agent coalition for collaborative task solving
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCoalition {
    pub coalition_id: String,
    pub members: Vec<AgentId>,
    pub task_description: String,
    pub shared_policy: SharedPolicy,
}

/// Shared policy negotiated via RLHF consensus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedPolicy {
    pub policy_weights: Vec<f64>,
    pub consensus_threshold: f64,
    pub reward_aggregation: RewardAggregation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RewardAggregation {
    Average,
    WeightedByReliability,
    MaxVoting,
}

/// Agent mesh protocol for quantum collaboration
pub struct AgentMeshProtocol {
    agents: Arc<RwLock<HashMap<AgentId, AgentProfile>>>,
    coalitions: Arc<RwLock<HashMap<String, AgentCoalition>>>,
    teleportation_channel: mpsc::Sender<TeleportationHandshake>,
}

impl AgentMeshProtocol {
    pub fn new() -> (Self, mpsc::Receiver<TeleportationHandshake>) {
        let (tx, rx) = mpsc::channel(100);
        
        (
            Self {
                agents: Arc::new(RwLock::new(HashMap::new())),
                coalitions: Arc::new(RwLock::new(HashMap::new())),
                teleportation_channel: tx,
            },
            rx,
        )
    }

    /// Register agent in the mesh
    pub async fn register_agent(&self, profile: AgentProfile) -> Result<()> {
        let mut agents = self.agents.write().await;
        agents.insert(profile.agent_id.clone(), profile);
        Ok(())
    }

    /// Quantum teleportation handshake for shard exchange
    pub async fn teleport_shard(
        &self,
        sender_id: AgentId,
        receiver_id: AgentId,
        shard: &MemoryShard,
    ) -> Result<()> {
        // Create entangled state for teleportation
        let entangled_state = self.create_entangled_pair();
        
        // Perform Bell measurement and extract classical bits
        let classical_bits = self.bell_measurement(&shard.quantum_state, &entangled_state);

        let handshake = TeleportationHandshake {
            sender_id,
            receiver_id,
            shard_id: shard.id.clone(),
            entangled_state,
            classical_bits,
        };

        self.teleportation_channel.send(handshake).await
            .map_err(|e| crate::error::QuantumError::ChannelError(e.to_string()))?;

        Ok(())
    }

    fn create_entangled_pair(&self) -> QuantumState {
        // Create Bell state |Φ⁺⟩ = (|00⟩ + |11⟩)/√2
        QuantumState::bell_state()
    }

    fn bell_measurement(&self, state: &QuantumState, entangled: &QuantumState) -> Vec<u8> {
        // Perform Bell basis measurement
        state.bell_measurement(entangled)
    }

    /// Shared policy negotiation via RLHF consensus
    pub async fn negotiate_shared_policy(
        &self,
        coalition_id: &str,
        member_policies: Vec<(AgentId, Vec<f64>)>,
    ) -> Result<SharedPolicy> {
        let agents = self.agents.read().await;
        
        // Weight policies by agent reliability
        let mut weighted_policy = vec![0.0; member_policies[0].1.len()];
        let mut total_weight = 0.0;

        for (agent_id, policy) in &member_policies {
            if let Some(profile) = agents.get(agent_id) {
                let reliability = profile.domain_reliability.values().sum::<f64>() 
                    / profile.domain_reliability.len() as f64;
                
                for (i, &weight) in policy.iter().enumerate() {
                    weighted_policy[i] += weight * reliability;
                }
                total_weight += reliability;
            }
        }

        // Normalize
        for weight in &mut weighted_policy {
            *weight /= total_weight;
        }

        Ok(SharedPolicy {
            policy_weights: weighted_policy,
            consensus_threshold: 0.75,
            reward_aggregation: RewardAggregation::WeightedByReliability,
        })
    }

    /// Form coalition for task
    pub async fn form_coalition(
        &self,
        task_description: String,
        required_specializations: Vec<AgentSpecialization>,
    ) -> Result<AgentCoalition> {
        let agents = self.agents.read().await;
        
        // Select agents based on specialization and availability
        let mut selected_agents = Vec::new();
        
        for specialization in &required_specializations {
            if let Some(agent_id) = self.find_best_agent(&agents, specialization) {
                selected_agents.push(agent_id);
            }
        }

        let coalition_id = format!("coalition_{}", uuid::Uuid::new_v4());
        
        // Negotiate shared policy
        let member_policies: Vec<_> = selected_agents.iter()
            .map(|id| (id.clone(), vec![1.0; 10])) // Placeholder policies
            .collect();
        
        let shared_policy = self.negotiate_shared_policy(&coalition_id, member_policies).await?;

        let coalition = AgentCoalition {
            coalition_id: coalition_id.clone(),
            members: selected_agents,
            task_description,
            shared_policy,
        };

        let mut coalitions = self.coalitions.write().await;
        coalitions.insert(coalition_id, coalition.clone());

        Ok(coalition)
    }

    fn find_best_agent(
        &self,
        agents: &HashMap<AgentId, AgentProfile>,
        specialization: &AgentSpecialization,
    ) -> Option<AgentId> {
        agents.iter()
            .filter(|(_, profile)| {
                profile.specializations.contains(specialization) &&
                profile.current_load < profile.max_capacity
            })
            .max_by(|(_, a), (_, b)| {
                let a_reliability = a.domain_reliability.values().sum::<f64>();
                let b_reliability = b.domain_reliability.values().sum::<f64>();
                a_reliability.partial_cmp(&b_reliability).unwrap()
            })
            .map(|(id, _)| id.clone())
    }
}

/// Task delegation engine with async messaging
pub struct TaskDelegationEngine {
    mesh: Arc<AgentMeshProtocol>,
    task_channels: Arc<RwLock<HashMap<AgentId, mpsc::Sender<Task>>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub task_id: String,
    pub description: String,
    pub specialization_required: AgentSpecialization,
    pub priority: u8,
    pub deadline: Option<u64>,
}

impl TaskDelegationEngine {
    pub fn new(mesh: Arc<AgentMeshProtocol>) -> Self {
        Self {
            mesh,
            task_channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Assign subtask based on agent specialization and domain reliability
    pub async fn delegate_task(&self, task: Task) -> Result<AgentId> {
        let agents = self.mesh.agents.read().await;
        
        // Find best agent for task
        let best_agent = agents.iter()
            .filter(|(_, profile)| {
                profile.specializations.contains(&task.specialization_required) &&
                profile.current_load < profile.max_capacity
            })
            .max_by(|(_, a), (_, b)| {
                let a_score = self.calculate_agent_score(a, &task);
                let b_score = self.calculate_agent_score(b, &task);
                a_score.partial_cmp(&b_score).unwrap()
            })
            .map(|(id, _)| id.clone());

        if let Some(agent_id) = best_agent {
            // Send task via Tokio channel
            let channels = self.task_channels.read().await;
            if let Some(tx) = channels.get(&agent_id) {
                tx.send(task).await
                    .map_err(|e| crate::error::QuantumError::ChannelError(e.to_string()))?;
            }
            Ok(agent_id)
        } else {
            Err(crate::error::QuantumError::NoAvailableAgent)
        }
    }

    fn calculate_agent_score(&self, profile: &AgentProfile, task: &Task) -> f64 {
        let reliability = profile.domain_reliability.values().sum::<f64>() 
            / profile.domain_reliability.len().max(1) as f64;
        let load_factor = 1.0 - (profile.current_load as f64 / profile.max_capacity as f64);
        
        reliability * 0.7 + load_factor * 0.3
    }

    /// Register task channel for agent
    pub async fn register_task_channel(&self, agent_id: AgentId, tx: mpsc::Sender<Task>) {
        let mut channels = self.task_channels.write().await;
        channels.insert(agent_id, tx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_registration() {
        let (mesh, _rx) = AgentMeshProtocol::new();
        
        let profile = AgentProfile {
            agent_id: "agent_1".to_string(),
            specializations: vec![AgentSpecialization::Translation],
            domain_reliability: HashMap::new(),
            current_load: 0,
            max_capacity: 10,
        };

        mesh.register_agent(profile).await.unwrap();
    }

    #[tokio::test]
    async fn test_coalition_formation() {
        let (mesh, _rx) = AgentMeshProtocol::new();
        
        let profile = AgentProfile {
            agent_id: "agent_1".to_string(),
            specializations: vec![AgentSpecialization::Translation],
            domain_reliability: HashMap::from([("legal".to_string(), 0.9)]),
            current_load: 0,
            max_capacity: 10,
        };

        mesh.register_agent(profile).await.unwrap();

        let coalition = mesh.form_coalition(
            "Translate legal document".to_string(),
            vec![AgentSpecialization::Translation],
        ).await.unwrap();

        assert!(!coalition.members.is_empty());
    }
}
