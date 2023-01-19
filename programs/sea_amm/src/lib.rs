#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]

pub mod dot;

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{self, AssociatedToken},
    token::{self, Mint, Token, TokenAccount},
};

use dot::program::*;
use std::{cell::RefCell, rc::Rc};

declare_id!("BBFDagoxxEadDkckRhXwRH2TmycytSjws4cErd6qKTYY");

pub mod seahorse_util {
    use super::*;

    #[cfg(feature = "pyth-sdk-solana")]
    pub use pyth_sdk_solana::{load_price_feed_from_account_info, PriceFeed};
    use std::{collections::HashMap, fmt::Debug, ops::Deref};

    pub struct Mutable<T>(Rc<RefCell<T>>);

    impl<T> Mutable<T> {
        pub fn new(obj: T) -> Self {
            Self(Rc::new(RefCell::new(obj)))
        }
    }

    impl<T> Clone for Mutable<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }

    impl<T> Deref for Mutable<T> {
        type Target = Rc<RefCell<T>>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<T: Debug> Debug for Mutable<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }

    impl<T: Default> Default for Mutable<T> {
        fn default() -> Self {
            Self::new(T::default())
        }
    }

    impl<T: Clone> Mutable<Vec<T>> {
        pub fn wrapped_index(&self, mut index: i128) -> usize {
            if index >= 0 {
                return index.try_into().unwrap();
            }

            index += self.borrow().len() as i128;

            return index.try_into().unwrap();
        }
    }

    impl<T: Clone, const N: usize> Mutable<[T; N]> {
        pub fn wrapped_index(&self, mut index: i128) -> usize {
            if index >= 0 {
                return index.try_into().unwrap();
            }

            index += self.borrow().len() as i128;

            return index.try_into().unwrap();
        }
    }

    #[derive(Clone)]
    pub struct Empty<T: Clone> {
        pub account: T,
        pub bump: Option<u8>,
    }

    #[derive(Clone, Debug)]
    pub struct ProgramsMap<'info>(pub HashMap<&'static str, AccountInfo<'info>>);

    impl<'info> ProgramsMap<'info> {
        pub fn get(&self, name: &'static str) -> AccountInfo<'info> {
            self.0.get(name).unwrap().clone()
        }
    }

    #[derive(Clone, Debug)]
    pub struct WithPrograms<'info, 'entrypoint, A> {
        pub account: &'entrypoint A,
        pub programs: &'entrypoint ProgramsMap<'info>,
    }

    impl<'info, 'entrypoint, A> Deref for WithPrograms<'info, 'entrypoint, A> {
        type Target = A;

        fn deref(&self) -> &Self::Target {
            &self.account
        }
    }

    pub type SeahorseAccount<'info, 'entrypoint, A> =
        WithPrograms<'info, 'entrypoint, Box<Account<'info, A>>>;

    pub type SeahorseSigner<'info, 'entrypoint> = WithPrograms<'info, 'entrypoint, Signer<'info>>;

    #[derive(Clone, Debug)]
    pub struct CpiAccount<'info> {
        #[doc = "CHECK: CpiAccounts temporarily store AccountInfos."]
        pub account_info: AccountInfo<'info>,
        pub is_writable: bool,
        pub is_signer: bool,
        pub seeds: Option<Vec<Vec<u8>>>,
    }

    #[macro_export]
    macro_rules! assign {
        ($ lval : expr , $ rval : expr) => {{
            let temp = $rval;

            $lval = temp;
        }};
    }

    #[macro_export]
    macro_rules! index_assign {
        ($ lval : expr , $ idx : expr , $ rval : expr) => {
            let temp_rval = $rval;
            let temp_idx = $idx;

            $lval[temp_idx] = temp_rval;
        };
    }
}

#[program]
mod sea_amm {
    use super::*;
    use seahorse_util::*;
    use std::collections::HashMap;

