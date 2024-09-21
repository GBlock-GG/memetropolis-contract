import * as anchor from "@coral-xyz/anchor";
import { Program, web3, BN } from "@coral-xyz/anchor";
import { PumpFun } from "../target/types/pump_fun";
import { MPL_TOKEN_METADATA_PROGRAM_ID, safeFetchAllDeprecatedMasterEditionV1  } from "@metaplex-foundation/mpl-token-metadata";
import { SystemProgram, Transaction, sendAndConfirmTransaction } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID, getAssociatedTokenAddressSync } from "@solana/spl-token";

import { assert } from "chai";
import { createConfig } from "./utils/create_config";
import { sleep } from "./utils/sleep";

describe("CreateToken", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.PumpFun as Program<PumpFun>;
  const payer = anchor.Wallet.local().payer;
  const connection = anchor.getProvider().connection;

  it("Create Token", async () => {
    // create Config Account
    const maxSupply = new BN(1_000_000);
    const initSupply = new BN(200_000);
    const defaultDecimals = 6;

    const { fee_receipt_kp, configPk } = await createConfig(
      program,
      payer,
      maxSupply,
      initSupply,
      defaultDecimals
    )

    // create Token Account
    const tokenMintKP = web3.Keypair.generate();
    const [ mintAuthorityPk ] = web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("pumpfun_mint_authority"),
        configPk.toBuffer()
      ],
      program.programId
    );

    // const lamports = await connection.getMinimumBalanceForRentExemption(0);
    // const txIns = SystemProgram.transfer({
    //   fromPubkey: payer.publicKey,
    //   lamports,
    //   toPubkey: mintAuthorityPk,
    // })
    
    // const tx0 = await sendAndConfirmTransaction(
    //   connection,
    //   new Transaction().add(txIns),
    //   [payer]
    // )
    // console.log("createMintAuthority:", tx0);


    const [ bondingCurve ] = web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("pumpfun_bonding_curve"),
        tokenMintKP.publicKey.toBuffer()
      ],
      program.programId
    );

    const associtedBondingCurve = getAssociatedTokenAddressSync(
      tokenMintKP.publicKey,
      bondingCurve,
      true
    );
    const metadata_program_id = new web3.PublicKey(MPL_TOKEN_METADATA_PROGRAM_ID)
    const [ metadataPDA ] = web3.PublicKey.findProgramAddressSync(
      [
          Buffer.from('metadata'),
          metadata_program_id.toBuffer(),
          tokenMintKP.publicKey.toBuffer(),
      ],
      metadata_program_id
    );

    console.log('payer:', payer.publicKey.toBase58());
    console.log('tokenMint:', tokenMintKP.publicKey.toBase58());
    console.log('mintAuthority:', mintAuthorityPk.toBase58());
    console.log('bondingCurve:', bondingCurve.toBase58());
    console.log('associtedBondingCurve:', associtedBondingCurve.toBase58());

    console.log('metadataAccount:', metadataPDA.toBase58());

    await sleep(5000);  
    const tx = await program.methods.createToken(
      "TOKEN_NAME", //token_name
      "TSYM", //symbol
      "ipfs://TOKEN_URI" // uri
    ).accounts({
      tokenMint: tokenMintKP.publicKey,
      mintAuthority: mintAuthorityPk,
      config: configPk,
      bondingCurve,
      associtedBondingCurve,
      metadata: metadataPDA,
      user: payer.publicKey,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      tokenProgram: TOKEN_PROGRAM_ID,
      tokenMetadataProgram: metadata_program_id,
      rent: web3.SYSVAR_RENT_PUBKEY,
      systemProgram: web3.SystemProgram.programId
    }).signers([tokenMintKP, payer]).rpc();

    console.log("createToken tx:", tx)

  });
});
