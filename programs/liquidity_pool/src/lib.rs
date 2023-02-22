use anchor_lang::prelude::*;
use std::{convert::TryInto};
use arrayref::array_ref;
use anchor_lang::solana_program::sysvar;

declare_id!("GPyMAqy5mDHXHmpvFEN7sJyhLu8upoYr57ADhzeeoJ8s");


#[program]
pub mod liquidity_pool {
    use anchor_lang::solana_program::entrypoint::ProgramResult;
    use anchor_lang::solana_program::program::{invoke, invoke_signed};
    use anchor_lang::solana_program::{system_instruction};
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
            return Err(ProgramError::InsufficientFunds);
        }
        if lottery.participants.contains(participant) {
            return Err(ProgramError::AccountAlreadyInitialized);
        }
        lottery.participants.push(**participant);
        lottery.total_amount += amount;
        Ok(())
    }

    pub fn pick_winner(ctx: Context<PickWinner>) -> ProgramResult {
        let lottery = &mut ctx.accounts.lottery;
        // Ensure there are participants in the lottery
        if lottery.participants.is_empty() {
            return Err(ProgramError::InvalidArgument);
        }

        // Generate a random index to select the winner
        let recent_slothashes = &ctx.accounts.recent_slothashes;
        let data = recent_slothashes.data.borrow();
        let most_recent = array_ref![data, 12, 8];

        let clock = Clock::get()?;
        // seed for the random number is a combination of the slot_hash - timestamp
        let seed = u64::from_le_bytes(*most_recent).saturating_sub(clock.unix_timestamp as u64);

        let index = (seed % lottery.participants.len() as u64) as usize;
        // Get the winner's address
        let winner_address = ctx.accounts.lottery.participants[index];

        // Calculate the winner and owner prizes
        let winner_prize = (ctx.accounts.lottery.total_amount * ctx.accounts.lottery.winner_prize)
            / 100;
        let owner_prize =
            (ctx.accounts.lottery.total_amount * ctx.accounts.lottery.owner_prize) / 100;

        // Transfer the winner's prize to their account
        let ix = system_instruction::transfer(
            &ctx.accounts.lottery.owner.key(),
            &ctx.accounts.winner.key,
            winner_prize,
        );
        let signers = &[&ctx.accounts.owner];
        let program = ctx.accounts.system_program.to_account_info().clone();
        let (address, _) = Pubkey::find_program_address(&[b"winner_prize".as_ref()], ctx.program_id);
        let accounts = &[ctx.accounts.lottery.to_account_info(), ctx.accounts.winner.to_account_info(), program];
        invoke_signed(&ix, accounts, &[&[b"winner_prize".as_ref(), &[0u8]]])?;

        // Transfer the owner's prize to their account
        let ix = system_instruction::transfer(
            &ctx.accounts.lottery.owner.key(),
            &ctx.accounts.owner.key,
            owner_prize,
        );
        invoke(&ix, accounts)?;

        // Set the winner and clear the participants list
        ctx.accounts.lottery.winner = Some(winner_address);
        ctx.accounts.lottery.participants.clear();

        Ok(())
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

#[derive(Accounts)]
pub struct NewPool<'info> {
    #[account(init, payer = owner, space = 8 + 32 + 64)]
    pub lottery: Account<'info, Pool>,

    /// CHECK: account is test account
    #[account(mut)]
    pub owner: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct JoinPool<'info> {
    #[account(mut)]
    pub lottery: Account<'info, Pool>,

    /// CHECK: test participants
    #[account(signer)]
    pub participant: AccountInfo<'info>,

}

#[derive(Accounts)]
pub struct PickWinner<'info> {
    #[account(mut)]
    pub lottery: Account<'info, Pool>,

    /// CHECK: test owner
    #[account(signer)]
    pub owner: AccountInfo<'info>,

    /// CHECK: winner account
    #[account(mut)]
    pub winner: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    /// CHECK: account constraints checked in account trait
    #[account(address = sysvar::slot_hashes::id())]
    recent_slothashes: UncheckedAccount<'info>,
}






