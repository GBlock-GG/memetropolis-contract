import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor"
import { Connection } from "@solana/web3.js";
import { PumpFun } from "../target/types/pump_fun"
import { createConfig } from "./utils/create_config"
import { createToken } from "./utils/create_token"
import { buy } from "./utils/buy"
import { getAccount, getOrCreateAssociatedTokenAccount } from "@solana/spl-token"
import { assert } from "chai"
import { getTransactionLogs } from "./utils/utils"

describe("BuyToken", () => {
  anchor.setProvider(anchor.AnchorProvider.env())

  const program = anchor.workspace.PumpFun as Program<PumpFun>
  const payer = anchor.Wallet.local().payer
  const connection: Connection = anchor.getProvider().connection

  it("buy", async () => {
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
      associtedBondingCurve,
    } = await createToken(
      program,
      payer,
      configPk,
      "TOKEN_NAME",
      "TSYM",
      "ipfs://TOKEN_URI"
    )

    const associtedUserTokenAccount = await getOrCreateAssociatedTokenAccount(connection, payer, tokenMint, payer.publicKey)
    
    const oldAmount = associtedUserTokenAccount.amount
    const oldBondingCurveInfo = await connection.getAccountInfo(bondingCurve)
    const oldSolAmount = oldBondingCurveInfo.lamports

    const buyAmount = new BN("1000000000")
    const sig = await buy(
      program,
      payer,
      buyAmount, //BUY AMOUNT 1000 token (decimal is 6)
      new BN("100000000"), //0.1SOL
      tokenMint,
      mintAuthorityPk,
      configPk,
      bondingCurve,
      associtedBondingCurve,
      associtedUserTokenAccount.address
    )
    await connection.confirmTransaction(sig, "finalized");

    await getTransactionLogs(connection, sig)
    const userTokenAccountInfo = await getAccount(connection, associtedUserTokenAccount.address)
    const newAmount = userTokenAccountInfo.amount
    // check to increased buy amount
    assert( BigInt(newAmount - oldAmount) === BigInt(buyAmount.toNumber()))
    // check to increased sol amount
    
    const newBondingCurveInfo = await connection.getAccountInfo(bondingCurve)
    const newSolAmount = newBondingCurveInfo.lamports
    //0.008020033 SOL for 1000 token
    console.log("oldSolAmount:", oldSolAmount)
    console.log("newSolAmount:", newSolAmount)
    assert(newSolAmount - oldSolAmount === 8020033)
  })
})