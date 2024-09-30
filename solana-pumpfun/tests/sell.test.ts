import * as anchor from "@coral-xyz/anchor";
import { Program, web3, BN } from "@coral-xyz/anchor"
import { PumpFun } from "../target/types/pump_fun"
import { createConfig } from "./utils/create_config"
import { createToken } from "./utils/create_token"
import { buy } from "./utils/buy"
import { sell } from "./utils/sell"
import { getAccount, getOrCreateAssociatedTokenAccount } from "@solana/spl-token"
import { assert } from "chai";

describe("SellToken", () => {
  anchor.setProvider(anchor.AnchorProvider.env())

  const program = anchor.workspace.PumpFun as Program<PumpFun>
  const payer = anchor.Wallet.local().payer
  const connection = anchor.getProvider().connection

  it("sell", async () => {
    const maxSupply = new BN(1_000_000_000_000); //with_decimal
    const initSupply = new BN(200_000_000_000);  //with_decimal
    const defaultDecimals = 6

    const { configPk } = await createConfig(
      program,
      payer,
      maxSupply,
      initSupply,
      defaultDecimals
    )
    // create Token Account
    const {
      tokenMint,
      mintAuthorityPk,
      bondingCurve,
      associtedBondingCurve
    } = await createToken(
      program,
      payer,
      configPk,
      "TOKEN_NAME",
      "TSYM",
      "ipfs://TOKEN_URI"
    )

    const associtedUserTokenAccount = await getOrCreateAssociatedTokenAccount(connection, payer, tokenMint, payer.publicKey)    
    await buy(
      program,
      payer,
      new BN("1000000000"), //BUY AMOUNT 1000 token (decimal is 6)
      new BN("100000000"), //0.1SOL
      tokenMint,
      mintAuthorityPk,
      configPk,
      bondingCurve,
      associtedBondingCurve,
      associtedUserTokenAccount.address
    )

    const oldAssocitedUserTokenAccount = await getAccount(connection, associtedUserTokenAccount.address)
    const oldBondingCurveInfo = await connection.getAccountInfo(bondingCurve)
    const oldAssocitedBondingInfo = await getAccount(connection, associtedBondingCurve)

    const sellAmount = new BN("1000000000") //sell 100 token
    const minSolOutput = new BN("1000")
    await sell(
      program,
      payer,
      sellAmount,
      minSolOutput,
      tokenMint,
      bondingCurve,
      associtedBondingCurve,
      associtedUserTokenAccount.address
    )
    const newAssocitedUserTokenAccount = await getAccount(connection, associtedUserTokenAccount.address)      
    const newBondingCurveInfo = await connection.getAccountInfo(bondingCurve)
    const newAssocitedBondingInfo = await getAccount(connection, associtedBondingCurve)

    
    //check sol amount for vault
    assert(oldBondingCurveInfo.lamports - newBondingCurveInfo.lamports === 8000 * 1000) //initial_price * token_amount
    //check token amount for vault
    assert(BigInt(newAssocitedBondingInfo.amount - oldAssocitedBondingInfo.amount) === BigInt(sellAmount.toNumber()))
    //check token amount for user
    assert(BigInt(oldAssocitedUserTokenAccount.amount - newAssocitedUserTokenAccount.amount) === BigInt(sellAmount.toNumber()))


  })
})