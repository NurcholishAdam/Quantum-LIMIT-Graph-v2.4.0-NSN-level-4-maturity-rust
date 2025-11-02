// -*- coding: utf-8 -*-
//! Provenance-Aware Edit Lineage
//! 
//! Every edit carries cryptographic lineage metadata with Merkle tree integrity.

use crate::error::Result;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;

/// Cryptographic provenance entry for each edit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceEntry {
    pub edit_id: String,
    pub contributor_id: String,
    pub backend_hash: String,
    pub domain_context: String,
    pub semantic_fingerprint: Vec<u8>,
    pub timestamp: u64,
    pub parent_edit_id: Option<String>,
    pub merkle_proof: Vec<String>,
}

/// Merkle tree node for integrity verification
#[derive(Debug, Clone)]
struct MerkleNode {
    hash: String,
    left: Option<Box<MerkleNode>>,
    right: Option<Box<MerkleNode>>,
}

/// Edit provenance ledger with Merkle tree integrity
pub struct EditProvenanceLedger {
    entries: Arc<RwLock<HashMap<String, ProvenanceEntry>>>,
    merkle_root: Arc<RwLock<Option<String>>>,
}

impl EditProvenanceLedger {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            merkle_root: Arc::new(RwLock::new(None)),
        }
    }

    /// Store provenance entry with cryptographic metadata
    pub async fn store_entry(&self, entry: ProvenanceEntry) -> Result<()> {
        let mut entries = self.entries.write().await;
        entries.insert(entry.edit_id.clone(), entry);
        
        // Rebuild Merkle tree
        self.rebuild_merkle_tree().await?;
        
        Ok(())
    }

    /// Generate semantic fingerprint for edit content
    pub fn generate_semantic_fingerprint(&self, content: &str, embedding: &[f64]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        
        // Include embedding in fingerprint
        for &val in embedding {
            hasher.update(val.to_le_bytes());
        }
        
        hasher.finalize().to_vec()
    }

    /// Verify integrity using Merkle proof
    pub async fn verify_integrity(&self, edit_id: &str) -> Result<bool> {
        let entries = self.entries.read().await;
        let merkle_root = self.merkle_root.read().await;
        
        if let Some(entry) = entries.get(edit_id) {
            if let Some(root) = merkle_root.as_ref() {
                return Ok(self.verify_merkle_proof(entry, root));
            }
        }
        
        Ok(false)
    }

    fn verify_merkle_proof(&self, entry: &ProvenanceEntry, root: &str) -> bool {
        let mut current_hash = self.hash_entry(entry);
        
        for sibling_hash in &entry.merkle_proof {
            current_hash = self.combine_hashes(&current_hash, sibling_hash);
        }
        
        current_hash == root
    }

    fn hash_entry(&self, entry: &ProvenanceEntry) -> String {
        let mut hasher = Sha256::new();
        hasher.update(entry.edit_id.as_bytes());
        hasher.update(entry.contributor_id.as_bytes());
        hasher.update(&entry.semantic_fingerprint);
        format!("{:x}", hasher.finalize())
    }

    fn combine_hashes(&self, hash1: &str, hash2: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(hash1.as_bytes());
        hasher.update(hash2.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    async fn rebuild_merkle_tree(&self) -> Result<()> {
        let entries = self.entries.read().await;
        let hashes: Vec<String> = entries.values()
            .map(|e| self.hash_entry(e))
            .collect();
        
        if let Some(root) = self.build_merkle_tree(&hashes) {
            let mut merkle_root = self.merkle_root.write().await;
            *merkle_root = Some(root.hash);
        }
        
        Ok(())
    }

    fn build_merkle_tree(&self, hashes: &[String]) -> Option<MerkleNode> {
        if hashes.is_empty() {
            return None;
        }
        
        if hashes.len() == 1 {
            return Some(MerkleNode {
                hash: hashes[0].clone(),
                left: None,
                right: None,
            });
        }
        
        let mid = hashes.len() / 2;
        let left = self.build_merkle_tree(&hashes[..mid]);
        let right = self.build_merkle_tree(&hashes[mid..]);
        
        if let (Some(l), Some(r)) = (&left, &right) {
            Some(MerkleNode {
                hash: self.combine_hashes(&l.hash, &r.hash),
                left: left.map(Box::new),
                right: right.map(Box::new),
            })
        } else {
            None
        }
    }

    /// Get edit lineage chain
    pub async fn get_lineage(&self, edit_id: &str) -> Result<Vec<ProvenanceEntry>> {
        let entries = self.entries.read().await;
        let mut lineage = Vec::new();
        let mut current_id = Some(edit_id.to_string());
        
        while let Some(id) = current_id {
            if let Some(entry) = entries.get(&id) {
                lineage.push(entry.clone());
                current_id = entry.parent_edit_id.clone();
            } else {
                break;
            }
        }
        
        Ok(lineage)
    }
}

/// Lineage visualizer for multimodal display
pub struct LineageVisualizer;

impl LineageVisualizer {
    pub fn generate_visual_lineage(
        &self,
        lineage: &[ProvenanceEntry],
    ) -> String {
        let mut viz = String::from("graph TD\n");
        
        for (i, entry) in lineage.iter().enumerate() {
            let node_id = format!("E{}", i);
            let label = format!("{}\\n{}\\n{}", 
                entry.edit_id, 
                entry.contributor_id,
                entry.domain_context
            );
            
            viz.push_str(&format!("    {}[\"{}\"]\n", node_id, label));
            
            if i > 0 {
                viz.push_str(&format!("    E{} --> {}\n", i - 1, node_id));
            }
        }
        
        viz
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_provenance_storage() {
        let ledger = EditProvenanceLedger::new();
        
        let entry = ProvenanceEntry {
            edit_id: "edit_1".to_string(),
            contributor_id: "user_1".to_string(),
            backend_hash: "backend_hash".to_string(),
            domain_context: "legal".to_string(),
            semantic_fingerprint: vec![1, 2, 3],
            timestamp: 1234567890,
            parent_edit_id: None,
            merkle_proof: vec![],
        };
        
        ledger.store_entry(entry).await.unwrap();
    }
}
