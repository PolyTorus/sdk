//! Core traits and interfaces for PolyTorus SDK
//! 
//! This module defines the interfaces for the 4-layer modular blockchain architecture.

use serde::{Deserialize, Serialize};

/// Hash type for blockchain data
pub type Hash = String;

/// Address type for accounts and contracts
pub type Address = String;

/// Generic result type
pub type Result<T> = anyhow::Result<T>;

// ============================================================================
// Core Data Structures
// ============================================================================

/// Transaction structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub hash: Hash,
    pub from: Address,
    pub to: Option<Address>,
    pub value: u64,
    pub gas_limit: u64,
    pub gas_price: u64,
    pub data: Vec<u8>,
    pub nonce: u64,
    pub signature: Vec<u8>,
    pub script_type: Option<ScriptTransactionType>,
}

/// Script transaction type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScriptTransactionType {
    /// Deploy a new script
    Deploy {
        script_data: Vec<u8>,
        init_params: Vec<u8>,
    },
    /// Call an existing script
    Call {
        script_hash: Hash,
        method: String,
        params: Vec<u8>,
    },
    /// Update script state
    StateUpdate {
        script_hash: Hash,
        updates: Vec<(Vec<u8>, Vec<u8>)>,
    },
}

/// Block structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub hash: Hash,
    pub parent_hash: Hash,
    pub number: u64,
    pub timestamp: u64,
    pub transactions: Vec<Transaction>,
    pub state_root: Hash,
    pub transaction_root: Hash,
    pub validator: Address,
    pub proof: Vec<u8>,
}

/// Account state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountState {
    pub balance: u64,
    pub nonce: u64,
    pub code_hash: Option<Hash>,
    pub storage_root: Option<Hash>,
}

// ============================================================================
// Execution Layer Types
// ============================================================================

/// Transaction execution receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionReceipt {
    pub tx_hash: Hash,
    pub success: bool,
    pub gas_used: u64,
    pub events: Vec<Event>,
}

/// Event emitted during execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub contract: Address,
    pub data: Vec<u8>,
    pub topics: Vec<Hash>,
}

/// Rollup batch for execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionBatch {
    pub batch_id: Hash,
    pub transactions: Vec<Transaction>,
    pub prev_state_root: Hash,
    pub new_state_root: Hash,
    pub timestamp: u64,
}

/// Script execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptExecutionContext {
    pub tx_hash: Hash,
    pub sender: Address,
    pub value: u64,
    pub gas_limit: u64,
    pub block_height: u64,
    pub timestamp: u64,
}

/// Script execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptExecutionResult {
    pub success: bool,
    pub gas_used: u64,
    pub return_data: Vec<u8>,
    pub logs: Vec<String>,
    pub state_changes: Vec<(Vec<u8>, Vec<u8>)>,
    pub events: Vec<Event>,
}

/// Script metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptMetadata {
    pub script_hash: Hash,
    pub owner: Address,
    pub deployed_at: u64,
    pub code_size: usize,
    pub version: u32,
    pub active: bool,
}

// ============================================================================
// Settlement Layer Types
// ============================================================================

/// Settlement finalization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementResult {
    pub settlement_root: Hash,
    pub settled_batches: Vec<Hash>,
    pub timestamp: u64,
}

/// Fraud proof for dispute resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FraudProof {
    pub batch_id: Hash,
    pub proof_data: Vec<u8>,
    pub expected_state_root: Hash,
    pub actual_state_root: Hash,
}

/// Settlement challenge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementChallenge {
    pub challenge_id: Hash,
    pub batch_id: Hash,
    pub proof: FraudProof,
    pub challenger: Address,
    pub timestamp: u64,
}

/// Challenge resolution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeResult {
    pub challenge_id: Hash,
    pub successful: bool,
    pub penalty: Option<u64>,
    pub timestamp: u64,
}

// ============================================================================
// Consensus Layer Types
// ============================================================================

/// Validator information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorInfo {
    pub address: Address,
    pub stake: u64,
    pub public_key: Vec<u8>,
    pub active: bool,
}

// ============================================================================
// Data Availability Types
// ============================================================================

/// Data availability proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailabilityProof {
    pub data_hash: Hash,
    pub merkle_proof: Vec<Hash>,
    pub root_hash: Hash,
    pub timestamp: u64,
}

/// Data storage entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataEntry {
    pub hash: Hash,
    pub data: Vec<u8>,
    pub size: usize,
    pub timestamp: u64,
    pub replicas: Vec<Address>,
}

