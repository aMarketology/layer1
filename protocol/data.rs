use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use aes_gcm::{Aes256Gcm, Key, Nonce, aead::{Aead, NewAead}};
use rand::Rng;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DataPoint {
    pub data_id: String,
    pub user_id: String,
    pub data_type: DataCategory,
    pub encrypted_content: String,
    pub metadata: DataMetadata,
    pub access_permissions: Vec<AccessPermission>,
    pub value_score: f64,
    pub created_at: DateTime<Utc>,
    pub last_accessed: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum DataCategory {
    SocialMedia {
        platform: String,
        content_type: String, // post, comment, like, share
        engagement_metrics: EngagementMetrics,
    },
    Personal {
        category: String, // demographics, preferences, behavior
        sensitivity_level: SensitivityLevel,
    },
    Financial {
        transaction_type: String,
        amount_range: String, // anonymized ranges
    },
    Health {
        data_type: String, // fitness, medical, wellness
        provider: Option<String>,
    },
    Location {
        precision_level: LocationPrecision,
        purpose: String,
    },
    Device {
        device_type: String,
        usage_patterns: HashMap<String, serde_json::Value>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EngagementMetrics {
    pub likes: u32,
    pub shares: u32,
    pub comments: u32,
    pub views: u32,
    pub reach: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SensitivityLevel {
    Public,
    Restricted,
    Private,
    Confidential,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum LocationPrecision {
    Country,
    State,
    City,
    Neighborhood,
    Exact,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DataMetadata {
    pub size_bytes: u64,
    pub format: String,
    pub source: String,
    pub quality_score: f64, // 0.0 to 1.0
    pub verification_status: VerificationStatus,
    pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum VerificationStatus {
    Unverified,
    SelfReported,
    ThirdPartyVerified,
    BlockchainVerified,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AccessPermission {
    pub requester_id: String,
    pub access_type: AccessType,
    pub purpose: String,
    pub duration: chrono::Duration,
    pub compensation: f64,
    pub granted_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum AccessType {
    View,
    Analyze,
    Aggregate,
    Download,
    Share,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DataMarketplace {
    pub listings: HashMap<String, DataListing>,
    pub transactions: Vec<DataTransaction>,
    pub user_data_vaults: HashMap<String, UserDataVault>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DataListing {
    pub listing_id: String,
    pub owner: String,
    pub data_type: DataCategory,
    pub description: String,
    pub price_per_access: f64,
    pub total_records: u64,
    pub sample_data: Option<String>,
    pub created_at: DateTime<Utc>,
    pub active: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DataTransaction {
    pub transaction_id: String,
    pub buyer: String,
    pub seller: String,
    pub data_id: String,
    pub access_type: AccessType,
    pub amount_paid: f64,
    pub timestamp: DateTime<Utc>,
    pub purpose: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserDataVault {
    pub user_id: String,
    pub encryption_key: String,
    pub data_points: Vec<String>, // Data IDs
    pub total_value: f64,
    pub earnings: f64,
    pub privacy_settings: PrivacySettings,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PrivacySettings {
    pub auto_sell_anonymous: bool,
    pub min_price_threshold: f64,
    pub allowed_categories: Vec<DataCategory>,
    pub blocked_buyers: Vec<String>,
    pub data_retention_days: u32,
}

pub struct DataEconomyEngine {
    pub data_points: HashMap<String, DataPoint>,
    pub marketplace: DataMarketplace,
    pub encryption_keys: HashMap<String, String>,
    pub data_valuations: HashMap<String, f64>,
}

impl DataEconomyEngine {
    pub fn new() -> Self {
        Self {
            data_points: HashMap::new(),
            marketplace: DataMarketplace {
                listings: HashMap::new(),
                transactions: Vec::new(),
                user_data_vaults: HashMap::new(),
            },
            encryption_keys: HashMap::new(),
            data_valuations: HashMap::new(),
        }
    }

    // Data Storage and Encryption
    pub fn store_encrypted_data(&mut self, user_id: &str, content: &str, data_type: DataCategory) -> Result<String, String> {
        let data_id = format!("data_{}_{}", user_id, chrono::Utc::now().timestamp_nanos());
        
        // Generate encryption key for user if not exists
        let encryption_key = self.get_or_create_user_key(user_id);
        
        // Encrypt the content
        let encrypted_content = self.encrypt_data(content, &encryption_key)?;
        
        // Calculate data value
        let value_score = self.calculate_data_value(&data_type, content.len());
        
        let data_point = DataPoint {
            data_id: data_id.clone(),
            user_id: user_id.to_string(),
            data_type,
            encrypted_content,
            metadata: DataMetadata {
                size_bytes: content.len() as u64,
                format: "text".to_string(),
                source: "social_app".to_string(),
                quality_score: 0.8,
                verification_status: VerificationStatus::SelfReported,
                tags: Vec::new(),
            },
            access_permissions: Vec::new(),
            value_score,
            created_at: Utc::now(),
            last_accessed: None,
        };

        self.data_points.insert(data_id.clone(), data_point);
        
        // Update user's data vault
        self.update_user_vault(user_id, &data_id, value_score);
        
        Ok(data_id)
    }

    fn encrypt_data(&self, content: &str, key: &str) -> Result<String, String> {
        let mut rng = rand::thread_rng();
        let nonce_bytes: [u8; 12] = rng.gen();
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let key_bytes = if key.len() >= 32 {
            &key.as_bytes()[..32]
        } else {
            let mut padded = [0u8; 32];
            let key_bytes = key.as_bytes();
            padded[..key_bytes.len()].copy_from_slice(key_bytes);
            &padded
        };
        
        let cipher_key = Key::from_slice(key_bytes);
        let cipher = Aes256Gcm::new(cipher_key);
        
        let ciphertext = cipher.encrypt(nonce, content.as_bytes())
            .map_err(|_| "Encryption failed")?;
        
        // Combine nonce and ciphertext
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);
        
        Ok(hex::encode(result))
    }

    pub fn decrypt_data(&self, user_id: &str, data_id: &str) -> Result<String, String> {
        let data_point = self.data_points.get(data_id)
            .ok_or("Data not found")?;
        
        if data_point.user_id != user_id {
            return Err("Access denied".to_string());
        }
        
        let encryption_key = self.encryption_keys.get(user_id)
            .ok_or("Encryption key not found")?;
        
        let encrypted_bytes = hex::decode(&data_point.encrypted_content)
            .map_err(|_| "Invalid encrypted data")?;
        
        if encrypted_bytes.len() < 12 {
            return Err("Invalid encrypted data format".to_string());
        }
        
        let (nonce_bytes, ciphertext) = encrypted_bytes.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let key_bytes = if encryption_key.len() >= 32 {
            &encryption_key.as_bytes()[..32]
        } else {
            let mut padded = [0u8; 32];
            let key_bytes = encryption_key.as_bytes();
            padded[..key_bytes.len()].copy_from_slice(key_bytes);
            &padded
        };
        
        let cipher_key = Key::from_slice(key_bytes);
        let cipher = Aes256Gcm::new(cipher_key);
        
        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|_| "Decryption failed")?;
        
        String::from_utf8(plaintext)
            .map_err(|_| "Invalid UTF-8 data".to_string())
    }

    fn get_or_create_user_key(&mut self, user_id: &str) -> String {
        if let Some(key) = self.encryption_keys.get(user_id) {
            key.clone()
        } else {
            let key = format!("key_{}_{}", user_id, chrono::Utc::now().timestamp());
            self.encryption_keys.insert(user_id.to_string(), key.clone());
            key
        }
    }

    fn calculate_data_value(&self, data_type: &DataCategory, content_size: usize) -> f64 {
        let base_value = match data_type {
            DataCategory::SocialMedia { engagement_metrics, .. } => {
                let engagement_score = engagement_metrics.likes as f64 * 0.1 +
                                     engagement_metrics.shares as f64 * 0.3 +
                                     engagement_metrics.comments as f64 * 0.5;
                0.05 + (engagement_score * 0.01)
            },
            DataCategory::Personal { sensitivity_level, .. } => {
                match sensitivity_level {
                    SensitivityLevel::Public => 0.01,
                    SensitivityLevel::Restricted => 0.05,
                    SensitivityLevel::Private => 0.15,
                    SensitivityLevel::Confidential => 0.3,
                }
            },
            DataCategory::Financial { .. } => 0.25,
            DataCategory::Health { .. } => 0.4,
            DataCategory::Location { precision_level, .. } => {
                match precision_level {
                    LocationPrecision::Country => 0.01,
                    LocationPrecision::State => 0.02,
                    LocationPrecision::City => 0.05,
                    LocationPrecision::Neighborhood => 0.1,
                    LocationPrecision::Exact => 0.2,
                }
            },
            DataCategory::Device { .. } => 0.08,
        };

        // Adjust for content size
        let size_multiplier = (content_size as f64 / 1000.0).min(5.0).max(0.1);
        base_value * size_multiplier
    }

    fn update_user_vault(&mut self, user_id: &str, data_id: &str, value: f64) {
        let vault = self.marketplace.user_data_vaults.entry(user_id.to_string())
            .or_insert_with(|| UserDataVault {
                user_id: user_id.to_string(),
                encryption_key: self.get_or_create_user_key(user_id),
                data_points: Vec::new(),
                total_value: 0.0,
                earnings: 0.0,
                privacy_settings: PrivacySettings {
                    auto_sell_anonymous: false,
                    min_price_threshold: 1.0,
                    allowed_categories: Vec::new(),
                    blocked_buyers: Vec::new(),
                    data_retention_days: 365,
                },
            });

        vault.data_points.push(data_id.to_string());
        vault.total_value += value;
    }

    // Data Marketplace Functions
    pub fn create_data_listing(&mut self, user_id: &str, data_type: DataCategory, description: &str, price: f64, record_count: u64) -> Result<String, String> {
        let listing_id = format!("listing_{}_{}", user_id, chrono::Utc::now().timestamp());
        
        let listing = DataListing {
            listing_id: listing_id.clone(),
            owner: user_id.to_string(),
            data_type,
            description: description.to_string(),
            price_per_access: price,
            total_records: record_count,
            sample_data: None,
            created_at: Utc::now(),
            active: true,
        };

        self.marketplace.listings.insert(listing_id.clone(), listing);
        Ok(listing_id)
    }

    pub fn purchase_data_access(&mut self, buyer: &str, listing_id: &str, purpose: &str) -> Result<String, String> {
        let listing = self.marketplace.listings.get(listing_id)
            .ok_or("Listing not found")?;
        
        if !listing.active {
            return Err("Listing is not active".to_string());
        }

        let transaction_id = format!("tx_{}_{}", buyer, chrono::Utc::now().timestamp());
        
        let transaction = DataTransaction {
            transaction_id: transaction_id.clone(),
            buyer: buyer.to_string(),
            seller: listing.owner.clone(),
            data_id: listing_id.to_string(),
            access_type: AccessType::View,
            amount_paid: listing.price_per_access,
            timestamp: Utc::now(),
            purpose: purpose.to_string(),
        };

        self.marketplace.transactions.push(transaction);
        
        // Update seller's earnings
        if let Some(vault) = self.marketplace.user_data_vaults.get_mut(&listing.owner) {
            vault.earnings += listing.price_per_access;
        }

        Ok(transaction_id)
    }

    // Analytics and Insights
    pub fn get_user_data_summary(&self, user_id: &str) -> HashMap<String, serde_json::Value> {
        let mut summary = HashMap::new();
        
        let user_data: Vec<_> = self.data_points.values()
            .filter(|dp| dp.user_id == user_id)
            .collect();

        summary.insert("total_data_points".to_string(), serde_json::Value::Number(serde_json::Number::from(user_data.len())));
        summary.insert("total_value".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(
            user_data.iter().map(|dp| dp.value_score).sum::<f64>()
        ).unwrap_or(serde_json::Number::from(0))));

        if let Some(vault) = self.marketplace.user_data_vaults.get(user_id) {
            summary.insert("total_earnings".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(vault.earnings).unwrap_or(serde_json::Number::from(0))));
        }

        summary
    }

    pub fn get_marketplace_stats(&self) -> HashMap<String, serde_json::Value> {
        let mut stats = HashMap::new();
        
        stats.insert("total_listings".to_string(), serde_json::Value::Number(serde_json::Number::from(self.marketplace.listings.len())));
        stats.insert("total_transactions".to_string(), serde_json::Value::Number(serde_json::Number::from(self.marketplace.transactions.len())));
        stats.insert("total_volume".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(
            self.marketplace.transactions.iter().map(|tx| tx.amount_paid).sum::<f64>()
        ).unwrap_or(serde_json::Number::from(0))));

        stats
    }
}