# sea_amm
# Built with Seahorse v0.2.6

from seahorse.prelude import *

declare_id('BBFDagoxxEadDkckRhXwRH2TmycytSjws4cErd6qKTYY')


class Pool(Account):
    bump: u8
    authority: Pubkey
    token_mint_a: Pubkey
    token_mint_b: Pubkey
    token_vault_a: Pubkey
    token_vault_b: Pubkey
    lp_token_mint: Pubkey


@instruction
def create_pool(
    authority: Signer,
    pool: Empty[Pool],
    token_mint_a: TokenMint,
    token_mint_b: TokenMint,
    token_vault_a: Empty[TokenAccount],
    token_vault_b: Empty[TokenAccount],
    lp_token_mint: Empty[TokenMint],
):
    bump = pool.bump()
    pool = pool.init(
        payer=authority,
        seeds=["pool", token_mint_a, token_mint_b]
    )
    token_vault_a = token_vault_a.init(
        payer=authority,
        authority=pool,
        mint=token_mint_a,
        seeds=["token-vault-a", token_mint_a]
    )
    token_vault_b = token_vault_b.init(
        payer=authority,
        authority=pool,
        mint=token_mint_b,
        seeds=["token-vault-b", token_mint_b]
    )
    lp_token_mint = lp_token_mint.init(
        payer=authority,
        seeds=["lp-token-mint", token_mint_a, token_mint_b],
        decimals=6,
        authority=pool
    )
    pool.bump = bump
    pool.authority = authority.key()
    pool.token_mint_a = token_mint_a.key()
    pool.token_mint_b = token_mint_b.key()
    pool.token_vault_a = token_vault_a.key()
    pool.token_vault_b = token_vault_b.key()
    pool.lp_token_mint = lp_token_mint.key()


@instruction
def swap(
    user: Signer,
    pool: Pool,
    token_in_mint: TokenMint,
    token_in_vault: TokenAccount,
    token_in_amount: u64,
    token_mint_a: TokenMint,
    token_mint_b: TokenMint,
    token_vault_a: TokenAccount,
    token_vault_b: TokenAccount,
):
    # account checks
    pool_pda = Pubkey.find_program_address(
        ["pool", token_mint_a, token_mint_b])[0]
    assert pool_pda == pool.key(), "Pool address is not valid"
    assert token_in_mint.key() == token_mint_a.key() or token_in_mint.key(
    ) == token_mint_b.key(), "Token not available in pool"
    assert token_in_amount > 0, "Token amount must be greater than zero"

    # determine which token is token in
    is_token_a = token_in_mint.key() == token_mint_a.key()
    token_out_vault = token_vault_a
    if is_token_a:
        token_out_vault = token_vault_b

    # calculate token out amount
    # dy = ydx / (x + dx)j
    token_out_amount = (token_out_vault.amount(
    ) * token_in_amount) // (token_in_vault.amount() + token_in_amount)

    # Transfer token in from user to pool
    token_in_vault.transfer(
        authority=user,
        to=token_out_vault,
        amount=token_in_amount
    )

    # Transfer token out from pool to user
    token_out_vault.transfer(
        authority=pool,
        to=token_in_vault,
        amount=token_out_amount,
        signer=["pool", token_mint_a, token_mint_b]
    )


@instruction
def add_liquidity(
    user: Signer,
    pool: Pool,
    token_mint_a: TokenMint,
    token_mint_b: TokenMint,
    user_token_account_a: TokenAccount,
    user_token_account_b: TokenAccount,
    pool_token_vault_a: TokenAccount,
    pool_token_vault_b: TokenAccount,
    lp_token_mint: TokenMint,
    user_lp_token_account: Empty[TokenAccount],
    token_amount_a: u64,
    token_amount_b: u64,
):
    # account checks
    pool_pda = Pubkey.find_program_address(
        ["pool", token_mint_a, token_mint_b])[0]
    assert pool_pda == pool.key(), "Pool address is not valid"
    lp_token_mint_pda = Pubkey.find_program_address(
        ["lp-token-mint", token_mint_a, token_mint_b])[0]
    assert lp_token_mint_pda == lp_token_mint.key(), "LP token mint address is not valid"
    assert token_amount_a > 0 and token_amount_b > 0, "Token amount must be greater than zero"

    # transfer tokens from user to pool
    user_token_account_a.transfer(
        authority=user,
        to=pool_token_vault_a,
        amount=token_amount_a,
    )
    user_token_account_b.transfer(
        authority=user,
        to=pool_token_vault_b,
        amount=token_amount_b
    )

    # check no price change after adding liquidity
    if pool_token_vault_a.amount() > 0 or pool_token_vault_b.amount() > 0:
        assert pool_token_vault_a.amount(
        ) * token_amount_b == pool_token_vault_b.amount() * token_amount_a, "Change amount of token a or token b to add liquidity"

    # calculate amount of lp tokens to mint
    total_lp_tokens = lp_token_mint.supply()
    lp_token_mint_amount = 0
    if total_lp_tokens == 0:
        lp_token_mint_amount = (token_amount_a * token_amount_b) ** 2
    else:
        lp_token_mint_amount = min(
            (token_amount_a * total_lp_tokens) //
            pool_token_vault_a.amount(),
            (token_amount_b * total_lp_tokens) //
            pool_token_vault_b.amount(),
        )
    assert lp_token_mint_amount > 0, "No lp tokens to mint"

    # mint lp tokens
    user_lp_token_account = user_lp_token_account.init(
        payer=user,
        authority=user,
        mint=lp_token_mint,
        seeds=["lp-token-account", lp_token_mint, user]
    )
    lp_token_mint.mint(
        authority=pool,
        to=user_lp_token_account,
        amount=lp_token_mint_amount,
        signer=["pool", token_mint_a, token_mint_b]
    )


@instruction
def remove_liquidity(
    user: Signer,
    pool: Pool,
    token_mint_a: TokenMint,
    token_mint_b: TokenMint,
    user_token_account_a: TokenAccount,
    user_token_account_b: TokenAccount,
    pool_token_vault_a: TokenAccount,
    pool_token_vault_b: TokenAccount,
    lp_token_mint: TokenMint,
    user_lp_token_account: TokenAccount
):
    # check accounts
    pool_pda = Pubkey.find_program_address(
        ["pool", token_mint_a, token_mint_b])[0]
    assert pool_pda == pool.key(), "Pool address is not valid"
    lp_token_mint_pda = Pubkey.find_program_address(
        ["lp-token-mint", token_mint_a, token_mint_b])[0]
    assert lp_token_mint_pda == lp_token_mint.key(), "LP token mint address is not valid"

    # calculate token amount to withdraw
    token_burn_amount = user_lp_token_account.amount()
    token_amount_a = pool_token_vault_a.amount(
    ) * token_burn_amount // lp_token_mint.supply()
    token_amount_b = pool_token_vault_b.amount(
    ) * token_burn_amount // lp_token_mint.supply()

    # burn lp tokens
    lp_token_mint.burn(
        authority=pool,
        holder=user_lp_token_account,
        amount=token_burn_amount,
        signer=["pool", token_mint_a, token_mint_b]
    )

    # transfer both tokens from pool to user
    pool_token_vault_a.transfer(
        authority=pool,
        to=user_token_account_a,
        amount=token_amount_a,
        signer=["pool", token_mint_a, token_mint_b]
    )
    pool_token_vault_b.transfer(
        authority=pool,
        to=user_token_account_b,
        amount=token_amount_b,
        signer=["pool", token_mint_a, token_mint_b]
    )
