use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SmartContract {
    pub contract_id: String,
    pub contract_type: ContractType,
    pub creator: String,
    pub participants: Vec<String>,
    pub state: ContractState,
    pub balance: f64,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ContractType {
    SocialWager,
    ChessGame,
    SportsStaking,
    FitnessChallenge,
    WordleGame,
    DataReward,
    ContentCreator,
    StakingPool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ContractState {
    Pending,
    Active,
    Completed,
    Cancelled,
    Disputed,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChessGameContract {
    pub game_id: String,
    pub white_player: String,
    pub black_player: String,
    pub wager_amount: f64,
    pub winner: Option<String>,
    pub moves: Vec<String>,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SportsStakingContract {
    pub event_id: String,
    pub event_type: SportType,
    pub event_description: String,
    pub prediction: String,
    pub stake_amount: f64,
    pub odds: f64,
    pub outcome: Option<String>,
    pub event_date: DateTime<Utc>,
    pub oracle_source: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SportType {
    NFL,
    NBA,
    Soccer,
    Tennis,
    Chess,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FitnessContract {
    pub gym_name: String,
    pub user: String,
    pub target_days: u32,
    pub current_days: u32,
    pub month: String,
    pub stake_amount: f64,
    pub reward_multiplier: f64,
    pub check_ins: Vec<DateTime<Utc>>,
    pub gym_verified: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WordleContract {
    pub player: String,
    pub daily_word: String,
    pub guesses: Vec<String>,
    pub completed: bool,
    pub score: Option<u32>,
    pub reward_amount: f64,
    pub date: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DataRewardContract {
    pub user: String,
    pub data_type: DataType,
    pub value_generated: f64,
    pub reward_rate: f64,
    pub total_earned: f64,
    pub last_payout: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum DataType {
    SocialPost,
    ProfileData,
    InteractionData,
    LocationData,
    PurchaseData,
    HealthData,
}

pub struct SmartContractEngine {
    pub contracts: HashMap<String, SmartContract>,
    pub chess_games: HashMap<String, ChessGameContract>,
    pub sports_stakes: HashMap<String, SportsStakingContract>,
    pub fitness_challenges: HashMap<String, FitnessContract>,
    pub wordle_games: HashMap<String, WordleContract>,
    pub data_rewards: HashMap<String, DataRewardContract>,
}

impl SmartContractEngine {
    pub fn new() -> Self {
        Self {
            contracts: HashMap::new(),
            chess_games: HashMap::new(),
            sports_stakes: HashMap::new(),
            fitness_challenges: HashMap::new(),
            wordle_games: HashMap::new(),
            data_rewards: HashMap::new(),
        }
    }

    // Chess Game Contract
    pub fn create_chess_wager(&mut self, white_player: &str, black_player: &str, wager_amount: f64) -> Result<String, String> {
        let game_id = format!("chess_{}", self.chess_games.len());
        
        let chess_game = ChessGameContract {
            game_id: game_id.clone(),
            white_player: white_player.to_string(),
            black_player: black_player.to_string(),
            wager_amount,
            winner: None,
            moves: Vec::new(),
            started_at: Utc::now(),
            ended_at: None,
        };

        let contract = SmartContract {
            contract_id: game_id.clone(),
            contract_type: ContractType::ChessGame,
            creator: white_player.to_string(),
            participants: vec![white_player.to_string(), black_player.to_string()],
            state: ContractState::Active,
            balance: wager_amount * 2.0, // Both players stake
            created_at: Utc::now(),
            expires_at: Some(Utc::now() + chrono::Duration::hours(24)),
            metadata: HashMap::new(),
        };

        self.contracts.insert(game_id.clone(), contract);
        self.chess_games.insert(game_id.clone(), chess_game);
        
        Ok(game_id)
    }

    pub fn finish_chess_game(&mut self, game_id: &str, winner: &str) -> Result<f64, String> {
        let game = self.chess_games.get_mut(game_id)
            .ok_or("Chess game not found")?;
        
        if game.winner.is_some() {
            return Err("Game already finished".to_string());
        }

        game.winner = Some(winner.to_string());
        game.ended_at = Some(Utc::now());

        let contract = self.contracts.get_mut(game_id)
            .ok_or("Contract not found")?;
        
        contract.state = ContractState::Completed;
        let reward = contract.balance;
        contract.balance = 0.0;

        Ok(reward)
    }

    // Sports Staking Contract
    pub fn create_sports_stake(&mut self, user: &str, event_description: &str, prediction: &str, stake_amount: f64, event_date: DateTime<Utc>) -> Result<String, String> {
        let event_id = format!("sports_{}", self.sports_stakes.len());
        
        let sports_stake = SportsStakingContract {
            event_id: event_id.clone(),
            event_type: SportType::NFL, // Default, can be specified
            event_description: event_description.to_string(),
            prediction: prediction.to_string(),
            stake_amount,
            odds: 2.0, // Default 2:1 odds
            outcome: None,
            event_date,
            oracle_source: "ESPN".to_string(),
        };

        let contract = SmartContract {
            contract_id: event_id.clone(),
            contract_type: ContractType::SportsStaking,
            creator: user.to_string(),
            participants: vec![user.to_string()],
            state: ContractState::Active,
            balance: stake_amount,
            created_at: Utc::now(),
            expires_at: Some(event_date + chrono::Duration::hours(6)),
            metadata: HashMap::new(),
        };

        self.contracts.insert(event_id.clone(), contract);
        self.sports_stakes.insert(event_id.clone(), sports_stake);
        
        Ok(event_id)
    }

    pub fn resolve_sports_stake(&mut self, event_id: &str, actual_outcome: &str) -> Result<f64, String> {
        let stake = self.sports_stakes.get_mut(event_id)
            .ok_or("Sports stake not found")?;
        
        stake.outcome = Some(actual_outcome.to_string());
        
        let contract = self.contracts.get_mut(event_id)
            .ok_or("Contract not found")?;
        
        let reward = if stake.prediction == actual_outcome {
            stake.stake_amount * stake.odds // Winner gets multiplied amount
        } else {
            0.0 // Loser gets nothing
        };

        contract.state = ContractState::Completed;
        contract.balance = 0.0;

        Ok(reward)
    }

    // Fitness Challenge Contract
    pub fn create_fitness_challenge(&mut self, user: &str, gym_name: &str, target_days: u32, stake_amount: f64) -> Result<String, String> {
        let challenge_id = format!("fitness_{}", self.fitness_challenges.len());
        
        let fitness_challenge = FitnessContract {
            gym_name: gym_name.to_string(),
            user: user.to_string(),
            target_days,
            current_days: 0,
            month: chrono::Utc::now().format("%Y-%m").to_string(),
            stake_amount,
            reward_multiplier: 2.5, // 2.5x return if successful
            check_ins: Vec::new(),
            gym_verified: false,
        };

        let contract = SmartContract {
            contract_id: challenge_id.clone(),
            contract_type: ContractType::FitnessChallenge,
            creator: user.to_string(),
            participants: vec![user.to_string()],
            state: ContractState::Active,
            balance: stake_amount,
            created_at: Utc::now(),
            expires_at: Some(Utc::now() + chrono::Duration::days(30)),
            metadata: HashMap::new(),
        };

        self.contracts.insert(challenge_id.clone(), contract);
        self.fitness_challenges.insert(challenge_id.clone(), fitness_challenge);
        
        Ok(challenge_id)
    }

    pub fn record_gym_checkin(&mut self, challenge_id: &str, gym_verification: bool) -> Result<u32, String> {
        let challenge = self.fitness_challenges.get_mut(challenge_id)
            .ok_or("Fitness challenge not found")?;
        
        if gym_verification {
            challenge.check_ins.push(Utc::now());
            challenge.current_days += 1;
            challenge.gym_verified = true;
        }

        // Check if challenge is completed
        if challenge.current_days >= challenge.target_days {
            let contract = self.contracts.get_mut(challenge_id)
                .ok_or("Contract not found")?;
            contract.state = ContractState::Completed;
        }

        Ok(challenge.current_days)
    }

    // Wordle Game Contract
    pub fn create_wordle_game(&mut self, player: &str, daily_word: &str) -> Result<String, String> {
        let game_id = format!("wordle_{}_{}", player, chrono::Utc::now().format("%Y%m%d"));
        
        let wordle_game = WordleContract {
            player: player.to_string(),
            daily_word: daily_word.to_string(),
            guesses: Vec::new(),
            completed: false,
            score: None,
            reward_amount: 1.0, // Base reward
            date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
        };

        let contract = SmartContract {
            contract_id: game_id.clone(),
            contract_type: ContractType::WordleGame,
            creator: player.to_string(),
            participants: vec![player.to_string()],
            state: ContractState::Active,
            balance: 5.0, // Pool reward
            created_at: Utc::now(),
            expires_at: Some(Utc::now() + chrono::Duration::hours(24)),
            metadata: HashMap::new(),
        };

        self.contracts.insert(game_id.clone(), contract);
        self.wordle_games.insert(game_id.clone(), wordle_game);
        
        Ok(game_id)
    }

    pub fn submit_wordle_guess(&mut self, game_id: &str, guess: &str) -> Result<String, String> {
        let game = self.wordle_games.get_mut(game_id)
            .ok_or("Wordle game not found")?;
        
        if game.completed {
            return Err("Game already completed".to_string());
        }

        game.guesses.push(guess.to_string());
        
        if guess == game.daily_word {
            game.completed = true;
            game.score = Some(game.guesses.len() as u32);
            
            // Calculate reward based on number of guesses
            let reward_multiplier = match game.guesses.len() {
                1 => 5.0,
                2 => 3.0,
                3 => 2.0,
                4 => 1.5,
                5 => 1.2,
                6 => 1.0,
                _ => 0.5,
            };
            
            game.reward_amount = game.reward_amount * reward_multiplier;
            
            let contract = self.contracts.get_mut(game_id)
                .ok_or("Contract not found")?;
            contract.state = ContractState::Completed;
            
            Ok(format!("Correct! Reward: {} L1", game.reward_amount))
        } else if game.guesses.len() >= 6 {
            game.completed = true;
            game.score = Some(0);
            
            let contract = self.contracts.get_mut(game_id)
                .ok_or("Contract not found")?;
            contract.state = ContractState::Completed;
            
            Ok("Game over! No reward.".to_string())
        } else {
            Ok(format!("Incorrect. {} guesses remaining.", 6 - game.guesses.len()))
        }
    }

    // Data Reward Contract
    pub fn create_data_reward_contract(&mut self, user: &str, data_type: DataType) -> Result<String, String> {
        let contract_id = format!("data_{}_{}", user, chrono::Utc::now().timestamp());
        
        let reward_rate = match data_type {
            DataType::SocialPost => 0.1,
            DataType::ProfileData => 0.05,
            DataType::InteractionData => 0.02,
            DataType::LocationData => 0.15,
            DataType::PurchaseData => 0.25,
            DataType::HealthData => 0.3,
        };

        let data_reward = DataRewardContract {
            user: user.to_string(),
            data_type: data_type.clone(),
            value_generated: 0.0,
            reward_rate,
            total_earned: 0.0,
            last_payout: Utc::now(),
        };

        let contract = SmartContract {
            contract_id: contract_id.clone(),
            contract_type: ContractType::DataReward,
            creator: user.to_string(),
            participants: vec![user.to_string()],
            state: ContractState::Active,
            balance: 0.0,
            created_at: Utc::now(),
            expires_at: None, // Ongoing contract
            metadata: HashMap::new(),
        };

        self.contracts.insert(contract_id.clone(), contract);
        self.data_rewards.insert(contract_id.clone(), data_reward);
        
        Ok(contract_id)
    }

    pub fn process_data_value(&mut self, contract_id: &str, data_value: f64) -> Result<f64, String> {
        let reward_contract = self.data_rewards.get_mut(contract_id)
            .ok_or("Data reward contract not found")?;
        
        let reward = data_value * reward_contract.reward_rate;
        reward_contract.value_generated += data_value;
        reward_contract.total_earned += reward;
        reward_contract.last_payout = Utc::now();

        Ok(reward)
    }

    // Utility Functions
    pub fn get_contract(&self, contract_id: &str) -> Option<&SmartContract> {
        self.contracts.get(contract_id)
    }

    pub fn get_active_contracts(&self, user: &str) -> Vec<&SmartContract> {
        self.contracts.values()
            .filter(|contract| {
                contract.participants.contains(&user.to_string()) && 
                matches!(contract.state, ContractState::Active)
            })
            .collect()
    }

    pub fn get_user_stats(&self, user: &str) -> HashMap<String, serde_json::Value> {
        let mut stats = HashMap::new();
        
        let user_contracts: Vec<_> = self.contracts.values()
            .filter(|c| c.participants.contains(&user.to_string()))
            .collect();

        stats.insert("total_contracts".to_string(), serde_json::Value::Number(serde_json::Number::from(user_contracts.len())));
        stats.insert("active_contracts".to_string(), serde_json::Value::Number(serde_json::Number::from(
            user_contracts.iter().filter(|c| matches!(c.state, ContractState::Active)).count()
        )));
        stats.insert("completed_contracts".to_string(), serde_json::Value::Number(serde_json::Number::from(
            user_contracts.iter().filter(|c| matches!(c.state, ContractState::Completed)).count()
        )));

        stats
    }
}