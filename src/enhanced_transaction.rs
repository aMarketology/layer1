use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};

/// Enhanced transaction with security features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedTransaction {
    pub id: String,                    // Unique transaction ID
    pub from: String,
    pub to: String,
    pub amount: f64,
    pub timestamp: u64,
    pub signature: String,
    pub nonce: u64,                    // Prevent replay attacks
    pub fee: f64,                      // Transaction fee
    pub data: Option<String>,          // Optional message/data
    pub status: TransactionStatus,     // Transaction status
    pub hash: String,                  // Transaction hash
}

/// Transaction status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
    Rejected,
    Expired,
}

impl EnhancedTransaction {
    pub fn new(from: String, to: String, amount: f64, fee: f64) -> Self {
        let mut tx = Self {
            id: Uuid::new_v4().to_string(),
            from,
            to,
            amount,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            signature: format!("sig_{}", rand::random::<u64>()),
            nonce: rand::random::<u64>(),
            fee,
            data: None,
            status: TransactionStatus::Pending,
            hash: String::new(),
        };
        tx.hash = tx.calculate_hash();
        tx
    }

    pub fn with_message(mut self, message: String) -> Self {
        self.data = Some(message);
        self.hash = self.calculate_hash();
        self
    }

    pub fn calculate_hash(&self) -> String {
        use sha2::{Sha256, Digest};
        
        let input = format!(
            "{}{}{}{}{}{}{}{}",
            self.id, self.from, self.to, self.amount, 
            self.timestamp, self.nonce, self.fee,
            self.data.as_deref().unwrap_or("")
        );
        
        let mut hasher = Sha256::new();
        hasher.update(input);
        format!("{:x}", hasher.finalize())
    }

    /// Calculate total cost (amount + fee)
    pub fn total_cost(&self) -> f64 {
        self.amount + self.fee
    }

    /// Check if transaction is expired (older than 1 hour)
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        now - self.timestamp > 3600 // 1 hour
    }

    /// Mark transaction as confirmed
    pub fn confirm(&mut self) {
        self.status = TransactionStatus::Confirmed;
        self.hash = self.calculate_hash();
    }

    /// Mark transaction as failed
    pub fn fail(&mut self) {
        self.status = TransactionStatus::Failed;
        self.hash = self.calculate_hash(); // Recalculate hash with new status
    }

    /// Mark transaction as rejected
    pub fn reject(&mut self) {
        self.status = TransactionStatus::Rejected;
        self.hash = self.calculate_hash(); // Recalculate hash with new status
    }

    /// Mark transaction as expired
    pub fn expire(&mut self) {
        self.status = TransactionStatus::Expired;
        self.hash = self.calculate_hash(); // Recalculate hash with new status
    }

    /// Get transaction priority score (higher is better)
    pub fn get_priority_score(&self) -> f64 {
        // Base priority on fee amount (not per gas)
        let fee_priority = self.fee;
        
        // Older transactions get slight priority boost
        let age_boost = (SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() - self.timestamp) as f64 * 0.001;
        
        fee_priority + age_boost
    }

    /// Validate transaction format
    pub fn validate(&self) -> Result<(), String> {
        if self.amount <= 0.0 {
            return Err("Amount must be positive".to_string());
        }

        if self.fee < 0.0 {
            return Err("Fee cannot be negative".to_string());
        }

        if self.from.is_empty() || self.to.is_empty() {
            return Err("From and to addresses cannot be empty".to_string());
        }

        if self.from == self.to {
            return Err("Cannot send to self".to_string());
        }

        // Validate data size (max 1KB)
        if let Some(ref data) = self.data {
            if data.len() > 1024 {
                return Err("Transaction data too large (max 1KB)".to_string());
            }
        }

        Ok(())
    }

    /// Convert to the original Transaction format for compatibility
    pub fn to_legacy_transaction(&self) -> crate::Transaction {
        crate::Transaction {
            from: self.from.clone(),
            to: self.to.clone(),
            amount: self.amount,
            timestamp: self.timestamp,
            signature: self.signature.clone(),
        }
    }

    /// Create from legacy transaction
    pub fn from_legacy_transaction(tx: &crate::Transaction, fee: f64) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            from: tx.from.clone(),
            to: tx.to.clone(),
            amount: tx.amount,
            timestamp: tx.timestamp,
            signature: tx.signature.clone(),
            nonce: rand::random::<u64>(),
            fee,
            data: None,
            status: TransactionStatus::Pending,
            hash: String::new(),
        }
    }

    /// Get transaction summary for logging
    pub fn summary(&self) -> String {
        format!(
            "TX[{}] {} -> {} (Amount: {}, Fee: {}, Status: {:?})",
            &self.id[..8], self.from, self.to, self.amount, self.fee, self.status
        )
    }
}

