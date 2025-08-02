//! PolyTorus SDK - Developer-friendly interface for the 4-layer modular blockchain
//!
//! This SDK provides a high-level, easy-to-use interface for developers to interact with
//! the PolyTorus blockchain platform. It abstracts the complexity of the 4-layer architecture
//! and provides simple methods for common blockchain operations.
//!
//! # Features
//!
//! - **Transaction Management**: Create, sign, and submit transactions
//! - **Wallet Integration**: Full HD wallet support with BIP32/BIP44
//! - **Smart Contracts**: Deploy and interact with WASM-based contracts
//! - **Block Operations**: Query blocks, mining, and validation
//! - **Data Availability**: Store and retrieve data with proofs
//! - **Layer Abstraction**: Direct access to individual layers when needed
//!
//! # Quick Start
//!
//! To use the PolyTorus SDK, you need to provide your own implementations of the four core traits:
//! - `ExecutionLayer`: Handles transaction execution and smart contracts
//! - `SettlementLayer`: Manages batch settlement and challenges
//! - `ConsensusLayer`: Provides block consensus and validation
//! - `DataAvailabilityLayer`: Ensures data availability and storage
//!
//! ```rust,ignore
//! use sdk::{PolyTorusClient, ClientConfig};
//! use traits::{ExecutionLayer, SettlementLayer, ConsensusLayer, DataAvailabilityLayer};
//!
//! // You must implement these traits for your specific blockchain architecture
//! struct MyExecutionLayer { /* your implementation */ }
//! struct MySettlementLayer { /* your implementation */ }
//! struct MyConsensusLayer { /* your implementation */ }
//! struct MyDataAvailabilityLayer { /* your implementation */ }
//!
//! // Implement the required traits for each layer
//! impl ExecutionLayer for MyExecutionLayer {
//!     // implement all required methods
//! }
//! // ... implement other traits
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create your layer implementations
//!     let execution_layer = MyExecutionLayer::new();
//!     let settlement_layer = MySettlementLayer::new();
//!     let consensus_layer = MyConsensusLayer::new();
//!     let data_availability_layer = MyDataAvailabilityLayer::new();
//!     
//!     // Create a new client with your layer implementations
//!     let client = PolyTorusClient::new(
//!         ClientConfig::default(),
//!         execution_layer,
//!         settlement_layer,
//!         consensus_layer,
//!         data_availability_layer,
//!     ).await?;
//!     
//!     // Create a wallet
//!     let wallet = client.create_wallet().await?;
//!     
//!     // Send a transaction
//!     let tx_hash = client.send_transaction(
//!         &wallet,
//!         "recipient_address",
//!         1000, // amount
//!         None  // data
//!     ).await?;
//!     
//!     println!("Transaction sent: {}", tx_hash);
//!     Ok(())
//! }
//! ```
//!
//! Note: This SDK provides the client structure and trait definitions. You are responsible
//! for implementing the actual layer logic according to your blockchain's requirements.

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// Trait definitions
pub mod traits;

// Re-export core types for convenience
pub use traits::{ExecutionLayer, SettlementLayer, ConsensusLayer, DataAvailabilityLayer};
pub use traits::{Transaction, Block, Hash, TransactionReceipt, Event, AccountState};
pub use traits::{Result, ScriptTransactionType};
pub use wallet::{HdWallet, Wallet, Address as WalletAddress, KeyPair, Signature, Mnemonic};

// ============================================================================
// SDK Configuration
// ============================================================================

/// Configuration for the PolyTorus client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    pub network: NetworkConfig,
    pub wallet: WalletConfig,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            network: NetworkConfig::default(),
            wallet: WalletConfig::default(),
        }
    }
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub chain_id: u64,
    pub network_name: String,
    pub is_testnet: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            chain_id: 1,
            network_name: "polytorus-mainnet".to_string(),
            is_testnet: false,
        }
    }
}

/// Wallet configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletConfig {
    pub derivation_path: String,
    pub address_format: String,
}

impl Default for WalletConfig {
    fn default() -> Self {
        Self {
            derivation_path: "m/44'/0'/0'".to_string(),
            address_format: "native_segwit".to_string(),
        }
    }
}

// ============================================================================
// SDK Client
// ============================================================================

