import * as anchor from "@coral-xyz/anchor";
import { Program, web3, BN } from "@coral-xyz/anchor";
import { IdoLaunchpad } from "../target/types/ido_launchpad";
import { getBlockTimestamp, createTokenMint } from "./utils";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { assert } from "chai";

describe("initialize test", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const owner = anchor.Wallet.local().payer;
  const program = anchor.workspace.IdoLaunchpad as Program<IdoLaunchpad>;
  const connection = anchor.getProvider().connection;

  it("initialize", async () => {
    const minInvest = 1000;
    const maxInvest = 1000000;
    const tokenPrice = 10;

    const currentBlockTime = await getBlockTimestamp(connection);

    const startTime = currentBlockTime + 20;
    const endTime = currentBlockTime + 1000;

    console.log("CurrentBlockTime:", currentBlockTime);

    //create meme Token Mint
    const memeMintKp = new web3.Keypair();
    await createTokenMint(connection, owner, memeMintKp);
    //create payment Token Mint
    const paymentMintKp = new web3.Keypair();
    await createTokenMint(connection, owner, paymentMintKp);

    console.log("MemeMint:", memeMintKp.publicKey.toBase58());
    console.log("paymentMint:", paymentMintKp.publicKey.toBase58());

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

    // Send transaction
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
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([])
      .rpc();

    // Confirm transaction
    await connection.confirmTransaction(txHash);

    // Fetch the created account
    const launchpadStateAccount = await program.account.launchpadState.fetch(
      launchpadStatePda
    );

    assert(
      launchpadStateAccount.admin.toBase58() == owner.publicKey.toBase58()
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
