// No imports needed: web3, anchor, pg and more are globally available
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";

import { Connection, Signer, Keypair } from "@solana/web3.js";
import {
  createMint,
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
  it("initialize", async () => {
    const minInvest = 1000;
    const maxInvest = 1000000;
    const tokenPrice = 10;

    const currentBlockTime = await getBlockTimestamp(pg.connection);

    const startTime = currentBlockTime + 20;
    const endTime = currentBlockTime + 1000;

    console.log("CurrentBlockTime:", currentBlockTime);

    //create meme Token Mint
    const memeMintKp = new web3.Keypair();
    await createTokenMint(pg.connection, pg.wallet.keypair, memeMintKp);
    //create payment Token Mint
    const paymentMintKp = new web3.Keypair();
    await createTokenMint(pg.connection, pg.wallet.keypair, paymentMintKp);

    console.log("MemeMint:", memeMintKp.publicKey.toBase58());
    console.log("paymentMint:", paymentMintKp.publicKey.toBase58());

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

    // Fetch the created account
    const launchpadStateAccount = await pg.program.account.launchpadState.fetch(
      launchpadStatePda
    );

    assert(
      launchpadStateAccount.admin.toBase58() == pg.wallet.publicKey.toBase58()
    );
    assert(launchpadStateAccount.bump == bump);
    assert(launchpadStateAccount.startTime.eq(new BN(startTime)));
    assert(launchpadStateAccount.endTime.eq(new BN(endTime)));
    assert(launchpadStateAccount.minInvest.eq(new BN(minInvest)));
    assert(launchpadStateAccount.maxInvest.eq(new BN(maxInvest)));
    assert(launchpadStateAccount.tokenPrice.eq(new BN(tokenPrice)));

    assert(
      launchpadStateAccount.memeMint.toBase58() ==
        memeMintKp.publicKey.toBase58()
    );
    assert(
      launchpadStateAccount.paymentMint.toBase58() ==
        paymentMintKp.publicKey.toBase58()
    );
    assert(launchpadStateAccount.claimedAmount.eq(new BN(0)));
    assert(launchpadStateAccount.totalSold.eq(new BN(0)));
  });
});
