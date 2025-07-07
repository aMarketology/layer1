use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

// Core Social Action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialAction {
    pub action_type: SocialActionType,
    pub user_address: String,
    pub post_id: String,
    pub target_user: Option<String>, // For likes/comments - who gets the reward
    pub timestamp: u64,
    pub reward_amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SocialActionType {
    Post,    // Creating a post (10 L1)
    Like,    // Liking someone's post (1/100000 to author)
    Comment, // Commenting on someone's post (1/100000 to commenter)
}

// Main Social Mining System
#[derive(Debug, Clone)]
pub struct SocialMiningSystem {
    pub actions: Vec<SocialAction>,
    pub daily_limits: HashMap<String, DailyLimits>, // user_address -> limits
}

#[derive(Debug, Clone)]
pub struct DailyLimits {
    pub date: String,
    pub posts: u64,
    pub likes: u64,
    pub comments: u64,
}

// API Request/Response Structures
#[derive(Deserialize)]
pub struct SocialPostRequest {
    pub user_address: String,
    pub post_id: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct SocialLikeRequest {
    pub user_address: String,
    pub post_id: String,
    pub post_author: String,
}

#[derive(Deserialize)]
pub struct SocialCommentRequest {
    pub user_address: String,
    pub post_id: String,
    pub post_author: String,
    pub comment_content: String,
}

#[derive(Serialize)]
pub struct SocialActionResponse {
    pub success: bool,
    pub message: String,
    pub reward_amount: f64,
    pub action_type: String,
}

#[derive(Serialize)]
pub struct SocialStatsResponse {
    pub total_posts: u64,
    pub total_likes: u64,
    pub total_comments: u64,
    pub total_rewards_distributed: f64,
    pub top_earners: Vec<UserEarnings>,
}

#[derive(Serialize)]
pub struct UserEarnings {
    pub user_address: String,
    pub username: Option<String>,
    pub total_earnings: f64,
    pub posts_count: u64,
}

impl SocialMiningSystem {
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
            daily_limits: HashMap::new(),
        }
    }

    // Get today as string for daily limits
    fn get_today() -> String {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let days = now / 86400; // Convert to days
        format!("day_{}", days)
    }

    // Check if user can perform action (daily limits)
    pub fn check_daily_limits(&mut self, user_address: &str, action_type: &SocialActionType) -> Result<(), String> {
        let today = Self::get_today();
        let limits = self.daily_limits
            .entry(user_address.to_string())
            .or_insert(DailyLimits {
                date: today.clone(),
                posts: 0,
                likes: 0,
                comments: 0,
            });

        // Reset if new day
        if limits.date != today {
            limits.date = today;
            limits.posts = 0;
            limits.likes = 0;
            limits.comments = 0;
        }

        // Check limits
        match action_type {
            SocialActionType::Post => {
                if limits.posts >= 50 { return Err("Daily post limit reached (50)".to_string()); }
            },
            SocialActionType::Like => {
                if limits.likes >= 1000 { return Err("Daily like limit reached (1000)".to_string()); }
            },
            SocialActionType::Comment => {
                if limits.comments >= 200 { return Err("Daily comment limit reached (200)".to_string()); }
            },
        }

        Ok(())
    }

    // Update daily limits after successful action
    pub fn update_daily_limits(&mut self, user_address: &str, action_type: &SocialActionType) {
        let today = Self::get_today();
        let limits = self.daily_limits
            .entry(user_address.to_string())
            .or_insert(DailyLimits {
                date: today.clone(),
                posts: 0,
                likes: 0,
                comments: 0,
            });

        match action_type {
            SocialActionType::Post => limits.posts += 1,
            SocialActionType::Like => limits.likes += 1,
            SocialActionType::Comment => limits.comments += 1,
        }
    }

    // Calculate reward amount based on action type
    pub fn calculate_reward(&self, action_type: &SocialActionType, max_supply: f64) -> f64 {
        match action_type {
            SocialActionType::Post => 10.0, // Fixed 10 L1 for posts
            SocialActionType::Like => max_supply / 100000.0, // 1/100000 of max supply
            SocialActionType::Comment => max_supply / 100000.0, // 1/100000 of max supply
        }
    }

    // Record a social action
    pub fn record_action(&mut self, action: SocialAction) {
        self.actions.push(action);
    }

    // Get social mining statistics
    pub fn get_stats(&self) -> SocialStatsResponse {
        let total_posts = self.actions.iter().filter(|a| matches!(a.action_type, SocialActionType::Post)).count() as u64;
        let total_likes = self.actions.iter().filter(|a| matches!(a.action_type, SocialActionType::Like)).count() as u64;
        let total_comments = self.actions.iter().filter(|a| matches!(a.action_type, SocialActionType::Comment)).count() as u64;
        let total_rewards_distributed = self.actions.iter().map(|a| a.reward_amount).sum();

        // Calculate top earners
        let mut earnings: HashMap<String, f64> = HashMap::new();
        let mut post_counts: HashMap<String, u64> = HashMap::new();

        for action in &self.actions {
            *earnings.entry(action.user_address.clone()).or_insert(0.0) += action.reward_amount;
            if matches!(action.action_type, SocialActionType::Post) {
                *post_counts.entry(action.user_address.clone()).or_insert(0) += 1;
            }
        }

        let mut top_earners: Vec<UserEarnings> = earnings
            .into_iter()
            .map(|(user_address, total_earnings)| UserEarnings {
                user_address: user_address.clone(),
                username: None, // Will be filled by blockchain
                total_earnings,
                posts_count: *post_counts.get(&user_address).unwrap_or(&0),
            })
            .collect();

        top_earners.sort_by(|a, b| b.total_earnings.partial_cmp(&a.total_earnings).unwrap());
        top_earners.truncate(10); // Top 10

        SocialStatsResponse {
            total_posts,
            total_likes,
            total_comments,
            total_rewards_distributed,
            top_earners,
        }
    }

    // Get user's social earnings
    pub fn get_user_earnings(&self, user_address: &str) -> f64 {
        self.actions
            .iter()
            .filter(|action| action.user_address == user_address)
            .map(|action| action.reward_amount)
            .sum()
    }

    // Cleanup old actions (keep last 1000 actions for performance)
    pub fn cleanup_old_actions(&mut self) {
        if self.actions.len() > 1000 {
            let keep_count = 1000;
            self.actions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
            self.actions.truncate(keep_count);
            println!("🧹 Social Mining: Cleaned up old actions, keeping latest {}", keep_count);
        }
    }
}