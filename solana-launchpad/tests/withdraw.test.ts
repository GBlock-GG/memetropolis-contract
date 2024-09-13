import { assert } from "chai";
import * as anchor from "@coral-xyz/anchor";
import { Program, web3, BN } from "@coral-xyz/anchor";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { IdoLaunchpad } from "../target/types/ido_launchpad";
import {
  createMint,
  getAccount,
  createAssociatedTokenAccount,
  mintTo,
} from "@solana/spl-token";
import { sleep, getBlockTimestamp, createTokenMint } from "./utils";

describe("test withdraw payment & meme by admin", async () => {
    // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const owner = anchor.Wallet.local().payer;
  const program = anchor.workspace.IdoLaunchpad as Program<IdoLaunchpad>;
  const connection = anchor.getProvider().connection;
  it("test withdraw", async () => {
    const minInvest = 10;
    const maxInvest = 1000000;
    const tokenPrice = 10;

    const currentBlockTime = await getBlockTimestamp(connection);

    const startTime = currentBlockTime + 5;
    const endTime = startTime + 5;

    //create meme Token Mint
    const memeMintKp = new web3.Keypair();
    await createTokenMint(connection, owner, memeMintKp);
    //create payment Token Mint
    const paymentMintKp = new web3.Keypair();
    await createTokenMint(connection, owner, paymentMintKp);

    const IDO_LAUNCHPAD_SEED = "ido_launchpad";

    const [launchpadStatePda, bump] = web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from(IDO_LAUNCHPAD_SEED),
        memeMintKp.publicKey.toBuffer(),
        owner.publicKey.toBuffer(),
      ],
      program.programId
    );

    const [memeTokenAccountPda] = web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from(IDO_LAUNCHPAD_SEED),
        launchpadStatePda.toBuffer(),
        memeMintKp.publicKey.toBuffer(),
      ],
      program.programId
    );

    const [paymentTokenAccountPda] = web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from(IDO_LAUNCHPAD_SEED),
        launchpadStatePda.toBuffer(),
        paymentMintKp.publicKey.toBuffer(),
      ],
      program.programId
    );

    // initialize
    const txHash = await program.methods
      .initialize(
        new BN(minInvest),
        new BN(maxInvest),
        new BN(tokenPrice),
        new BN(startTime),
        new BN(endTime)
      )
      .accounts({
        launchpadState: launchpadStatePda,
        memeTokenAccount: memeTokenAccountPda,
        paymentTokenAccount: paymentTokenAccountPda,
        memeMint: memeMintKp.publicKey,
        paymentMint: paymentMintKp.publicKey,
        signer: owner.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([])
      .rpc();
    // Confirm transaction
    await connection.confirmTransaction(txHash);

    //mint 100_000 meme token to launchpad meme token account by admin  (deposit by admin)
    await mintTo(
        connection,
        owner,
        memeMintKp.publicKey,
        memeTokenAccountPda,
        owner,
        100_000
      );


    const [userStakePda, userStakePdaBump] =
      web3.PublicKey.findProgramAddressSync(
        [
          Buffer.from(IDO_LAUNCHPAD_SEED),
          launchpadStatePda.toBuffer(),
          owner.publicKey.toBuffer(),
        ],
        program.programId
      );

    const userPaymentTokenAccount = await createAssociatedTokenAccount(
      connection,
      owner, //payer
      paymentMintKp.publicKey,
      owner.publicKey //owner
    );
    // mint 100_000 payment token to user's payment token account for testing..
    await mintTo(
      connection,
      owner,
      paymentMintKp.publicKey,
      userPaymentTokenAccount,
      owner,
      100_000
    );

    const buyTokenAmount = 10;

    await sleep(5000); //sleep for 5 seconds to active sale
    //buy with 10 paymentToken
    const txHash1 = await program.methods
      .buyTokens(new BN(buyTokenAmount))
      .accounts({
        launchpadState: launchpadStatePda,
        userStake: userStakePda,
        userPaymentTokenAccount,
        launchpadPaymentTokenAccount: paymentTokenAccountPda,
        paymentMint: paymentMintKp.publicKey,
        memeMint: memeMintKp.publicKey,
        signer: owner.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([])
      .rpc();
    await connection.confirmTransaction(txHash1);
    await sleep(5000); //sleep for 5 seconds to end sale
    //claim token
    //create userMemeTokenAccount
    const userMemeTokenAccount = await createAssociatedTokenAccount(
      connection,
      owner, //payer
      memeMintKp.publicKey,
      owner.publicKey //owner
    );
    const txHash2 = await program.methods
      .claimTokens()
      .accounts({
        launchpadState: launchpadStatePda,
        userStake: userStakePda,
        userMemeTokenAccount,
        launchpadMemeTokenAccount: memeTokenAccountPda,
        paymentMint: paymentMintKp.publicKey,
        memeMint: memeMintKp.publicKey,
        signer: owner.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([])
      .rpc();
    await connection.confirmTransaction(txHash2);

    //withdraw meme

    const txHash3 = await program.methods
      .withdrawMeme()
      .accounts({
        launchpadState: launchpadStatePda,
        launchpadMemeTokenAccount: memeTokenAccountPda,
        beneficiaryMemeTokenAccount: userMemeTokenAccount,
        memeMint: memeMintKp.publicKey,
        signer: owner.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: web3.SystemProgram.programId,
      }).signers([]).rpc();
    await connection.confirmTransaction(txHash3);
    const userMemeTokenAccountInfo = await getAccount(
        connection,
        userMemeTokenAccount
    );
    assert(userMemeTokenAccountInfo.amount == BigInt(100_000))

    //withdraw payment
    const txHash4 = await program.methods
      .withdrawPayment()
      .accounts({
        launchpadState: launchpadStatePda,
        launchpadPaymentTokenAccount: paymentTokenAccountPda,
        beneficiaryPaymentTokenAccount: userPaymentTokenAccount,
        paymentMint: paymentMintKp.publicKey,
        signer: owner.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: web3.SystemProgram.programId,
      }).signers([]).rpc();
    await connection.confirmTransaction(txHash4);
    const userPaymentTokenAccountInfo = await getAccount(
        connection,
        userPaymentTokenAccount
    );
    console.log("amount:", userPaymentTokenAccountInfo.amount)
    assert(userPaymentTokenAccountInfo.amount == BigInt(100_000))
  });
});