/// Transaction pool with enhanced features
pub struct TransactionPool {
    pending: Vec<EnhancedTransaction>,
    confirmed: Vec<EnhancedTransaction>,
    failed: Vec<EnhancedTransaction>,
    rejected: Vec<EnhancedTransaction>,
    expired: Vec<EnhancedTransaction>,
    max_pool_size: usize,
    max_history_size: usize,
    min_fee: f64,
}

impl TransactionPool {
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
            confirmed: Vec::new(),
            failed: Vec::new(),
            rejected: Vec::new(),
            expired: Vec::new(),
            max_pool_size: 1000, // Maximum pending transactions
            max_history_size: 10000, // Maximum historical transactions
            min_fee: 0.001, // Minimum transaction fee
        }
    }

    pub fn with_config(max_pool_size: usize, max_history_size: usize, min_fee: f64) -> Self {
        Self {
            pending: Vec::new(),
            confirmed: Vec::new(),
            failed: Vec::new(),
            rejected: Vec::new(),
            expired: Vec::new(),
            max_pool_size,
            max_history_size,
            min_fee,
        }
    }

    pub fn add_transaction(&mut self, tx: EnhancedTransaction) -> Result<(), String> {
        // Validate transaction
        tx.validate()?;

        // Check pool capacity
        if self.pending.len() >= self.max_pool_size {
            return Err("Transaction pool is full".to_string());
        }

        // Check minimum fee
        if tx.fee < self.min_fee {
            return Err(format!("Transaction fee too low. Minimum: {}", self.min_fee));
        }

        // Check for duplicate transaction IDs
        if self.pending.iter().any(|existing| existing.id == tx.id) {
            return Err("Duplicate transaction ID".to_string());
        }

        // Check if transaction is expired
        if tx.is_expired() {
            return Err("Transaction is expired".to_string());
        }

        // Check for nonce reuse (prevent replay attacks)
        if self.pending.iter().any(|existing| existing.from == tx.from && existing.nonce == tx.nonce) {
            return Err("Nonce already used for this address".to_string());
        }

        println!("📥 Transaction added to pool: {}", tx.summary());
        self.pending.push(tx);
        Ok(())
    }

    pub fn get_pending_transactions(&self) -> &Vec<EnhancedTransaction> {
        &self.pending
    }

    pub fn get_confirmed_transactions(&self) -> &Vec<EnhancedTransaction> {
        &self.confirmed
    }

    pub fn get_failed_transactions(&self) -> &Vec<EnhancedTransaction> {
        &self.failed
    }

    pub fn get_rejected_transactions(&self) -> &Vec<EnhancedTransaction> {
        &self.rejected
    }

    pub fn get_expired_transactions(&self) -> &Vec<EnhancedTransaction> {
        &self.expired
    }

    pub fn confirm_transaction(&mut self, id: &str) -> Result<(), String> {
        if let Some(pos) = self.pending.iter().position(|tx| tx.id == id) {
            let mut tx = self.pending.remove(pos);
            tx.confirm(); // REMOVED: gas_used parameter
            
            println!("✅ Transaction confirmed: {}", tx.summary());
            
            if self.confirmed.len() >= self.max_history_size {
                self.confirmed.remove(0);
            }
            
            self.confirmed.push(tx);
            Ok(())
        } else {
            Err("Transaction not found in pending pool".to_string())
        }
    }

    pub fn fail_transaction(&mut self, id: &str) -> Result<(), String> {
        if let Some(pos) = self.pending.iter().position(|tx| tx.id == id) {
            let mut tx = self.pending.remove(pos);
            tx.fail();
            
            println!("❌ Transaction failed: {}", tx.summary());
            
            // Maintain history size limit
            if self.failed.len() >= self.max_history_size {
                self.failed.remove(0);
            }
            
            self.failed.push(tx);
            Ok(())
        } else {
            Err("Transaction not found in pending pool".to_string())
        }
    }

    pub fn reject_transaction(&mut self, id: &str) -> Result<(), String> {
        if let Some(pos) = self.pending.iter().position(|tx| tx.id == id) {
            let mut tx = self.pending.remove(pos);
            tx.reject();
            
            println!("🚫 Transaction rejected: {}", tx.summary());
            
            // Maintain history size limit
            if self.rejected.len() >= self.max_history_size {
                self.rejected.remove(0);
            }
            
            self.rejected.push(tx);
            Ok(())
        } else {
            Err("Transaction not found in pending pool".to_string())
        }
    }

    pub fn clear_pending(&mut self) {
        let count = self.pending.len();
        self.pending.clear();
        if count > 0 {
            println!("🧹 Cleared {} pending transactions", count);
        }
    }

    pub fn remove_transaction(&mut self, id: &str) -> Option<EnhancedTransaction> {
        if let Some(pos) = self.pending.iter().position(|tx| tx.id == id) {
            let tx = self.pending.remove(pos);
            println!("🗑️ Transaction removed: {}", tx.summary());
            Some(tx)
        } else {
            None
        }
    }

    pub fn get_transaction_by_id(&self, id: &str) -> Option<&EnhancedTransaction> {
        self.pending.iter()
            .chain(self.confirmed.iter())
            .chain(self.failed.iter())
            .chain(self.rejected.iter())
            .chain(self.expired.iter())
            .find(|tx| tx.id == id)
    }

    pub fn get_transactions_by_fee(&self, min_fee: f64) -> Vec<&EnhancedTransaction> {
        self.pending
            .iter()
            .filter(|tx| tx.fee >= min_fee)
            .collect()
    }

    pub fn get_transactions_by_address(&self, address: &str) -> Vec<&EnhancedTransaction> {
        self.pending
            .iter()
            .chain(self.confirmed.iter())
            .chain(self.failed.iter())
            .chain(self.rejected.iter())
            .chain(self.expired.iter())
            .filter(|tx| tx.from == address || tx.to == address)
            .collect()
    }

    /// Get transactions sorted by priority (highest first) for mining
    pub fn get_transactions_by_priority(&self) -> Vec<&EnhancedTransaction> {
        let mut txs: Vec<&EnhancedTransaction> = self.pending.iter().collect();
        txs.sort_by(|a, b| {
            b.get_priority_score()
                .partial_cmp(&a.get_priority_score())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        txs
    }

    /// Get transactions sorted by fee (highest first)
    pub fn get_transactions_by_fee_desc(&self) -> Vec<&EnhancedTransaction> {
        let mut txs: Vec<&EnhancedTransaction> = self.pending.iter().collect();
        txs.sort_by(|a, b| b.fee.partial_cmp(&a.fee).unwrap_or(std::cmp::Ordering::Equal));
        txs
    }

    /// Get transactions by status
    pub fn get_transactions_by_status(&self, status: &TransactionStatus) -> Vec<&EnhancedTransaction> {
        match status {
            TransactionStatus::Pending => self.pending.iter().collect(),
            TransactionStatus::Confirmed => self.confirmed.iter().collect(),
            TransactionStatus::Failed => self.failed.iter().collect(),
            TransactionStatus::Rejected => self.rejected.iter().collect(),
            TransactionStatus::Expired => self.expired.iter().collect(),
        }
    }

    /// Remove expired transactions and move them to expired pool
    pub fn cleanup_expired(&mut self) -> usize {
        let initial_count = self.pending.len();
        let mut expired_txs = Vec::new();

        // Find expired transactions
        self.pending.retain(|tx| {
            if tx.is_expired() {
                expired_txs.push(tx.clone());
                false
            } else {
                true
            }
        });

        // Move expired transactions to expired pool
        for mut tx in expired_txs {
            tx.expire();
            
            // Maintain history size limit
            if self.expired.len() >= self.max_history_size {
                self.expired.remove(0);
            }
            
            self.expired.push(tx);
        }

        let expired_count = initial_count - self.pending.len();
        if expired_count > 0 {
            println!("⏰ Moved {} expired transactions to expired pool", expired_count);
        }
        
        expired_count
    }

    /// Clean up old historical transactions
    pub fn cleanup_history(&mut self) {
        let mut cleaned = 0;

        // Clean confirmed transactions
        if self.confirmed.len() > self.max_history_size {
            let to_remove = self.confirmed.len() - self.max_history_size;
            self.confirmed.drain(0..to_remove);
            cleaned += to_remove;
        }

        // Clean failed transactions
        if self.failed.len() > self.max_history_size {
            let to_remove = self.failed.len() - self.max_history_size;
            self.failed.drain(0..to_remove);
            cleaned += to_remove;
        }

        // Clean rejected transactions
        if self.rejected.len() > self.max_history_size {
            let to_remove = self.rejected.len() - self.max_history_size;
            self.rejected.drain(0..to_remove);
            cleaned += to_remove;
        }

        // Clean expired transactions
        if self.expired.len() > self.max_history_size {
            let to_remove = self.expired.len() - self.max_history_size;
            self.expired.drain(0..to_remove);
            cleaned += to_remove;
        }

        if cleaned > 0 {
            println!("🧹 Cleaned up {} old historical transactions", cleaned);
        }
    }

    /// Get pool statistics
    pub fn get_stats(&self) -> PoolStats {
        let total_fees: f64 = self.pending.iter().map(|tx| tx.fee).sum();
        let total_volume: f64 = self.pending.iter().map(|tx| tx.amount).sum();
        let avg_fee = if !self.pending.is_empty() { 
            total_fees / self.pending.len() as f64 
        } else { 
            0.0 
        };

        PoolStats {
            pending_count: self.pending.len(),
            confirmed_count: self.confirmed.len(),
            failed_count: self.failed.len(),
            rejected_count: self.rejected.len(),
            expired_count: self.expired.len(),
            total_transactions: self.pending.len() + self.confirmed.len() + 
                              self.failed.len() + self.rejected.len() + self.expired.len(),
            average_fee: avg_fee,
            total_volume: total_volume,
            total_fees: total_fees,
            min_fee: self.min_fee,
            max_pool_size: self.max_pool_size,
        }
    }

    /// Get detailed statistics
    pub fn get_detailed_stats(&self) -> DetailedPoolStats {
        let stats = self.get_stats();
        
        // Calculate fee distribution
        let mut fees: Vec<f64> = self.pending.iter().map(|tx| tx.fee).collect();
        fees.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        
        let min_fee_pending = fees.first().copied().unwrap_or(0.0);
        let max_fee_pending = fees.last().copied().unwrap_or(0.0);
        let median_fee = if !fees.is_empty() {
            fees[fees.len() / 2]
        } else {
            0.0
        };

        DetailedPoolStats {
            basic_stats: stats,
            min_fee_pending,
            max_fee_pending,
            median_fee,
            pool_utilization: (self.pending.len() as f64 / self.max_pool_size as f64) * 100.0,
        }
    }

    /// Set minimum fee
    pub fn set_min_fee(&mut self, min_fee: f64) {
        self.min_fee = min_fee;
        println!("💰 Minimum transaction fee updated to: {}", min_fee);
    }

    /// Get current minimum fee
    pub fn get_min_fee(&self) -> f64 {
        self.min_fee
    }
}

/// Pool statistics
#[derive(Serialize, Debug)]
pub struct PoolStats {
    pub pending_count: usize,
    pub confirmed_count: usize,
    pub failed_count: usize,
    pub rejected_count: usize,
    pub expired_count: usize,
    pub total_transactions: usize,
    pub average_fee: f64,
    pub total_volume: f64,
    pub total_fees: f64,
    pub min_fee: f64,
    pub max_pool_size: usize,
}

/// Detailed pool statistics
#[derive(Serialize, Debug)]
pub struct DetailedPoolStats {
    pub basic_stats: PoolStats,
    pub min_fee_pending: f64,
    pub max_fee_pending: f64,
    pub median_fee: f64,
    pub pool_utilization: f64, // Percentage
}

/// Transaction receipt for confirmed transactions
#[derive(Serialize, Debug)]
pub struct TransactionReceipt {
    pub transaction_hash: String,
    pub transaction_id: String,
    pub block_number: Option<u64>,
    pub status: TransactionStatus,
    pub timestamp: u64,
    pub from: String,
    pub to: String,
    pub amount: f64,
    pub fee: f64,
    pub data: Option<String>,
}

impl TransactionReceipt {
    pub fn new(tx: &EnhancedTransaction, block_number: Option<u64>) -> Self {
        Self {
            transaction_hash: tx.hash.clone(),
            transaction_id: tx.id.clone(),
            block_number,
            status: tx.status.clone(),
            timestamp: tx.timestamp,
            from: tx.from.clone(),
            to: tx.to.clone(),
            amount: tx.amount,
            fee: tx.fee,
            data: tx.data.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_transaction_creation() {
        let tx = EnhancedTransaction::new(
            "alice".to_string(),
            "bob".to_string(),
            50.0,
            1.0
        );

        assert!(!tx.id.is_empty());
        assert_eq!(tx.from, "alice");
        assert_eq!(tx.to, "bob");
        assert_eq!(tx.amount, 50.0);
        assert_eq!(tx.fee, 1.0);
        assert_eq!(tx.total_cost(), 51.0);
        assert_eq!(tx.status, TransactionStatus::Pending);
        assert!(!tx.hash.is_empty());
    }

    #[test]
    fn test_transaction_with_message() {
        let tx = EnhancedTransaction::new(
            "alice".to_string(),
            "bob".to_string(),
            50.0,
            1.0
        ).with_message("Hello Bob!".to_string());

        assert_eq!(tx.data, Some("Hello Bob!".to_string()));
        assert!(tx.hash.len() > 64); // Increased due to data
    }

    #[test]
    fn test_transaction_validation() {
        let tx = EnhancedTransaction::new(
            "alice".to_string(),
            "bob".to_string(),
            50.0,
            1.0
        );
        assert!(tx.validate().is_ok());

        // Test invalid amount
        let mut invalid_tx = tx.clone();
        invalid_tx.amount = -10.0;
        assert!(invalid_tx.validate().is_err());

        // Test self-transaction
        let mut self_tx = tx.clone();
        self_tx.to = self_tx.from.clone();
        assert!(self_tx.validate().is_err());
    }

    #[test]
    fn test_transaction_pool() {
        let mut pool = TransactionPool::new();
        
        let tx = EnhancedTransaction::new(
            "alice".to_string(),
            "bob".to_string(),
            50.0,
            1.0
        );

        let tx_id = tx.id.clone();
        assert!(pool.add_transaction(tx).is_ok());
        assert_eq!(pool.get_pending_transactions().len(), 1);

        // Test confirmation
        assert!(pool.confirm_transaction(&tx_id).is_ok());
        assert_eq!(pool.get_pending_transactions().len(), 0);
        assert_eq!(pool.get_confirmed_transactions().len(), 1);
    }

    #[test]
    fn test_transaction_priority() {
        let mut pool = TransactionPool::new();
        
        let tx1 = EnhancedTransaction::new("alice".to_string(), "bob".to_string(), 10.0, 1.0);
        let tx2 = EnhancedTransaction::new("bob".to_string(), "charlie".to_string(), 10.0, 5.0);
        let tx3 = EnhancedTransaction::new("charlie".to_string(), "alice".to_string(), 10.0, 3.0);

        pool.add_transaction(tx1).unwrap();
        pool.add_transaction(tx2).unwrap();
        pool.add_transaction(tx3).unwrap();

        let priority_txs = pool.get_transactions_by_priority();
        
        // Should be sorted by priority score (fee-based)
        assert!(priority_txs[0].fee >= priority_txs[1].fee);
        assert!(priority_txs[1].fee >= priority_txs[2].fee);
    }

    #[test]
    fn test_duplicate_id_prevention() {
        let mut pool = TransactionPool::new();
        
        let tx1 = EnhancedTransaction::new("alice".to_string(), "bob".to_string(), 10.0, 1.0);
        let mut tx2 = tx1.clone();
        tx2.from = "charlie".to_string(); // Different sender but same ID

        assert!(pool.add_transaction(tx1).is_ok());
        assert!(pool.add_transaction(tx2).is_err()); // Should fail due to duplicate ID
    }

    #[test]
    fn test_nonce_replay_prevention() {
        let mut pool = TransactionPool::new();
        
        let tx1 = EnhancedTransaction::new("alice".to_string(), "bob".to_string(), 10.0, 1.0);
        let mut tx2 = EnhancedTransaction::new("alice".to_string(), "charlie".to_string(), 20.0, 1.0);
        tx2.nonce = tx1.nonce; // Same nonce from same sender

        assert!(pool.add_transaction(tx1).is_ok());
        assert!(pool.add_transaction(tx2).is_err()); // Should fail due to nonce reuse
    }

    #[test]
    fn test_minimum_fee_enforcement() {
        let mut pool = TransactionPool::with_config(100, 1000, 5.0); // Min fee: 5.0
        
        let low_fee_tx = EnhancedTransaction::new("alice".to_string(), "bob".to_string(), 10.0, 1.0);
        let high_fee_tx = EnhancedTransaction::new("alice".to_string(), "bob".to_string(), 10.0, 10.0);

        assert!(pool.add_transaction(low_fee_tx).is_err()); // Should fail due to low fee
        assert!(pool.add_transaction(high_fee_tx).is_ok()); // Should succeed
    }

    #[test]
    fn test_pool_statistics() {
        let mut pool = TransactionPool::new();
        
        let tx1 = EnhancedTransaction::new("alice".to_string(), "bob".to_string(), 100.0, 2.0);
        let tx2 = EnhancedTransaction::new("bob".to_string(), "charlie".to_string(), 200.0, 4.0);
        
        pool.add_transaction(tx1).unwrap();
        pool.add_transaction(tx2).unwrap();

        let stats = pool.get_stats();
        assert_eq!(stats.pending_count, 2);
        assert_eq!(stats.total_volume, 300.0);
        assert_eq!(stats.total_fees, 6.0);
        assert_eq!(stats.average_fee, 3.0);
    }
}