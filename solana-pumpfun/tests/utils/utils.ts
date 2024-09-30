import { web3 } from "@coral-xyz/anchor"
export const getTransactionLogs = async (conn: web3.Connection, signature: string) => {
  // const rawConfig :web3.GetVersionedTransactionConfig = {
  //   maxSupportedTransactionVersion: 0,
  //   commitment: 'confirmed'
  // }
  const txDetails = await conn.getTransaction(signature, {"commitment": "confirmed"});
  console.log('---log-----')
  console.log(txDetails);
}