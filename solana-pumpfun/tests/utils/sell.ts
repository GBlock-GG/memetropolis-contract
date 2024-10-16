import { Program, web3, BN } from "@coral-xyz/anchor";
import { PublicKey, Connection, SystemProgram, Transaction } from "@solana/web3.js";
import { getAssociatedTokenAddressSync, ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID, getOrCreateAssociatedTokenAccount } from "@solana/spl-token";

export const sell = async (
    program: Program,
    payer: web3.Keypair,
    sellAmount: BN,
    minSolOutput: BN,
    tokenMint: PublicKey,
    bondingCurve: PublicKey,
    associtedBondingCurve: PublicKey,
    associtedUserTokenAccount: PublicKey,
) => {    
    const tx = await program.methods.sell(
      sellAmount,
      minSolOutput
    ).accounts({
      tokenMint, 
      bondingCurve,
      associtedBondingCurve,
      associtedUserTokenAccount,
      user: payer.publicKey,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId
    }).signers([payer]).rpc()
    console.log('Sell Sig:', tx)
}
