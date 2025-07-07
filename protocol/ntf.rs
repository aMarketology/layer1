use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use crate::protocol::smart_contracts::{SmartContract, ContractType, ContractState};
use crate::protocol::data::{DataCategory, DataPoint, AccessType};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DataNFT {
    pub nft_id: String,
    pub token_id: u64,
    pub owner: String,
    pub data_period: DataPeriod,
    pub data_summary: DataSummary,
    pub unlock_conditions: UnlockConditions,
    pub metadata: NFTMetadata,
    pub current_bid: Option<Bid>,
    pub unlocked_by: Vec<UnlockRecord>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub status: NFTStatus,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DataPeriod {
    pub period_type: PeriodType,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub data_point_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum PeriodType {
    Daily,
    Weekly,
    Monthly,
    Custom { days: u32 },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DataSummary {
    pub total_data_points: u64,
    pub categories: HashMap<String, u64>, // Category -> count
    pub engagement_score: f64,
    pub quality_score: f64,
    pub estimated_value: f64,
    pub demographic_info: DemographicSummary,
    pub activity_patterns: ActivityPatterns,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DemographicSummary {
    pub age_range: Option<String>,
    pub location_region: Option<String>,
    pub interests: Vec<String>,
    pub activity_level: ActivityLevel,
    pub spending_category: SpendingCategory,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ActivityLevel {
    Low,
    Medium,
    High,
    VeryHigh,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SpendingCategory {
    Budget,
    Moderate,
    Premium,
    Luxury,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ActivityPatterns {
    pub most_active_hours: Vec<u8>, // 0-23
    pub platform_usage: HashMap<String, f64>, // Platform -> hours
    pub content_preferences: Vec<String>,
    pub interaction_frequency: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UnlockConditions {
    pub minimum_payment: f64,
    pub allowed_advertiser_types: Vec<AdvertiserType>,
    pub usage_restrictions: UsageRestrictions,
    pub auto_unlock_threshold: Option<f64>,
    pub exclusive_access_period: Duration,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum AdvertiserType {
    TechCompany,
    RetailBrand,
    HealthcareProvider,
    FinancialServices,
    Entertainment,
    Education,
    NonProfit,
    Government,
    Any,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UsageRestrictions {
    pub can_aggregate: bool,
    pub can_profile: bool,
    pub can_retarget: bool,
    pub max_usage_days: u32,
    pub attribution_required: bool,
    pub data_deletion_required: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NFTMetadata {
    pub name: String,
    pub description: String,
    pub image_url: Option<String>,
    pub attributes: Vec<NFTAttribute>,
    pub rarity_score: f64,
    pub collection: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NFTAttribute {
    pub trait_type: String,
    pub value: String,
    pub rarity: f64, // 0.0 to 1.0
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Bid {
    pub bidder: String,
    pub amount: f64,
    pub advertiser_type: AdvertiserType,
    pub campaign_purpose: String,
    pub bid_timestamp: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UnlockRecord {
    pub advertiser: String,
    pub amount_paid: f64,
    pub unlock_timestamp: DateTime<Utc>,
    pub access_duration: Duration,
    pub campaign_id: String,
    pub data_used: Vec<String>, // Specific data points accessed
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum NFTStatus {
    Active,
    Locked,
    Unlocked,
    Expired,
    Transferred,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AdvertiserUnlockContract {
    pub contract_id: String,
    pub advertiser: String,
    pub nft_id: String,
    pub payment_amount: f64,
    pub campaign_details: CampaignDetails,
    pub unlock_timestamp: DateTime<Utc>,
    pub access_expires_at: DateTime<Utc>,
    pub data_access_log: Vec<DataAccessLog>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CampaignDetails {
    pub campaign_id: String,
    pub campaign_name: String,
    pub advertiser_type: AdvertiserType,
    pub target_audience: String,
    pub campaign_purpose: String,
    pub compliance_certifications: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DataAccessLog {
    pub timestamp: DateTime<Utc>,
    pub data_point_id: String,
    pub access_type: AccessType,
    pub purpose: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NFTCollection {
    pub collection_id: String,
    pub name: String,
    pub description: String,
    pub creator: String,
    pub nfts: Vec<String>, // NFT IDs
    pub floor_price: f64,
    pub total_volume: f64,
    pub royalty_percentage: f64,
}

pub struct DataNFTEngine {
    pub nfts: HashMap<String, DataNFT>,
    pub collections: HashMap<String, NFTCollection>,
    pub unlock_contracts: HashMap<String, AdvertiserUnlockContract>,
    pub user_nfts: HashMap<String, Vec<String>>, // User -> NFT IDs
    pub advertiser_unlocks: HashMap<String, Vec<String>>, // Advertiser -> Contract IDs
    pub marketplace_bids: HashMap<String, Vec<Bid>>, // NFT ID -> Bids
    pub next_token_id: u64,
}

impl DataNFTEngine {
    pub fn new() -> Self {
        Self {
            nfts: HashMap::new(),
            collections: HashMap::new(),
            unlock_contracts: HashMap::new(),
            user_nfts: HashMap::new(),
            advertiser_unlocks: HashMap::new(),
            marketplace_bids: HashMap::new(),
            next_token_id: 1,
        }
    }

    // NFT Minting Functions
    pub fn mint_data_nft(&mut self, user_id: &str, data_points: Vec<&DataPoint>, period_type: PeriodType) -> Result<String, String> {
        if data_points.is_empty() {
            return Err("No data points provided".to_string());
        }

        let nft_id = format!("nft_{}_{}", user_id, self.next_token_id);
        let token_id = self.next_token_id;
        self.next_token_id += 1;

        // Calculate period dates
        let (start_date, end_date) = self.calculate_period_dates(&period_type);
        
        // Generate data summary
        let data_summary = self.generate_data_summary(data_points)?;
        
        // Set unlock conditions based on data value
        let unlock_conditions = self.generate_unlock_conditions(&data_summary);
        
        // Create NFT metadata
        let metadata = self.generate_nft_metadata(user_id, &data_summary, &period_type);
        
        let data_nft = DataNFT {
            nft_id: nft_id.clone(),
            token_id,
            owner: user_id.to_string(),
            data_period: DataPeriod {
                period_type,
                start_date,
                end_date,
                data_point_ids: data_points.iter().map(|dp| dp.data_id.clone()).collect(),
            },
            data_summary,
            unlock_conditions,
            metadata,
            current_bid: None,
            unlocked_by: Vec::new(),
            created_at: Utc::now(),
            expires_at: end_date + Duration::days(30), // NFT valid for 30 days after period
            status: NFTStatus::Active,
        };

        // Add to collections
        self.add_to_collection(&nft_id, user_id)?;
        
        // Store NFT
        self.nfts.insert(nft_id.clone(), data_nft);
        
        // Update user's NFT list
        self.user_nfts.entry(user_id.to_string())
            .or_insert_with(Vec::new)
            .push(nft_id.clone());

        Ok(nft_id)
    }

    fn calculate_period_dates(&self, period_type: &PeriodType) -> (DateTime<Utc>, DateTime<Utc>) {
        let now = Utc::now();
        match period_type {
            PeriodType::Daily => {
                let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
                let end = start + Duration::days(1);
                (start, end)
            },
            PeriodType::Weekly => {
                let days_since_monday = now.weekday().num_days_from_monday();
                let start = (now - Duration::days(days_since_monday as i64)).date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
                let end = start + Duration::weeks(1);
                (start, end)
            },
            PeriodType::Monthly => {
                let start = now.date_naive().with_day(1).unwrap().and_hms_opt(0, 0, 0).unwrap().and_utc();
                let end = if start.month() == 12 {
                    start.with_year(start.year() + 1).unwrap().with_month(1).unwrap()
                } else {
                    start.with_month(start.month() + 1).unwrap()
                };
                (start, end)
            },
            PeriodType::Custom { days } => {
                let start = now - Duration::days(*days as i64);
                (start, now)
            },
        }
    }

    fn generate_data_summary(&self, data_points: Vec<&DataPoint>) -> Result<DataSummary, String> {
        let mut categories = HashMap::new();
        let mut total_engagement = 0.0;
        let mut total_quality = 0.0;
        let mut total_value = 0.0;

        for data_point in &data_points {
            // Count categories
            let category_name = match &data_point.data_type {
                crate::protocol::data::DataCategory::SocialMedia { content_type, .. } => format!("social_{}", content_type),
                crate::protocol::data::DataCategory::Personal { category, .. } => format!("personal_{}", category),
                crate::protocol::data::DataCategory::Financial { transaction_type, .. } => format!("financial_{}", transaction_type),
                crate::protocol::data::DataCategory::Health { data_type, .. } => format!("health_{}", data_type),
                crate::protocol::data::DataCategory::Location { .. } => "location".to_string(),
                crate::protocol::data::DataCategory::Device { device_type, .. } => format!("device_{}", device_type),
            };
            
            *categories.entry(category_name).or_insert(0) += 1;
            
            // Calculate engagement score
            if let crate::protocol::data::DataCategory::SocialMedia { engagement_metrics, .. } = &data_point.data_type {
                total_engagement += (engagement_metrics.likes + engagement_metrics.shares * 2 + engagement_metrics.comments * 3) as f64;
            }
            
            total_quality += data_point.metadata.quality_score;
            total_value += data_point.value_score;
        }

        let data_count = data_points.len() as f64;
        
        Ok(DataSummary {
            total_data_points: data_points.len() as u64,
            categories,
            engagement_score: total_engagement,
            quality_score: total_quality / data_count,
            estimated_value: total_value,
            demographic_info: DemographicSummary {
                age_range: Some("25-34".to_string()), // Would be calculated from actual data
                location_region: Some("North America".to_string()),
                interests: vec!["Technology".to_string(), "Finance".to_string()],
                activity_level: ActivityLevel::High,
                spending_category: SpendingCategory::Moderate,
            },
            activity_patterns: ActivityPatterns {
                most_active_hours: vec![9, 12, 15, 18, 21],
                platform_usage: [("social_app".to_string(), 3.5)].iter().cloned().collect(),
                content_preferences: vec!["Tech News".to_string(), "Crypto".to_string()],
                interaction_frequency: 8.5,
            },
        })
    }

    fn generate_unlock_conditions(&self, data_summary: &DataSummary) -> UnlockConditions {
        let base_price = data_summary.estimated_value * 2.0; // 2x multiplier for NFT value
        
        UnlockConditions {
            minimum_payment: base_price.max(1.0),
            allowed_advertiser_types: vec![AdvertiserType::Any], // Can be customized by user
            usage_restrictions: UsageRestrictions {
                can_aggregate: true,
                can_profile: true,
                can_retarget: false, // Default to false for privacy
                max_usage_days: 30,
                attribution_required: true,
                data_deletion_required: true,
            },
            auto_unlock_threshold: Some(base_price * 3.0), // Auto-unlock at 3x price
            exclusive_access_period: Duration::days(7),
        }
    }

    fn generate_nft_metadata(&self, user_id: &str, data_summary: &DataSummary, period_type: &PeriodType) -> NFTMetadata {
        let period_name = match period_type {
            PeriodType::Daily => "Daily",
            PeriodType::Weekly => "Weekly",
            PeriodType::Monthly => "Monthly",
            PeriodType::Custom { days } => "Custom",
        };

        let name = format!("{} Data Collection #{}", period_name, self.next_token_id - 1);
        let description = format!(
            "Exclusive access to {} data points from user activity. Includes {} engagement score and {} quality rating.",
            data_summary.total_data_points,
            data_summary.engagement_score,
            data_summary.quality_score
        );

        // Calculate rarity based on data quality and engagement
        let rarity_score = (data_summary.quality_score * 0.4 + 
                           (data_summary.engagement_score / 100.0).min(1.0) * 0.6)
                           .min(1.0);

        let attributes = vec![
            NFTAttribute {
                trait_type: "Data Points".to_string(),
                value: data_summary.total_data_points.to_string(),
                rarity: if data_summary.total_data_points > 100 { 0.8 } else { 0.4 },
            },
            NFTAttribute {
                trait_type: "Quality Score".to_string(),
                value: format!("{:.2}", data_summary.quality_score),
                rarity: data_summary.quality_score,
            },
            NFTAttribute {
                trait_type: "Engagement Level".to_string(),
                value: if data_summary.engagement_score > 1000.0 { "High" } else { "Medium" }.to_string(),
                rarity: (data_summary.engagement_score / 2000.0).min(1.0),
            },
            NFTAttribute {
                trait_type: "Period Type".to_string(),
                value: period_name.to_string(),
                rarity: match period_type {
                    PeriodType::Monthly => 0.9,
                    PeriodType::Weekly => 0.6,
                    PeriodType::Daily => 0.3,
                    PeriodType::Custom { .. } => 0.7,
                },
            },
        ];

        NFTMetadata {
            name,
            description,
            image_url: Some(format!("https://api.layer1.social/nft/image/{}", self.next_token_id - 1)),
            attributes,
            rarity_score,
            collection: format!("{}_data_collection", user_id),
        }
    }

    fn add_to_collection(&mut self, nft_id: &str, user_id: &str) -> Result<(), String> {
        let collection_id = format!("{}_data_collection", user_id);
        
        let collection = self.collections.entry(collection_id.clone())
            .or_insert_with(|| NFTCollection {
                collection_id: collection_id.clone(),
                name: format!("{}'s Data Collection", user_id),
                description: "Personal data NFTs representing user activity and engagement".to_string(),
                creator: user_id.to_string(),
                nfts: Vec::new(),
                floor_price: 1.0,
                total_volume: 0.0,
                royalty_percentage: 5.0, // 5% royalty to user
            });

        collection.nfts.push(nft_id.to_string());
        Ok(())
    }

    // Advertiser Unlock Functions
    pub fn create_unlock_bid(&mut self, nft_id: &str, advertiser: &str, amount: f64, advertiser_type: AdvertiserType, campaign_purpose: &str) -> Result<String, String> {
        let nft = self.nfts.get(nft_id)
            .ok_or("NFT not found")?;

        if amount < nft.unlock_conditions.minimum_payment {
            return Err(format!("Bid amount {} is below minimum {}", amount, nft.unlock_conditions.minimum_payment));
        }

        if !nft.unlock_conditions.allowed_advertiser_types.contains(&advertiser_type) && 
           !nft.unlock_conditions.allowed_advertiser_types.contains(&AdvertiserType::Any) {
            return Err("Advertiser type not allowed".to_string());
        }

        let bid = Bid {
            bidder: advertiser.to_string(),
            amount,
            advertiser_type,
            campaign_purpose: campaign_purpose.to_string(),
            bid_timestamp: Utc::now(),
            expires_at: Utc::now() + Duration::hours(24),
        };

        self.marketplace_bids.entry(nft_id.to_string())
            .or_insert_with(Vec::new)
            .push(bid);

        // Check for auto-unlock
        if let Some(auto_threshold) = nft.unlock_conditions.auto_unlock_threshold {
            if amount >= auto_threshold {
                return self.execute_unlock(nft_id, advertiser, amount, campaign_purpose);
            }
        }

        Ok("Bid placed successfully".to_string())
    }

    pub fn execute_unlock(&mut self, nft_id: &str, advertiser: &str, amount: f64, campaign_purpose: &str) -> Result<String, String> {
        let nft = self.nfts.get_mut(nft_id)
            .ok_or("NFT not found")?;

        if nft.status != NFTStatus::Active {
            return Err("NFT is not available for unlock".to_string());
        }

        let contract_id = format!("unlock_{}_{}", nft_id, chrono::Utc::now().timestamp());
        
        // Create unlock contract
        let unlock_contract = AdvertiserUnlockContract {
            contract_id: contract_id.clone(),
            advertiser: advertiser.to_string(),
            nft_id: nft_id.to_string(),
            payment_amount: amount,
            campaign_details: CampaignDetails {
                campaign_id: format!("campaign_{}", chrono::Utc::now().timestamp()),
                campaign_name: campaign_purpose.to_string(),
                advertiser_type: AdvertiserType::Any, // Would be specified in real implementation
                target_audience: "General".to_string(),
                campaign_purpose: campaign_purpose.to_string(),
                compliance_certifications: vec!["GDPR".to_string(), "CCPA".to_string()],
            },
            unlock_timestamp: Utc::now(),
            access_expires_at: Utc::now() + nft.unlock_conditions.exclusive_access_period,
            data_access_log: Vec::new(),
        };

        // Record unlock
        let unlock_record = UnlockRecord {
            advertiser: advertiser.to_string(),
            amount_paid: amount,
            unlock_timestamp: Utc::now(),
            access_duration: nft.unlock_conditions.exclusive_access_period,
            campaign_id: unlock_contract.campaign_details.campaign_id.clone(),
            data_used: Vec::new(),
        };

        nft.unlocked_by.push(unlock_record);
        nft.status = NFTStatus::Unlocked;

        // Store contract
        self.unlock_contracts.insert(contract_id.clone(), unlock_contract);
        
        // Update advertiser's unlocks
        self.advertiser_unlocks.entry(advertiser.to_string())
            .or_insert_with(Vec::new)
            .push(contract_id.clone());

        // Update collection volume
        if let Some(collection) = self.collections.get_mut(&nft.metadata.collection) {
            collection.total_volume += amount;
        }

        Ok(contract_id)
    }

    // Data Access Functions
    pub fn access_nft_data(&mut self, contract_id: &str, data_point_id: &str, access_type: AccessType, purpose: &str) -> Result<String, String> {
        let contract = self.unlock_contracts.get_mut(contract_id)
            .ok_or("Unlock contract not found")?;

        if Utc::now() > contract.access_expires_at {
            return Err("Access period has expired".to_string());
        }

        let nft = self.nfts.get(contract.nft_id.as_str())
            .ok_or("NFT not found")?;

        if !nft.data_period.data_point_ids.contains(&data_point_id.to_string()) {
            return Err("Data point not included in this NFT".to_string());
        }

        // Log access
        let access_log = DataAccessLog {
            timestamp: Utc::now(),
            data_point_id: data_point_id.to_string(),
            access_type,
            purpose: purpose.to_string(),
        };

        contract.data_access_log.push(access_log);

        Ok(format!("Access granted to data point {}", data_point_id))
    }

    // Query Functions
    pub fn get_user_nfts(&self, user_id: &str) -> Vec<&DataNFT> {
        if let Some(nft_ids) = self.user_nfts.get(user_id) {
            nft_ids.iter()
                .filter_map(|id| self.nfts.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_marketplace_nfts(&self) -> Vec<&DataNFT> {
        self.nfts.values()
            .filter(|nft| nft.status == NFTStatus::Active)
            .collect()
    }

    pub fn get_nft_details(&self, nft_id: &str) -> Option<&DataNFT> {
        self.nfts.get(nft_id)
    }

    pub fn get_advertiser_unlocks(&self, advertiser: &str) -> Vec<&AdvertiserUnlockContract> {
        if let Some(contract_ids) = self.advertiser_unlocks.get(advertiser) {
            contract_ids.iter()
                .filter_map(|id| self.unlock_contracts.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_collection_stats(&self, collection_id: &str) -> Option<&NFTCollection> {
        self.collections.get(collection_id)
    }

    // Analytics Functions
    pub fn get_nft_analytics(&self) -> HashMap<String, serde_json::Value> {
        let mut analytics = HashMap::new();
        
        analytics.insert("total_nfts".to_string(), serde_json::Value::Number(serde_json::Number::from(self.nfts.len())));
        analytics.insert("total_unlocks".to_string(), serde_json::Value::Number(serde_json::Number::from(self.unlock_contracts.len())));
        
        let total_volume: f64 = self.collections.values().map(|c| c.total_volume).sum();
        analytics.insert("total_marketplace_volume".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(total_volume).unwrap_or(serde_json::Number::from(0))));
        
        let active_nfts = self.nfts.values().filter(|nft| nft.status == NFTStatus::Active).count();
        analytics.insert("active_nfts".to_string(), serde_json::Value::Number(serde_json::Number::from(active_nfts)));

        analytics
    }
}