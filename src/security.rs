use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Basic security error types
#[derive(Debug)]
pub enum SecurityError {
    RateLimitExceeded,
    InvalidTransaction(String),
    ValidationFailed(String),
    BlacklistedAddress,
}

/// Simple rate limiter to prevent spam
pub struct RateLimiter {
    requests: HashMap<String, Vec<Instant>>,
    max_requests: usize,
    window_duration: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_seconds: u64) -> Self {
        Self {
            requests: HashMap::new(),
            max_requests,
            window_duration: Duration::from_secs(window_seconds),
        }
    }

    pub fn check_rate_limit(&mut self, identifier: &str) -> Result<(), SecurityError> {
        let now = Instant::now();
        let window_start = now - self.window_duration;

        // Get or create request history for this identifier
        let request_times = self.requests.entry(identifier.to_string()).or_insert_with(Vec::new);

        // Remove old requests outside the window
        request_times.retain(|&time| time > window_start);

        // Check if we're within the rate limit
        if request_times.len() >= self.max_requests {
            return Err(SecurityError::RateLimitExceeded);
        }

        // Add current request
        request_times.push(now);
        Ok(())
    }

    pub fn cleanup_old_entries(&mut self) {
        let now = Instant::now();
        let window_start = now - self.window_duration;

        self.requests.retain(|_, times| {
            times.retain(|&time| time > window_start);
            !times.is_empty()
        });
    }

    /// Get current request count for an identifier
    pub fn get_current_requests(&self, identifier: &str) -> usize {
        if let Some(times) = self.requests.get(identifier) {
            let now = Instant::now();
            let window_start = now - self.window_duration;
            times.iter().filter(|&&time| time > window_start).count()
        } else {
            0
        }
    }
}

/// Transaction validator with basic security checks
pub struct TransactionValidator {
    min_amount: f64,
    max_amount: f64,
    max_transaction_size: usize,
    blacklisted_addresses: std::collections::HashSet<String>,
    suspicious_patterns: Vec<String>,
}

impl TransactionValidator {
    pub fn new() -> Self {
        Self {
            min_amount: 0.0001,
            max_amount: 1_000_000.0,
            max_transaction_size: 1024, // 1KB for now
            blacklisted_addresses: std::collections::HashSet::new(),
            suspicious_patterns: vec![
                "hack".to_string(),
                "exploit".to_string(),
                "phishing".to_string(),
            ],
        }
    }

    pub fn validate_transaction(&self, from: &str, to: &str, amount: f64) -> Result<(), SecurityError> {
        // Amount validation
        if amount <= 0.0 {
            return Err(SecurityError::InvalidTransaction("Amount must be positive".to_string()));
        }

        if amount < self.min_amount {
            return Err(SecurityError::InvalidTransaction(
                format!("Amount too small. Minimum: {}", self.min_amount)
            ));
        }

        if amount > self.max_amount {
            return Err(SecurityError::InvalidTransaction(
                format!("Amount too large. Maximum: {}", self.max_amount)
            ));
        }

        // Address validation
        self.validate_address(from)?;
        self.validate_address(to)?;

        // Self-transaction check
        if from == to {
            return Err(SecurityError::InvalidTransaction("Cannot send to self".to_string()));
        }

        // Blacklist check
        if self.blacklisted_addresses.contains(from) || self.blacklisted_addresses.contains(to) {
            return Err(SecurityError::BlacklistedAddress);
        }

        // Suspicious pattern check
        if self.contains_suspicious_pattern(from) || self.contains_suspicious_pattern(to) {
            return Err(SecurityError::InvalidTransaction("Suspicious address pattern detected".to_string()));
        }

        Ok(())
    }

    fn validate_address(&self, address: &str) -> Result<(), SecurityError> {
        if address.is_empty() {
            return Err(SecurityError::InvalidTransaction("Empty address".to_string()));
        }

        if address.len() > 64 {
            return Err(SecurityError::InvalidTransaction("Address too long".to_string()));
        }

        // Allow system addresses
        if ["genesis", "mining_reward", "connection_reward", "system"].contains(&address) {
            return Ok(());
        }

        // Basic format validation for wallet addresses
        if address.starts_with("wallet_") && address.len() < 8 {
            return Err(SecurityError::InvalidTransaction("Invalid wallet address format".to_string()));
        }

        Ok(())
    }

