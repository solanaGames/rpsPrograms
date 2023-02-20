use anchor_lang::prelude::*;
use anchor_spl::token::*;

use rps::cpi::accounts::CreatePlayerInfo;
use rps::cpi::create_player_info;
use rps::program::Rps;
use rps::{self, PlayerInfo};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

const POOL_SEED: &'static [u8; 4] = b"pool";
const AUTHORITY_SEED: &'static [u8; 9] = b"authority";
const MINT_SEED: &'static [u8; 4] = b"mint";

#[program]
pub mod blp {
    use anchor_spl::token::{
        initialize_mint2, spl_token::instruction::mint_to_checked, InitializeMint2,
    };

    use super::*;

    pub fn create_pool(ctx: Context<CreatePool>, seed: u64) -> Result<()> {
        ctx.accounts.pool.seed = seed;
        ctx.accounts.pool.authority = ctx.accounts.pool_authority.key();
        ctx.accounts.pool.bot_authority = ctx.accounts.bot_authority.key();
        ctx.accounts.pool.lp_token_mint = ctx.accounts.lp_token_mint.key();

        // creating lp token mint
        initialize_mint2(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                InitializeMint2 {
                    mint: ctx.accounts.lp_token_mint.to_account_info(),
                },
            ),
            9, // because sol got 9
            &ctx.accounts.pool_authority.key(),
            Some(&ctx.accounts.pool_authority.key()),
        )?;

        // registering the pool to be able to play
        create_player_info(CpiContext::new(
            ctx.accounts.rps_program.to_account_info(),
            CreatePlayerInfo {
                owner: ctx.accounts.pool_authority.to_account_info(),
                player_info: ctx.accounts.pool_authority_player_info.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
            },
        ))?;

        Ok(())
    }

    // to change stuff for the bot
    // pub fn update_pool(ctx: Context<CreatePool>) -> Result<()> {
    //     Ok(())
    // }

    pub fn deposit(ctx: Context<Deposit>, deposit_amount: u64) -> Result<()> {
        let deposits = (ctx.accounts.pool_authority.to_account_info().lamports() as u128)
            .checked_add(2 * ctx.accounts.pool_authority_player_info.amount_in_games as u128) // doubled because have to assume won all outstanding games
            .unwrap();
        let lp_total = ctx.accounts.lp_token_mint.supply as u128;
        let mint_amount_u128: u128;
        if deposits != 0 {
            mint_amount_u128 = (deposit_amount as u128)
                .checked_mul(lp_total)
                .unwrap()
                .checked_div(deposits)
                .unwrap();
        } else if lp_total == 0 {
            // first deposit so have rate be 1
            mint_amount_u128 = (deposit_amount as u128);
        } else {
            panic!("pool blew up no deposits allowed");
        }

        let mint_amount = u64::try_from(mint_amount_u128).unwrap();
        mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.lp_token_mint.to_account_info(),
                    to: ctx.accounts.user_lp_token_account.to_account_info(),
                    authority: ctx.accounts.pool_authority.to_account_info(),
                },
                &[&[
                    AUTHORITY_SEED.as_ref(),
                    ctx.accounts.pool.key().as_ref(),
                    &[*ctx.bumps.get("pool_authority").unwrap()],
                ]],
            ),
            mint_amount,
        )?;
        anchor_lang::system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: ctx.accounts.user_authority.to_account_info(),
                    to: ctx.accounts.pool_authority.to_account_info(),
                },
            ),
            deposit_amount,
        )?;

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, withdraw_amount:u64) -> Result<()> {
        let deposits = (ctx.accounts.pool_authority.to_account_info().lamports() as u128);
        // not considering money in games because must assume lost all
        let lp_total = ctx.accounts.lp_token_mint.supply as u128;
        let sol_withdraw_amount_u128: u128;
        if lp_total != 0 && deposits != 0 {
            sol_withdraw_amount_u128 = (withdraw_amount as u128)
                .checked_mul(deposits)
                .unwrap()
                .checked_div(lp_total)
                .unwrap();
        } else if deposits == 0 {
            panic!("no money to withdraw, all stuck in games or pool blew up");
        } else {
            panic!("no lp tokens outstanding");
        }
        let sol_withdraw_amount = u64::try_from(sol_withdraw_amount_u128).unwrap();

        // let deposits = (ctx.accounts.pool_authority.to_account_info().lamports() as u128);
        // let lp_total = ctx.accounts.lp_token_mint.supply as u128;

        Ok(())
    }

    pub fn bot_play(ctx: Context<BotPlay>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(pool_seed: u64)]
