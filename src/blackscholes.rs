use core::f64;
use statrs::distribution::{Continuous, ContinuousCDF, Normal};
use rust_decimal::{Decimal, RoundingStrategy, dec};
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};

#[derive(Debug, Default, Clone)]
/// Environmental variables that affect an option's price. 
/// All member variables should not be negative.
pub struct Environment {
    /// Current stock price
    pub stock: f64,
    /// Constant riskfree rate
    pub risk_free: f64,
    /// Constant stock price volatility. (E.g 4% would be 0.04).
    pub vol: f64,
    /// Constant dividend yield of the stock. (E.g 16% would be 0.16).
    pub div_yield: f64,
}

#[derive(Debug, Default, Clone)]
/// Variables specific to an option contract that affects it's price. 
/// All member variables should not be negative.
pub struct Contract {
    /// Strike price of the option
    pub strike: f64,
    /// Time left to expiry of the option
    pub expiry: f64,
    
}

#[derive(Debug, Default, Clone)]
/// A potential future result of a stock price.
/// All member variables should not be negative.
pub struct Movement {
    /// Final stock price after the movement
    pub stock: f64,
    /// Time frame/length of the price movement
    pub time: f64,
}
impl Movement {
    /// Updates the Environment and Contract struct given such that they reflect
    /// what happens at the movement end duration. So the only things that are overwritten are:
    ///  - environ.stock
    ///  - con.expiry
    /// 
    /// Contract expiry of the output is clamped to always be non-negative.
    pub fn apply(&self, environ: Environment, con: Contract) -> (Environment, Contract) {
        return (
            Environment {
                stock: self.stock,
                ..environ
            },
            Contract {
                expiry: f64::max(con.expiry - self.time, 0.0),
                ..con
            }
        )
    }
}


