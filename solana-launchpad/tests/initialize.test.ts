import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";

import { assert } from "chai";

import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
describe("Launchpad", () => {
  it("initialize", async () => {
    const minInvest = 1000;
    const maxInvest = 1000000;
    const tokenPrice = 10;
    const startTime = 0;
    const endTime = 1000;

    const IDO_LAUNCHPAD_SEED = "ido_launchpad";

    const memeMintKp = new web3.Keypair();
    const paymentMintKp = new web3.Keypair();

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
    assert(launchpadStateAccount.admin == pg.wallet.publicKey);
    assert(launchpadStateAccount.bump == bump);
    assert(launchpadStateAccount.startTime.eq(new BN(startTime)));
    assert(launchpadStateAccount.endTime.eq(new BN(endTime)));
    assert(launchpadStateAccount.minInvest.eq(new BN(minInvest)));
    assert(launchpadStateAccount.maxInvest.eq(new BN(maxInvest)));
    assert(launchpadStateAccount.memeMint == memeMintKp.publicKey);
    assert(launchpadStateAccount.paymentMint == paymentMintKp.publicKey);
  });
});
