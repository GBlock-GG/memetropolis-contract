import { Connection, Signer, Keypair } from "@solana/web3.js";
import {
  createMint,
} from "@solana/spl-token";

export async function createTokenMint(
	connection: Connection,
	payer: Signer,
	keypair: Keypair
) {
	await createMint(
		connection,
		payer,
		payer.publicKey,
		payer.publicKey,
		9,
		keypair
	);
}

export async function getBlockTimestamp(connection: Connection): Promise<number> {
  let slot = await connection.getSlot();
	const blockTime = await connection.getBlockTime(slot);
  return blockTime?? 0;
}

export async function sleep(ms) {
	return new Promise(resolve => setTimeout(resolve, ms));
}