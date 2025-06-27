use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use warp::Filter;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::time;

// Transaction structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Transaction {
    from: String,
    to: String,
    amount: f64,
    timestamp: u64,
    signature: String,
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
            "{}{}{}{}{}{}",
            self.index, self.timestamp, transactions_data, 
            self.previous_hash, self.nonce, self.miner
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

// Enhanced Blockchain structure with connection tracking
struct Blockchain {
    chain: Vec<Block>,
    difficulty: usize,
    pending_transactions: Vec<Transaction>,
    balances: HashMap<String, f64>,
    mining_reward: f64,
    connections: HashMap<String, Connection>,
    max_supply: f64,
    circulating_supply: f64,
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
            max_supply: 21_000_000.0, // Like Bitcoin
            circulating_supply: 0.0,
        };
        blockchain.create_genesis_block();
        blockchain
    }

    fn create_genesis_block(&mut self) {
        let mut genesis_block = Block::new(0, Vec::new(), "0".to_string(), "genesis".to_string());
        genesis_block.mine_block(self.difficulty);
        self.chain.push(genesis_block);
    }

    fn connect_user(&mut self, address: String) -> Result<String, String> {
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

    fn get_connection_info(&self, address: &str) -> Option<&Connection> {
        self.connections.get(address)
    }

    fn get_all_connections(&self) -> Vec<&Connection> {
        self.connections.values().filter(|c| c.is_active).collect()
    }

    fn create_transaction(&mut self, from: String, to: String, amount: f64) -> Result<String, String> {
        if from != "genesis" && from != "mining_reward" && from != "connection_reward" {
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
            signature: format!("sig_{}", rand::random::<u64>()),
        };

        self.pending_transactions.push(transaction);
        Ok("Transaction added to pending pool".to_string())
    }

    fn mine_pending_transactions(&mut self, miner_address: String) {
        let reward_tx = Transaction {
            from: "mining_reward".to_string(),
            to: miner_address.clone(),
            amount: self.mining_reward,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            signature: "reward".to_string(),
        };
        self.pending_transactions.push(reward_tx);

        let previous_block = self.chain.last().unwrap();
        let mut new_block = Block::new(
            previous_block.index + 1,
            self.pending_transactions.clone(),
            previous_block.hash.clone(),
            miner_address,
        );
        
        new_block.mine_block(self.difficulty);
        self.chain.push(new_block);
        
        self.update_balances();
        self.pending_transactions.clear();
    }

    fn update_balances(&mut self) {
        self.balances.clear();
        self.circulating_supply = 0.0;
        
        for block in &self.chain {
            for tx in &block.transactions {
                if tx.from != "mining_reward" && tx.from != "genesis" && tx.from != "connection_reward" {
                    *self.balances.entry(tx.from.clone()).or_insert(0.0) -= tx.amount;
                }
                *self.balances.entry(tx.to.clone()).or_insert(0.0) += tx.amount;
                self.circulating_supply += tx.amount;
            }
        }
    }

    fn get_balance(&self, address: &str) -> f64 {
        *self.balances.get(address).unwrap_or(&0.0)
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
}

// Request/Response structures
#[derive(Deserialize)]
struct TransactionRequest {
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

#[derive(Deserialize)]
struct CreateWalletRequest {
    user_id: String,
    username: String,
}

#[derive(Serialize)]
struct WalletResponse {
    wallet_address: String,
    initial_balance: f64,
    transaction_hash: String,
}

#[derive(Deserialize)]
struct TipRequest {
    from_address: String,
    to_address: String,
    amount: f64,
    post_id: Option<String>,
    message: Option<String>,
}

#[derive(Deserialize)]
struct TransactionHistoryRequest {
    address: String,
    limit: Option<i32>,
    offset: Option<i32>,
}

#[derive(Serialize)]
struct TransactionHistory {
    transactions: Vec<Transaction>,
    total_count: usize,
}

#[derive(Serialize)]
struct UserWalletInfo {
    address: String,
    balance: f64,
    total_sent: f64,
    total_received: f64,
    transaction_count: usize,
}

#[tokio::main]
async fn main() {
    let blockchain = Arc::new(Mutex::new(Blockchain::new()));
    
    // Initialize with some test balances
    {
        let mut bc = blockchain.lock().unwrap();
        bc.create_transaction("genesis".to_string(), "alice".to_string(), 100.0).unwrap();
        bc.create_transaction("genesis".to_string(), "bob".to_string(), 50.0).unwrap();
        bc.mine_pending_transactions("miner1".to_string());
    }
    
    // Start connection reward processing task
    let blockchain_rewards = blockchain.clone();
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(5)); // Check every 5 seconds
        loop {
            interval.tick().await;
            let mut bc = blockchain_rewards.lock().unwrap();
            bc.process_connection_rewards();
        }
    });
    
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type"])
        .allow_methods(vec!["GET", "POST"]);
    
    // Clone blockchain for different routes
    let bc_chain = blockchain.clone();
    let bc_tx = blockchain.clone();
    let bc_mine = blockchain.clone();
    let bc_balance = blockchain.clone();
    let bc_connect = blockchain.clone();
    let bc_disconnect = blockchain.clone();
    let bc_connections = blockchain.clone();
    let bc_stats = blockchain.clone();
    
    // GET blockchain
    let get_blockchain = warp::path("blockchain")
        .and(warp::get())
        .map(move || {
            let bc = bc_chain.lock().unwrap();
            warp::reply::json(&bc.chain)
        });
    
    // POST transaction
    let create_transaction = warp::path("rpc")
        .and(warp::path("transaction"))
        .and(warp::post())
        .and(warp::body::json())
        .map(move |req: TransactionRequest| {
            let mut bc = bc_tx.lock().unwrap();
            match bc.create_transaction(req.from, req.to, req.amount) {
                Ok(msg) => warp::reply::json(&serde_json::json!({"success": true, "message": msg})),
                Err(err) => warp::reply::json(&serde_json::json!({"success": false, "error": err})),
            }
        });
    
    // POST mine block
    let mine_block = warp::path("rpc")
        .and(warp::path("mine"))
        .and(warp::post())
        .and(warp::body::json())
        .map(move |req: MineRequest| {
            let mut bc = bc_mine.lock().unwrap();
            bc.mine_pending_transactions(req.miner_address);
            warp::reply::json(&serde_json::json!({"success": true, "message": "Block mined successfully"}))
        });
    
    // GET balance
    let get_balance = warp::path("rpc")
        .and(warp::path("balance"))
        .and(warp::path::param::<String>())
        .and(warp::get())
        .map(move |address: String| {
            let bc = bc_balance.lock().unwrap();
            let balance = bc.get_balance(&address);
            warp::reply::json(&BalanceResponse { address, balance })
        });
    
    // POST connect user
    let connect_user = warp::path("rpc")
        .and(warp::path("connect"))
        .and(warp::post())
        .and(warp::body::json())
        .map(move |req: ConnectRequest| {
            let mut bc = bc_connect.lock().unwrap();
            match bc.connect_user(req.address) {
                Ok(msg) => warp::reply::json(&serde_json::json!({"success": true, "message": msg})),
                Err(err) => warp::reply::json(&serde_json::json!({"success": false, "error": err})),
            }
        });
    
    // POST disconnect user
    let disconnect_user = warp::path("rpc")
        .and(warp::path("disconnect"))
        .and(warp::post())
        .and(warp::body::json())
        .map(move |req: ConnectRequest| {
            let mut bc = bc_disconnect.lock().unwrap();
            match bc.disconnect_user(&req.address) {
                Ok(msg) => warp::reply::json(&serde_json::json!({"success": true, "message": msg})),
                Err(err) => warp::reply::json(&serde_json::json!({"success": false, "error": err})),
            }
        });
    
    // GET connections
    let get_connections = warp::path("rpc")
        .and(warp::path("connections"))
        .and(warp::get())
        .map(move || {
            let bc = bc_connections.lock().unwrap();
            warp::reply::json(&bc.get_all_connections())
        });
    
    // GET network stats
    let get_stats = warp::path("rpc")
        .and(warp::path("stats"))
        .and(warp::get())
        .map(move || {
            let bc = bc_stats.lock().unwrap();
            warp::reply::json(&bc.get_network_stats())
        });
    
    // POST create wallet (for new user signup)
    let create_wallet = warp::path("rpc")
        .and(warp::path("create-wallet"))
        .and(warp::post())
        .and(warp::body::json())
        .map(move |req: CreateWalletRequest| {
            let mut bc = blockchain.clone().lock().unwrap();
            
            // Generate unique wallet address
            let wallet_address = format!("wallet_{}", req.user_id);
            
            // Create signup bonus transaction
            match bc.create_transaction(
                "genesis".to_string(), 
                wallet_address.clone(), 
                1000.0
            ) {
                Ok(_) => {
                    // Mine the transaction immediately for instant credit
                    bc.mine_pending_transactions("system".to_string());
                    
                    warp::reply::json(&WalletResponse {
                        wallet_address: wallet_address.clone(),
                        initial_balance: 1000.0,
                        transaction_hash: format!("signup_bonus_{}", req.user_id),
                    })
                },
                Err(err) => {
                    warp::reply::json(&serde_json::json!({
                        "success": false, 
                        "error": err
                    }))
                }
            }
        });
    
    // GET transaction history for an address
    let get_transaction_history = warp::path("rpc")
        .and(warp::path("history"))
        .and(warp::path::param::<String>())
        .and(warp::get())
        .map(move |address: String| {
            let bc = blockchain.clone().lock().unwrap();
            let mut user_transactions = Vec::new();
            
            // Collect all transactions involving this address
            for block in &bc.chain {
                for tx in &block.transactions {
                    if tx.from == address || tx.to == address {
                        user_transactions.push(tx.clone());
                    }
                }
            }
            
            // Sort by timestamp (newest first)
            user_transactions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
            
            warp::reply::json(&TransactionHistory {
                transactions: user_transactions,
                total_count: user_transactions.len(),
            })
        });
    
    // GET detailed wallet info
    let get_wallet_info = warp::path("rpc")
        .and(warp::path("wallet-info"))
        .and(warp::path::param::<String>())
        .and(warp::get())
        .map(move |address: String| {
            let bc = blockchain.clone().lock().unwrap();
            let balance = bc.get_balance(&address);
            
            let mut total_sent = 0.0;
            let mut total_received = 0.0;
            let mut transaction_count = 0;
            
            for block in &bc.chain {
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
            
            warp::reply::json(&UserWalletInfo {
                address,
                balance,
                total_sent,
                total_received,
                transaction_count,
            })
        });
    
    // POST tip transaction (for social media features)
    let send_tip = warp::path("rpc")
        .and(warp::path("tip"))
        .and(warp::post())
        .and(warp::body::json())
        .map(move |req: TipRequest| {
            let mut bc = blockchain.clone().lock().unwrap();
            match bc.create_transaction(req.from_address, req.to_address, req.amount) {
                Ok(msg) => {
                    warp::reply::json(&serde_json::json!({
                        "success": true, 
                        "message": msg,
                        "tip_amount": req.amount,
                        "post_id": req.post_id,
                        "tip_message": req.message
                    }))
                },
                Err(err) => {
                    warp::reply::json(&serde_json::json!({
                        "success": false, 
                        "error": err
                    }))
                }
            }
        });
    
    let root = warp::path::end()
        .map(|| warp::reply::html("Blockchain server is running! Open the HTML file to use the interface."));

    let routes = root
        .or(get_blockchain)
        .or(create_transaction)
        .or(mine_block)
        .or(get_balance)
        .or(connect_user)
        .or(disconnect_user)
        .or(get_connections)
        .or(get_stats)
        .or(create_wallet)
        .or(get_transaction_history)
        .or(get_wallet_info)
        .or(send_tip)
        .with(cors);
    
    println!("🔗 Blockchain RPC Server running on http://localhost:3030");
    println!("Endpoints:");
    println!("  GET  /blockchain - View full blockchain");
    println!("  POST /rpc/transaction - Create transaction");
    println!("  POST /rpc/mine - Mine pending transactions");
    println!("  GET  /rpc/balance/{{address}} - Get balance");
    println!("  POST /rpc/connect - Connect to network");
    println!("  POST /rpc/disconnect - Disconnect from network");
    println!("  GET  /rpc/connections - View active connections");
    println!("  GET  /rpc/stats - Network statistics");
    println!("  POST /rpc/create-wallet - Create a new wallet (signup)");
    println!("  GET  /rpc/history/{{address}} - Get transaction history for an address");
    println!("  GET  /rpc/wallet-info/{{address}} - Get detailed wallet info");
    println!("  POST /rpc/tip - Send a tip transaction");
    
    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}