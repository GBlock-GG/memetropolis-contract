import { Program, web3 } from "@coral-xyz/anchor";
import { createMint, getAssociatedTokenAddressSync, ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID,
  createAssociatedTokenAccount,
  createAssociatedTokenAccountInstruction
 } from "@solana/spl-token";
import { MPL_TOKEN_METADATA_PROGRAM_ID  } from "@metaplex-foundation/mpl-token-metadata";
import { PublicKey, SystemProgram, Transaction, sendAndConfirmTransaction } from "@solana/web3.js";
import { PumpFun } from "../../target/types/pump_fun";
import { OftTools } from '@layerzerolabs/lz-solana-sdk-v2'
export const createToken = async (
  program: Program<PumpFun>,
  payer: web3.Keypair,
  tokenName: string,
  tokenSymbol: string,
  tokenUri: string,
):Promise<{
  tokenMint: PublicKey,
  // mintAuthorityPk: PublicKey,
  bondingCurve: PublicKey,
  associtedBondingCurve: PublicKey,
  associtedUserTokenAccount: PublicKey,
  metadataPDA: PublicKey,
}> => {
  const tokenMintKP = web3.Keypair.generate();
  // const [ mintAuthorityPk ] = web3.PublicKey.findProgramAddressSync(
  //   [
  //     Buffer.from("pumpfun_mint_authority"),
  //     configPk.toBuffer()
  //   ],
  //   program.programId
  // )

  const [ bondingCurve ] = web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from("pumpfun_bonding_curve"),
      tokenMintKP.publicKey.toBuffer()
    ],
    program.programId
  )

  const associtedBondingCurve = getAssociatedTokenAddressSync(
    tokenMintKP.publicKey,
    bondingCurve,
    true
  )

  const associtedUserTokenAccount = getAssociatedTokenAddressSync(
    tokenMintKP.publicKey,
    payer.publicKey,
  )

  const metadata_program_id = new web3.PublicKey(MPL_TOKEN_METADATA_PROGRAM_ID)
  const [ metadataPDA ] = web3.PublicKey.findProgramAddressSync(
    [
        Buffer.from('metadata'),
        metadata_program_id.toBuffer(),
        tokenMintKP.publicKey.toBuffer(),
    ],
    metadata_program_id
  );

  const [ oftConfig ] = PublicKey.findProgramAddressSync(
    [
      Buffer.from("Oft"),
      tokenMintKP.publicKey.toBuffer()
    ],
    program.programId
  )

  const [ lzReceiveTypesAccounts ] = PublicKey.findProgramAddressSync(
    [
      Buffer.from("LzReceiveTypes"),
      oftConfig.toBuffer()
    ],
    program.programId
  )
  const endpointProgram = new PublicKey("76y77prsiCMvXMjuoZ5VRrhG5qYBrUMYTE5WgHqgjEn6")

  //create token mint
  console.log('------create-token: tx0')
  const tx0 = await createMint(program.provider.connection, payer, oftConfig, null, 9, tokenMintKP)

  //create bonding_curve
  // console.log('------create bonding-curve')
  // const lamports = await program.provider.connection.getMinimumBalanceForRentExemption(0);
  // const transaction = new Transaction().add(
  //   SystemProgram.createAccount({
  //     fromPubkey:payer.publicKey,
  //     lamports,
  //     newAccountPubkey: bondingCurve,
  //     space: 0,
  //     programId: program.programId
  //   })
  // )
  // await sendAndConfirmTransaction(program.provider.connection, transaction,[payer])
  //associated boding curve
  console.log('----------Create associated bonding curve')

  const associatedToken = getAssociatedTokenAddressSync(tokenMintKP.publicKey, bondingCurve, true)

  const transaction = new Transaction().add(
      createAssociatedTokenAccountInstruction(
          payer.publicKey,
          associatedToken,
          bondingCurve,
          tokenMintKP.publicKey,
      )
  );

  await sendAndConfirmTransaction(program.provider.connection, transaction, [payer])

  console.log('--------associted_user_token_account')
  await createAssociatedTokenAccount(program.provider.connection, payer, tokenMintKP.publicKey, payer.publicKey)


  const [oappRegistry] =  PublicKey.findProgramAddressSync([Buffer.from("OApp", "utf8"), oftConfig.toBuffer()], endpointProgram)
  const [ eventAuthority] = PublicKey.findProgramAddressSync([Buffer.from("__event_authority", "utf8")], endpointProgram)
  console.log('-------createMemeToken')
  const tx = await program.methods.createToken(
    {
      name: Buffer.from(tokenName),
      symbol: Buffer.from(tokenSymbol),
      uri: Buffer.from(tokenUri),
      endpointProgram
    }
  ).accounts({
    payer: payer.publicKey,
    tokenMint: tokenMintKP.publicKey,
    oftConfig,
    // lzReceiveTypesAccounts,
    bondingCurve,
    associtedBondingCurve,
    // associtedUserTokenAccount,
    metadata: metadataPDA,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    tokenProgram: TOKEN_PROGRAM_ID,
    tokenMetadataProgram: metadata_program_id,
    rent: web3.SYSVAR_RENT_PUBKEY,
    systemProgram: web3.SystemProgram.programId,
  }).signers([payer]).rpc()

  console.log('-----Create OFT----------')
  const initOftIx = await OftTools.createInitNativeOftIx(
    program.programId,
    payer.publicKey,
    payer.publicKey,
    tokenMintKP.publicKey,
    oftConfig
  )
  const transaction1 = new Transaction()
  transaction1.add(initOftIx)
  await sendAndConfirmTransaction(program.provider.connection, transaction1, [payer])


  console.log(tx)
  return {
    tokenMint: tokenMintKP.publicKey,
    // mintAuthorityPk,
    bondingCurve,
    associtedBondingCurve,
    associtedUserTokenAccount,
    metadataPDA
  }
}