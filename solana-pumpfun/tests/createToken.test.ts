import * as anchor from "@coral-xyz/anchor";
import { Program, web3, BN } from "@coral-xyz/anchor";
import { PumpFun } from "../target/types/pump_fun";
import { assert } from "chai";
import { createConfig } from "./utils/create_config";
import { createToken } from "./utils/create_token";
import { fetchMetadata, mplTokenMetadata, findMetadataPda } from "@metaplex-foundation/mpl-token-metadata";
import { createUmi } from '@metaplex-foundation/umi-bundle-defaults'
import { PublicKey, publicKey } from "@metaplex-foundation/umi";

describe("CreateToken", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.PumpFun as Program<PumpFun>;
  const payer = anchor.Wallet.local().payer;
  const connection = anchor.getProvider().connection;
  const umi = createUmi(connection.rpcEndpoint).use(mplTokenMetadata())
  console.log(connection.rpcEndpoint)

  it("Create Token", async () => {
    // create Config Account
    const maxSupply = new BN(1_000_000);
    const initSupply = new BN(200_000);
    const defaultDecimals = 6;

    const { fee_receipt_kp, configPk } = await createConfig(
      program,
      payer,
      maxSupply,
      initSupply,
      defaultDecimals
    )

    const {
      tokenMint,
      mintAuthorityPk,
      bondingCurve,
      associtedBondingCurve,
      metadataPDA
    } = await createToken(
      program,
      payer,
      configPk,
      "TOKEN_NAME",
      "TSYM",
      "ipfs://TOKEN_URI"
    )   

    console.log('payer:', payer.publicKey.toBase58());
    console.log('tokenMint:', tokenMint.toBase58());
    console.log('mintAuthority:', mintAuthorityPk.toBase58());
    console.log('bondingCurve:', bondingCurve.toBase58());
    console.log('associtedBondingCurve:', associtedBondingCurve.toBase58());
    console.log('metadataAccount:', metadataPDA.toBase58());

    const metadataInfo = await connection.getAccountInfo(metadataPDA)

    const metadatapda = findMetadataPda(umi, {
      mint: publicKey(tokenMint),
    })
    console.log(metadatapda)
    const res = await fetchMetadata(umi, metadatapda)
    console.log(res)
  });
});
