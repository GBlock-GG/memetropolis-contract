import { Program, web3, BN } from "@coral-xyz/anchor";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID } from "@solana/spl-token";

export const withdraw = async (
    program: Program,
    payer: web3.Keypair,
    tokenMint: PublicKey,
    bondingCurve: PublicKey,
    associtedBondingCurve: PublicKey,
    associtedUserTokenAccount: PublicKey,
) => {    
    const tx = await program.methods.withdraw().accounts({
      authority: payer.publicKey,
      tokenMint, 
      bondingCurve,
      associtedBondingCurve,
      associtedUserTokenAccount, //receive
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId
    }).signers([payer]).rpc()
    console.log('withdraw Sig:', tx)
    return tx;
}