/// High-level client for interacting with PolyTorus blockchain
pub struct PolyTorusClient<E, S, C, D>
where
    E: ExecutionLayer + Send + Sync + 'static,
    S: SettlementLayer + Send + Sync + 'static,
    C: ConsensusLayer + Send + Sync + 'static,
    D: DataAvailabilityLayer + Send + Sync + 'static,
{
    pub config: ClientConfig,
    execution_layer: Arc<RwLock<E>>,
    settlement_layer: Arc<RwLock<S>>,
    consensus_layer: Arc<RwLock<C>>,
    data_availability_layer: Arc<RwLock<D>>,
    wallets: Arc<RwLock<HashMap<String, HdWallet>>>,
}

impl<E, S, C, D> PolyTorusClient<E, S, C, D>
where
    E: ExecutionLayer + Send + Sync + 'static,
    S: SettlementLayer + Send + Sync + 'static,
    C: ConsensusLayer + Send + Sync + 'static,
    D: DataAvailabilityLayer + Send + Sync + 'static,
{
    /// Create a new PolyTorus client with provided layer implementations
    pub async fn new(
        config: ClientConfig,
        execution_layer: E,
        settlement_layer: S,
        consensus_layer: C,
        data_availability_layer: D,
    ) -> Result<Self> {
        Ok(Self {
            config,
            execution_layer: Arc::new(RwLock::new(execution_layer)),
            settlement_layer: Arc::new(RwLock::new(settlement_layer)),
            consensus_layer: Arc::new(RwLock::new(consensus_layer)),
            data_availability_layer: Arc::new(RwLock::new(data_availability_layer)),
            wallets: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    // ========================================================================
    // Wallet Management
    // ========================================================================

    /// Create a new HD wallet
    pub async fn create_wallet(&self) -> Result<WalletInfo> {
        let wallet = HdWallet::new(wallet::KeyType::Ed25519)
            .map_err(|e| anyhow!("Failed to create wallet: {}", e))?;
        
        let mnemonic_phrase = wallet.get_mnemonic().phrase().to_string();
        
        let mut base_wallet = wallet.derive_wallet("m/44'/9999'/0'/0/0", wallet::KeyType::Ed25519)
            .map_err(|e| anyhow!("Failed to derive wallet: {}", e))?;
        let address = base_wallet.default_address()
            .map_err(|e| anyhow!("Failed to get address: {}", e))?.value;
        
        let wallet_id = uuid::Uuid::new_v4().to_string();
        
        let mut wallets = self.wallets.write().await;
        wallets.insert(wallet_id.clone(), wallet);
        
        Ok(WalletInfo {
            id: wallet_id,
            address,
            mnemonic: mnemonic_phrase,
            derivation_path: self.config.wallet.derivation_path.clone(),
        })
    }

    /// Import wallet from mnemonic
    pub async fn import_wallet(&self, mnemonic: &str, _passphrase: Option<&str>) -> Result<WalletInfo> {
        let wallet = HdWallet::from_phrase(mnemonic, wallet::KeyType::Ed25519)
            .map_err(|e| anyhow!("Failed to import wallet: {}", e))?;
        
        let mut base_wallet = wallet.derive_wallet("m/44'/9999'/0'/0/0", wallet::KeyType::Ed25519)
            .map_err(|e| anyhow!("Failed to derive wallet: {}", e))?;
        let address = base_wallet.default_address()
            .map_err(|e| anyhow!("Failed to get address: {}", e))?.value;
        
        let wallet_id = uuid::Uuid::new_v4().to_string();
        
        let mut wallets = self.wallets.write().await;
        wallets.insert(wallet_id.clone(), wallet);
        
        Ok(WalletInfo {
            id: wallet_id,
            address,
            mnemonic: mnemonic.to_string(),
            derivation_path: self.config.wallet.derivation_path.clone(),
        })
    }

    /// Get wallet information
    pub async fn get_wallet(&self, wallet_id: &str) -> Result<Option<WalletInfo>> {
        let wallets = self.wallets.read().await;
        
        if let Some(wallet) = wallets.get(wallet_id) {
            let mut base_wallet = wallet.derive_wallet("m/44'/9999'/0'/0/0", wallet::KeyType::Ed25519)
                .map_err(|e| anyhow!("Failed to derive wallet: {}", e))?;
            let address = base_wallet.default_address()
                .map_err(|e| anyhow!("Failed to get address: {}", e))?.value;
            
            Ok(Some(WalletInfo {
                id: wallet_id.to_string(),
                address,
                mnemonic: String::new(), // Don't expose mnemonic in get operations
                derivation_path: self.config.wallet.derivation_path.clone(),
            }))
        } else {
            Ok(None)
        }
    }

    // ========================================================================
    // Transaction Operations
    // ========================================================================

    /// Send a simple transaction
    pub async fn send_transaction(
        &self,
        wallet_info: &WalletInfo,
        to: &str,
        amount: u64,
        data: Option<Vec<u8>>,
    ) -> Result<Hash> {
        let wallets = self.wallets.read().await;
        let wallet = wallets.get(&wallet_info.id)
            .ok_or_else(|| anyhow!("Wallet not found"))?;
        
        let keypair = wallet.derive_key(0)
            .map_err(|e| anyhow!("Failed to derive keypair: {}", e))?;
        
        let nonce = self.get_account_nonce(&wallet_info.address).await?;
        
        let tx = Transaction {
            hash: self.generate_transaction_hash(),
            from: wallet_info.address.clone(),
            to: Some(to.to_string()),
            value: amount,
            gas_limit: 21000, // Standard gas limit
            gas_price: 1,     // Default gas price
            data: data.unwrap_or_default(),
            nonce,
            signature: vec![], // Will be filled after signing
            script_type: None,
        };
        
        let signed_tx = self.sign_transaction(tx, &keypair).await?;
        self.submit_transaction(signed_tx).await
    }

    /// Deploy a smart contract
    pub async fn deploy_contract(
        &self,
        wallet_info: &WalletInfo,
        contract_code: &[u8],
        init_params: &[u8],
        gas_limit: u64,
    ) -> Result<Hash> {
        let wallets = self.wallets.read().await;
        let wallet = wallets.get(&wallet_info.id)
            .ok_or_else(|| anyhow!("Wallet not found"))?;
        
        let keypair = wallet.derive_key(0)
            .map_err(|e| anyhow!("Failed to derive keypair: {}", e))?;
        
        let nonce = self.get_account_nonce(&wallet_info.address).await?;
        
        let tx = Transaction {
            hash: self.generate_transaction_hash(),
            from: wallet_info.address.clone(),
            to: None, // Contract deployment
            value: 0,
            gas_limit,
            gas_price: 1,
            data: vec![],
            nonce,
            signature: vec![],
            script_type: Some(ScriptTransactionType::Deploy {
                script_data: contract_code.to_vec(),
                init_params: init_params.to_vec(),
            }),
        };
        
        let signed_tx = self.sign_transaction(tx, &keypair).await?;
        self.submit_transaction(signed_tx).await
    }

    /// Call a smart contract method
    pub async fn call_contract(
        &self,
        wallet_info: &WalletInfo,
        contract_hash: &Hash,
        method: &str,
        params: &[u8],
        gas_limit: u64,
    ) -> Result<Hash> {
        let wallets = self.wallets.read().await;
        let wallet = wallets.get(&wallet_info.id)
            .ok_or_else(|| anyhow!("Wallet not found"))?;
        
        let keypair = wallet.derive_key(0)
            .map_err(|e| anyhow!("Failed to derive keypair: {}", e))?;
        
        let nonce = self.get_account_nonce(&wallet_info.address).await?;
        
        let tx = Transaction {
            hash: self.generate_transaction_hash(),
            from: wallet_info.address.clone(),
            to: Some(contract_hash.clone()),
            value: 0,
            gas_limit,
            gas_price: 1,
            data: vec![],
            nonce,
            signature: vec![],
            script_type: Some(ScriptTransactionType::Call {
                script_hash: contract_hash.clone(),
                method: method.to_string(),
                params: params.to_vec(),
            }),
        };
        
        let signed_tx = self.sign_transaction(tx, &keypair).await?;
        self.submit_transaction(signed_tx).await
    }

    // ========================================================================
    // Query Operations
    // ========================================================================

    /// Get account balance
    pub async fn get_balance(&self, address: &str) -> Result<u64> {
        let execution = self.execution_layer.read().await;
        let account_state = execution.get_account_state(&address.to_string()).await?;
        Ok(account_state.balance)
    }

    /// Get account nonce
    pub async fn get_account_nonce(&self, address: &str) -> Result<u64> {
        let execution = self.execution_layer.read().await;
        let account_state = execution.get_account_state(&address.to_string()).await?;
        Ok(account_state.nonce)
    }

    /// Get block by hash
    pub async fn get_block(&self, block_hash: &Hash) -> Result<Option<Block>> {
        let consensus = self.consensus_layer.read().await;
        consensus.get_block_by_hash(block_hash).await
    }

    /// Get current block height
    pub async fn get_block_height(&self) -> Result<u64> {
        let consensus = self.consensus_layer.read().await;
        consensus.get_block_height().await
    }

    /// Get transaction receipt (simplified)
    pub async fn get_transaction_receipt(&self, _tx_hash: &Hash) -> Result<Option<TransactionReceipt>> {
        // This would typically query a transaction pool or blockchain storage
        // For now, returning None as this requires more complex state management
        Ok(None)
    }

    // ========================================================================
    // Data Availability Operations
    // ========================================================================

    /// Store data on the blockchain
    pub async fn store_data(&self, data: &[u8]) -> Result<Hash> {
        let mut da_layer = self.data_availability_layer.write().await;
        da_layer.store_data(data).await
    }

    /// Retrieve data from the blockchain
    pub async fn retrieve_data(&self, data_hash: &Hash) -> Result<Option<Vec<u8>>> {
        let da_layer = self.data_availability_layer.read().await;
        da_layer.retrieve_data(data_hash).await
    }

    /// Verify data availability
    pub async fn verify_data_availability(&self, data_hash: &Hash) -> Result<bool> {
        let da_layer = self.data_availability_layer.read().await;
        da_layer.verify_availability(data_hash).await
    }

    // ========================================================================
    // Mining Operations
    // ========================================================================

    /// Mine a new block with pending transactions
    pub async fn mine_block(&self) -> Result<Block> {
        let mut consensus = self.consensus_layer.write().await;
        // For simplicity, mining an empty block
        // In a real implementation, this would gather pending transactions
        consensus.mine_block(vec![]).await
    }

    /// Set mining difficulty
    pub async fn set_mining_difficulty(&self, difficulty: usize) -> Result<()> {
        let mut consensus = self.consensus_layer.write().await;
        consensus.set_difficulty(difficulty).await
    }

    // ========================================================================
    // Layer Access
    // ========================================================================

    /// Get direct access to execution layer
    pub fn execution_layer(&self) -> Arc<RwLock<E>> {
        self.execution_layer.clone()
    }

    /// Get direct access to settlement layer
    pub fn settlement_layer(&self) -> Arc<RwLock<S>> {
        self.settlement_layer.clone()
    }

    /// Get direct access to consensus layer
    pub fn consensus_layer(&self) -> Arc<RwLock<C>> {
        self.consensus_layer.clone()
    }

    /// Get direct access to data availability layer
    pub fn data_availability_layer(&self) -> Arc<RwLock<D>> {
        self.data_availability_layer.clone()
    }

    // ========================================================================
    // Internal Helper Methods
    // ========================================================================

    async fn sign_transaction(&self, mut tx: Transaction, keypair: &KeyPair) -> Result<Transaction> {
        let tx_data = serde_json::to_vec(&tx)?;
        let signature_vec = keypair.sign(&tx_data)
            .map_err(|e| anyhow!("Failed to sign transaction: {}", e))?;
        tx.signature = signature_vec;
        Ok(tx)
    }

    async fn submit_transaction(&self, tx: Transaction) -> Result<Hash> {
        let mut execution = self.execution_layer.write().await;
        let receipt = execution.execute_transaction(&tx).await?;
        
        if receipt.success {
            Ok(tx.hash)
        } else {
            Err(anyhow!("Transaction execution failed"))
        }
    }

    fn generate_transaction_hash(&self) -> Hash {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(chrono::Utc::now().timestamp().to_string());
        hasher.update(uuid::Uuid::new_v4().to_string());
        hex::encode(hasher.finalize())
    }
}

// ============================================================================
// SDK Types
// ============================================================================

/// Wallet information returned by SDK
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletInfo {
    pub id: String,
    pub address: String,
    pub mnemonic: String,
    pub derivation_path: String,
}

/// Contract deployment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractDeployment {
    pub contract_hash: Hash,
    pub transaction_hash: Hash,
    pub gas_used: u64,
}

/// Contract call result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractCallResult {
    pub transaction_hash: Hash,
    pub return_data: Vec<u8>,
    pub gas_used: u64,
    pub events: Vec<Event>,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_config_creation() {
        let config = ClientConfig::default();
        assert_eq!(config.network.chain_id, 1);
        assert_eq!(config.network.network_name, "polytorus-mainnet");
        assert!(!config.network.is_testnet);
        assert_eq!(config.wallet.derivation_path, "m/44'/0'/0'");
        assert_eq!(config.wallet.address_format, "native_segwit");
    }

    #[test]
    fn test_network_config_default() {
        let network_config = NetworkConfig::default();
        assert_eq!(network_config.chain_id, 1);
        assert_eq!(network_config.network_name, "polytorus-mainnet");
        assert!(!network_config.is_testnet);
    }

    #[test]
    fn test_wallet_config_default() {
        let wallet_config = WalletConfig::default();
        assert_eq!(wallet_config.derivation_path, "m/44'/0'/0'");
        assert_eq!(wallet_config.address_format, "native_segwit");
    }

    #[test]
    fn test_wallet_info_serialization() {
        let wallet_info = WalletInfo {
            id: "test-id".to_string(),
            address: "test-address".to_string(),
            mnemonic: "test mnemonic words".to_string(),
            derivation_path: "m/44'/0'/0'".to_string(),
        };

        // Test that WalletInfo can be serialized and deserialized
        let serialized = serde_json::to_string(&wallet_info).unwrap();
        let deserialized: WalletInfo = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(wallet_info.id, deserialized.id);
        assert_eq!(wallet_info.address, deserialized.address);
        assert_eq!(wallet_info.mnemonic, deserialized.mnemonic);
        assert_eq!(wallet_info.derivation_path, deserialized.derivation_path);
    }

    #[test]
    fn test_contract_deployment_serialization() {
        let deployment = ContractDeployment {
            contract_hash: "test-contract-hash".to_string(),
            transaction_hash: "test-tx-hash".to_string(),
            gas_used: 50000,
        };

        // Test that ContractDeployment can be serialized and deserialized
        let serialized = serde_json::to_string(&deployment).unwrap();
        let deserialized: ContractDeployment = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(deployment.contract_hash, deserialized.contract_hash);
        assert_eq!(deployment.transaction_hash, deserialized.transaction_hash);
        assert_eq!(deployment.gas_used, deserialized.gas_used);
    }

    #[test]
    fn test_contract_call_result_serialization() {
        let call_result = ContractCallResult {
            transaction_hash: "test-tx-hash".to_string(),
            return_data: vec![1, 2, 3, 4],
            gas_used: 30000,
            events: vec![],
        };

        // Test that ContractCallResult can be serialized and deserialized
        let serialized = serde_json::to_string(&call_result).unwrap();
        let deserialized: ContractCallResult = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(call_result.transaction_hash, deserialized.transaction_hash);
        assert_eq!(call_result.return_data, deserialized.return_data);
        assert_eq!(call_result.gas_used, deserialized.gas_used);
        assert_eq!(call_result.events.len(), deserialized.events.len());
    }

    #[test]
    fn test_trait_re_exports() {
        // Test that types are properly re-exported
        // This is a compile-time test - if it compiles, the re-exports work
        use crate::{Transaction, Block, Hash, TransactionReceipt, Event, AccountState};
        use crate::{ScriptTransactionType};
        
        // These are just type checks to ensure the re-exports are working
        let _tx: Option<Transaction> = None;
        let _block: Option<Block> = None;
        let _hash: Option<Hash> = None;
        let _receipt: Option<TransactionReceipt> = None;
        let _event: Option<Event> = None;
        let _account: Option<AccountState> = None;
        let _script_type: Option<ScriptTransactionType> = None;
        
        // Test that we can access trait types through the SDK
        assert!(std::any::type_name::<Transaction>().contains("Transaction"));
        assert!(std::any::type_name::<Block>().contains("Block"));
        assert!(std::any::type_name::<Hash>().contains("String"));
    }
}