    #[derive(Accounts)]
    # [instruction (token_amount_a : u64 , token_amount_b : u64)]
    pub struct AddLiquidity<'info> {
        #[account(mut)]
        pub user: Signer<'info>,
        #[account(mut)]
        pub pool: Box<Account<'info, dot::program::Pool>>,
        #[account(mut)]
        pub token_mint_a: Box<Account<'info, Mint>>,
        #[account(mut)]
        pub token_mint_b: Box<Account<'info, Mint>>,
        #[account(mut)]
        pub user_token_account_a: Box<Account<'info, TokenAccount>>,
        #[account(mut)]
        pub user_token_account_b: Box<Account<'info, TokenAccount>>,
        #[account(mut)]
        pub pool_token_vault_a: Box<Account<'info, TokenAccount>>,
        #[account(mut)]
        pub pool_token_vault_b: Box<Account<'info, TokenAccount>>,
        #[account(mut)]
        pub lp_token_mint: Box<Account<'info, Mint>>,
        # [account (init , payer = user , seeds = ["lp-token-account" . as_bytes () . as_ref () , lp_token_mint . key () . as_ref () , user . key () . as_ref ()] , bump , token :: mint = lp_token_mint , token :: authority = user)]
        pub user_lp_token_account: Box<Account<'info, TokenAccount>>,
        pub rent: Sysvar<'info, Rent>,
        pub system_program: Program<'info, System>,
        pub token_program: Program<'info, Token>,
    }

    pub fn add_liquidity(
        ctx: Context<AddLiquidity>,
        token_amount_a: u64,
        token_amount_b: u64,
    ) -> Result<()> {
        let mut programs = HashMap::new();

        programs.insert(
            "system_program",
            ctx.accounts.system_program.to_account_info(),
        );

        programs.insert(
            "token_program",
            ctx.accounts.token_program.to_account_info(),
        );

        let programs_map = ProgramsMap(programs);
        let user = SeahorseSigner {
            account: &ctx.accounts.user,
            programs: &programs_map,
        };

        let pool = dot::program::Pool::load(&mut ctx.accounts.pool, &programs_map);
        let token_mint_a = SeahorseAccount {
            account: &ctx.accounts.token_mint_a,
            programs: &programs_map,
        };

        let token_mint_b = SeahorseAccount {
            account: &ctx.accounts.token_mint_b,
            programs: &programs_map,
        };

        let user_token_account_a = SeahorseAccount {
            account: &ctx.accounts.user_token_account_a,
            programs: &programs_map,
        };

        let user_token_account_b = SeahorseAccount {
            account: &ctx.accounts.user_token_account_b,
            programs: &programs_map,
        };

        let pool_token_vault_a = SeahorseAccount {
            account: &ctx.accounts.pool_token_vault_a,
            programs: &programs_map,
        };

        let pool_token_vault_b = SeahorseAccount {
            account: &ctx.accounts.pool_token_vault_b,
            programs: &programs_map,
        };

        let lp_token_mint = SeahorseAccount {
            account: &ctx.accounts.lp_token_mint,
            programs: &programs_map,
        };

        let user_lp_token_account = Empty {
            account: SeahorseAccount {
                account: &ctx.accounts.user_lp_token_account,
                programs: &programs_map,
            },
            bump: ctx.bumps.get("user_lp_token_account").map(|bump| *bump),
        };

        add_liquidity_handler(
            user.clone(),
            pool.clone(),
            token_mint_a.clone(),
            token_mint_b.clone(),
            user_token_account_a.clone(),
            user_token_account_b.clone(),
            pool_token_vault_a.clone(),
            pool_token_vault_b.clone(),
            lp_token_mint.clone(),
            user_lp_token_account.clone(),
            token_amount_a,
            token_amount_b,
        );

        dot::program::Pool::store(pool);

        return Ok(());
    }

    #[derive(Accounts)]
    pub struct CreatePool<'info> {
        #[account(mut)]
        pub authority: Signer<'info>,
        # [account (init , space = std :: mem :: size_of :: < dot :: program :: Pool > () + 8 , payer = authority , seeds = ["pool" . as_bytes () . as_ref () , token_mint_a . key () . as_ref () , token_mint_b . key () . as_ref ()] , bump)]
        pub pool: Box<Account<'info, dot::program::Pool>>,
        #[account(mut)]
        pub token_mint_a: Box<Account<'info, Mint>>,
        #[account(mut)]
        pub token_mint_b: Box<Account<'info, Mint>>,
        # [account (init , payer = authority , seeds = ["token-vault-a" . as_bytes () . as_ref () , token_mint_a . key () . as_ref ()] , bump , token :: mint = token_mint_a , token :: authority = pool)]
        pub token_vault_a: Box<Account<'info, TokenAccount>>,
        # [account (init , payer = authority , seeds = ["token-vault-b" . as_bytes () . as_ref () , token_mint_b . key () . as_ref ()] , bump , token :: mint = token_mint_b , token :: authority = pool)]
        pub token_vault_b: Box<Account<'info, TokenAccount>>,
        # [account (init , payer = authority , seeds = ["lp-token-mint" . as_bytes () . as_ref () , token_mint_a . key () . as_ref () , token_mint_b . key () . as_ref ()] , bump , mint :: decimals = 6 , mint :: authority = pool)]
        pub lp_token_mint: Box<Account<'info, Mint>>,
        pub rent: Sysvar<'info, Rent>,
        pub system_program: Program<'info, System>,
        pub token_program: Program<'info, Token>,
    }

    pub fn create_pool(ctx: Context<CreatePool>) -> Result<()> {
        let mut programs = HashMap::new();

        programs.insert(
            "system_program",
            ctx.accounts.system_program.to_account_info(),
        );

        programs.insert(
            "token_program",
            ctx.accounts.token_program.to_account_info(),
        );

        let programs_map = ProgramsMap(programs);
        let authority = SeahorseSigner {
            account: &ctx.accounts.authority,
            programs: &programs_map,
        };

        let pool = Empty {
            account: dot::program::Pool::load(&mut ctx.accounts.pool, &programs_map),
            bump: ctx.bumps.get("pool").map(|bump| *bump),
        };

        let token_mint_a = SeahorseAccount {
            account: &ctx.accounts.token_mint_a,
            programs: &programs_map,
        };

        let token_mint_b = SeahorseAccount {
            account: &ctx.accounts.token_mint_b,
            programs: &programs_map,
        };

        let token_vault_a = Empty {
            account: SeahorseAccount {
                account: &ctx.accounts.token_vault_a,
                programs: &programs_map,
            },
            bump: ctx.bumps.get("token_vault_a").map(|bump| *bump),
        };

        let token_vault_b = Empty {
            account: SeahorseAccount {
                account: &ctx.accounts.token_vault_b,
                programs: &programs_map,
            },
            bump: ctx.bumps.get("token_vault_b").map(|bump| *bump),
        };

        let lp_token_mint = Empty {
            account: SeahorseAccount {
                account: &ctx.accounts.lp_token_mint,
                programs: &programs_map,
            },
            bump: ctx.bumps.get("lp_token_mint").map(|bump| *bump),
        };

        create_pool_handler(
            authority.clone(),
            pool.clone(),
            token_mint_a.clone(),
            token_mint_b.clone(),
            token_vault_a.clone(),
            token_vault_b.clone(),
            lp_token_mint.clone(),
        );

        dot::program::Pool::store(pool.account);

        return Ok(());
    }

    #[derive(Accounts)]
    pub struct RemoveLiquidity<'info> {
        #[account(mut)]
        pub user: Signer<'info>,
        #[account(mut)]
        pub pool: Box<Account<'info, dot::program::Pool>>,
        #[account(mut)]
        pub token_mint_a: Box<Account<'info, Mint>>,
        #[account(mut)]
        pub token_mint_b: Box<Account<'info, Mint>>,
        #[account(mut)]
        pub user_token_account_a: Box<Account<'info, TokenAccount>>,
        #[account(mut)]
        pub user_token_account_b: Box<Account<'info, TokenAccount>>,
        #[account(mut)]
        pub pool_token_vault_a: Box<Account<'info, TokenAccount>>,
        #[account(mut)]
        pub pool_token_vault_b: Box<Account<'info, TokenAccount>>,
        #[account(mut)]
        pub lp_token_mint: Box<Account<'info, Mint>>,
        #[account(mut)]
        pub user_lp_token_account: Box<Account<'info, TokenAccount>>,
        pub token_program: Program<'info, Token>,
    }

    pub fn remove_liquidity(ctx: Context<RemoveLiquidity>) -> Result<()> {
        let mut programs = HashMap::new();

        programs.insert(
            "token_program",
            ctx.accounts.token_program.to_account_info(),
        );

        let programs_map = ProgramsMap(programs);
        let user = SeahorseSigner {
            account: &ctx.accounts.user,
            programs: &programs_map,
        };

        let pool = dot::program::Pool::load(&mut ctx.accounts.pool, &programs_map);
        let token_mint_a = SeahorseAccount {
            account: &ctx.accounts.token_mint_a,
            programs: &programs_map,
        };

        let token_mint_b = SeahorseAccount {
            account: &ctx.accounts.token_mint_b,
            programs: &programs_map,
        };

        let user_token_account_a = SeahorseAccount {
            account: &ctx.accounts.user_token_account_a,
            programs: &programs_map,
        };

        let user_token_account_b = SeahorseAccount {
            account: &ctx.accounts.user_token_account_b,
            programs: &programs_map,
        };

        let pool_token_vault_a = SeahorseAccount {
            account: &ctx.accounts.pool_token_vault_a,
            programs: &programs_map,
        };

        let pool_token_vault_b = SeahorseAccount {
            account: &ctx.accounts.pool_token_vault_b,
            programs: &programs_map,
        };

        let lp_token_mint = SeahorseAccount {
            account: &ctx.accounts.lp_token_mint,
            programs: &programs_map,
        };

        let user_lp_token_account = SeahorseAccount {
            account: &ctx.accounts.user_lp_token_account,
            programs: &programs_map,
        };

        remove_liquidity_handler(
            user.clone(),
            pool.clone(),
            token_mint_a.clone(),
            token_mint_b.clone(),
            user_token_account_a.clone(),
            user_token_account_b.clone(),
            pool_token_vault_a.clone(),
            pool_token_vault_b.clone(),
            lp_token_mint.clone(),
            user_lp_token_account.clone(),
        );

        dot::program::Pool::store(pool);

        return Ok(());
    }

    #[derive(Accounts)]
    # [instruction (token_in_amount : u64)]
    pub struct Swap<'info> {
        #[account(mut)]
        pub user: Signer<'info>,
        #[account(mut)]
        pub pool: Box<Account<'info, dot::program::Pool>>,
        #[account(mut)]
        pub token_in_mint: Box<Account<'info, Mint>>,
        #[account(mut)]
        pub token_in_vault: Box<Account<'info, TokenAccount>>,
        #[account(mut)]
        pub token_mint_a: Box<Account<'info, Mint>>,
        #[account(mut)]
        pub token_mint_b: Box<Account<'info, Mint>>,
        #[account(mut)]
        pub token_vault_a: Box<Account<'info, TokenAccount>>,
        #[account(mut)]
        pub token_vault_b: Box<Account<'info, TokenAccount>>,
        pub token_program: Program<'info, Token>,
    }

    pub fn swap(ctx: Context<Swap>, token_in_amount: u64) -> Result<()> {
        let mut programs = HashMap::new();

        programs.insert(
            "token_program",
            ctx.accounts.token_program.to_account_info(),
        );

        let programs_map = ProgramsMap(programs);
        let user = SeahorseSigner {
            account: &ctx.accounts.user,
            programs: &programs_map,
        };

        let pool = dot::program::Pool::load(&mut ctx.accounts.pool, &programs_map);
        let token_in_mint = SeahorseAccount {
            account: &ctx.accounts.token_in_mint,
            programs: &programs_map,
        };

        let token_in_vault = SeahorseAccount {
            account: &ctx.accounts.token_in_vault,
            programs: &programs_map,
        };

        let token_mint_a = SeahorseAccount {
            account: &ctx.accounts.token_mint_a,
            programs: &programs_map,
        };

        let token_mint_b = SeahorseAccount {
            account: &ctx.accounts.token_mint_b,
            programs: &programs_map,
        };

        let token_vault_a = SeahorseAccount {
            account: &ctx.accounts.token_vault_a,
            programs: &programs_map,
        };

        let token_vault_b = SeahorseAccount {
            account: &ctx.accounts.token_vault_b,
            programs: &programs_map,
        };

        swap_handler(
            user.clone(),
            pool.clone(),
            token_in_mint.clone(),
            token_in_vault.clone(),
            token_in_amount,
            token_mint_a.clone(),
            token_mint_b.clone(),
            token_vault_a.clone(),
            token_vault_b.clone(),
        );

        dot::program::Pool::store(pool);

        return Ok(());
    }
}
