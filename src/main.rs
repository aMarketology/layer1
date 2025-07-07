#![recursion_limit = "512"]

use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use warp::Filter;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::time;
use crate::token_launch::TokenHolding;
extern crate rand; // Add this line

// Add the new modules
mod security;
mod enhanced_transaction;
mod token_launch;
mod social_mining;

// Import the new types
use security::{SecurityManager, SecurityError, SecurityStats};
use enhanced_transaction::{EnhancedTransaction, TransactionPool, PoolStats, TransactionReceipt};
use token_launch::{
    TokenLaunchSystem, LaunchTokenRequest, BuyTokenRequest, SellTokenRequest,
    UserPortfolioResponse, Token, TokenTrade
};
// Add social mining imports
use social_mining::{
    SocialMiningSystem, SocialPostRequest, SocialLikeRequest, SocialCommentRequest,
    SocialActionResponse, SocialStatsResponse
};

// Original Transaction structure (keep for compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Transaction {
    from: String,
    to: String,
    amount: f64,
    timestamp: u64,
    signature: String,
}

// Add this new structure for enhanced transaction requests
#[derive(Deserialize)]
struct EnhancedTransactionRequest {
    from: String,
    to: String,
    amount: f64,
    fee: f64,
    message: Option<String>,
}

// Updated Block structure
#[derive(Debug, Clone, Serialize)]
struct Block {
    index: u64,
    timestamp: u64,
    transactions: Vec<Transaction>,
    previous_hash: String,
    hash: String,
    nonce: u64,
    miner: String,
    reward: f64,
}

// Connection tracking structure
#[derive(Debug, Clone, Serialize)]
struct Connection {
    address: String,
    connected_at: u64,
    last_activity: u64,
    total_rewards: f64,
    is_active: bool,
}

// Address label structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AddressLabel {
    username: String,
    address: String,
    registered_at: u64,
    is_verified: bool,
}

// Request structures
#[derive(Deserialize)]
struct TransactionRequest {
    from: String,
    to: String,
    amount: f64,
}

#[derive(Deserialize)]
struct TransactionWithUsernamesRequest {
    from: String,
    to: String,
    amount: f64,
}

#[derive(Deserialize)]
struct MineRequest {
    miner_address: String,
}

#[derive(Deserialize)]
struct ConnectRequest {
    address: String,
}

#[derive(Deserialize)]
struct DisconnectRequest {
    address: String,
}

#[derive(Deserialize)]
struct UsernameRegisterRequest {
    username: String,
}

#[derive(Deserialize)]
struct UsernameResolveRequest {
    username: String,
}

#[derive(Deserialize)]
struct TipRequest {
    from: String,
    to: String,
    amount: f64,
    message: Option<String>,
}

#[derive(Deserialize)]
struct AdminBlacklistRequest {
    address: String,
    reason: Option<String>,
}

#[derive(Deserialize)]
struct AdminUnblacklistRequest {
    address: String,
}

// Response structures
#[derive(Serialize)]
struct BalanceResponse {
    address: String,
    balance: f64,
}

#[derive(Serialize)]
struct NetworkStats {
    total_supply: f64,
    circulating_supply: f64,
    remaining_supply: f64,
    current_reward_rate: f64,
    active_connections: usize,
    total_blocks: usize,
}

#[derive(Serialize)]
struct UserWalletInfo {
    address: String,
    balance: f64,
    total_sent: f64,
    total_received: f64,
    transaction_count: u32,
}

#[derive(Serialize)]
struct TransactionHistoryResponse {
    address: String,
    transactions: Vec<serde_json::Value>,
    total_count: usize,
}

#[derive(Serialize)]
struct WalletInfoResponse {
    address: String,
    balance: f64,
    username: Option<String>,
    is_verified: bool,
    total_sent: f64,
    total_received: f64,
    transaction_count: u32,
    connection_info: Option<Connection>,
}

impl Block {
    fn new(index: u64, transactions: Vec<Transaction>, previous_hash: String, miner: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let mut block = Block {
            index,
            timestamp,
            transactions,
            previous_hash,
            hash: String::new(),
            nonce: 0,
            miner,
            reward: 10.0,
        };
        
        block.hash = block.calculate_hash();
        block
    }

    fn calculate_hash(&self) -> String {
        let transactions_data = serde_json::to_string(&self.transactions).unwrap_or_default();
        let input = format!(
            "{}{}{}{}{}{}{}",
            self.index, self.timestamp, transactions_data, 
            self.previous_hash, self.nonce, self.miner, self.reward
        );
        let mut hasher = Sha256::new();
        hasher.update(input);
        format!("{:x}", hasher.finalize())
    }

    fn mine_block(&mut self, difficulty: usize) {
        let target = "0".repeat(difficulty);
        println!("Mining block {}...", self.index);
        while &self.hash[..difficulty] != target {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }
        println!("Block {} mined! Hash: {}", self.index, self.hash);
    }
}

// Enhanced Blockchain structure with security and enhanced transactions
#[derive(Serialize)]
struct Blockchain {
    chain: Vec<Block>,
    difficulty: usize,
    pending_transactions: Vec<Transaction>,
    balances: HashMap<String, f64>,
    mining_reward: f64,
    connections: HashMap<String, Connection>,
    max_supply: f64,
    circulating_supply: f64,
    address_labels: HashMap<String, AddressLabel>,
    address_to_username: HashMap<String, String>,
    // New security and enhanced transaction fields
    #[serde(skip)] // Skip serialization for complex types
    security_manager: SecurityManager,
    #[serde(skip)]
    enhanced_tx_pool: TransactionPool,
    #[serde(skip)]
    token_system: TokenLaunchSystem,
    #[serde(skip)]
    social_mining: SocialMiningSystem,
}

impl Blockchain {
    fn new() -> Self {
        let mut blockchain = Blockchain {
            chain: Vec::new(),
            difficulty: 2,
            pending_transactions: Vec::new(),
            balances: HashMap::new(),
            mining_reward: 10.0,
            connections: HashMap::new(),
            max_supply: 21_000_000.0,
            circulating_supply: 0.0,
            address_labels: HashMap::new(),
            address_to_username: HashMap::new(),
            // Initialize security and enhanced features
            security_manager: SecurityManager::new(),
            enhanced_tx_pool: TransactionPool::new(),
            token_system: TokenLaunchSystem::new(),
            social_mining: SocialMiningSystem::new(),
        };
        blockchain.create_genesis_block();
        blockchain
    }

    fn create_genesis_block(&mut self) {
        let genesis_tx = Transaction {
            from: "genesis".to_string(),
            to: "genesis".to_string(),
            amount: 0.0,
            timestamp: 0,
            signature: "genesis".to_string(),
        };

        let genesis_block = Block::new(0, vec![genesis_tx], "0".to_string(), "genesis".to_string());
        self.chain.push(genesis_block);
        self.update_balances();
    }

    // Original transaction creation (keep for compatibility)
    fn create_transaction(&mut self, from: String, to: String, amount: f64) -> Result<String, String> {
        if from != "genesis" && from != "mining_reward" && from != "connection_reward" && from != "social_mining" {
            let balance = self.get_balance(&from);
            if balance < amount {
                return Err(format!("Insufficient balance. Have: {}, Need: {}", balance, amount));
            }
        }

        let transaction = Transaction {
            from: from.clone(),
            to: to.clone(),
            amount,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            signature: format!("sig_{}_{}", from, rand::random::<u64>()),
        };

        self.pending_transactions.push(transaction);
        Ok("Transaction added to pending pool".to_string())
    }

