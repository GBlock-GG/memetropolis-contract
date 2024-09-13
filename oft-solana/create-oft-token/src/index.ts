import { OftTools } from '@layerzerolabs/lz-solana-sdk-v2'
import bs58 from 'bs58'
import * as dotenv from "dotenv"
dotenv.config()

import { Keypair, PublicKey, SystemProgram, Connection, Transaction } from '@solana/web3.js'
import { getMintLen, TOKEN_PROGRAM_ID, createInitializeMintInstruction } from '@solana/spl-token'

const OFT_SEED = 'Oft'
const OFT_PROGRAM_ID = process.env.OFT_PROGRAM_ID + ''
const PAYER_PRIV_KEY = process.env.PAYER_PRIV_KEY + ''
const RPC_URL = process.env.RPC_URL + ''
const SOLANA_OFT_TOKEN_DECIMALS = Number(process.env.OFT_TOKEN_DECIMALS)
const ENDPOINT_PROGRAM_ID = '76y77prsiCMvXMjuoZ5VRrhG5qYBrUMYTE5WgHqgjEn6'
const main = async ()=>{
  const mintKp = Keypair.generate()
  const payer = Keypair.fromSecretKey(bs58.decode(PAYER_PRIV_KEY))

  const [oftConfigPda] = PublicKey.findProgramAddressSync(
    [Buffer.from(OFT_SEED, 'utf8'), mintKp.publicKey.toBuffer()],
    new PublicKey(OFT_PROGRAM_ID)
  )

  const connection = new Connection(RPC_URL, "confirmed")

  // step 1, create the mint token
  const createMintIxs = [
    SystemProgram.createAccount({
        fromPubkey: payer.publicKey,
        newAccountPubkey: mintKp.publicKey,
        space: getMintLen([]),
        lamports: await connection.getMinimumBalanceForRentExemption(getMintLen([])),
        programId: TOKEN_PROGRAM_ID,
    }),
    createInitializeMintInstruction(mintKp.publicKey, SOLANA_OFT_TOKEN_DECIMALS, oftConfigPda, oftConfigPda),
  ]

  // step 2, create the OFT token
  const initOftIx = await OftTools.createInitNativeOftIx(
    new PublicKey(OFT_PROGRAM_ID),
    payer.publicKey,
    payer.publicKey,
    mintKp.publicKey,
    payer.publicKey,
    SOLANA_OFT_TOKEN_DECIMALS,
    new PublicKey(ENDPOINT_PROGRAM_ID),
    TOKEN_PROGRAM_ID
  )
  await connection.sendTransaction(new Transaction().add(initOftIx), [payer])

}

main().catch(e => {
    console.error(e)
})