    fn contains_suspicious_pattern(&self, address: &str) -> bool {
        let address_lower = address.to_lowercase();
        self.suspicious_patterns.iter().any(|pattern| address_lower.contains(pattern))
    }

    // Admin functions for managing security
    pub fn add_to_blacklist(&mut self, address: String) {
        self.blacklisted_addresses.insert(address);
    }

    pub fn remove_from_blacklist(&mut self, address: &str) -> bool {
        self.blacklisted_addresses.remove(address)
    }

    pub fn is_blacklisted(&self, address: &str) -> bool {
        self.blacklisted_addresses.contains(address)
    }

    pub fn add_suspicious_pattern(&mut self, pattern: String) {
        self.suspicious_patterns.push(pattern.to_lowercase());
    }

    pub fn get_blacklisted_addresses(&self) -> Vec<&String> {
        self.blacklisted_addresses.iter().collect()
    }
}

/// Combined security manager for transactions
pub struct SecurityManager {
    pub transaction_limiter: RateLimiter,
    pub mining_limiter: RateLimiter,
    pub connection_limiter: RateLimiter,
    pub validator: TransactionValidator,
    failed_attempts: HashMap<String, u32>,
    max_failed_attempts: u32,
}

impl SecurityManager {
    pub fn new() -> Self {
        Self {
            transaction_limiter: RateLimiter::new(10, 60), // 10 transactions per minute
            mining_limiter: RateLimiter::new(5, 300),      // 5 mining attempts per 5 minutes
            connection_limiter: RateLimiter::new(3, 30),   // 3 connections per 30 seconds
            validator: TransactionValidator::new(),
            failed_attempts: HashMap::new(),
            max_failed_attempts: 5,
        }
    }

    /// Comprehensive transaction security check
    pub fn check_transaction_security(&mut self, from: &str, to: &str, amount: f64) -> Result<(), SecurityError> {
        // Check if address is temporarily blocked due to failed attempts
        if let Some(&attempts) = self.failed_attempts.get(from) {
            if attempts >= self.max_failed_attempts {
                return Err(SecurityError::InvalidTransaction(
                    "Address temporarily blocked due to multiple failed attempts".to_string()
                ));
            }
        }

        // Rate limiting check
        self.transaction_limiter.check_rate_limit(from)?;

        // Transaction validation
        self.validator.validate_transaction(from, to, amount)?;

        // If we get here, reset failed attempts for this address
        self.failed_attempts.remove(from);

        Ok(())
    }

    pub fn check_mining_security(&mut self, miner: &str) -> Result<(), SecurityError> {
        // Rate limiting for mining
        self.mining_limiter.check_rate_limit(miner)?;

        // Basic miner validation
        if self.validator.is_blacklisted(miner) {
            return Err(SecurityError::BlacklistedAddress);
        }

        Ok(())
    }

    pub fn check_connection_security(&mut self, address: &str) -> Result<(), SecurityError> {
        // Rate limiting for connections
        self.connection_limiter.check_rate_limit(address)?;

        // Connection validation
        if self.validator.is_blacklisted(address) {
            return Err(SecurityError::BlacklistedAddress);
        }

        Ok(())
    }

    /// Record a failed transaction attempt
    pub fn record_failed_attempt(&mut self, address: &str) {
        let attempts = self.failed_attempts.entry(address.to_string()).or_insert(0);
        *attempts += 1;
        
        println!("⚠️ Failed attempt recorded for {}: {} attempts", address, attempts);
        
        // Auto-blacklist after too many failed attempts
        if *attempts >= self.max_failed_attempts {
            self.validator.add_to_blacklist(address.to_string());
            println!("🚫 Address {} automatically blacklisted due to multiple failed attempts", address);
        }
    }

