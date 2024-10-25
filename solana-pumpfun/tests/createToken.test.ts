import * as anchor from "@coral-xyz/anchor";
import { Program, web3, BN } from "@coral-xyz/anchor";
import { PumpFun } from "../target/types/pump_fun";
import { assert } from "chai";
import { createConfig } from "./utils/create_config";
import { createToken } from "./utils/create_token";
import { fetchMetadata, mplTokenMetadata, findMetadataPda, fetchDigitalAsset  } from "@metaplex-foundation/mpl-token-metadata";
import { createUmi } from '@metaplex-foundation/umi-bundle-defaults'
import { PublicKey, publicKey } from "@metaplex-foundation/umi";
import { getMint, getAccount } from "@solana/spl-token"

describe("CreateToken", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.PumpFun as Program<PumpFun>;
  const payer = anchor.Wallet.local().payer;
  const connection = anchor.getProvider().connection;
  const umi = createUmi(connection).use(mplTokenMetadata())

  it("Create Token", async () => {
    // create Config Account
    // const maxSupply = new BN(1_000_000);
    // const initSupply = new BN(200_000);
    // const defaultDecimals = 6;

    // const { configPk } = await createConfig(
    //   program,
    //   payer,
    //   maxSupply,
    //   initSupply,
    //   defaultDecimals
    // )

    const {
      tokenMint,
      // mintAuthorityPk,
      bondingCurve,
      associtedUserTokenAccount,
      associtedBondingCurve,
      metadataPDA
    } = await createToken(
      program,
      payer,
      // configPk,
      "AAA",
      "AAA",
      "URI123456789"
    )

    console.log('payer:', payer.publicKey.toBase58());
    console.log('tokenMint:', tokenMint.toBase58());
    // console.log('mintAuthority:', mintAuthorityPk.toBase58());
    console.log('bondingCurve:', bondingCurve.toBase58());
    console.log('associtedBondingCurve:', associtedBondingCurve.toBase58());
    console.log('metadataAccount:', metadataPDA.toBase58());

    const mint  = await getMint(connection, tokenMint)
    assert( mint.decimals === 9)
    // assert( mint.mintAuthority.toBase58() === mintAuthorityPk.toBase58())
    // assert( mint.freezeAuthority == null)

    const metadataInfo = await fetchMetadata(umi, publicKey(metadataPDA.toBase58()))
    console.log(metadataInfo)
    // assert( metadataInfo.name === "TOKEN_NAME")
    // assert( metadataInfo.symbol === "TSYM")
    // assert( metadataInfo.uri === "ipfs://TOKEN_URI")

    // const userTokenAccount = await getAccount(connection, associtedUserTokenAccount)
    // assert( userTokenAccount.amount === BigInt(0))
    // assert( userTokenAccount.mint.toBase58() === tokenMint.toBase58())
    // assert( userTokenAccount.owner.toBase58() === payer.publicKey.toBase58())

    // const associtedBondingCurveInfo = await getAccount(connection, associtedBondingCurve)
    // // assert( associtedBondingCurveInfo.amount === BigInt(maxSupply.toNumber() * (10 **defaultDecimals))) 
    // assert( associtedBondingCurveInfo.mint.toBase58() === tokenMint.toBase58())
    // assert( associtedBondingCurveInfo.owner.toBase58() === bondingCurve.toBase58())

  });
});
