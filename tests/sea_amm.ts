import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { createAccount, createMint, getMint, mintTo } from "@solana/spl-token";
import { SeaAmm } from "../target/types/sea_amm";

// what is dy given dx?
// dy = ydx / x
// what is dx given dy?
// dx = xdy / y

describe("sea_amm", () => {
	const provider = anchor.AnchorProvider.env();
	const connection = provider.connection;
	anchor.setProvider(provider);

	const program = anchor.workspace.SeaAmm as Program<SeaAmm>;
	const programIdPk = new anchor.web3.PublicKey(program.programId);

	const authority = anchor.web3.Keypair.generate();
	const authorityPk = authority.publicKey;

	const user = anchor.web3.Keypair.generate();
	const userPk = user.publicKey;

	let poolPk: anchor.web3.PublicKey;
	let tokenMintAPk: anchor.web3.PublicKey;
	let tokenMintBPk: anchor.web3.PublicKey;
	let lpTokenMintPk: anchor.web3.PublicKey;
	let userTokenAccountAPk: anchor.web3.PublicKey;
	let userTokenAccountBPk: anchor.web3.PublicKey;
	let userLPTokenAccountPk: anchor.web3.PublicKey;
	let poolTokenVaultAPk: anchor.web3.PublicKey;
	let poolTokenVaultBPk: anchor.web3.PublicKey;

	before(async () => {
		// request airdrops
		const authorityAirdropSign = await connection.requestAirdrop(
			authorityPk,
			anchor.web3.LAMPORTS_PER_SOL * 5
		);
		await connection.confirmTransaction(authorityAirdropSign);
		const userAirdropSign = await connection.requestAirdrop(
			userPk,
			anchor.web3.LAMPORTS_PER_SOL
		);
		await connection.confirmTransaction(userAirdropSign);

		// create tokens
		tokenMintAPk = await createMint(
			connection,
			authority,
			authorityPk,
			authorityPk,
			6
		);

		tokenMintBPk = await createMint(
			connection,
			authority,
			authorityPk,
			authorityPk,
			6
		);

		// create token accounts for user
		userTokenAccountAPk = await createAccount(
			connection,
			authority,
			tokenMintAPk,
			userPk
		);

		userTokenAccountBPk = await createAccount(
			connection,
			authority,
			tokenMintBPk,
			userPk
		);

		// mint tokens to user account
		await mintTo(
			connection,
			authority,
			tokenMintAPk,
			userTokenAccountAPk,
			authority,
			1000_000_000
		);

		await mintTo(
			connection,
			authority,
			tokenMintBPk,
			userTokenAccountBPk,
			authority,
			1000_000_000
		);

		// check total mint supply
		const mintAccountA = await getMint(connection, tokenMintAPk);
		console.log("Mint A supply", mintAccountA.supply.toString());
		const mintAccountB = await getMint(connection, tokenMintBPk);
		console.log("Mint B supply", mintAccountB.supply.toString());

		// get pda accounts
		[poolPk] = anchor.web3.PublicKey.findProgramAddressSync(
			[Buffer.from("pool"), tokenMintAPk.toBuffer(), tokenMintBPk.toBuffer()],
			programIdPk
		);
		[lpTokenMintPk] = anchor.web3.PublicKey.findProgramAddressSync(
			[
				Buffer.from("lp-token-mint"),
				tokenMintAPk.toBuffer(),
				tokenMintBPk.toBuffer(),
			],
			programIdPk
		);
		[poolTokenVaultAPk] = anchor.web3.PublicKey.findProgramAddressSync(
			[Buffer.from("token-vault-a"), tokenMintAPk.toBuffer()],
			programIdPk
		);
		[poolTokenVaultBPk] = anchor.web3.PublicKey.findProgramAddressSync(
			[Buffer.from("token-vault-b"), tokenMintBPk.toBuffer()],
			programIdPk
		);
		[userLPTokenAccountPk] = anchor.web3.PublicKey.findProgramAddressSync(
			[
				Buffer.from("lp-token-account"),
				lpTokenMintPk.toBuffer(),
				userPk.toBuffer(),
			],
			programIdPk
		);
	});

	it("create pool", async () => {
		const tx = await program.methods
			.createPool()
			.accounts({
				authority: authorityPk,
				pool: poolPk,
				tokenMintA: tokenMintAPk,
				tokenMintB: tokenMintBPk,
				tokenVaultA: poolTokenVaultAPk,
				tokenVaultB: poolTokenVaultBPk,
				lpTokenMint: lpTokenMintPk,
			})
			.signers([authority])
			.rpc();
		console.log("Your transaction signature", tx);
	});
});
