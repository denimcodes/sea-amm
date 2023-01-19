#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
use crate::{assign, id, index_assign, seahorse_util::*};
use anchor_lang::{prelude::*, solana_program};
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use std::{cell::RefCell, rc::Rc};

#[account]
#[derive(Debug)]
pub struct Pool {
    pub bump: u8,
    pub authority: Pubkey,
    pub token_mint_a: Pubkey,
    pub token_mint_b: Pubkey,
    pub token_vault_a: Pubkey,
    pub token_vault_b: Pubkey,
    pub lp_token_mint: Pubkey,
}

impl<'info, 'entrypoint> Pool {
    pub fn load(
        account: &'entrypoint mut Box<Account<'info, Self>>,
        programs_map: &'entrypoint ProgramsMap<'info>,
    ) -> Mutable<LoadedPool<'info, 'entrypoint>> {
        let bump = account.bump;
        let authority = account.authority.clone();
        let token_mint_a = account.token_mint_a.clone();
        let token_mint_b = account.token_mint_b.clone();
        let token_vault_a = account.token_vault_a.clone();
        let token_vault_b = account.token_vault_b.clone();
        let lp_token_mint = account.lp_token_mint.clone();

        Mutable::new(LoadedPool {
            __account__: account,
            __programs__: programs_map,
            bump,
            authority,
            token_mint_a,
            token_mint_b,
            token_vault_a,
            token_vault_b,
            lp_token_mint,
        })
    }

    pub fn store(loaded: Mutable<LoadedPool>) {
        let mut loaded = loaded.borrow_mut();
        let bump = loaded.bump;

        loaded.__account__.bump = bump;

        let authority = loaded.authority.clone();

        loaded.__account__.authority = authority;

        let token_mint_a = loaded.token_mint_a.clone();

        loaded.__account__.token_mint_a = token_mint_a;

        let token_mint_b = loaded.token_mint_b.clone();

        loaded.__account__.token_mint_b = token_mint_b;

        let token_vault_a = loaded.token_vault_a.clone();

        loaded.__account__.token_vault_a = token_vault_a;

        let token_vault_b = loaded.token_vault_b.clone();

        loaded.__account__.token_vault_b = token_vault_b;

        let lp_token_mint = loaded.lp_token_mint.clone();

        loaded.__account__.lp_token_mint = lp_token_mint;
    }
}

#[derive(Debug)]
pub struct LoadedPool<'info, 'entrypoint> {
    pub __account__: &'entrypoint mut Box<Account<'info, Pool>>,
    pub __programs__: &'entrypoint ProgramsMap<'info>,
    pub bump: u8,
    pub authority: Pubkey,
    pub token_mint_a: Pubkey,
    pub token_mint_b: Pubkey,
    pub token_vault_a: Pubkey,
    pub token_vault_b: Pubkey,
    pub lp_token_mint: Pubkey,
}

pub fn add_liquidity_handler<'info>(
    mut user: SeahorseSigner<'info, '_>,
    mut pool: Mutable<LoadedPool<'info, '_>>,
    mut token_mint_a: SeahorseAccount<'info, '_, Mint>,
    mut token_mint_b: SeahorseAccount<'info, '_, Mint>,
    mut user_token_account_a: SeahorseAccount<'info, '_, TokenAccount>,
    mut user_token_account_b: SeahorseAccount<'info, '_, TokenAccount>,
    mut pool_token_vault_a: SeahorseAccount<'info, '_, TokenAccount>,
    mut pool_token_vault_b: SeahorseAccount<'info, '_, TokenAccount>,
    mut lp_token_mint: SeahorseAccount<'info, '_, Mint>,
    mut user_lp_token_account: Empty<SeahorseAccount<'info, '_, TokenAccount>>,
    mut token_amount_a: u64,
    mut token_amount_b: u64,
) -> () {
    let mut pool_pda = Pubkey::find_program_address(
        Mutable::new(vec![
            "pool".to_string().as_bytes().as_ref(),
            token_mint_a.key().as_ref(),
            token_mint_b.key().as_ref(),
        ])
        .borrow()
        .as_slice(),
        &id(),
    )
    .0;

    if !(pool_pda == pool.borrow().__account__.key()) {
        panic!("Pool address is not valid");
    }

    let mut lp_token_mint_pda = Pubkey::find_program_address(
        Mutable::new(vec![
            "lp-token-mint".to_string().as_bytes().as_ref(),
            token_mint_a.key().as_ref(),
            token_mint_b.key().as_ref(),
        ])
        .borrow()
        .as_slice(),
        &id(),
    )
    .0;

    if !(lp_token_mint_pda == lp_token_mint.key()) {
        panic!("LP token mint address is not valid");
    }

    if !((token_amount_a > 0) && (token_amount_b > 0)) {
        panic!("Token amount must be greater than zero");
    }

    token::transfer(
        CpiContext::new(
            user_token_account_a.programs.get("token_program"),
            token::Transfer {
                from: user_token_account_a.to_account_info(),
                authority: user.clone().to_account_info(),
                to: pool_token_vault_a.clone().to_account_info(),
            },
        ),
        token_amount_a.clone(),
    )
    .unwrap();

    token::transfer(
        CpiContext::new(
            user_token_account_b.programs.get("token_program"),
            token::Transfer {
                from: user_token_account_b.to_account_info(),
                authority: user.clone().to_account_info(),
                to: pool_token_vault_b.clone().to_account_info(),
            },
        ),
        token_amount_b.clone(),
    )
    .unwrap();

    if (pool_token_vault_a.amount > 0) || (pool_token_vault_b.amount > 0) {
        if !((pool_token_vault_a.amount * token_amount_b)
            == (pool_token_vault_b.amount * token_amount_a))
        {
            panic!("Change amount of token a or token b to add liquidity");
        }
    }

    let mut total_lp_tokens = lp_token_mint.supply;
    let mut lp_token_mint_amount = 0;

    if total_lp_tokens == 0 {
        lp_token_mint_amount = (token_amount_a * token_amount_b).pow(2);
    } else {
        lp_token_mint_amount = ((token_amount_a * total_lp_tokens) / pool_token_vault_a.amount)
            .min((token_amount_b * total_lp_tokens) / pool_token_vault_b.amount);
    }

    if !(lp_token_mint_amount > 0) {
        panic!("No lp tokens to mint");
    }

    let mut user_lp_token_account = user_lp_token_account.account.clone();

    token::mint_to(
        CpiContext::new_with_signer(
            lp_token_mint.programs.get("token_program"),
            token::MintTo {
                mint: lp_token_mint.to_account_info(),
                authority: pool.borrow().__account__.to_account_info(),
                to: user_lp_token_account.clone().to_account_info(),
            },
            &[Mutable::new(vec![
                "pool".to_string().as_bytes().as_ref(),
                token_mint_a.key().as_ref(),
                token_mint_b.key().as_ref(),
            ])
            .borrow()
            .as_slice()],
        ),
        lp_token_mint_amount.clone(),
    )
    .unwrap();
}