// ============================================================================
// Layer Traits
// ============================================================================

/// Execution Layer Interface
#[async_trait::async_trait]
pub trait ExecutionLayer: Send + Sync {
    /// Execute a single transaction
    async fn execute_transaction(&mut self, tx: &Transaction) -> Result<TransactionReceipt>;

    /// Execute a batch of transactions (rollup)
    async fn execute_batch(&mut self, transactions: Vec<Transaction>) -> Result<ExecutionBatch>;

    /// Get current state root
    async fn get_state_root(&self) -> Result<Hash>;

    /// Get account state
    async fn get_account_state(&self, address: &Address) -> Result<AccountState>;

    /// Begin execution context
    async fn begin_execution(&mut self) -> Result<()>;

    /// Commit execution results
    async fn commit_execution(&mut self) -> Result<Hash>;

    /// Rollback execution
    async fn rollback_execution(&mut self) -> Result<()>;

    /// Deploy a script
    async fn deploy_script(
        &mut self,
        owner: &Address,
        script_data: &[u8],
        init_params: &[u8],
    ) -> Result<Hash>;

    /// Execute a script
    async fn execute_script(
        &mut self,
        script_hash: &Hash,
        method: &str,
        params: &[u8],
        context: ScriptExecutionContext,
    ) -> Result<ScriptExecutionResult>;

    /// Get script metadata
    async fn get_script_metadata(&self, script_hash: &Hash) -> Result<Option<ScriptMetadata>>;
}

/// Settlement Layer Interface
#[async_trait::async_trait]
pub trait SettlementLayer: Send + Sync {
    /// Settle execution batch
    async fn settle_batch(&mut self, batch: &ExecutionBatch) -> Result<SettlementResult>;

    /// Submit fraud proof challenge
    async fn submit_challenge(&mut self, challenge: SettlementChallenge) -> Result<()>;

    /// Process challenge resolution
    async fn process_challenge(&mut self, challenge_id: &Hash) -> Result<ChallengeResult>;

    /// Get settlement root
    async fn get_settlement_root(&self) -> Result<Hash>;

    /// Get settlement history
    async fn get_settlement_history(&self, limit: usize) -> Result<Vec<SettlementResult>>;
}

/// Consensus Layer Interface
#[async_trait::async_trait]
pub trait ConsensusLayer: Send + Sync {
    /// Propose new block
    async fn propose_block(&mut self, block: Block) -> Result<()>;

    /// Validate block proposal
    async fn validate_block(&self, block: &Block) -> Result<bool>;

    /// Get canonical chain
    async fn get_canonical_chain(&self) -> Result<Vec<Hash>>;

    /// Get current block height
    async fn get_block_height(&self) -> Result<u64>;

    /// Get block by hash
    async fn get_block_by_hash(&self, hash: &Hash) -> Result<Option<Block>>;

    /// Add validated block to chain
    async fn add_block(&mut self, block: Block) -> Result<()>;

    /// Check if node is validator
    async fn is_validator(&self) -> Result<bool>;

    /// Get validator set
    async fn get_validator_set(&self) -> Result<Vec<ValidatorInfo>>;

    /// Mine a new block with PoW
    async fn mine_block(&mut self, transactions: Vec<Transaction>) -> Result<Block>;

    /// Get current mining difficulty
    async fn get_difficulty(&self) -> Result<usize>;

    /// Set mining difficulty
    async fn set_difficulty(&mut self, difficulty: usize) -> Result<()>;
}

/// Data Availability Layer Interface
#[async_trait::async_trait]
pub trait DataAvailabilityLayer: Send + Sync {
    /// Store data and return hash
    async fn store_data(&mut self, data: &[u8]) -> Result<Hash>;

    /// Retrieve data by hash
    async fn retrieve_data(&self, hash: &Hash) -> Result<Option<Vec<u8>>>;

    /// Verify data availability
    async fn verify_availability(&self, hash: &Hash) -> Result<bool>;

    /// Broadcast data to network
    async fn broadcast_data(&mut self, hash: &Hash, data: &[u8]) -> Result<()>;

    /// Request data from peers
    async fn request_data(&mut self, hash: &Hash) -> Result<()>;

    /// Get availability proof
    async fn get_availability_proof(&self, hash: &Hash) -> Result<Option<AvailabilityProof>>;

    /// Get data entry metadata
    async fn get_data_entry(&self, hash: &Hash) -> Result<Option<DataEntry>>;
}