import * as anchor from "@coral-xyz/anchor";
import { Program, web3, BN } from "@coral-xyz/anchor";
import { PumpFun } from "../target/types/pump_fun";
import { assert } from "chai";

import { createConfig } from "./utils/create_config";

describe("CreateConfig", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.PumpFun as Program<PumpFun>;
  const payer = anchor.Wallet.local().payer;

  it("Create Config", async () => {
    const maxSupply = new BN(1_000_000_000_000); //with_decimal
    const initSupply = new BN(200_000_000_000);  //with_decimal
    const defaultDecimals = 6;

    const { feeReceiptPk, configPk } = await createConfig(
      program,
      payer,
      maxSupply,
      initSupply,
      defaultDecimals
    )

    // Fetch the created account
    const configAccount = await program.account.config.fetch(
      configPk
    );
    assert( configAccount.authority.toBase58() === payer.publicKey.toBase58() )
    assert( configAccount.defaultDecimals === defaultDecimals )
    assert( configAccount.feeRecipient.toBase58() === feeReceiptPk.toBase58() )
    assert( configAccount.maxSupply.eq(maxSupply) )
    assert( configAccount.initSupply.eq(initSupply) )
  });
});
