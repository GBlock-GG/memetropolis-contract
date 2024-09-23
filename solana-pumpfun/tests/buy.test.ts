import * as anchor from "@coral-xyz/anchor";
import { Program, web3, BN } from "@coral-xyz/anchor"
import { PumpFun } from "../target/types/pump_fun"
import { createConfig } from "./utils/create_config"

describe("BuyToken", () => {
  anchor.setProvider(anchor.AnchorProvider.env())

  const program = anchor.workspace.PumpFun as Program<PumpFun>
  const payer = anchor.Wallet.local().payer
  const connection = anchor.getProvider().connection

  it("buy", async () => {
    const maxSupply = new BN(1_000_000)
    const initSupply = new BN(200_000)
    const defaultDecimals = 6

    const { fee_receipt_kp, configPk } = await createConfig(
      program,
      payer,
      maxSupply,
      initSupply,
      defaultDecimals
    )
    // create Token Account
    

  })
})