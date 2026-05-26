#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, Symbol};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Station,     // Address of the gas station owner
    FuelPrice,   // Wholesale price per liter in stablecoin
    TotalPooled, // Accumulated stablecoin amount
    TotalLiters, // Liters secured in this current batch
    Token,       // Stablecoin contract address used for payments
    DriverBal(Address), // Liters pre-purchased per individual driver account
}

#[contract]
pub struct GasoLinxContract;

#[contractimpl]
impl GasoLinxContract {
    /// Initializes the wholesale fuel procurement campaign batch.
    pub fn initialize(env: Env, station: Address, token: Address, price_per_liter: i128) {
        if env.storage().instance().has(&DataKey::Station) {
            panic!("Contract already initialized");
        }
        env.storage().instance().set(&DataKey::Station, &station);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage().instance().set(&DataKey::FuelPrice, &price_per_liter);
        env.storage().instance().set(&DataKey::TotalPooled, &0i128);
        env.storage().instance().set(&DataKey::TotalLiters, &0i128);
    }

    /// Drivers call this to pledge stablecoins to the pool to buy future fuel allocations at a fixed wholesale discount rate.
    pub fn pre_purchase_fuel(env: Env, driver: Address, amount: i128) {
        driver.require_auth();

        let token_addr: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let price_per_liter: i128 = env.storage().instance().get(&DataKey::FuelPrice).unwrap();

        // Calculate exact fractional fuel allocation (liters)
        let purchased_liters = amount / price_per_liter;
        if purchased_liters <= 0 {
            panic!("Deposit amount insufficient to purchase at least one liter");
        }

        // Pull stablecoin funds into this contract escrow from driver wallet
        let client = token::Client::new(&env, &token_addr);
        client.transfer(&driver, &env.current_contract_address(), &amount);

        // Update Global pooled tracker data state
        let current_pooled: i128 = env.storage().instance().get(&DataKey::TotalPooled).unwrap_or(0);
        let current_liters: i128 = env.storage().instance().get(&DataKey::TotalLiters).unwrap_or(0);
        env.storage().instance().set(&DataKey::TotalPooled, &(current_pooled + amount));
        env.storage().instance().set(&DataKey::TotalLiters, &(current_liters + purchased_liters));

        // Update Driver's specific allocation records
        let key = DataKey::DriverBal(driver.clone());
        let current_driver_bal: i128 = env.storage().instance().get(&key).unwrap_or(0);
        env.storage().instance().set(&key, &(current_driver_bal + purchased_liters));
    }

    /// Allows the station operator to unlock the aggregated pooled stablecoins to pay the fuel haulage distributor wholesale.
    pub fn dispatch_wholesale_payout(env: Env) -> i128 {
        let station: Address = env.storage().instance().get(&DataKey::Station).unwrap();
        station.require_auth();

        let token_addr: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let pooled_funds: i128 = env.storage().instance().get(&DataKey::TotalPooled).unwrap();

        if pooled_funds <= 0 {
            panic!("No liquidity pooled to pay out");
        }

        // Transfer funds out from escrow to the verified distribution station operator account
        let client = token::Client::new(&env, &token_addr);
        client.transfer(&env.current_contract_address(), &station, &pooled_funds);

        // Reset tracking pool variables for subsequent delivery cycles
        env.storage().instance().set(&DataKey::TotalPooled, &0i128);

        pooled_funds
    }

    /// Read function to check a driver's locked down fuel volume allocation balance.
    pub fn get_driver_liters(env: Env, driver: Address) -> i128 {
        env.storage().instance().get(&DataKey::DriverBal(driver)).unwrap_or(0)
    }
}

mod test;