    // New enhanced transaction creation with security
    fn create_enhanced_transaction(&mut self, req: EnhancedTransactionRequest) -> Result<String, String> {
        // Security checks
        match self.security_manager.check_transaction_security(&req.from, &req.to, req.amount) {
            Ok(_) => {},
            Err(SecurityError::RateLimitExceeded) => {
                self.security_manager.record_failed_attempt(&req.from);
                return Err("Rate limit exceeded. Please wait before sending another transaction.".to_string());
            },
            Err(SecurityError::InvalidTransaction(msg)) => {
                self.security_manager.record_failed_attempt(&req.from);
                return Err(msg);
            },
            Err(SecurityError::BlacklistedAddress) => {
                return Err("Address is blacklisted and cannot perform transactions.".to_string());
            },
            Err(SecurityError::ValidationFailed(msg)) => {
                self.security_manager.record_failed_attempt(&req.from);
                return Err(msg);
            },
        }

        // Balance check including fee
        if req.from != "genesis" && req.from != "mining_reward" && req.from != "connection_reward" && req.from != "social_mining" {
            let balance = self.get_balance(&req.from);
            let total_needed = req.amount + req.fee;
            if balance < total_needed {
                self.security_manager.record_failed_attempt(&req.from);
                return Err(format!("Insufficient balance. Have: {}, Need: {} (including fee: {})", 
                                 balance, total_needed, req.fee));
            }
        }

        // Create enhanced transaction
        let mut enhanced_tx = EnhancedTransaction::new(req.from.clone(), req.to.clone(), req.amount, req.fee);
        
        // Add message if provided
        if let Some(message) = req.message {
            enhanced_tx = enhanced_tx.with_message(message);
        }

        let tx_id = enhanced_tx.id.clone();

        // Add to enhanced pool
        if let Err(e) = self.enhanced_tx_pool.add_transaction(enhanced_tx.clone()) {
            return Err(e);
        }

        // Also add to legacy pool for compatibility
        let legacy_tx = enhanced_tx.to_legacy_transaction();
        self.pending_transactions.push(legacy_tx);

        println!("🔒 Enhanced transaction created: {} -> {} (Amount: {}, Fee: {}, ID: {})", 
                 req.from, req.to, req.amount, req.fee, tx_id);

        Ok(format!("Enhanced transaction created with ID: {}", tx_id))
    }

    // Enhanced mining with security and transaction fees
    fn mine_enhanced_block(&mut self, miner_address: String) -> Result<String, String> {
        // Security checks for mining
        match self.security_manager.check_mining_security(&miner_address) {
            Ok(_) => {},
            Err(SecurityError::RateLimitExceeded) => {
                return Err("Mining rate limit exceeded. Please wait before mining again.".to_string());
            },
            Err(SecurityError::BlacklistedAddress) => {
                return Err("Miner address is blacklisted.".to_string());
            },
            Err(_) => {
                return Err("Mining security check failed.".to_string());
            },
        }

        if self.pending_transactions.is_empty() {
            return Err("No pending transactions to mine".to_string());
        }

        // Get transactions sorted by priority (fee)
        let priority_txs = self.enhanced_tx_pool.get_transactions_by_priority();
        let mut total_fees = 0.0;
        let mut confirmed_tx_ids = Vec::new();

        // Process enhanced transactions and calculate fees
        for enhanced_tx in priority_txs.iter().take(100) { // Limit block size
            total_fees += enhanced_tx.fee;
            confirmed_tx_ids.push(enhanced_tx.id.clone());
        }

        // Mining reward transaction (includes collected fees)
        let total_reward = self.mining_reward + total_fees;
        let reward_tx = Transaction {
            from: "mining_reward".to_string(),
            to: miner_address.clone(),
            amount: total_reward,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            signature: format!("mining_reward_{}", rand::random::<u64>()),
        };
        self.pending_transactions.push(reward_tx);

        let previous_block = self.chain.last().unwrap();
        let mut new_block = Block::new(
            previous_block.index + 1,
            self.pending_transactions.clone(),
            previous_block.hash.clone(),
            miner_address.clone(),
        );
        
        new_block.mine_block(self.difficulty);
        self.chain.push(new_block);
        
        // Confirm transactions in enhanced pool
        for tx_id in confirmed_tx_ids {
            let _ = self.enhanced_tx_pool.confirm_transaction(&tx_id);
        }
        
        self.update_balances();
        self.pending_transactions.clear();

        println!("⛏️ Enhanced block mined by {} with {} total reward (including {} fees)", 
                 miner_address, total_reward, total_fees);

        Ok(format!("Enhanced block mined successfully! Total reward: {}", total_reward))
    }

    // Keep original mining method for compatibility
    fn mine_pending_transactions(&mut self, miner_address: String) {
        if self.pending_transactions.is_empty() {
            println!("No pending transactions to mine");
            return;
        }

        let reward_tx = Transaction {
            from: "mining_reward".to_string(),
            to: miner_address.clone(),
            amount: self.mining_reward,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            signature: format!("mining_reward_{}", rand::random::<u64>()),
        };
        self.pending_transactions.push(reward_tx);

        let previous_block = self.chain.last().unwrap();
        let mut new_block = Block::new(
            previous_block.index + 1,
            self.pending_transactions.clone(),
            previous_block.hash.clone(),
            miner_address.clone(),
        );
        
        new_block.mine_block(self.difficulty);
        self.chain.push(new_block);
        
        self.update_balances();
        self.pending_transactions.clear();
    }

    fn connect_user(&mut self, address: String) -> Result<String, String> {
        // Security check for connections
        match self.security_manager.check_connection_security(&address) {
            Ok(_) => {},
            Err(SecurityError::RateLimitExceeded) => {
                return Err("Connection rate limit exceeded. Please wait before connecting again.".to_string());
            },
            Err(SecurityError::BlacklistedAddress) => {
                return Err("Address is blacklisted and cannot connect.".to_string());
            },
            Err(_) => {
                return Err("Connection security check failed.".to_string());
            },
        }

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        
        if self.connections.contains_key(&address) {
            // Update existing connection
            if let Some(conn) = self.connections.get_mut(&address) {
                conn.last_activity = now;
                conn.is_active = true;
                return Ok(format!("Welcome back, {}! Connection reactivated.", address));
            }
        }
        
        // Create new connection
        let connection = Connection {
            address: address.clone(),
            connected_at: now,
            last_activity: now,
            total_rewards: 0.0,
            is_active: true,
        };
        
        self.connections.insert(address.clone(), connection);
        println!("👤 New user connected: {}", address);
        
        Ok(format!("Welcome, {}! You're now connected to the network.", address))
    }

    fn disconnect_user(&mut self, address: &str) -> Result<String, String> {
        if let Some(conn) = self.connections.get_mut(address) {
            conn.is_active = false;
            Ok(format!("User {} disconnected.", address))
        } else {
            Err("User not found in connections.".to_string())
        }
    }

    fn calculate_connection_reward(&self) -> f64 {
        // Exponential decay based on circulating supply
        let remaining_percentage = (self.max_supply - self.circulating_supply) / self.max_supply;
        let base_reward = 1.0; // Base reward per minute
        
        // Exponential decay: reward = base * e^(-decay_rate * supply_used)
        let decay_rate = 2.0;
        let supply_used_percentage = 1.0 - remaining_percentage;
        let reward = base_reward * (-decay_rate * supply_used_percentage).exp();
        
        // Minimum reward of 0.001
        reward.max(0.001)
    }

    fn process_connection_rewards(&mut self) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let reward_per_minute = self.calculate_connection_reward();
        
        let mut rewards_given = Vec::new();
        
        for (address, connection) in self.connections.iter_mut() {
            if !connection.is_active {
                continue;
            }
            
            let connected_duration = now - connection.connected_at;
            
            // Give reward every minute (60 seconds)
            if connected_duration >= 60 && (connected_duration % 60) < 5 { // 5 second window
                if self.circulating_supply < self.max_supply {
                    connection.total_rewards += reward_per_minute;
                    *self.balances.entry(address.clone()).or_insert(0.0) += reward_per_minute;
                    self.circulating_supply += reward_per_minute;
                    
                    rewards_given.push((address.clone(), reward_per_minute));
                    
                    // Create reward transaction
                    let reward_tx = Transaction {
                        from: "connection_reward".to_string(),
                        to: address.clone(),
                        amount: reward_per_minute,
                        timestamp: now,
                        signature: "connection_reward".to_string(),
                    };
                    self.pending_transactions.push(reward_tx);
                }
            }
            
            connection.last_activity = now;
        }
        
