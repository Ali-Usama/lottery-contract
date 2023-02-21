use anchor_lang::prelude::*;

declare_id!("GPyMAqy5mDHXHmpvFEN7sJyhLu8upoYr57ADhzeeoJ8s");

#[program]
pub mod liquidity_pool {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
