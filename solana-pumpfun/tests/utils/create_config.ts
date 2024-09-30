
import { Program, BN } from "@coral-xyz/anchor";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
export const createConfig = async (
    program: Program,
    payer: Keypair,
    maxSupply: BN,
    initSupply: BN,
    defaultDecimals: number
): Promise<{
  feeReceiptPk: PublicKey,
  configPk: PublicKey,
}> => {
    const feeReceiptKp = Keypair.generate();
    
    const authorityPk = payer.publicKey; //admin
    const[ configPk ] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("pumpfun_config"),
        authorityPk.toBuffer()
      ],
      program.programId
    );


    const tx = await program.methods.createConfig(
      feeReceiptKp.publicKey,  //fee_receipt
      maxSupply, //max_supply
      initSupply, //init_supply
      defaultDecimals, //default_decimals
    ).accounts({
      authority: authorityPk,
      config: configPk,
      systemProgram: SystemProgram.programId,
    }).signers([])
    .rpc();

    console.log("Your transaction signature:", tx);
    console.log("configPk:", configPk.toBase58())
    return {
      feeReceiptPk: feeReceiptKp.publicKey,
      configPk,
    }
}