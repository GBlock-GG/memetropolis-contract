import { Program, web3, BN } from "@coral-xyz/anchor";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID } from "@solana/spl-token";

export const buy = async (
    program: Program,
    payer: web3.Keypair,
    buyAmount: BN,
    maxSolAmount: BN,
    tokenMint: PublicKey,
    mintAuthority: PublicKey,
    config: PublicKey,
    bondingCurve: PublicKey,
    associtedBondingCurve: PublicKey,
    associtedUserTokenAccount: PublicKey,
) => {    
    const tx = await program.methods.buy(
      buyAmount,
      maxSolAmount
    ).accounts({
      tokenMint, 
      mintAuthority,
      config,
      bondingCurve,
      associtedBondingCurve,
      associtedUserTokenAccount,
      user: payer.publicKey,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId
    }).signers([payer]).rpc()
    console.log('buy Sig:', tx)
    return tx;
}
