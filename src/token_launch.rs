use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub symbol: String,
    pub name: String,
    pub description: String,
    pub creator: String,
    pub total_supply: f64,
    pub circulating_supply: f64,
    pub created_at: u64,
    pub image_url: Option<String>,
    pub website: Option<String>,
    pub twitter: Option<String>,
    pub telegram: Option<String>,
    pub contract_address: String,
    pub is_verified: bool,
    pub market_cap: f64,
    pub price_in_l1: f64,
    pub liquidity_pool: f64,
    pub holders_count: usize,
    pub trade_count: u64,
    pub status: TokenStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokenStatus {
    Launching,    // Just created, building liquidity
    Trading,      // Available for trading
    Graduated,    // Moved to full DEX
    Paused,       // Trading temporarily paused
    Rugpulled,    // Marked as suspicious
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenHolding {
    pub token_symbol: String,
    pub amount: f64,
    pub acquired_at: u64,
    pub average_price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenTrade {
    pub id: String,
    pub token_symbol: String,
    pub trader: String,
    pub trade_type: TradeType,
    pub amount: f64,
    pub price: f64,
    pub l1_amount: f64,
    pub timestamp: u64,
    pub slippage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradeType {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityPool {
    pub token_symbol: String,
    pub token_reserve: f64,
    pub l1_reserve: f64,
    pub k_constant: f64, // x * y = k for AMM
    pub lp_token_supply: f64,
    pub fee_rate: f64, // 0.3% default
}

pub struct TokenLaunchSystem {
    pub tokens: HashMap<String, Token>,
    pub token_holdings: HashMap<String, HashMap<String, TokenHolding>>, // user -> token -> holding
    pub liquidity_pools: HashMap<String, LiquidityPool>,
    pub recent_trades: Vec<TokenTrade>,
    pub launch_fee: f64,
    pub min_liquidity: f64,
    pub graduation_threshold: f64, // Market cap needed to graduate
}

impl TokenLaunchSystem {
    pub fn new() -> Self {
        Self {
            tokens: HashMap::new(),
            token_holdings: HashMap::new(),
            liquidity_pools: HashMap::new(),
            recent_trades: Vec::new(),
            launch_fee: 10.0, // 10 L1 to launch a token
            min_liquidity: 100.0, // Minimum L1 liquidity needed
            graduation_threshold: 50000.0, // 50k L1 market cap to graduate
        }
    }

    pub fn launch_token(&mut self, req: LaunchTokenRequest, creator_balance: f64) -> Result<Token, String> {
        // Validate launch fee
        if creator_balance < self.launch_fee {
            return Err(format!("Insufficient balance. Need {} L1 to launch token", self.launch_fee));
        }

        // Validate token symbol (must be unique)
        if self.tokens.contains_key(&req.symbol) {
            return Err("Token symbol already exists".to_string());
        }

        // Validate inputs
        if req.symbol.len() < 2 || req.symbol.len() > 10 {
            return Err("Token symbol must be 2-10 characters".to_string());
        }

        if req.name.len() < 3 || req.name.len() > 50 {
            return Err("Token name must be 3-50 characters".to_string());
        }

        if req.total_supply < 1000000.0 || req.total_supply > 1000000000000.0 {
            return Err("Total supply must be between 1M and 1T tokens".to_string());
        }

        // Generate contract address
        let contract_address = self.generate_contract_address(&req.symbol, &req.creator);
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        // Create token (clone values to avoid borrow issues)
        let token = Token {
            symbol: req.symbol.clone(),
            name: req.name.clone(),
            description: req.description.clone(),
            creator: req.creator.clone(),
            total_supply: req.total_supply,
            circulating_supply: 0.0,
            created_at: now,
            image_url: req.image_url.clone(),
            website: req.website.clone(),
            twitter: req.twitter.clone(),
            telegram: req.telegram.clone(),
            contract_address: contract_address.clone(),
            is_verified: false,
            market_cap: 0.0,
            price_in_l1: req.initial_price,
            liquidity_pool: 0.0,
            holders_count: 0,
            trade_count: 0,
            status: TokenStatus::Launching,
        };

        // Create initial liquidity pool
        let pool = LiquidityPool {
            token_symbol: req.symbol.clone(),
            token_reserve: req.total_supply * 0.8, // 80% of supply goes to pool
            l1_reserve: req.initial_liquidity,
            k_constant: (req.total_supply * 0.8) * req.initial_liquidity,
            lp_token_supply: ((req.total_supply * 0.8) * req.initial_liquidity).sqrt(),
            fee_rate: 0.003, // 0.3% fee
        };

        // Give creator 20% of tokens
        let creator_tokens = req.total_supply * 0.2;
        self.add_token_holding(&req.creator, &req.symbol, creator_tokens, req.initial_price);

        // Store token and pool
        self.tokens.insert(req.symbol.clone(), token.clone());
        self.liquidity_pools.insert(req.symbol.clone(), pool);

        // Now we can use the cloned values in println!
        println!("🚀 Token launched: {} ({}) by {}", req.name, req.symbol, req.creator);
        println!("📊 Initial supply: {}, Creator allocation: {}", req.total_supply, creator_tokens);

        Ok(token)
    }

    pub fn buy_token(&mut self, req: BuyTokenRequest, buyer_balance: f64) -> Result<TokenTrade, String> {
        // Check if token exists
        let token = self.tokens.get_mut(&req.token_symbol)
            .ok_or("Token not found")?;

        // Check if pool exists
        let pool = self.liquidity_pools.get_mut(&req.token_symbol)
            .ok_or("Liquidity pool not found")?;

        // Check buyer balance
        if buyer_balance < req.l1_amount {
            return Err("Insufficient L1 balance".to_string());
        }

        // Calculate tokens to receive using AMM formula
        // tokens_out = (token_reserve * l1_in) / (l1_reserve + l1_in)
        let l1_after_fee = req.l1_amount * (1.0 - pool.fee_rate);
        let tokens_out = (pool.token_reserve * l1_after_fee) / (pool.l1_reserve + l1_after_fee);
        
        // Check slippage
        let expected_price = req.l1_amount / tokens_out;
        let current_price = pool.l1_reserve / pool.token_reserve;
        let slippage = ((expected_price - current_price) / current_price * 100.0).abs();
        
        if slippage > req.max_slippage {
            return Err(format!("Slippage too high: {:.2}% (max: {:.2}%)", slippage, req.max_slippage));
        }

        // Update pool reserves
        pool.l1_reserve += req.l1_amount;
        pool.token_reserve -= tokens_out;

        // Update token stats
        token.circulating_supply += tokens_out;
        token.price_in_l1 = pool.l1_reserve / pool.token_reserve;
        token.market_cap = token.circulating_supply * token.price_in_l1;
        token.liquidity_pool = pool.l1_reserve;
        token.trade_count += 1;

        // Add tokens to buyer
        self.add_token_holding(&req.buyer, &req.token_symbol, tokens_out, expected_price);

        // Create trade record
        let trade = TokenTrade {
            id: format!("trade_{}_{}", req.token_symbol, SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()),
            token_symbol: req.token_symbol.clone(),
            trader: req.buyer.clone(),
            trade_type: TradeType::Buy,
            amount: tokens_out,
            price: expected_price,
            l1_amount: req.l1_amount,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            slippage,
        };

        self.recent_trades.push(trade.clone());
        self.update_token_status(&req.token_symbol);

        println!("💰 Token purchase: {} bought {:.2} {} for {:.2} L1", 
                 req.buyer, tokens_out, req.token_symbol, req.l1_amount);

        Ok(trade)
    }

    pub fn sell_token(&mut self, req: SellTokenRequest) -> Result<TokenTrade, String> {
        // Check if user has enough tokens
        let user_holdings = self.token_holdings.get_mut(&req.seller)
            .ok_or("No token holdings found")?;
        
        let holding = user_holdings.get_mut(&req.token_symbol)
            .ok_or("You don't own this token")?;

        if holding.amount < req.token_amount {
            return Err(format!("Insufficient tokens. You have: {}, trying to sell: {}", 
                             holding.amount, req.token_amount));
        }

        // Get token and pool
        let token = self.tokens.get_mut(&req.token_symbol)
            .ok_or("Token not found")?;
        
        let pool = self.liquidity_pools.get_mut(&req.token_symbol)
            .ok_or("Liquidity pool not found")?;

        // Calculate L1 to receive using AMM formula
        // l1_out = (l1_reserve * tokens_in) / (token_reserve + tokens_in)
        let l1_out_before_fee = (pool.l1_reserve * req.token_amount) / (pool.token_reserve + req.token_amount);
        let l1_out = l1_out_before_fee * (1.0 - pool.fee_rate);

        // Check slippage
        let expected_price = l1_out / req.token_amount;
        let current_price = pool.l1_reserve / pool.token_reserve;
        let slippage = ((current_price - expected_price) / current_price * 100.0).abs();

        if slippage > req.max_slippage {
            return Err(format!("Slippage too high: {:.2}% (max: {:.2}%)", slippage, req.max_slippage));
        }

        // Update pool reserves
        pool.l1_reserve -= l1_out;
        pool.token_reserve += req.token_amount;

        // Update token stats
        token.circulating_supply -= req.token_amount;
        token.price_in_l1 = pool.l1_reserve / pool.token_reserve;
        token.market_cap = token.circulating_supply * token.price_in_l1;
        token.liquidity_pool = pool.l1_reserve;
        token.trade_count += 1;

        // Remove tokens from seller
        holding.amount -= req.token_amount;
        if holding.amount <= 0.0 {
            user_holdings.remove(&req.token_symbol);
        }

        // Create trade record
        let trade = TokenTrade {
            id: format!("trade_{}_{}", req.token_symbol, SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()),
            token_symbol: req.token_symbol.clone(),
            trader: req.seller.clone(),
            trade_type: TradeType::Sell,
            amount: req.token_amount,
            price: expected_price,
            l1_amount: l1_out,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            slippage,
        };

        self.recent_trades.push(trade.clone());
        self.update_token_status(&req.token_symbol);

        println!("💸 Token sale: {} sold {:.2} {} for {:.2} L1", 
                 req.seller, req.token_amount, req.token_symbol, l1_out);

        Ok(trade)
    }

    fn add_token_holding(&mut self, user: &str, token_symbol: &str, amount: f64, price: f64) {
        let user_holdings = self.token_holdings.entry(user.to_string()).or_insert_with(HashMap::new);
        
        if let Some(existing) = user_holdings.get_mut(token_symbol) {
            // Update average price
            let total_value = (existing.amount * existing.average_price) + (amount * price);
            existing.amount += amount;
            existing.average_price = total_value / existing.amount;
        } else {
            // New holding
            user_holdings.insert(token_symbol.to_string(), TokenHolding {
                token_symbol: token_symbol.to_string(),
                amount,
                acquired_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                average_price: price,
            });
            
            // Update holders count
            if let Some(token) = self.tokens.get_mut(token_symbol) {
                token.holders_count += 1;
            }
        }
    }

    fn update_token_status(&mut self, token_symbol: &str) {
        if let Some(token) = self.tokens.get_mut(token_symbol) {
            match token.status {
                TokenStatus::Launching => {
                    if token.market_cap >= self.graduation_threshold {
                        token.status = TokenStatus::Graduated;
                        println!("🎓 Token {} has graduated to full DEX!", token_symbol);
                    } else if token.liquidity_pool >= self.min_liquidity {
                        token.status = TokenStatus::Trading;
                        println!("📈 Token {} is now actively trading!", token_symbol);
                    }
                },
                _ => {}
            }
        }
    }

    fn generate_contract_address(&self, symbol: &str, creator: &str) -> String {
        let input = format!("{}{}{}",
            symbol,
            creator,
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()
        );
        let mut hasher = Sha256::new();
        hasher.update(input);
        format!("token_{:x}", hasher.finalize())[0..20].to_string()
    }

    pub fn get_token_info(&self, symbol: &str) -> Option<&Token> {
        self.tokens.get(symbol)
    }

    pub fn get_user_holdings(&self, user: &str) -> Option<&HashMap<String, TokenHolding>> {
        self.token_holdings.get(user)
    }

    pub fn get_trending_tokens(&self, limit: usize) -> Vec<&Token> {
        let mut tokens: Vec<&Token> = self.tokens.values().collect();
        tokens.sort_by(|a, b| b.trade_count.cmp(&a.trade_count));
        tokens.into_iter().take(limit).collect()
    }

    pub fn get_recent_trades(&self, limit: usize) -> Vec<&TokenTrade> {
        self.recent_trades.iter().rev().take(limit).collect()
    }

    pub fn get_all_tokens(&self) -> Vec<&Token> {
        self.tokens.values().collect()
    }
}

// Request structures
#[derive(Deserialize)]
pub struct LaunchTokenRequest {
    pub symbol: String,
    pub name: String,
    pub description: String,
    pub creator: String,
    pub total_supply: f64,
    pub initial_price: f64,
    pub initial_liquidity: f64,
    pub image_url: Option<String>,
    pub website: Option<String>,
    pub twitter: Option<String>,
    pub telegram: Option<String>,
}

#[derive(Deserialize)]
pub struct BuyTokenRequest {
    pub token_symbol: String,
    pub buyer: String,
    pub l1_amount: f64,
    pub max_slippage: f64, // percentage
}

#[derive(Deserialize)]
pub struct SellTokenRequest {
    pub token_symbol: String,
    pub seller: String,
    pub token_amount: f64,
    pub max_slippage: f64, // percentage
}

// Response structures
#[derive(Serialize)]
pub struct TokenListResponse {
    pub tokens: Vec<Token>,
    pub total_count: usize,
}

#[derive(Serialize)]
pub struct UserPortfolioResponse {
    pub user: String,
    pub holdings: Vec<TokenHolding>,
    pub total_value_l1: f64,
    pub total_pnl: f64,
}

#[derive(Serialize)]
pub struct TokenStatsResponse {
    pub token: Token,
    pub recent_trades: Vec<TokenTrade>,
    pub price_chart: Vec<PricePoint>,
}

#[derive(Serialize)]
pub struct PricePoint {
    pub timestamp: u64,
    pub price: f64,
    pub volume: f64,
}