        // Log rewards given
        for (address, reward) in rewards_given {
            println!("🎁 Connection reward given: {} received {:.4} L1", address, reward);
        }
    }

    fn register_username(&mut self, username: String) -> Result<(String, String), String> {
        // Validate username format
        if username.len() < 3 || username.len() > 20 {
            return Err("Username must be between 3-20 characters".to_string());
        }
        
        if !username.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err("Username can only contain letters, numbers, underscores, and hyphens".to_string());
        }
        
        // Check if username is already taken
        if self.address_labels.contains_key(&username) {
            return Err("Username is already taken".to_string());
        }
        
        // Generate wallet address from username
        let wallet_address = format!("wallet_{}", username);
        
        // Check if this generated address already exists (shouldn't happen but safety check)
        if self.address_to_username.contains_key(&wallet_address) {
            return Err("Generated address already exists".to_string());
        }
        
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        
        let label = AddressLabel {
            username: username.clone(),
            address: wallet_address.clone(),
            registered_at: now,
            is_verified: true, // Auto-verify since we control the address generation
        };
        
        self.address_labels.insert(username.clone(), label);
        self.address_to_username.insert(wallet_address.clone(), username.clone());
        
        // Give initial balance to new wallet (signup bonus)
        match self.create_transaction("genesis".to_string(), wallet_address.clone(), 1000.0) {
            Ok(_) => {
                println!("📝 Username registered: {} -> {} (with 1000 L1 signup bonus)", username, wallet_address);
                
                // Auto-mine the signup bonus transaction
                self.mine_pending_transactions("system".to_string());
                
                Ok((username, wallet_address))
            },
            Err(e) => {
                // Clean up if transaction fails
                self.address_labels.remove(&username);
                self.address_to_username.remove(&wallet_address);
                Err(format!("Failed to create signup bonus: {}", e))
            }
        }
    }

    fn resolve_username(&self, username: &str) -> Result<&AddressLabel, String> {
        self.address_labels.get(username)
            .ok_or_else(|| format!("Username '{}' not found", username))
    }
    
    fn get_username_by_address(&self, address: &str) -> Option<&String> {
        self.address_to_username.get(address)
    }
    
    fn get_all_labels(&self) -> Vec<&AddressLabel> {
        self.address_labels.values().collect()
    }
    
    // Enhanced create_transaction that supports usernames
    fn create_transaction_with_labels(&mut self, from: String, to: String, amount: f64) -> Result<String, String> {
        // Resolve 'from' address if it's a username
        let from_address = if from.starts_with('@') || self.address_labels.contains_key(&from) {
            let username = if from.starts_with('@') { &from[1..] } else { &from };
            match self.resolve_username(username) {
                Ok(label) => label.address.clone(),
                Err(e) => return Err(format!("From address resolution failed: {}", e)),
            }
        } else {
            from
        };
        
        // Resolve 'to' address if it's a username
        let to_address = if to.starts_with('@') || self.address_labels.contains_key(&to) {
            let username = if to.starts_with('@') { &to[1..] } else { &to };
            match self.resolve_username(username) {
                Ok(label) => label.address.clone(),
                Err(e) => return Err(format!("To address resolution failed: {}", e)),
            }
        } else {
            to
        };
        
        // Use the existing create_transaction method with resolved addresses
        self.create_transaction(from_address, to_address, amount)
    }
    
    // Enhanced transaction display with usernames
    fn format_transaction_with_labels(&self, tx: &Transaction) -> serde_json::Value {
        let from_display = self.get_username_by_address(&tx.from)
            .map(|username| format!("@{}", username))
            .unwrap_or_else(|| tx.from.clone());
            
        let to_display = self.get_username_by_address(&tx.to)
            .map(|username| format!("@{}", username))
            .unwrap_or_else(|| tx.to.clone());
        
        serde_json::json!({
            "from": tx.from,
            "from_display": from_display,
            "to": tx.to,
            "to_display": to_display,
            "amount": tx.amount,
            "timestamp": tx.timestamp,
            "signature": tx.signature
        })
    }

    // Get enhanced transaction pool statistics
    fn get_pool_stats(&self) -> PoolStats {
        self.enhanced_tx_pool.get_stats()
    }

    // Get transaction receipt
    fn get_transaction_receipt(&self, tx_id: &str) -> Option<TransactionReceipt> {
        // Check confirmed transactions
        for tx in self.enhanced_tx_pool.get_confirmed_transactions() {
            if tx.id == tx_id {
                // Find block number
                let block_number = self.chain.iter().enumerate()
                    .find(|(_, block)| {
                        block.transactions.iter().any(|t| {
                            format!("sig_{}_{}", t.from, t.timestamp.to_string()) == tx.signature
                        })
                    })
                    .map(|(i, _)| i as u64);
                
                return Some(TransactionReceipt::new(tx, block_number));
            }
        }
        None
    }

    // Cleanup expired transactions and security components
    fn cleanup(&mut self) {
        let expired_count = self.enhanced_tx_pool.cleanup_expired();
        if expired_count > 0 {
            println!("🧹 Cleaned up {} expired transactions", expired_count);
        }
        
        self.security_manager.cleanup();
    }

    // Get security statistics
    fn get_security_stats(&self) -> SecurityStats {
        self.security_manager.get_security_stats()
    }

    // Admin methods for security management
    fn admin_blacklist_address(&mut self, address: String, reason: Option<String>) {
        self.security_manager.admin_blacklist(address, reason);
    }

    fn admin_unblacklist_address(&mut self, address: &str) -> bool {
        self.security_manager.admin_unblacklist(address)
    }

    fn update_balances(&mut self) {
        self.balances.clear();
        self.circulating_supply = 0.0;
        
        for block in &self.chain {
            for transaction in &block.transactions {
                if transaction.from != "genesis" && transaction.from != "mining_reward" && transaction.from != "connection_reward" && transaction.from != "social_mining" {
                    *self.balances.entry(transaction.from.clone()).or_insert(0.0) -= transaction.amount;
                }
                
                *self.balances.entry(transaction.to.clone()).or_insert(0.0) += transaction.amount;
                
                if transaction.from == "mining_reward" || transaction.from == "connection_reward" || transaction.from == "genesis" || transaction.from == "social_mining" {
                    self.circulating_supply += transaction.amount;
                }
            }
        }
        
        self.balances.retain(|_, &mut balance| balance > 0.0);
    }

    fn get_balance(&self, address: &str) -> f64 {
        *self.balances.get(address).unwrap_or(&0.0)
    }

    fn get_connection_info(&self, address: &str) -> Option<&Connection> {
        self.connections.get(address)
    }

    fn get_all_connections(&self) -> Vec<&Connection> {
        self.connections.values().filter(|c| c.is_active).collect()
    }

    fn get_network_stats(&self) -> NetworkStats {
        NetworkStats {
            total_supply: self.max_supply,
            circulating_supply: self.circulating_supply,
            remaining_supply: self.max_supply - self.circulating_supply,
            current_reward_rate: self.calculate_connection_reward(),
            active_connections: self.connections.values().filter(|c| c.is_active).count(),
            total_blocks: self.chain.len(),
        }
    }

    fn get_all_balances(&self) -> Vec<BalanceResponse> {
        self.balances
            .iter()
            .filter(|(_, &balance)| balance > 0.0)
            .map(|(address, &balance)| BalanceResponse {
                address: address.clone(),
                balance,
            })
            .collect()
    }

    fn get_user_wallet(&self, user_id: &str) -> Option<UserWalletInfo> {
        let wallet_address = format!("wallet_{}", user_id);
        let balance = self.get_balance(&wallet_address);
        
        if balance > 0.0 || self.balances.contains_key(&wallet_address) {
            let mut total_sent = 0.0;
            let mut total_received = 0.0;
            let mut transaction_count = 0;
            
            for block in &self.chain {
                for tx in &block.transactions {
                    if tx.from == wallet_address || tx.to == wallet_address {
                        transaction_count += 1;
                        if tx.from == wallet_address && tx.from != "genesis" && tx.from != "mining_reward" {
                            total_sent += tx.amount;
                        }
                        if tx.to == wallet_address {
                            total_received += tx.amount;
                        }
                    }
                }
            }
            
            Some(UserWalletInfo {
                address: wallet_address,
                balance,
                total_sent,
                total_received,
                transaction_count,
            })
        } else {
            None
        }
    }

    fn create_user_wallet(&self, user_id: &str) -> Result<UserWalletInfo, String> {
        match self.get_user_wallet(user_id) {
            Some(wallet) => Ok(wallet),
            None => {
                // Return a new wallet info for users that don't exist yet
                let wallet_address = format!("wallet_{}", user_id);
                Ok(UserWalletInfo {
                    address: wallet_address,
                    balance: 0.0,
                    total_sent: 0.0,
                    total_received: 0.0,
                    transaction_count: 0,
                })
            }
        }
    }

    fn get_user_wallet_by_username(&self, username: &str) -> Option<WalletInfoResponse> {
        // Try to resolve username first
        if let Ok(label) = self.resolve_username(username) {
            let address = &label.address;
            let balance = self.get_balance(address);
            
            let mut total_sent = 0.0;
            let mut total_received = 0.0;
            let mut transaction_count = 0;
            
            for block in &self.chain {
                for tx in &block.transactions {
                    if tx.from == *address || tx.to == *address {
                        transaction_count += 1;
                        if tx.from == *address && tx.from != "genesis" && tx.from != "mining_reward" {
                            total_sent += tx.amount;
                        }
                        if tx.to == *address {
                            total_received += tx.amount;
                        }
                    }
                }
            }
            
            Some(WalletInfoResponse {
                address: address.clone(),
                balance,
                username: Some(username.to_string()),
                is_verified: label.is_verified,
                total_sent,
                total_received,
                transaction_count,
                connection_info: self.get_connection_info(address).cloned(),
            })
        } else {
            None
        }
    }

    fn get_transaction_history(&self, address: &str) -> TransactionHistoryResponse {
        let mut transactions = Vec::new();
        
        for block in &self.chain {
            for tx in &block.transactions {
                if tx.from == address || tx.to == address {
                    transactions.push(serde_json::json!({
                        "from": tx.from,
                        "to": tx.to,
                        "amount": tx.amount,
                        "timestamp": tx.timestamp,
                        "signature": tx.signature,
                        "block_index": block.index
                    }));
                }
            }
        }
        
        // Sort by timestamp (newest first)
        transactions.sort_by(|a, _b| {
            let timestamp_a = a["timestamp"].as_u64().unwrap_or(0);
            let timestamp_b = a["timestamp"].as_u64().unwrap_or(0);
            timestamp_b.cmp(&timestamp_a)
        });
        
        TransactionHistoryResponse {
            address: address.to_string(),
            transactions: transactions.clone(),
            total_count: transactions.len(),
        }
    }

    fn get_transaction_history_with_labels(&self, address: &str) -> TransactionHistoryResponse {
        let mut transactions = Vec::new();
        
        for block in &self.chain {
            for tx in &block.transactions {
                if tx.from == address || tx.to == address {
                    transactions.push(self.format_transaction_with_labels(tx));
                }
            }
        }
        
        // Sort by timestamp (newest first)
        transactions.sort_by(|a, _b| {
            let timestamp_a = a["timestamp"].as_u64().unwrap_or(0);
            let timestamp_b = a["timestamp"].as_u64().unwrap_or(0);
            timestamp_b.cmp(&timestamp_a)
        });
        
        TransactionHistoryResponse {
            address: address.to_string(),
            transactions: transactions.clone(),
            total_count: transactions.len(),
        }
    }

    fn get_wallet_info(&self, address: &str) -> WalletInfoResponse {
        let balance = self.get_balance(address);
        let username = self.get_username_by_address(address).cloned();
        let is_verified = username.as_ref()
            .and_then(|u| self.address_labels.get(u))
            .map(|label| label.is_verified)
            .unwrap_or(false);
        
        let mut total_sent = 0.0;
        let mut total_received = 0.0;
        let mut transaction_count = 0;
        
        for block in &self.chain {
            for tx in &block.transactions {
                if tx.from == address || tx.to == address {
                    transaction_count += 1;
                    if tx.from == address && tx.from != "genesis" && tx.from != "mining_reward" {
                        total_sent += tx.amount;
                    }
                    if tx.to == address {
                        total_received += tx.amount;
                    }
                }
            }
        }
        
        WalletInfoResponse {
            address: address.to_string(),
            balance,
            username,
            is_verified,
            total_sent,
            total_received,
            transaction_count,
            connection_info: self.get_connection_info(address).cloned(),
        }
    }

    fn send_tip(&mut self, from: String, to: String, amount: f64, message: Option<String>) -> Result<String, String> {
        // First create the transaction
        let result = self.create_transaction_with_labels(from.clone(), to.clone(), amount);
        
        match result {
            Ok(_) => {
                let tip_message = match message {
                    Some(msg) => format!(" with message: '{}'", msg),
                    None => String::new(),
                };
                
                println!("💝 Tip sent: {} -> {} (Amount: {}){}", from, to, amount, tip_message);
                Ok(format!("Tip of {} L1 sent successfully{}", amount, tip_message))
            },
            Err(e) => Err(e)
        }
    }

    // Token system methods
    fn launch_token(&mut self, req: LaunchTokenRequest) -> Result<Token, String> {
        // First, resolve the creator address if it's a username
        let creator_address = if req.creator.starts_with('@') || self.address_labels.contains_key(&req.creator) {
            // It's a username, resolve it
            let username = if req.creator.starts_with('@') { &req.creator[1..] } else { &req.creator };
            match self.resolve_username(username) {
                Ok(label) => label.address.clone(),
                Err(_) => req.creator.clone(), // Fallback to original if resolution fails
            }
        } else {
            // Check if we have this username in our system
            self.address_labels.get(&req.creator)
                .map(|label| label.address.clone())
                .unwrap_or(req.creator.clone())
        };

        // Check creator balance using the resolved address
        let creator_balance = self.get_balance(&creator_address);
        
        // Create a new request with the resolved address
        let resolved_req = LaunchTokenRequest {
            symbol: req.symbol,
            name: req.name,
            description: req.description,
            creator: creator_address.clone(), // Use resolved address
            total_supply: req.total_supply,
            initial_price: req.initial_price,
            initial_liquidity: req.initial_liquidity,
            image_url: req.image_url,
            website: req.website,
            twitter: req.twitter,
            telegram: req.telegram,
        };

        let token = self.token_system.launch_token(resolved_req, creator_balance)?;
        
        // Create transaction for launch fee using resolved address
        let launch_fee = self.token_system.launch_fee;
        match self.create_transaction(creator_address, "token_launch_fees".to_string(), launch_fee) {
            Ok(_) => {
                println!("💰 Token launch fee collected: {} L1", launch_fee);
                Ok(token)
            },
            Err(e) => Err(format!("Failed to collect launch fee: {}", e))
        }
    }

    fn buy_token(&mut self, req: BuyTokenRequest) -> Result<(TokenTrade, String), String> {
        // Resolve buyer address if it's a username
        let buyer_address = if req.buyer.starts_with('@') || self.address_labels.contains_key(&req.buyer) {
            let username = if req.buyer.starts_with('@') { &req.buyer[1..] } else { &req.buyer };
            match self.resolve_username(username) {
                Ok(label) => label.address.clone(),
                Err(_) => req.buyer.clone(),
            }
        } else {
            self.address_labels.get(&req.buyer)
                .map(|label| label.address.clone())
                .unwrap_or(req.buyer.clone())
        };

        let buyer_balance = self.get_balance(&buyer_address);
        
        let resolved_req = BuyTokenRequest {
            token_symbol: req.token_symbol,
            buyer: buyer_address.clone(),
            l1_amount: req.l1_amount,
            max_slippage: req.max_slippage,
        };
        
        let trade = self.token_system.buy_token(resolved_req, buyer_balance)?;
        
        // Create L1 transaction for the purchase
        let tx_result = self.create_transaction(
            buyer_address,
            format!("token_pool_{}", trade.token_symbol),
            trade.l1_amount
        );
        
        match tx_result {
            Ok(msg) => Ok((trade, msg)),
            Err(e) => Err(format!("Failed to process L1 transaction: {}", e))
        }
    }

    fn sell_token(&mut self, req: SellTokenRequest) -> Result<(TokenTrade, String), String> {
        // Resolve seller address if it's a username
        let seller_address = if req.seller.starts_with('@') || self.address_labels.contains_key(&req.seller) {
            let username = if req.seller.starts_with('@') { &req.seller[1..] } else { &req.seller };
            match self.resolve_username(username) {
                Ok(label) => label.address.clone(),
                Err(_) => req.seller.clone(),
            }
        } else {
            self.address_labels.get(&req.seller)
                .map(|label| label.address.clone())
                .unwrap_or(req.seller.clone())
        };

        let resolved_req = SellTokenRequest {
            token_symbol: req.token_symbol,
            seller: seller_address.clone(),
            token_amount: req.token_amount,
            max_slippage: req.max_slippage,
        };
        
        let trade = self.token_system.sell_token(resolved_req)?;
        
        // Create L1 transaction to give seller their L1
        let tx_result = self.create_transaction(
            format!("token_pool_{}", trade.token_symbol),
            seller_address,
            trade.l1_amount
        );
        
        match tx_result {
            Ok(msg) => Ok((trade, msg)),
            Err(e) => Err(format!("Failed to process L1 payout: {}", e))
        }
    }

    fn get_user_token_portfolio(&self, user: &str) -> UserPortfolioResponse {
        // Resolve user address if it's a username
        let user_address = if user.starts_with('@') || self.address_labels.contains_key(user) {
            let username = if user.starts_with('@') { &user[1..] } else { user };
            match self.resolve_username(username) {
                Ok(label) => label.address.clone(),
                Err(_) => user.to_string(),
            }
        } else {
            self.address_labels.get(user)
                .map(|label| label.address.clone())
                .unwrap_or(user.to_string())
        };

        let holdings: Vec<TokenHolding> = self.token_system.get_user_holdings(&user_address)
            .map(|h| h.values().cloned().collect())
            .unwrap_or_default();
        
        let mut total_value_l1 = 0.0;
        let mut total_pnl = 0.0;
        
        for holding in &holdings {
            if let Some(token) = self.token_system.get_token_info(&holding.token_symbol) {
                let current_value = holding.amount * token.price_in_l1;
                let original_value = holding.amount * holding.average_price;
                total_value_l1 += current_value;
                total_pnl += current_value - original_value;
            }
        }
        
        UserPortfolioResponse {
            user: user.to_string(),
            holdings,
            total_value_l1,
            total_pnl,
        }
    }

    // Social Mining Methods

    fn process_social_post(&mut self, req: SocialPostRequest) -> Result<SocialActionResponse, String> {
        // Resolve user address if username provided
        let user_address = self.resolve_user_address(&req.user_address)?;

        // Check daily limits
        self.social_mining.check_daily_limits(&user_address, &social_mining::SocialActionType::Post)?;

        // Calculate reward (fixed 10 tokens for posting)
        let reward_amount = self.social_mining.calculate_reward(&social_mining::SocialActionType::Post, self.max_supply);

        // Check if we have enough supply left
        if self.circulating_supply + reward_amount > self.max_supply {
            return Err("Maximum supply reached, no more social rewards available".to_string());
        }

        // Create reward transaction
        match self.create_transaction("social_mining".to_string(), user_address.clone(), reward_amount) {
            Ok(_) => {
                // Record the social action
                let action = social_mining::SocialAction {
                    action_type: social_mining::SocialActionType::Post,
                    user_address: user_address.clone(),
                    post_id: req.post_id.clone(),
                    target_user: None,
                    timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    reward_amount,
                };

                self.social_mining.record_action(action);
                self.social_mining.update_daily_limits(&user_address, &social_mining::SocialActionType::Post);

                // Auto-mine the reward
                self.mine_pending_transactions("social_system".to_string());

                println!("📝 Social Post Reward: {} received {} L1 for post {}", user_address, reward_amount, req.post_id);

                Ok(SocialActionResponse {
                    success: true,
                    message: format!("Post reward of {} L1 awarded!", reward_amount),
                    reward_amount,
                    action_type: "post".to_string(),
                })
            },
            Err(e) => Err(format!("Failed to create reward transaction: {}", e))
        }
    }

    fn process_social_like(&mut self, req: SocialLikeRequest) -> Result<SocialActionResponse, String> {
        // Resolve user addresses
        let user_address = self.resolve_user_address(&req.user_address)?;
        let post_author_address = self.resolve_user_address(&req.post_author)?;

        // Prevent self-liking
        if user_address == post_author_address {
            return Err("Cannot like your own post".to_string());
        }

        // Check daily limits
        self.social_mining.check_daily_limits(&user_address, &social_mining::SocialActionType::Like)?;

        // Calculate reward (1/100000 of total supply)
        let reward_amount = self.social_mining.calculate_reward(&social_mining::SocialActionType::Like, self.max_supply);

        // Check supply
        if self.circulating_supply + reward_amount > self.max_supply {
            return Err("Maximum supply reached, no more social rewards available".to_string());
        }

        // Create reward transaction (reward goes to the POST AUTHOR, not the liker)
        match self.create_transaction("social_mining".to_string(), post_author_address.clone(), reward_amount) {
            Ok(_) => {
                // Record the social action
                let action = social_mining::SocialAction {
                    action_type: social_mining::SocialActionType::Like,
                    user_address: user_address.clone(),
                    post_id: req.post_id.clone(),
                    target_user: Some(post_author_address.clone()),
                    timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    reward_amount,
                };

                self.social_mining.record_action(action);
                self.social_mining.update_daily_limits(&user_address, &social_mining::SocialActionType::Like);

                // Auto-mine the reward
                self.mine_pending_transactions("social_system".to_string());

                println!("👍 Social Like Reward: {} received {} L1 for like on post {} by {}", 
                         post_author_address, reward_amount, req.post_id, user_address);

                Ok(SocialActionResponse {
                    success: true,
                    message: format!("Like recorded! Post author received {} L1 reward", reward_amount),
                    reward_amount,
                    action_type: "like".to_string(),
                })
            },
            Err(e) => Err(format!("Failed to create reward transaction: {}", e))
        }
    }

    fn process_social_comment(&mut self, req: SocialCommentRequest) -> Result<SocialActionResponse, String> {
        // Resolve user addresses
        let user_address = self.resolve_user_address(&req.user_address)?;
        let post_author_address = self.resolve_user_address(&req.post_author)?;

        // Check daily limits
        self.social_mining.check_daily_limits(&user_address, &social_mining::SocialActionType::Comment)?;

        // Calculate reward (1/100000 of total supply)
        let reward_amount = self.social_mining.calculate_reward(&social_mining::SocialActionType::Comment, self.max_supply);

        // Check supply
        if self.circulating_supply + reward_amount > self.max_supply {
            return Err("Maximum supply reached, no more social rewards available".to_string());
        }

        // Create reward transaction (reward goes to the COMMENTER)
        match self.create_transaction("social_mining".to_string(), user_address.clone(), reward_amount) {
            Ok(_) => {
                // Record the social action
                let action = social_mining::SocialAction {
                    action_type: social_mining::SocialActionType::Comment,
                    user_address: user_address.clone(),
                    post_id: req.post_id.clone(),
                    target_user: Some(post_author_address.clone()),
                    timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    reward_amount,
                };

                self.social_mining.record_action(action);
                self.social_mining.update_daily_limits(&user_address, &social_mining::SocialActionType::Comment);

                // Auto-mine the reward
                self.mine_pending_transactions("social_system".to_string());

                println!("💬 Social Comment Reward: {} received {} L1 for commenting on post {} by {}", 
                         user_address, reward_amount, req.post_id, post_author_address);

                Ok(SocialActionResponse {
                    success: true,
                    message: format!("Comment reward of {} L1 awarded!", reward_amount),
                    reward_amount,
                    action_type: "comment".to_string(),
                })
            },
            Err(e) => Err(format!("Failed to create reward transaction: {}", e))
        }
    }

    fn get_social_stats(&self) -> SocialStatsResponse {
        let mut stats = self.social_mining.get_stats();
        
        // Add usernames to top earners
        for earner in &mut stats.top_earners {
            earner.username = self.get_username_by_address(&earner.user_address).cloned();
        }
        
        stats
    }

    fn resolve_user_address(&self, input: &str) -> Result<String, String> {
        // If it starts with @ or is a known username, resolve it
        if input.starts_with('@') || self.address_labels.contains_key(input) {
            let username = if input.starts_with('@') { &input[1..] } else { input };
            match self.resolve_username(username) {
                Ok(label) => Ok(label.address.clone()),
                Err(_) => Err(format!("Username '{}' not found", username))
            }
        } else {
            // Assume it's already an address
            Ok(input.to_string())
        }
    }
}