pub trait BlackScholes {
    fn bsm_price(env: &Environment, contract: &Contract) -> f64;
    #[allow(non_snake_case)]
    fn bsm_price_k(env: &Environment, contract: &Contract) -> f64;
    #[allow(non_snake_case)]
    fn bsm_price_t(env: &Environment, contract: &Contract) -> f64;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Call;
impl BlackScholes for Call {
    /// Returns the price of a call option under the black-scholes pricing model.
    /// 
    /// NaN is return upon unexpected/erroneous arguments. E.g negative volatility.
    #[allow(non_snake_case)]
    fn bsm_price(env: &Environment, contract: &Contract) -> f64 {
        let stock = env.stock;
        let risk_free = env.risk_free;
        let div_yield = env.div_yield;
        let vol = env.vol;
        let strike = contract.strike;
        let time_left = contract.expiry;
        let d_1 = (f64::ln(stock / strike) + time_left * (risk_free - div_yield + (vol.powi(2) / 2.0))) / (vol * time_left.sqrt());
        let d_2 = d_1 - vol * time_left.sqrt();
        let std_normal_dist = Normal::new(0.0, 1.0).unwrap();
        let stock_PV = stock * f64::exp(-div_yield * time_left);
        let strike_PV = strike * f64::exp(-risk_free * time_left);
        let call_price = std_normal_dist.cdf(d_1) * stock_PV - std_normal_dist.cdf(d_2) * strike_PV;
        return call_price;
    }
    /// Returns the partial derivative of a call option with respect to the strike price under the black-scholes pricing model.
    /// 
    /// NaN is return upon unexpected/erroneous arguments. E.g negative volatility.
    fn bsm_price_k(env: &Environment, contract: &Contract) -> f64 {
        let stock = env.stock;
        let risk_free = env.risk_free;
        let div_yield = env.div_yield;
        let vol = env.vol;
        let strike = contract.strike;
        let time_left = contract.expiry;
        let d_1 = (f64::ln(stock / strike) + time_left * (risk_free - div_yield + (vol.powi(2) / 2.0))) / (vol * time_left.sqrt());
        let d_2 = d_1 - vol * time_left.sqrt();
        let std_normal_dist = Normal::new(0.0, 1.0).unwrap();
        let dual_delta = -f64::exp(-risk_free * time_left) * std_normal_dist.cdf(d_2);
        return dual_delta;
    }
    /// Returns the partial derivative of a call option with respect to time under the black-scholes pricing model.
    /// 
    /// NaN is return upon unexpected/erroneous arguments. E.g negative volatility.
    fn bsm_price_t(env: &Environment, contract: &Contract) -> f64 {
        let stock = env.stock;
        let risk_free = env.risk_free;
        let div_yield = env.div_yield;
        let vol = env.vol;
        let strike = contract.strike;
        let time_left = contract.expiry;
        let d_1 = (f64::ln(stock / strike) + time_left * (risk_free - div_yield + (vol.powi(2) / 2.0))) / (vol * time_left.sqrt());
        let d_2 = d_1 - vol * time_left.sqrt();
        let std_normal_dist = Normal::new(0.0, 1.0).unwrap();
        let a = ((stock * vol * f64::exp(- div_yield*time_left))/(2.0*time_left.sqrt())) * std_normal_dist.pdf(d_1);
        let b = risk_free * strike * f64::exp(-risk_free*time_left) * std_normal_dist.cdf(d_2);
        let c = -div_yield * stock * f64::exp(-div_yield*time_left) * std_normal_dist.cdf(d_1);
        let theta = a + b + c;
        return theta;
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Put;
impl BlackScholes for Put {
    /// Returns the price of a put option under the black-scholes pricing model.
    /// 
    /// NaN is return upon unexpected/erroneous arguments. E.g negative volatility.
    #[allow(non_snake_case)]
    fn bsm_price(env: &Environment, contract: &Contract) -> f64 {
        let stock = env.stock;
        let risk_free = env.risk_free;
        let div_yield = env.div_yield;
        let vol = env.vol;
        let strike = contract.strike;
        let time_left = contract.expiry;
        let d_1 = (f64::ln(stock / strike) + time_left * (risk_free - div_yield + (vol.powi(2) / 2.0))) / (vol * time_left.sqrt());
        let d_2 = d_1 - vol * time_left.sqrt();
        let std_normal_dist = Normal::new(0.0, 1.0).unwrap();
        let stock_PV = stock * f64::exp(-div_yield * time_left);
        let strike_PV = strike * f64::exp(-risk_free * time_left);
        let put_price = std_normal_dist.cdf(-d_2) * strike_PV - std_normal_dist.cdf(-d_1) * stock_PV;
        return put_price;
    }
    /// Returns the partial derivative of a put option with respect to the strike price under the black-scholes pricing model.
    /// 
    /// NaN is return upon unexpected/erroneous arguments. E.g negative volatility.
    fn bsm_price_k(env: &Environment, contract: &Contract) -> f64 {
        let stock = env.stock;
        let risk_free = env.risk_free;
        let div_yield = env.div_yield;
        let vol = env.vol;
        let strike = contract.strike;
        let time_left = contract.expiry;
        let d_1 = (f64::ln(stock / strike) + time_left * (risk_free - div_yield + (vol.powi(2) / 2.0))) / (vol * time_left.sqrt());
        let d_2 = d_1 - vol * time_left.sqrt();
        let std_normal_dist = Normal::new(0.0, 1.0).unwrap();
        let dual_delta = f64::exp(-risk_free * time_left) * (1.0 - std_normal_dist.cdf(d_2));
        return dual_delta;
    }
    /// Returns the partial derivative of a put option with respect to time under the black-scholes pricing model.
    /// 
    /// NaN is return upon unexpected/erroneous arguments. E.g negative volatility.
    fn bsm_price_t(env: &Environment, contract: &Contract) -> f64 {
        let stock = env.stock;
        let risk_free = env.risk_free;
        let div_yield = env.div_yield;
        let vol = env.vol;
        let strike = contract.strike;
        let time_left = contract.expiry;
        let d_1 = (f64::ln(stock / strike) + time_left * (risk_free - div_yield + (vol.powi(2) / 2.0))) / (vol * time_left.sqrt());
        let d_2 = d_1 - vol * time_left.sqrt();
        let std_normal_dist = Normal::new(0.0, 1.0).unwrap();
        let a = ((stock * vol * f64::exp(-div_yield*time_left))/(2.0*time_left.sqrt())) * std_normal_dist.pdf(d_1);
        let b = risk_free * strike * f64::exp(-risk_free*time_left) * std_normal_dist.pdf(-d_2);
        let c = -div_yield * stock * f64::exp(-div_yield*time_left) * std_normal_dist.pdf(-d_1);
        let theta = a + b + c;
        return theta;
    }
}

/// Rounds the a given floating point price to what would be the real-world buy price
/// This means rounding upwards to 2 d.p. with a price minimum of 0.01.
fn buy_rounding(num: f64) -> Decimal {
    // 0.001 to ensure >0.01 is guaranteed after rounding up
    Decimal::from_f64(num.max(0.001))
        .unwrap_or(dec!(0.01))
        .round_dp_with_strategy(2, RoundingStrategy::AwayFromZero)
}

/// Rounds the a given floating point price to what would be the real-world buy price
/// This means rounding downwards to 2 d.p. with a price minimum of 0.00.
fn sell_rounding(num: f64) -> Decimal {
    Decimal::from_f64(num.max(0.0))
        .unwrap_or(dec!(0.00))
        .round_dp_with_strategy(2, RoundingStrategy::ToZero)
}

pub trait BlackScholesRounded: BlackScholes {
    /// Returns the real world buying price of the option.
    /// This basically means rounding up to the nearest cent
    fn bsm_price_buy(env: &Environment, contract: &Contract) -> Decimal {
        buy_rounding(Self::bsm_price(env, contract))
    }
    /// Returns the real world selling price of the option.
    /// This basically means rounding down to the nearest cent
    #[allow(dead_code)]
    fn bsm_price_sell(env: &Environment, contract: &Contract) -> Decimal {
        sell_rounding(Self::bsm_price(env, contract))
    }
}
impl BlackScholesRounded for Call {}
impl BlackScholesRounded for Put {}

/// The minimum threshold for the exit price in the ROI calculation such that the exit price is rounded down to 0
const ROI_FLOOR_THRESHOLD: f64 = 0.00001;

pub trait BlackScholesROI: BlackScholes {
    /// Returns the (buying_price, selling_price) from purchasing the option imediately in the given environment and then selling at the movement endpoint
    fn buy_sell_prices(start_env: &Environment, end_env: &Environment, contract: &Contract, movement: &Movement) -> (f64, f64) {
        let start_env = start_env.clone();
        let start_con = contract.clone();
        let (end_env, end_con) = movement.apply(end_env.clone(), contract.clone());
        let entry = Self::bsm_price(&start_env, &start_con);
        let exit = Self::bsm_price(&end_env, &end_con);
        return (entry, exit);
    }

    /// Returns the ROI from purchasing the option imediately in the given environment and then selling at the movement endpoint
    #[allow(dead_code)]
    fn roi(start_env: &Environment, end_env: &Environment, contract: &Contract, movement: &Movement) -> f64 {
        let (mut entry, mut exit) = Self::buy_sell_prices(start_env, end_env, contract, movement);
        // Ensure non-zero division
        entry = f64::max(ROI_FLOOR_THRESHOLD, entry);
        if exit <= ROI_FLOOR_THRESHOLD {
            exit = 0.0
        }
        let roi = exit / entry;
        return roi;
    }

    /// Compute first partial derivative of ROI with respect to the strike price of the chosen option
    fn roi_k(start_env: &Environment, end_env: &Environment, contract: &Contract, movement: &Movement) -> f64 {
        let (mut entry, mut exit) = Self::buy_sell_prices(start_env, end_env, contract, movement);
        entry = f64::max(ROI_FLOOR_THRESHOLD, entry);
        if exit <= ROI_FLOOR_THRESHOLD {
            exit = 0.0
        }
        let (updated_end_env, end_contract) = movement.apply(end_env.clone(), contract.clone());
        let entry_k = Self::bsm_price_k(&start_env, contract);
        let exit_k = Self::bsm_price_k(&updated_end_env, &end_contract);
        // Using quotient rule...
        let roi_k = (entry*exit_k - exit*entry_k) / entry.powi(2);
        return roi_k
    }

    #[allow(dead_code)]
    /// Compute first partial derivative of ROI with respect to the expiry time of the chosen call option
    fn roi_t(start_env: &Environment, end_env: &Environment, contract: &Contract, movement: &Movement) -> f64 {
        let (mut entry, mut exit) = Self::buy_sell_prices(start_env, end_env, contract, movement);
        entry = f64::max(ROI_FLOOR_THRESHOLD, entry);
        if exit <= ROI_FLOOR_THRESHOLD {
            exit = 0.0
        }
        let (updated_end_env, end_contract) = movement.apply(end_env.clone(), contract.clone());
        let entry_t = Self::bsm_price_t(&start_env, contract);
        let exit_t = Self::bsm_price_t(&updated_end_env, &end_contract);
        // Using quotient rule...
        let roi_t = (entry*exit_t - exit*entry_t) / entry.powi(2);
        return roi_t
    }

    /// Computes the contract that generates the highest ROI (using gradient ascent)
    fn find_best_contract(start_env: &Environment, end_env: &Environment, movement: &Movement) -> Contract {
        let start_env = start_env.clone();
        let end_env = end_env.clone();
        let mut answer = Contract {strike: movement.stock, expiry: movement.time + 0.0001};

        for _ in 0..5000 {
            // Optimal option expiry is automatically done when matching the price movement duration
            // So the only parameter left to optimise is option strike.
            // Todo: Pretty confident on this fact but need to find formal proof later
            let grad = Self::roi_k(&start_env, &end_env, &answer, movement);
            let mut step_mult: f64 = 0.1;
            let step_max = 0.01;
            // Adjust step multiplier to ensure step magnitude does not exceed step_max
            step_mult = step_mult.min(step_max / grad.abs());
            answer.strike += step_mult * grad;
        }

        return answer;
    }
}
impl BlackScholesROI for Call {}
impl BlackScholesROI for Put {}

pub trait BlackScholesROIRounded: BlackScholesROI + BlackScholesRounded {
    /// Returns the real-world (buying_price, selling_price) from purchasing the option imediately in the given environment and then selling at the movement endpoint
    fn buy_sell_prices_practical (start_env: &Environment, end_env: &Environment, contract: &Contract, movement: &Movement) -> (Decimal, Decimal) {
        let (entry, exit) = Self::buy_sell_prices(start_env, end_env, contract, movement);
        return (buy_rounding(entry), sell_rounding(exit));
    }

    /// Returns the practical/real-world ROI from purchasing the option imediately in the given environment and then selling at the movement endpoint.
    /// This means the theoretical purchase price is rounded up and the theoretical selling price is rounded down
    fn roi_practical(start_env: &Environment, end_env: &Environment, contract: &Contract, movement: &Movement) -> f64 {
        // buy price is guaranteed to be >0.01
        let (entry, exit) = Self::buy_sell_prices_practical(start_env, end_env, contract, movement);
        let roi = exit / entry;
        return roi.to_f64().unwrap();
    }
}
impl BlackScholesROIRounded for Call {}
impl BlackScholesROIRounded for Put {}