pub fn create_pool_handler<'info>(
    mut authority: SeahorseSigner<'info, '_>,
    mut pool: Empty<Mutable<LoadedPool<'info, '_>>>,
    mut token_mint_a: SeahorseAccount<'info, '_, Mint>,
    mut token_mint_b: SeahorseAccount<'info, '_, Mint>,
    mut token_vault_a: Empty<SeahorseAccount<'info, '_, TokenAccount>>,
    mut token_vault_b: Empty<SeahorseAccount<'info, '_, TokenAccount>>,
    mut lp_token_mint: Empty<SeahorseAccount<'info, '_, Mint>>,
) -> () {
    let mut bump = pool.bump.unwrap();
    let mut pool = pool.account.clone();
    let mut token_vault_a = token_vault_a.account.clone();
    let mut token_vault_b = token_vault_b.account.clone();
    let mut lp_token_mint = lp_token_mint.account.clone();

    assign!(pool.borrow_mut().bump, bump);

    assign!(pool.borrow_mut().authority, authority.key());

    assign!(pool.borrow_mut().token_mint_a, token_mint_a.key());

    assign!(pool.borrow_mut().token_mint_b, token_mint_b.key());

    assign!(pool.borrow_mut().token_vault_a, token_vault_a.key());

    assign!(pool.borrow_mut().token_vault_b, token_vault_b.key());

    assign!(pool.borrow_mut().lp_token_mint, lp_token_mint.key());
}

