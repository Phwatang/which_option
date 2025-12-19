use core::f64;
use statrs::distribution::{Continuous, ContinuousCDF, Normal};

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
/// A prediction of the future result of a stock price.
/// All member variables should not be negative.
pub struct Movement {
    /// Predicted stock price after the movement
    pub stock: f64,
    /// Time frame/length of the price movement
    pub time: f64,
}
impl Movement {
    /// Updates the Environment and Contract struct given such that they reflect
    /// what happens at the prediction end date. So the only things that are overwritten are:
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

/// The minimum threshold for the exit price in the ROI calculation such that the exit price is rounded down to 0
const ROI_FLOOR_THRESHOLD: f64 = 0.00001;

pub trait BlackScholesROI: BlackScholes {
    /// Returns the ROI from purchasing the option imediately in the given environment and then selling at the predicted endpoint
    fn roi(start_env: &Environment, end_env: &Environment, contract: &Contract, predict: &Movement) -> f64 {
        let start_env = start_env.clone();
        let start_con = contract.clone();
        let (end_env, end_con) = predict.apply(end_env.clone(), contract.clone());
        let mut entry = Self::bsm_price(&start_env, &start_con);
        let mut exit = Self::bsm_price(&end_env, &end_con);
        entry = f64::max(ROI_FLOOR_THRESHOLD, entry);
        if exit <= ROI_FLOOR_THRESHOLD {
            exit = 0.0
        }
        let roi = exit / entry;
        return roi;
    }
    /// Compute first partial derivative of ROI with respect to the strike price of the chosen option
    fn roi_k(start_env: &Environment, end_env: &Environment, contract: &Contract, predict: &Movement) -> f64 {
        let start_env = start_env.clone();
        let start_con = contract.clone();
        let (end_env, end_con) = predict.apply(end_env.clone(), contract.clone());
        let mut entry = Self::bsm_price(&start_env, &start_con);
        let mut exit = Self::bsm_price(&end_env, &end_con);
        entry = f64::max(ROI_FLOOR_THRESHOLD, entry);
        if exit <= ROI_FLOOR_THRESHOLD {
            exit = 0.0
        }
        let entry_k = Self::bsm_price_k(&start_env, &start_con);
        let exit_k = Self::bsm_price_k(&end_env, &end_con);
        // Using quotient rule...
        let roi_k = (entry*exit_k - exit*entry_k) / entry.powi(2);
        return roi_k
    }

    #[allow(dead_code)]
    /// Compute first partial derivative of ROI with respect to the expiry time of the chosen call option
    fn roi_t(start_env: &Environment, end_env: &Environment, contract: &Contract, predict: &Movement) -> f64 {
        let start_env = start_env.clone();
        let start_con = contract.clone();
        let (end_env, end_con) = predict.apply(end_env.clone(), contract.clone());
        let mut entry = Self::bsm_price(&start_env, &start_con);
        let mut exit = Self::bsm_price(&end_env, &end_con);
        entry = f64::max(ROI_FLOOR_THRESHOLD, entry);
        if exit <= ROI_FLOOR_THRESHOLD {
            exit = 0.0
        }
        let entry_t = Self::bsm_price_t(&start_env, &start_con);
        let exit_t = Self::bsm_price_t(&end_env, &end_con);
        // Using quotient rule...
        let roi_t = (entry*exit_t - exit*entry_t) / entry.powi(2);
        return roi_t
    }
    /// Computes the contract that generates the highest ROI (using gradient ascent)
    fn find_best_contract(start_env: &Environment, end_env: &Environment, predict: &Movement) -> Contract {
        let start_env = start_env.clone();
        let end_env = end_env.clone();
        let mut answer = Contract {strike: predict.stock, expiry: predict.time + 0.0001};

        for _ in 0..5000 {
            // Time should already be optimal when matching the prediction time
            // So the only parameter left to optimise is option strike.
            let grad = Self::roi_k(&start_env, &end_env, &answer, predict);
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


