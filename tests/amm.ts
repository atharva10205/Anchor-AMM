import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Amm } from "../target/types/amm";
import {
  Keypair,
  LAMPORTS_PER_SOL,
  SystemProgram,
  PublicKey,
} from "@solana/web3.js";
import {
  createMint,
  TOKEN_PROGRAM_ID as tokenProgram,
  ASSOCIATED_TOKEN_PROGRAM_ID as associatedTokenProgram,
  getOrCreateAssociatedTokenAccount,
  getAssociatedTokenAddressSync,
  mintTo,
} from "@solana/spl-token";
import { assert } from "chai";

describe("amm", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.amm as Program<Amm>;
  const provider = anchor.getProvider();
  const connection = provider.connection;

  const admin = Keypair.generate();
  const user = Keypair.generate();

  const seed = new anchor.BN(696969);
  const fee = 30;
  const DECIMALS = 6;

  let mint_x: PublicKey;
  let mint_y: PublicKey;
  let vault_x: PublicKey;
  let vault_y: PublicKey;
  let user_x: PublicKey;
  let user_y: PublicKey;
  let user_lp: PublicKey;

  const [config, configBump] = PublicKey.findProgramAddressSync(
    [Buffer.from("config"), seed.toArrayLike(Buffer, "le", 8)],
    program.programId
  );

  const [mint_liquidity_pool] = PublicKey.findProgramAddressSync(
    [Buffer.from("lp"), config.toBuffer()],
    program.programId
  );

  before("Airdrop and create Mints", async () => {
    const adminAirdropSig = await connection.requestAirdrop(
      admin.publicKey,
      10 * LAMPORTS_PER_SOL
    );
    await connection.confirmTransaction(adminAirdropSig);
    console.log(`Airdropped 10 SOL to admin: ${admin.publicKey.toBase58()}`);

    const userAirdropSig = await connection.requestAirdrop(
      user.publicKey,
      10 * LAMPORTS_PER_SOL
    );
    await connection.confirmTransaction(userAirdropSig);
    console.log(`Airdropped 10 SOL to user: ${user.publicKey.toBase58()}`);

    const latestBlockhash = await connection.getLatestBlockhash();
    console.log(`Latest Blockhash: ${latestBlockhash.blockhash}`);

    mint_x = await createMint(
      connection,
      admin,
      admin.publicKey,
      null,
      DECIMALS
    );
    mint_y = await createMint(
      connection,
      admin,
      admin.publicKey,
      null,
      DECIMALS
    );

    vault_x = getAssociatedTokenAddressSync(mint_x, config, true);
    vault_y = getAssociatedTokenAddressSync(mint_y, config, true);

    user_x = (
      await getOrCreateAssociatedTokenAccount(
        connection,
        user,
        mint_x,
        user.publicKey
      )
    ).address;
    user_y = (
      await getOrCreateAssociatedTokenAccount(
        connection,
        user,
        mint_y,
        user.publicKey
      )
    ).address;

    await mintTo(
      connection,
      admin,
      mint_x,
      user_x,
      admin,
      1000 * 10 ** DECIMALS
    );
    await mintTo(
      connection,
      admin,
      mint_y,
      user_y,
      admin,
      1000 * 10 ** DECIMALS
    );
  });

  it("initialize pool", async () => {
    await program.methods
      .initialize(seed, fee, admin.publicKey)
      .accountsStrict({
        signer: admin.publicKey,
        mintX: mint_x,
        mintY: mint_y,
        mintLiquidityPool: mint_liquidity_pool,
        vaultX: vault_x,
        vaultY: vault_y,
        config: config,
        tokenProgram,
        associatedTokenProgram,
        systemProgram: SystemProgram.programId,
      })
      .signers([admin])
      .rpc();

    const configAccount = await program.account.config.fetch(config);
    assert.ok(configAccount.authority.equals(admin.publicKey));
    assert.equal(configAccount.fee, fee);
    assert.equal(configAccount.locked, false);
  });

  it("deposit liquidity", async () => {
    user_lp = (
      await getOrCreateAssociatedTokenAccount(
        connection,
        user,
        mint_liquidity_pool,
        user.publicKey
      )
    ).address;

    const depositAmount = new anchor.BN(100 * 10 ** DECIMALS);
    const maxX = new anchor.BN(50 * 10 ** DECIMALS);
    const maxY = new anchor.BN(50 * 10 ** DECIMALS);

    await program.methods
      .deposit(depositAmount, maxX, maxY)
      .accountsStrict({
        signer: user.publicKey,
        mintX: mint_x,
        mintY: mint_y,
        config: config,
        mintLiquidityPool: mint_liquidity_pool,
        vaultX: vault_x,
        vaultY: vault_y,
        userX: user_x,
        userY: user_y,
        userLiquidityPool: user_lp,
        tokenProgram,
        associatedTokenProgram,
        systemProgram: SystemProgram.programId,
      })
      .signers([user])
      .rpc();

    const user_Liquidity_Pool_Balance = await connection.getTokenAccountBalance(
      user_lp
    );
    assert.equal(
      user_Liquidity_Pool_Balance.value.amount,
      depositAmount.toString()
    );
  });

  it("Swap X for Y", async () => {
    const swapAmount = new anchor.BN(5 * 10 ** DECIMALS);
    const minOut = new anchor.BN(1);

    const user_Y_Before = await connection.getTokenAccountBalance(user_y);

    await program.methods
      .swap(true, swapAmount, minOut)
      .accountsStrict({
        signer: user.publicKey,
        mintX: mint_x,
        mintY: mint_y,
        config: config,
        mintLp: mint_liquidity_pool,
        vaultX: vault_x,
        vaultY: vault_y,
        userX: user_x,
        userY: user_y,
        tokenProgram,
        associatedTokenProgram,
        systemProgram: SystemProgram.programId,
      })
      .signers([user])
      .rpc();

    const user_Y_After = await connection.getTokenAccountBalance(user_y);

    assert.ok(
      BigInt(user_Y_After.value.amount) > BigInt(user_Y_Before.value.amount)
    );
  });

  it("lock and unlock pool", async () => {
    await program.methods
      .lock()
      .accountsStrict({
        signer: admin.publicKey,
        config: config,
      })
      .signers([admin])
      .rpc();
    let configAccount = await program.account.config.fetch(config);
    assert.equal(configAccount.locked, true);

    await program.methods
      .unlock()
      .accountsStrict({
        signer: admin.publicKey,
        config: config,
      })
      .signers([admin])
      .rpc();
    configAccount = await program.account.config.fetch(config);
    assert.equal(configAccount.locked, false);
  });

  it("withdraw liquidity", async () => {
    const user_lp_before = await connection.getTokenAccountBalance(user_lp);
    const userXBefore = await connection.getTokenAccountBalance(user_x);
    const userYBefore = await connection.getTokenAccountBalance(user_y);

    const withdrawAmount = new anchor.BN(user_lp_before.value.amount).div(
      new anchor.BN(2)
    );
    const minX = new anchor.BN(1);
    const minY = new anchor.BN(1);

    await program.methods.withdraw(withdrawAmount, minX, minY).accountsStrict({
      signer: user.publicKey,
      mintX: mint_x,
      mintY: mint_y,
      config: config,
      mintLp: mint_liquidity_pool,
      vaultX: vault_x,
      vaultY: vault_y,
      userX: user_x,
      userY: user_y,
      userLp: user_lp,
      tokenProgram,
      associatedTokenProgram,
      systemProgram: SystemProgram.programId,
    }).signers([user])
    .rpc();

    const userLpAfter = await connection.getTokenAccountBalance(user_lp);
    const userXAfter = await connection.getTokenAccountBalance(user_x);
    const userYAfter = await connection.getTokenAccountBalance(user_y);

    assert.ok(BigInt(userLpAfter.value.amount) < BigInt(user_lp_before.value.amount));
    assert.ok(BigInt(userXAfter.value.amount) > BigInt(userXBefore.value.amount));
    assert.ok(BigInt(userYAfter.value.amount) > BigInt(userYBefore.value.amount));
  });
});
