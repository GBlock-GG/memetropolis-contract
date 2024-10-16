import { Program, web3, BN } from "@coral-xyz/anchor";
import { getAssociatedTokenAddressSync, ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { MPL_TOKEN_METADATA_PROGRAM_ID  } from "@metaplex-foundation/mpl-token-metadata";
import { PublicKey } from "@solana/web3.js";

export const createToken = async (
  program: Program,
  payer: web3.Keypair,
  configPk: web3.PublicKey,
  tokenName: string,
  tokenSymbol: string,
  tokenUri: string,
):Promise<{
  tokenMint: PublicKey,
  mintAuthorityPk: PublicKey,
  bondingCurve: PublicKey,
  associtedBondingCurve: PublicKey,
  associtedUserTokenAccount: PublicKey,
  metadataPDA: PublicKey,
}> => {
  const tokenMintKP = web3.Keypair.generate();
  const [ mintAuthorityPk ] = web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from("pumpfun_mint_authority"),
      configPk.toBuffer()
    ],
    program.programId
  )

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

  const tx = await program.methods.createToken(
    tokenName,
    tokenSymbol,
    tokenUri,
  ).accounts({
    tokenMint: tokenMintKP.publicKey,
    mintAuthority: mintAuthorityPk,
    config: configPk,
    bondingCurve,
    associtedBondingCurve,
    associtedUserTokenAccount,
    metadata: metadataPDA,
    user: payer.publicKey,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    tokenProgram: TOKEN_PROGRAM_ID,
    tokenMetadataProgram: metadata_program_id,
    rent: web3.SYSVAR_RENT_PUBKEY,
    systemProgram: web3.SystemProgram.programId
  }).signers([tokenMintKP, payer]).rpc()
  console.log('createToken Sig:', tx)
  return {
    tokenMint: tokenMintKP.publicKey,
    mintAuthorityPk,
    bondingCurve,
    associtedBondingCurve,
    associtedUserTokenAccount,
    metadataPDA
  }
}