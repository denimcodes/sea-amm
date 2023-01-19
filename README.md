## Seahorse Constant Product AMM

An example of a Solana program built with Seahorse. It is an automated marked maker program. Users can add liquidity, remove liquidity and swap tokens using the program.

A constant product automated market maker (CPMM) uses a constant product formula (xy=k) to determine the prices of assets traded on the platform, providing a stable trading environment. CPMMs are often used in decentralized finance (DeFi) applications.

### Get started

1. Install [solana](https://docs.solana.com/cli/install-solana-cli-tools), [anchor](https://www.anchor-lang.com/docs/installation) and [seahorse](https://seahorse-lang.org/docs/installation). 
2. After project is created, `cd` to program folder.
2. Run `seahorse init <program-name>` command to create new seahorse project.
3. Run `seahorse build` to build the program. The first build might take few minutes.
4. After the build is complete, start writing the program in `<program-name>.py` file under `programs_py` folder.

### Program state

Solana program states are stored in data accounts. 
Here we have a `pool` account which is used for storing a pair of tokens.

### Program Instructions

Instructions are functions where logic of the program is stored. We can create new accounts, create tokens, mint and transfer tokens with instructions. Instructions can be called from client programs.

There are three instructions in this program.

1. `create_pool` -> Create a new pool account for a pair of tokens.
2. `add_liquidity` -> Users can add liquidity to the pool for a specific pair of tokens and mint lp tokens. The lp tokens value are proportional to the value of added tokens.
3. `remove_liquidity` -> Users can remove liquidity from the pool.
4. `swap` -> Swap between two pairs of tokens. 

### Test
Run `anchor test` command to run test programs on solana localhost.
