// -*- coding: utf-8 -*-
//! Level 4 Maturity: Self-Reflective Quantum Agents
//! 
//! This module implements advanced agent capabilities including:
//! - Self-audit and reflection
//! - Cross-agent collaboration via quantum teleportation
//! - Cryptographic edit provenance
//! - Autonomous benchmark evolution
//! - Quantum-driven retrieval planning
//! - Ethical self-governance

pub mod self_audit;
pub mod agent_mesh;
pub mod provenance;
pub mod benchmark_synth;
pub mod quantum_retrieval;
pub mod ethical_governance;
pub mod integration;

pub use self_audit::{SelfAuditModule, ReflectionLoop, AuditReport};
pub use agent_mesh::{AgentMeshProtocol, TaskDelegationEngine, AgentCoalition};
pub use provenance::{EditProvenanceLedger, ProvenanceEntry, LineageVisualizer};
pub use benchmark_synth::{BenchmarkSynthesizer, BenchmarkValidator, SyntheticBenchmark};
pub use quantum_retrieval::{QuantumRetrievalPlanner, SemanticTeleportRouter, RetrievalPlan};
pub use ethical_governance::{EthicalReasoningModule, PolicyAdaptationLayer, EthicalConstraint};
pub use integration::Level4Agent;