    /// Clean up old entries and reset counters
    pub fn cleanup(&mut self) {
        self.transaction_limiter.cleanup_old_entries();
        self.mining_limiter.cleanup_old_entries();
        self.connection_limiter.cleanup_old_entries();
        
        // Reset failed attempts after some time (optional)
        // In a real implementation, you might want to implement time-based cleanup
    }

    /// Get security statistics
    pub fn get_security_stats(&self) -> SecurityStats {
        SecurityStats {
            blacklisted_addresses: self.validator.blacklisted_addresses.len(),
            failed_attempts_tracked: self.failed_attempts.len(),
            max_failed_attempts: self.max_failed_attempts,
            rate_limits: RateLimitStats {
                transaction_limit: self.transaction_limiter.max_requests,
                mining_limit: self.mining_limiter.max_requests,
                connection_limit: self.connection_limiter.max_requests,
            },
        }
    }

    /// Admin function to manually blacklist an address
    pub fn admin_blacklist(&mut self, address: String, reason: Option<String>) {
        self.validator.add_to_blacklist(address.clone());
        println!("🔒 Admin blacklisted address: {} (Reason: {})", 
                 address, reason.unwrap_or_else(|| "No reason provided".to_string()));
    }

    /// Admin function to remove from blacklist
    pub fn admin_unblacklist(&mut self, address: &str) -> bool {
        let removed = self.validator.remove_from_blacklist(address);
        if removed {
            println!("✅ Admin removed {} from blacklist", address);
            // Also clear failed attempts
            self.failed_attempts.remove(address);
        }
        removed
    }
}

/// Security statistics structure
#[derive(serde::Serialize, Debug)]
pub struct SecurityStats {
    pub blacklisted_addresses: usize,
    pub failed_attempts_tracked: usize,
    pub max_failed_attempts: u32,
    pub rate_limits: RateLimitStats,
}

#[derive(serde::Serialize, Debug)]
pub struct RateLimitStats {
    pub transaction_limit: usize,
    pub mining_limit: usize,
    pub connection_limit: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_rate_limiting() {
        let mut limiter = RateLimiter::new(2, 1); // 2 requests per second
        
        // First two requests should succeed
        assert!(limiter.check_rate_limit("user1").is_ok());
        assert!(limiter.check_rate_limit("user1").is_ok());
        
        // Third request should fail
        assert!(limiter.check_rate_limit("user1").is_err());
        
        // Different user should still work
        assert!(limiter.check_rate_limit("user2").is_ok());
    }

    #[test]
    fn test_transaction_validation() {
        let validator = TransactionValidator::new();
        
        // Valid transaction
        assert!(validator.validate_transaction("alice", "bob", 10.0).is_ok());
        
        // Invalid: self-transaction
        assert!(validator.validate_transaction("alice", "alice", 10.0).is_err());
        
        // Invalid: negative amount
        assert!(validator.validate_transaction("alice", "bob", -10.0).is_err());
        
        // Invalid: too small amount
        assert!(validator.validate_transaction("alice", "bob", 0.00001).is_err());
    }

    #[test]
    fn test_blacklisting() {
        let mut validator = TransactionValidator::new();
        
        // Should work initially
        assert!(validator.validate_transaction("badguy", "alice", 10.0).is_ok());
        
        // Add to blacklist
        validator.add_to_blacklist("badguy".to_string());
        
        // Should fail now
        assert!(validator.validate_transaction("badguy", "alice", 10.0).is_err());
        assert!(validator.validate_transaction("alice", "badguy", 10.0).is_err());
    }

    #[test]
    fn test_security_manager() {
        let mut security = SecurityManager::new();
        
        // Should work initially
        assert!(security.check_transaction_security("alice", "bob", 10.0).is_ok());
        
        // Simulate multiple failures
        for _ in 0..6 {
            security.record_failed_attempt("attacker");
        }
        
        // Should be blocked now
        assert!(security.check_transaction_security("attacker", "bob", 10.0).is_err());
    }
}