use anchor_lang::prelude::*;
use std::convert::TryInto;

declare_id!("GPyMAqy5mDHXHmpvFEN7sJyhLu8upoYr57ADhzeeoJ8s");


#[program]
mod liquidity_pool {
    use anchor_lang::solana_program::entrypoint::ProgramResult;
    use super::*;

    pub fn new_lottery(
        ctx: Context<NewPool>,
        pool_value: u64,
        winner_prize: u64,
        owner_prize: u64,
    ) -> ProgramResult {
        let lottery = &mut ctx.accounts.lottery;
        lottery.owner = *ctx.accounts.owner.to_account_info().key;
        lottery.total_amount = 0;
        lottery.pool_value = pool_value;
        lottery.winner_prize = winner_prize;
        lottery.owner_prize = owner_prize;
        lottery.participants = Vec::new();
        lottery.winner = None;
        Ok(())
    }

    pub fn join_lottery(
        ctx: Context<JoinPool>,
        amount: u64,
    ) -> ProgramResult {
        let lottery = &mut ctx.accounts.lottery;
        let participant = &ctx.accounts.participant.to_account_info().key;
        if amount != lottery.pool_value {
            return Err("Value should be 0.1 SOL".into());
        }
        if lottery.participants.contains(participant) {
            return Err("User already a participant".into());
        }
        lottery.participants.push(**participant);
        lottery.total_amount += amount;
        Ok(())
    }

    pub fn pick_winner(ctx: Context<PickWinner>) -> ProgramResult {
        let lottery = &mut ctx.accounts.lottery;
        if lottery.participants.is_empty() {
            return Err("No participant".into());
        }
        let index = (lottery.random() % lottery.participants.len() as u64) as usize;
        let winner_address = lottery.participants[index];
        let winner_prize = lottery.total_amount * lottery.winner_prize / 100;
        let owner_prize = lottery.total_amount * lottery.owner_prize / 100;
        ctx.accounts
            .winner
            .to_account_info()
            .try_borrow_mut_lamports()?;
        ctx.accounts.owner.try_borrow_mut_lamports()?;
        **ctx.accounts.winner.to_account_info().lamports.borrow_mut() -= winner_prize;
        **ctx.accounts.owner.lamports.borrow_mut() -= owner_prize;
        **ctx.accounts
            .winner
            .to_account_info()
            .lamports
            .borrow_mut() += winner_prize + owner_prize;
        lottery.winner = Some(winner_address);
        lottery.participants.clear();
        Ok(())
    }

    #[derive(Accounts)]
    pub struct NewPool<'info> {
        #[account(init, payer = owner)]
        pub lottery: Account<'info, Pool>,
        pub owner: AccountInfo<'info>,
        pub rent: Sysvar<'info, Rent>,
    }

    #[derive(Accounts)]
    pub struct JoinPool<'info> {
        #[account(mut)]
        pub lottery: Account<'info, Pool>,
        #[account(signer)]
        pub participant: AccountInfo<'info>,
    }

    #[derive(Accounts)]
    pub struct PickWinner<'info> {
        #[account(mut)]
        pub lottery: Account<'info, Pool>,
        #[account(signer)]
        pub owner: AccountInfo<'info>,
        #[account(mut)]
        pub winner: AccountInfo<'info>,
    }
}

#[account]
pub struct Pool {
    pub owner: Pubkey,
    pub total_amount: u64,
    pub pool_value: u64,
    pub winner_prize: u64,
    pub owner_prize: u64,
    pub participants: Vec<Pubkey>,
    pub winner: Option<Pubkey>,
}






