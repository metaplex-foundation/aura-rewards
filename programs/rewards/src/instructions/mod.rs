//! Program instructions

mod claim;
mod close_mining;
mod deposit_mining;
mod distribute_rewards;
mod extend_stake;
mod fill_vault;
mod initialize_mining;
mod initialize_pool;
mod withdraw_mining;

pub use claim::*;
pub use close_mining::*;
pub use deposit_mining::*;
pub use distribute_rewards::*;
pub use extend_stake::*;
pub use fill_vault::*;
pub use initialize_mining::*;
pub use initialize_pool::*;
pub use withdraw_mining::*;
