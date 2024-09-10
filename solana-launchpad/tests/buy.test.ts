// No imports needed: web3, anchor, pg and more are globally available
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";

import { Connection, Signer, Keypair } from "@solana/web3.js";
import {
  createMint,
  getAccount,
  createAssociatedTokenAccount,
  mintTo,
} from "@solana/spl-token";

async function createTokenMint(
  connection: Connection,
  payer: Signer,
  keypair: Keypair
) {
  await createMint(
    connection,
    payer,
    payer.publicKey,
    payer.publicKey,
    9,
    keypair
  );
}

async function getBlockTimestamp(connection: Connection): Promise<number> {
  let slot = await connection.getSlot();
  return await connection.getBlockTime(slot);
}

describe("Test", async () => {
  it("buy_tokens", async () => {
    const minInvest = 10;
    const maxInvest = 1000000;
    const tokenPrice = 10;

    const currentBlockTime = await getBlockTimestamp(pg.connection);

    const startTime = currentBlockTime + 10;
    const endTime = currentBlockTime + 25;

    //create meme Token Mint
    const memeMintKp = new web3.Keypair();
    await createTokenMint(pg.connection, pg.wallet.keypair, memeMintKp);
    //create payment Token Mint
    const paymentMintKp = new web3.Keypair();
    await createTokenMint(pg.connection, pg.wallet.keypair, paymentMintKp);

    const IDO_LAUNCHPAD_SEED = "ido_launchpad";

    const [launchpadStatePda, bump] = web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from(IDO_LAUNCHPAD_SEED),
        memeMintKp.publicKey.toBuffer(),
        pg.wallet.publicKey.toBuffer(),
      ],
      pg.PROGRAM_ID
    );

    const [memeTokenAccountPda] = web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from(IDO_LAUNCHPAD_SEED),
        launchpadStatePda.toBuffer(),
        memeMintKp.publicKey.toBuffer(),
      ],
      pg.PROGRAM_ID
    );

    const [paymentTokenAccountPda] = web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from(IDO_LAUNCHPAD_SEED),
        launchpadStatePda.toBuffer(),
        paymentMintKp.publicKey.toBuffer(),
      ],
      pg.PROGRAM_ID
    );

    // Send transaction
    const txHash = await pg.program.methods
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
        signer: pg.wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([])
      .rpc();
    // Confirm transaction
    await pg.connection.confirmTransaction(txHash);

    const [userStakePda, userStakePdaBump] =
      web3.PublicKey.findProgramAddressSync(
        [
          Buffer.from(IDO_LAUNCHPAD_SEED),
          launchpadStatePda.toBuffer(),
          pg.wallet.publicKey.toBuffer(),
        ],
        pg.PROGRAM_ID
      );

    const userPaymentTokenAccount = await createAssociatedTokenAccount(
      pg.connection,
      pg.wallet.keypair, //payer
      paymentMintKp.publicKey,
      pg.wallet.publicKey //owner
    );

    await mintTo(
      pg.connection,
      pg.wallet.keypair,
      paymentMintKp.publicKey,
      userPaymentTokenAccount,
      pg.wallet.keypair,
      100_000
    );

    const buyTokenAmount = 10;

    await sleep(5000); //sleep for 5 seconds to active sale

    const txHash1 = await pg.program.methods
      .buyTokens(new BN(buyTokenAmount))
      .accounts({
        launchpadState: launchpadStatePda,
        userStake: userStakePda,
        userPaymentTokenAccount,
        launchpadPaymentTokenAccount: paymentTokenAccountPda,
        paymentMint: paymentMintKp.publicKey,
        memeMint: memeMintKp.publicKey,
        signer: pg.wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([])
      .rpc();
    // Confirm transaction
    await pg.connection.confirmTransaction(txHash1);

    const launchpadStateAccount = await pg.program.account.launchpadState.fetch(
      launchpadStatePda
    );
    assert(launchpadStateAccount.claimedAmount.eq(new BN(0)));
    assert(launchpadStateAccount.totalSold.eq(new BN(buyTokenAmount)));
    const userStakeAccount = await pg.program.account.userStake.fetch(
      userStakePda
    );
    assert(userStakeAccount.bump == userStakePdaBump);
    assert(userStakeAccount.hasClaimedTokens == false);
    assert(userStakeAccount.isInitialized == true);
    assert(userStakeAccount.invests.eq(new BN(tokenPrice * buyTokenAmount)));
    assert(userStakeAccount.purchased.eq(new BN(buyTokenAmount)));
    const paymentTokenAccount = await getAccount(
      pg.connection,
      paymentTokenAccountPda
    );
    assert(paymentTokenAccount.amount == BigInt(buyTokenAmount));

    const userPaymentTokenAccountInfo = await getAccount(
      pg.connection,
      userPaymentTokenAccount
    );
    assert(
      userPaymentTokenAccountInfo.amount == BigInt(100_000 - buyTokenAmount)
    );
  });
});
