
import { Program, web3, BN } from "@coral-xyz/anchor";

export const createConfig = async (
    program: Program,
    payer: web3.Keypair,
    maxSupply: BN,
    initSupply: BN,
    defaultDecimals: number
) => {
    const fee_receipt_kp = web3.Keypair.generate();
    
    const authorityPk = payer.publicKey; //admin
    const[ configPk ] = web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("pumpfun_config"),
        authorityPk.toBuffer()
      ],
      program.programId
    );


    const tx = await program.methods.createConfig(
      fee_receipt_kp.publicKey,  //fee_receipt
      maxSupply, //max_supply
      initSupply, //init_supply
      defaultDecimals, //default_decimals
    ).accounts({
      authority: authorityPk,
      config: configPk,
      systemProgram: web3.SystemProgram.programId,
    }).signers([])
    .rpc();

    console.log("Your transaction signature:", tx);
    console.log("configPk:", configPk.toBase58())
    return {
        fee_receipt_kp,
        configPk,
    }
}