pub fn remove_liquidity_handler<'info>(
    mut user: SeahorseSigner<'info, '_>,
    mut pool: Mutable<LoadedPool<'info, '_>>,
    mut token_mint_a: SeahorseAccount<'info, '_, Mint>,
    mut token_mint_b: SeahorseAccount<'info, '_, Mint>,
    mut user_token_account_a: SeahorseAccount<'info, '_, TokenAccount>,
    mut user_token_account_b: SeahorseAccount<'info, '_, TokenAccount>,
    mut pool_token_vault_a: SeahorseAccount<'info, '_, TokenAccount>,
    mut pool_token_vault_b: SeahorseAccount<'info, '_, TokenAccount>,
    mut lp_token_mint: SeahorseAccount<'info, '_, Mint>,
    mut user_lp_token_account: SeahorseAccount<'info, '_, TokenAccount>,
) -> () {
    let mut pool_pda = Pubkey::find_program_address(
        Mutable::new(vec![
            "pool".to_string().as_bytes().as_ref(),
            token_mint_a.key().as_ref(),
            token_mint_b.key().as_ref(),
        ])
        .borrow()
        .as_slice(),
        &id(),
    )
    .0;

    if !(pool_pda == pool.borrow().__account__.key()) {
        panic!("Pool address is not valid");
    }

    let mut lp_token_mint_pda = Pubkey::find_program_address(
        Mutable::new(vec![
            "lp-token-mint".to_string().as_bytes().as_ref(),
            token_mint_a.key().as_ref(),
            token_mint_b.key().as_ref(),
        ])
        .borrow()
        .as_slice(),
        &id(),
    )
    .0;

    if !(lp_token_mint_pda == lp_token_mint.key()) {
        panic!("LP token mint address is not valid");
    }

    let mut token_burn_amount = user_lp_token_account.amount;
    let mut token_amount_a = (pool_token_vault_a.amount * token_burn_amount) / lp_token_mint.supply;
    let mut token_amount_b = (pool_token_vault_b.amount * token_burn_amount) / lp_token_mint.supply;

    token::burn(
        CpiContext::new_with_signer(
            lp_token_mint.programs.get("token_program"),
            token::Burn {
                mint: lp_token_mint.to_account_info(),
                authority: pool.borrow().__account__.to_account_info(),
                from: user_lp_token_account.clone().to_account_info(),
            },
            &[Mutable::new(vec![
                "pool".to_string().as_bytes().as_ref(),
                token_mint_a.key().as_ref(),
                token_mint_b.key().as_ref(),
            ])
            .borrow()
            .as_slice()],
        ),
        token_burn_amount.clone(),
    )
    .unwrap();

    token::transfer(
        CpiContext::new_with_signer(
            pool_token_vault_a.programs.get("token_program"),
            token::Transfer {
                from: pool_token_vault_a.to_account_info(),
                authority: pool.borrow().__account__.to_account_info(),
                to: user_token_account_a.clone().to_account_info(),
            },
            &[Mutable::new(vec![
                "pool".to_string().as_bytes().as_ref(),
                token_mint_a.key().as_ref(),
                token_mint_b.key().as_ref(),
            ])
            .borrow()
            .as_slice()],
        ),
        token_amount_a.clone(),
    )
    .unwrap();

    token::transfer(
        CpiContext::new_with_signer(
            pool_token_vault_b.programs.get("token_program"),
            token::Transfer {
                from: pool_token_vault_b.to_account_info(),
                authority: pool.borrow().__account__.to_account_info(),
                to: user_token_account_b.clone().to_account_info(),
            },
            &[Mutable::new(vec![
                "pool".to_string().as_bytes().as_ref(),
                token_mint_a.key().as_ref(),
                token_mint_b.key().as_ref(),
            ])
            .borrow()
            .as_slice()],
        ),
        token_amount_b.clone(),
    )
    .unwrap();
}

pub fn swap_handler<'info>(
    mut user: SeahorseSigner<'info, '_>,
    mut pool: Mutable<LoadedPool<'info, '_>>,
    mut token_in_mint: SeahorseAccount<'info, '_, Mint>,
    mut token_in_vault: SeahorseAccount<'info, '_, TokenAccount>,
    mut token_in_amount: u64,
    mut token_mint_a: SeahorseAccount<'info, '_, Mint>,
    mut token_mint_b: SeahorseAccount<'info, '_, Mint>,
    mut token_vault_a: SeahorseAccount<'info, '_, TokenAccount>,
    mut token_vault_b: SeahorseAccount<'info, '_, TokenAccount>,
) -> () {
    let mut pool_pda = Pubkey::find_program_address(
        Mutable::new(vec![
            "pool".to_string().as_bytes().as_ref(),
            token_mint_a.key().as_ref(),
            token_mint_b.key().as_ref(),
        ])
        .borrow()
        .as_slice(),
        &id(),
    )
    .0;

    if !(pool_pda == pool.borrow().__account__.key()) {
        panic!("Pool address is not valid");
    }

    if !((token_in_mint.key() == token_mint_a.key()) || (token_in_mint.key() == token_mint_b.key()))
    {
        panic!("Token not available in pool");
    }

    if !(token_in_amount > 0) {
        panic!("Token amount must be greater than zero");
    }

    let mut is_token_a = token_in_mint.key() == token_mint_a.key();
    let mut token_out_vault = token_vault_a;

    if is_token_a {
        token_out_vault = token_vault_b;
    }

    let mut token_out_amount =
        (token_out_vault.amount * token_in_amount) / (token_in_vault.amount + token_in_amount);

    token::transfer(
        CpiContext::new(
            token_in_vault.programs.get("token_program"),
            token::Transfer {
                from: token_in_vault.to_account_info(),
                authority: user.clone().to_account_info(),
                to: token_out_vault.clone().to_account_info(),
            },
        ),
        token_in_amount.clone(),
    )
    .unwrap();

    token::transfer(
        CpiContext::new_with_signer(
            token_out_vault.programs.get("token_program"),
            token::Transfer {
                from: token_out_vault.to_account_info(),
                authority: pool.borrow().__account__.to_account_info(),
                to: token_in_vault.clone().to_account_info(),
            },
            &[Mutable::new(vec![
                "pool".to_string().as_bytes().as_ref(),
                token_mint_a.key().as_ref(),
                token_mint_b.key().as_ref(),
            ])
            .borrow()
            .as_slice()],
        ),
        token_out_amount.clone(),
    )
    .unwrap();
}