#[tokio::main]
async fn main() {
    let blockchain = Arc::new(Mutex::new(Blockchain::new()));

    // Create clones for different endpoint handlers
    let blockchain_clone = blockchain.clone();
    let bc_transaction = blockchain.clone();
    let bc_enhanced_tx = blockchain.clone();
    let bc_tx_usernames = blockchain.clone();
    let bc_mine = blockchain.clone();
    let bc_enhanced_mine = blockchain.clone();
    let bc_balance = blockchain.clone();
    let bc_connect = blockchain.clone();
    let bc_disconnect = blockchain.clone();
    let bc_connections = blockchain.clone();
    let bc_stats = blockchain.clone();
    let bc_pool_stats = blockchain.clone();
    let bc_create_wallet = blockchain.clone();
    let bc_get_wallet = blockchain.clone();
    let bc_get_wallet_username = blockchain.clone();
    let bc_tx_history = blockchain.clone();
    let bc_tx_history_labels = blockchain.clone();
    let bc_wallet_info = blockchain.clone();
    let bc_tx_receipt = blockchain.clone();
    let bc_tip = blockchain.clone();
    let bc_register = blockchain.clone();
    let bc_resolve = blockchain.clone();
    let bc_labels = blockchain.clone();
    let bc_launch_token = blockchain.clone();
    let bc_buy_token = blockchain.clone();
    let bc_sell_token = blockchain.clone();
    let bc_all_tokens = blockchain.clone();
    let bc_trending_tokens = blockchain.clone();
    let bc_token_info = blockchain.clone();
    let bc_portfolio = blockchain.clone();
    let bc_social_post = blockchain.clone();
    let bc_social_like = blockchain.clone();
    let bc_social_comment = blockchain.clone();
    let bc_social_stats = blockchain.clone();
     let bc_stats2 = blockchain.clone();  // For get_all_balances
    let bc_stats3 = blockchain.clone();  // For admin_blacklist  
    let bc_stats4 = blockchain.clone();  // For admin_unblacklist
    let bc_security_stats = blockchain.clone();  // For get_security_stats
    let bc_network_stats = blockchain.clone();   // For get_network_stats

    // Start connection reward processing (every 30 seconds)
    let bc_rewards = blockchain.clone();
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            let mut bc = bc_rewards.lock().unwrap();
            bc.process_connection_rewards();
        }
    });

    // Cleanup task for security, expired transactions, and social mining
    let blockchain_cleanup = blockchain.clone();
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(300)); // Every 5 minutes
        loop {
            interval.tick().await;
            let mut bc = blockchain_cleanup.lock().unwrap();
            bc.cleanup();
            bc.social_mining.cleanup_old_actions();
        }
    });

    // Health check endpoint
    let health_check = warp::path("health")
        .and(warp::get())
        .map(|| warp::reply::json(&serde_json::json!({
            "status": "healthy",
            "blockchain": "Layer1",
            "version": "2.0.0"
        })));

    // GET blockchain state
    let get_blockchain = warp::path("blockchain")
        .and(warp::get())
        .map(move || {
            let bc = blockchain_clone.lock().unwrap();
            warp::reply::json(&*bc)
        });

    // POST transaction
    let create_transaction = warp::path("transaction")
        .and(warp::post())
        .and(warp::body::json())
        .map(move |req: TransactionRequest| {
            let mut bc = bc_transaction.lock().unwrap();
            match bc.create_transaction(req.from, req.to, req.amount) {
                Ok(msg) => warp::reply::json(&serde_json::json!({
                    "success": true,
                    "message": msg
                })),
                Err(err) => warp::reply::json(&serde_json::json!({
                    "success": false,
                    "error": err
                })),
            }
        });

    // POST enhanced transaction with security
    let create_enhanced_transaction = warp::path("rpc")
        .and(warp::path("transaction"))
        .and(warp::path("enhanced"))
        .and(warp::post())
        .and(warp::body::json())
        .map(move |req: EnhancedTransactionRequest| {
            let mut bc = bc_enhanced_tx.lock().unwrap();
            match bc.create_enhanced_transaction(req) {
                Ok(msg) => warp::reply::json(&serde_json::json!({
                    "success": true,
                    "message": msg
                })),
                Err(err) => warp::reply::json(&serde_json::json!({
                    "success": false,
                    "error": err
                })),
            }
        });

    // POST transaction with username support
    let create_transaction_with_usernames = warp::path("rpc")
        .and(warp::path("transaction"))
        .and(warp::post())
        .and(warp::body::json())
        .map(move |req: TransactionWithUsernamesRequest| {
            let mut bc = bc_tx_usernames.lock().unwrap();
            match bc.create_transaction_with_labels(req.from, req.to, req.amount) {
                Ok(msg) => warp::reply::json(&serde_json::json!({
                    "success": true,
                    "message": msg
                })),
                Err(err) => warp::reply::json(&serde_json::json!({
                    "success": false,
                    "error": err
                })),
            }
        });

    // POST mine block
    let mine_block = warp::path("mine")
        .and(warp::post())
        .and(warp::body::json())
        .map(move |req: MineRequest| {
            let mut bc = bc_mine.lock().unwrap();
            bc.mine_pending_transactions(req.miner_address.clone());
            warp::reply::json(&serde_json::json!({
                "success": true,
                "message": format!("Block mined by {}", req.miner_address)
            }))
        });

    // POST enhanced mine block with security
    let mine_enhanced_block = warp::path("rpc")
        .and(warp::path("mine"))
        .and(warp::post())
        .and(warp::body::json())
        .map(move |req: MineRequest| {
            let mut bc = bc_enhanced_mine.lock().unwrap();
            match bc.mine_enhanced_block(req.miner_address) {
                Ok(msg) => warp::reply::json(&serde_json::json!({
                    "success": true,
                    "message": msg
                })),
                Err(err) => warp::reply::json(&serde_json::json!({
                    "success": false,
                    "error": err
                })),
            }
        });

    // GET balance
    let get_balance = warp::path("balance")
        .and(warp::path::param::<String>())
        .and(warp::get())
        .map(move |address: String| {
            let bc = bc_balance.lock().unwrap();
            let balance = bc.get_balance(&address);
            warp::reply::json(&serde_json::json!({
                "address": address,
                "balance": balance
            }))
        });

    // GET all balances
    let get_all_balances = warp::path("rpc")
        .and(warp::path("balances"))
        .and(warp::get())
        .map(move || {
            let bc = bc_stats2.lock().unwrap();  // Fix: use bc_stats instead of blockchain
            warp::reply::json(&bc.get_all_balances())
        });

    // POST connect user
    let connect_user = warp::path("connect")
        .and(warp::post())
        .and(warp::body::json())
        .map(move |req: ConnectRequest| {
            let mut bc = bc_connect.lock().unwrap();
            match bc.connect_user(req.address) {
                Ok(msg) => warp::reply::json(&serde_json::json!({
                    "success": true,
                    "message": msg
                })),
                Err(err) => warp::reply::json(&serde_json::json!({
                    "success": false,
                    "error": err
                })),
            }
        });

    // POST disconnect user
    let disconnect_user = warp::path("disconnect")
        .and(warp::post())
        .and(warp::body::json())
        .map(move |req: DisconnectRequest| {
            let mut bc = bc_disconnect.lock().unwrap();
            match bc.disconnect_user(&req.address) {
                Ok(msg) => warp::reply::json(&serde_json::json!({
                    "success": true,
                    "message": msg
                })),
                Err(err) => warp::reply::json(&serde_json::json!({
                    "success": false,
                    "error": err
                })),
            }
        });

    // GET connections
    let get_connections = warp::path("connections")
        .and(warp::get())
        .map(move || {
            let bc = bc_connections.lock().unwrap();
            warp::reply::json(&bc.get_all_connections())
        });

    // GET network stats
    let get_stats = warp::path("stats")
        .and(warp::get())
        .map(move || {
            let bc = bc_stats.lock().unwrap();
            warp::reply::json(&bc.get_network_stats())
        });

    // GET transaction pool stats
    let get_pool_stats = warp::path("rpc")
        .and(warp::path("pool"))
        .and(warp::path("stats"))
        .and(warp::get())
        .map(move || {
            let bc = bc_pool_stats.lock().unwrap();
            warp::reply::json(&bc.get_pool_stats())
        });

    // GET security statistics
    let get_security_stats = warp::path("rpc")
        .and(warp::path("security"))
        .and(warp::path("stats"))
        .and(warp::get())
        .map(move || {
            let bc = blockchain.lock().unwrap();
            warp::reply::json(&bc.get_security_stats())
        });

    // POST create wallet
    let create_wallet = warp::path("wallet")
        .and(warp::post())
        .and(warp::body::json())
        .map(move |req: serde_json::Value| {
            let user_id = req.get("user_id").and_then(|v| v.as_str()).unwrap_or("anonymous");
            let bc = bc_create_wallet.lock().unwrap();
            match bc.create_user_wallet(user_id) {
                Ok(wallet_info) => warp::reply::json(&wallet_info),
                Err(err) => warp::reply::json(&serde_json::json!({
                    "success": false,
                    "error": err
                })),
            }
        });

    // GET wallet by address
    let get_wallet = warp::path("wallet")
        .and(warp::path::param::<String>())
        .and(warp::get())
        .map(move |address: String| {
            let bc = bc_get_wallet.lock().unwrap();
            match bc.get_user_wallet(&address) {
                Some(wallet_info) => warp::reply::json(&wallet_info),
                None => {
                    // Create a default wallet response for new users
                    let wallet_address = format!("wallet_{}", address); // Fix: use address instead of user_id
                    let wallet_info = UserWalletInfo {
                        address: wallet_address,
                        balance: 0.0,
                        total_sent: 0.0,
                        total_received: 0.0,
                        transaction_count: 0,
                    };
                    warp::reply::json(&wallet_info)
                }
            }
        });

    // GET wallet by username - fixed version
    let get_wallet_by_username = warp::path("wallet")
        .and(warp::path("username"))
        .and(warp::path::param::<String>())
        .and(warp::get())
        .map(move |username: String| {
            let bc = bc_get_wallet_username.lock().unwrap();
            match bc.get_user_wallet_by_username(&username) {
                Some(wallet_info) => warp::reply::json(&wallet_info),
                None => warp::reply::json(&serde_json::json!({
                    "success": false,
                    "error": "Wallet not found"
                })),
            }
        });

    // POST tip (send L1 with optional message)
    let send_tip = warp::path("rpc")
        .and(warp::path("tip"))
        .and(warp::post())
        .and(warp::body::json())
        .map(move |req: TipRequest| {
            let mut bc = bc_tip.lock().unwrap();
            match bc.send_tip(req.from, req.to, req.amount, req.message) {
                Ok(msg) => warp::reply::json(&serde_json::json!({
                    "success": true,
                    "message": msg
                })),
                Err(err) => warp::reply::json(&serde_json::json!({
                    "success": false,
                    "error": err
                })),
            }
        });

    // POST username registration
    let register_username = warp::path("rpc")
        .and(warp::path("username"))
        .and(warp::post())
        .and(warp::body::json())
        .map(move |req: UsernameRegisterRequest| {
            let mut bc = bc_register.lock().unwrap();
            match bc.register_username(req.username) {
                Ok((username, address)) => warp::reply::json(&serde_json::json!({
                    "success": true,
                    "username": username,
                    "address": address
                })),
                Err(err) => warp::reply::json(&serde_json::json!({
                    "success": false,
                    "error": err
                })),
            }
        });

    // GET username resolution - fixed version
    let resolve_username = warp::path("rpc")
        .and(warp::path("username"))
        .and(warp::path("resolve"))
        .and(warp::path::param::<String>())
        .and(warp::get())
        .map(move |username: String| {
            let bc = bc_resolve.lock().unwrap();
            match bc.resolve_username(&username) {
                Ok(label) => warp::reply::json(&label),
                Err(err) => warp::reply::json(&serde_json::json!({
                    "success": false,
                    "error": err
                })),
            }
        });

    // GET all usernames and addresses
    let get_all_usernames = warp::path("rpc")
        .and(warp::path("usernames"))
        .and(warp::get())
        .map(move || {
            let bc = bc_labels.lock().unwrap();
            let labels = bc.get_all_labels();
            warp::reply::json(&labels)
        });

    // POST token launch
    let token_launch = warp::path("rpc")
        .and(warp::path("token"))
        .and(warp::path("launch"))
        .and(warp::post())
        .and(warp::body::json())
        .map(move |req: LaunchTokenRequest| {
            let mut bc = bc_launch_token.lock().unwrap();  // Fix: use bc_launch_token instead of blockchain
            match bc.launch_token(req) {
                Ok(token) => warp::reply::json(&token),
                Err(err) => warp::reply::json(&serde_json::json!({
                    "success": false,
                    "error": err
                })),
            }
        });

    // POST buy token
    let buy_token = warp::path("rpc")
        .and(warp::path("token"))
        .and(warp::path("buy"))
        .and(warp::post())
        .and(warp::body::json())
        .map(move |req: BuyTokenRequest| {
            let mut bc = bc_buy_token.lock().unwrap();  // Fix: use bc_buy_token instead of blockchain
            match bc.buy_token(req) {
                Ok((trade, msg)) => warp::reply::json(&serde_json::json!({
                    "success": true,
                    "trade": trade,
                    "message": msg
                })),
                Err(err) => warp::reply::json(&serde_json::json!({
                    "success": false,
                    "error": err
                })),
            }
        });

    // POST sell token
    let sell_token = warp::path("rpc")
        .and(warp::path("token"))
        .and(warp::path("sell"))
        .and(warp::post())
        .and(warp::body::json())
        .map(move |req: SellTokenRequest| {
            let mut bc = bc_sell_token.lock().unwrap();  // Fix: use bc_sell_token instead of blockchain
            match bc.sell_token(req) {
                Ok((trade, msg)) => warp::reply::json(&serde_json::json!({
                    "success": true,
                    "trade": trade,
                    "message": msg
                })),
                Err(err) => warp::reply::json(&serde_json::json!({
                    "success": false,
                    "error": err
                })),
            }
        });

    // POST admin blacklist
    let admin_blacklist = warp::path("admin")
        .and(warp::path("blacklist"))
        .and(warp::post())
        .and(warp::body::json())
        .map(move |req: AdminBlacklistRequest| {
            let mut bc = bc_stats3.lock().unwrap();  // Fix: use bc_stats instead of blockchain
            bc.admin_blacklist_address(req.address.clone(), req.reason.clone());
            warp::reply::json(&serde_json::json!({
                "success": true,
                "address": req.address,
                "reason": req.reason.unwrap_or_default()
            }))
        });

    // POST admin unblacklist
    let admin_unblacklist = warp::path("admin")
        .and(warp::path("unblacklist"))
        .and(warp::post())
        .and(warp::body::json())
        .map(move |req: AdminUnblacklistRequest| {
            let mut bc = bc_stats4.lock().unwrap();  // Fix: use bc_stats instead of blockchain
            let result = bc.admin_unblacklist_address(&req.address);
            warp::reply::json(&serde_json::json!({
                "success": result,
                "address": req.address
            }))
        });

        // GET all tokens
    let get_all_tokens = warp::path("rpc")
        .and(warp::path("tokens"))
        .and(warp::get())
        .map(move || {
            let bc = bc_all_tokens.lock().unwrap();
            warp::reply::json(&bc.token_system.get_all_tokens())
        });

    // GET trending tokens
    let get_trending_tokens = warp::path("rpc")
        .and(warp::path("trending-tokens"))
        .and(warp::get())
        .map(move || {
            let bc = bc_trending_tokens.lock().unwrap();
            warp::reply::json(&bc.token_system.get_trending_tokens(10))
        });

    // GET token info
    let get_token_info = warp::path("rpc")
        .and(warp::path("token"))
        .and(warp::path::param::<String>())
        .and(warp::get())
        .map(move |symbol: String| {
            let bc = bc_token_info.lock().unwrap();
            match bc.token_system.get_token_info(&symbol) {
                Some(token) => warp::reply::json(&token),
                None => warp::reply::json(&serde_json::json!({
                    "success": false,
                    "error": "Token not found"
                })),
            }
        });

    // GET user portfolio
    let get_user_portfolio = warp::path("rpc")
        .and(warp::path("portfolio"))
        .and(warp::path::param::<String>())
        .and(warp::get())
        .map(move |user: String| {
            let bc = bc_portfolio.lock().unwrap();
            warp::reply::json(&bc.get_user_token_portfolio(&user))
        });

    // POST social post
    let social_post = warp::path("rpc")
        .and(warp::path("social"))
        .and(warp::path("post"))
        .and(warp::post())
        .and(warp::body::json())
        .map(move |req: SocialPostRequest| {
            let mut bc = bc_social_post.lock().unwrap();
            match bc.process_social_post(req) {
                Ok(response) => warp::reply::json(&response),
                Err(err) => warp::reply::json(&serde_json::json!({
                    "success": false,
                    "error": err
                })),
            }
        });

    // POST social like
    let social_like = warp::path("rpc")
        .and(warp::path("social"))
        .and(warp::path("like"))
        .and(warp::post())
        .and(warp::body::json())
        .map(move |req: SocialLikeRequest| {
            let mut bc = bc_social_like.lock().unwrap();
            match bc.process_social_like(req) {
                Ok(response) => warp::reply::json(&response),
                Err(err) => warp::reply::json(&serde_json::json!({
                    "success": false,
                    "error": err
                })),
            }
        });

    // POST social comment
    let social_comment = warp::path("rpc")
        .and(warp::path("social"))
        .and(warp::path("comment"))
        .and(warp::post())
        .and(warp::body::json())
        .map(move |req: SocialCommentRequest| {
            let mut bc = bc_social_comment.lock().unwrap();
            match bc.process_social_comment(req) {
                Ok(response) => warp::reply::json(&response),
                Err(err) => warp::reply::json(&serde_json::json!({
                    "success": false,
                    "error": err
                })),
            }
        });

    // GET social stats
    let get_social_stats = warp::path("rpc")
        .and(warp::path("social"))
        .and(warp::path("stats"))
        .and(warp::get())
        .map(move || {
            let bc = bc_social_stats.lock().unwrap();
            warp::reply::json(&bc.get_social_stats())
        });

    // CORS configuration
    
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type", "x-user-id", "x-username", "x-tx-id"])
        .allow_methods(vec!["GET", "POST", "DELETE"]);

    // CORS configuration
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type", "x-user-id", "x-username", "x-tx-id"])
        .allow_methods(vec!["GET", "POST", "DELETE"]);

    println!("🚀 Layer1 Blockchain Server Starting...");
    println!("📡 Server running on http://0.0.0.0:3030");
    println!("");
    println!("📋 Available API Endpoints:");
    println!("");
    println!("🔹 Basic Operations:");
    println!("  GET  /health - Health check");
    println!("  GET  /blockchain - Full blockchain state");
    println!("  GET  /stats - Network statistics");
    println!("");
    println!("💰 Transactions:");
    println!("  POST /transaction - Create basic transaction");
    println!("  POST /rpc/transaction - Create transaction with usernames");
    println!("");
    println!("🔹 Basic Operations:");
    println!("  GET  /health - Health check");
    println!("  GET  /blockchain - Full blockchain state");
    println!("  GET  /stats - Network statistics");
    println!("");
    println!("💰 Transactions:");
    println!("  POST /transaction - Create basic transaction");
    println!("  POST /rpc/transaction - Create transaction with usernames");
    println!("  POST /rpc/transaction/enhanced - Create enhanced transaction with fees");
    println!("  POST /rpc/tip - Send tip with message");
    println!("");
    println!("⛏️ Mining:");
    println!("  POST /mine - Mine block (basic)");
    println!("  POST /rpc/mine - Mine block (enhanced with security)");
    println!("");
    println!("👤 User Management:");
    println!("  POST /connect - Connect user to network");
    println!("  POST /disconnect - Disconnect user");
    println!("  GET  /connections - Active connections");
    println!("  GET  /wallet/{{address}} - Wallet info");
    println!("  GET  /rpc/token/{{symbol}} - Token information");
    println!("  GET  /rpc/portfolio/{{user}} - User token portfolio");
    println!("");
    println!("🔒 Security:");
    println!("  GET  /rpc/security/stats - Security statistics");
    println!("  GET  /rpc/pool/stats - Transaction pool stats");
    println!("  GET  /rpc/transaction/receipt - Transaction receipt");
    println!("  POST /admin/blacklist - Admin blacklist address");
    println!("  POST /admin/unblacklist - Admin unblacklist address");
    println!("");
    println!("🪙 Token Launch & Trading:");
    println!("  POST /rpc/launch-token - Launch new token (10 L1 fee)");
    println!("  POST /rpc/buy-token - Buy token with L1");
    println!("  POST /rpc/sell-token - Sell token for L1");
    println!("  GET  /rpc/tokens - All launched tokens");
    println!("  GET  /rpc/trending-tokens - Trending tokens");
    println!("  GET  /rpc/token/{{symbol}} - Token information");
    println!("  GET  /rpc/portfolio/{{user}} - User token portfolio");
    println!("");
    println!("📱 Social Mining:");
    println!("  POST /rpc/social/post - Create post (earn 10 L1)");
    println!("  POST /rpc/social/like - Like post (author earns L1)");
    println!("  POST /rpc/social/comment - Comment (earn L1)");
    println!("  GET  /rpc/social/stats - Social mining statistics");
    println!("");
    println!("🎯 Features Active:");
    println!("  ✅ Enhanced Security & Rate Limiting");
    println!("  ✅ Username System with Auto-Wallets");
    println!("  ✅ Connection Rewards (Auto-mining)");
    println!("  ✅ Token Launch Platform");
    println!("  ✅ Social Mining System");
    println!("  ✅ Multi-layer Transaction Security");
    println!("");

    // Complete routes definition
    let routes = health_check
        .or(get_blockchain)
        .or(create_transaction)
        .or(create_enhanced_transaction)
        .or(create_transaction_with_usernames)
        .or(mine_block)
        .or(mine_enhanced_block)
        .or(get_balance)
        .or(get_all_balances)
        .or(connect_user)
        .or(disconnect_user)
        .or(get_connections)
        .or(get_stats)
        .or(get_pool_stats)
        .or(get_security_stats)
        .or(create_wallet)
        .or(get_wallet)
        .or(get_wallet_by_username)
        .or(send_tip)
        .or(register_username)
        .or(resolve_username)
        .or(get_all_usernames)
        .or(token_launch)
        .or(buy_token)
        .or(sell_token)
        .or(get_all_tokens)
        .or(get_trending_tokens)
        .or(get_token_info)
        .or(get_user_portfolio)
        .or(social_post)
        .or(social_like)
        .or(social_comment)
        .or(get_social_stats)
        .or(admin_blacklist)
        .or(admin_unblacklist)
        .with(cors);

    warp::serve(routes)
        .run(([0, 0, 0, 0], 3030))
        .await;
    println!("🛑 Server stopped.");
}