pub struct CreatePool<'info> {
    #[account(
        init,
        seeds = [POOL_SEED.as_ref(), &pool_seed.to_le_bytes()],
        bump,
        payer = bot_authority,
        space = Pool::space()
    )]
    pub pool: Account<'info, Pool>,

    #[account(mut, seeds = [AUTHORITY_SEED.as_ref(), pool.key().as_ref()], bump)]
    pub pool_authority: AccountInfo<'info>,

    #[account(mut)]
    pub pool_authority_player_info: Account<'info, PlayerInfo>,

    #[account(
        init,
        seeds = [MINT_SEED.as_ref(), pool.key().as_ref()],
        bump,
        payer = bot_authority,
        space = Mint::LEN,
    )]
    pub lp_token_mint: Account<'info, Mint>,

    #[account(mut)]
    pub bot_authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub rps_program: Program<'info, Rps>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(
        seeds = [POOL_SEED.as_ref(), &pool.seed.to_le_bytes()],
        bump,
    )]
    pub pool: Account<'info, Pool>,

    #[account(mut, seeds = [AUTHORITY_SEED.as_ref(), pool.key().as_ref()], bump)]
    pub pool_authority: AccountInfo<'info>,

    #[account(constraint = pool_authority_player_info.owner.key() == pool_authority.key())]
    pub pool_authority_player_info: Account<'info, PlayerInfo>,

    #[account(
        seeds = [MINT_SEED.as_ref(), pool.key().as_ref()],
        bump,
    )]
    pub lp_token_mint: Account<'info, Mint>,

    // depositing from
    #[account(mut)]
    pub user_authority: Signer<'info>,

    // where to mint lp tokens to
    #[account(
        mut,
        constraint = user_lp_token_account.mint == lp_token_mint.key(),
        constraint = user_lp_token_account.owner == user_authority.key(),
    )]
    pub user_lp_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        seeds = [POOL_SEED.as_ref(), &pool.seed.to_le_bytes()],
        bump,
    )]
    pub pool: Account<'info, Pool>,

    #[account(mut, seeds = [AUTHORITY_SEED.as_ref(), pool.key().as_ref()], bump)]
    pub pool_authority: AccountInfo<'info>,

    #[account(constraint = pool_authority_player_info.owner.key() == pool_authority.key())]
    pub pool_authority_player_info: Account<'info, PlayerInfo>,

    #[account(
        seeds = [MINT_SEED.as_ref(), pool.key().as_ref()],
        bump,
    )]
    pub lp_token_mint: Account<'info, Mint>,

    // depositing from
    #[account(mut)]
    pub user_authority: Signer<'info>,

    // where to mint lp tokens to
    #[account(
        mut,
        constraint = user_lp_token_account.mint == lp_token_mint.key(),
        constraint = user_lp_token_account.owner == user_authority.key(),
    )]
    pub user_lp_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct BotPlay {}

#[account]
#[derive()]
pub struct Pool {
    pub seed: u64,

    // will be native sol only for now so no need for a token account
    pub authority: Pubkey,

    // rps player info for the pool (needed so we can see outstanding wager)
    pub authority_player_info: Pubkey,

    // pubkey for the bot which is allowed to call play for the pool
    pub bot_authority: Pubkey,

    // lp token mint for pool depositors
    pub lp_token_mint: Pubkey,
    // actually i guess this isn't possible
    // // amount which is currently locked in games
    // pub in_games_amount: u64,
}

impl Pool {
    pub fn space() -> usize {
        // idk lmao
        1000
    }
}
