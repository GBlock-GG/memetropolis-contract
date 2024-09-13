import { assert } from "chai";
import { getBlockTimestamp, createTokenMint, sleep } from "./utils";
import * as anchor from "@coral-xyz/anchor";
import { Program, web3, BN } from "@coral-xyz/anchor";
import { IdoLaunchpad } from "../target/types/ido_launchpad";
import { TOKEN_PROGRAM_ID, createAssociatedTokenAccount, mintTo, getAccount } from "@solana/spl-token";

describe("buy test", async () => {
    // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const owner = anchor.Wallet.local().payer;
  const program = anchor.workspace.IdoLaunchpad as Program<IdoLaunchpad>;
  const connection = anchor.getProvider().connection;

    it("buy_tokens", async () => {
      const minInvest = 10;
      const maxInvest = 1000000;
      const tokenPrice = 10;
  
      const currentBlockTime = await getBlockTimestamp(connection);
  
      const startTime = currentBlockTime + 10;
      const endTime = currentBlockTime + 25;
  
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
          systemProgram: web3.SystemProgram.programId,
        })
        .signers([])
        .rpc();
      // Confirm transaction
      await connection.confirmTransaction(txHash);
  
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
      // Confirm transaction
      await connection.confirmTransaction(txHash1);
  
      const launchpadStateAccount = await program.account.launchpadState.fetch(
        launchpadStatePda
      );
      assert(launchpadStateAccount.claimedAmount.eq(new BN(0)));
      assert(launchpadStateAccount.totalSold.eq(new BN(buyTokenAmount)));
      const userStakeAccount = await program.account.userStake.fetch(
        userStakePda
      );
      assert(userStakeAccount.bump == userStakePdaBump);
      assert(userStakeAccount.hasClaimedTokens == false);
      assert(userStakeAccount.isInitialized == true);
      assert(userStakeAccount.invests.eq(new BN(tokenPrice * buyTokenAmount)));
      assert(userStakeAccount.purchased.eq(new BN(buyTokenAmount)));
      const paymentTokenAccount = await getAccount(
        connection,
        paymentTokenAccountPda
      );
      assert(paymentTokenAccount.amount == BigInt(buyTokenAmount));
  
      const userPaymentTokenAccountInfo = await getAccount(
        connection,
        userPaymentTokenAccount
      );
      assert(
        userPaymentTokenAccountInfo.amount == BigInt(100_000 - buyTokenAmount)
      );
    });
  });
  