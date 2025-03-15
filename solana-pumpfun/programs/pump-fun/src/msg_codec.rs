
use crate::*;

const MEME_TOKEN_ADDRESS_OFFSET: usize = 1;
const RECEIPT_ADDRESS_OFFSET: usize = 33;
const SOL_AMOUNT_OFFSET: usize = 65;
const TOKEN_AMOUNT_OFFSET: usize = 81;

// make msg to cross chain
pub fn encode(
  msg_type: u8, // buy: 1, sell: 2
  meme_token_addr: [u8; 32],
  to_addr: [u8; 32],
  eth_amount: u128, //16 byte
  token_amount: u128, // 16 byte,  evm contract has 32 bytes, so increase 16bytes more
  sender: Pubkey,
  compose_msg: &Option<Vec<u8>>,
) -> Vec<u8> {
  if let Some(msg) = compose_msg {
      let mut encoded = Vec::with_capacity(145 + msg.len()); // 1 + 32 + 32 + 16 + 32 + 32
      encoded.extend_from_slice(&(vec![msg_type]));
      encoded.extend_from_slice(&meme_token_addr);
      encoded.extend_from_slice(&to_addr);
      encoded.extend_from_slice(&eth_amount.to_be_bytes());
      //32 bytes for token_amount
      encoded.extend_from_slice(&vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]);
      encoded.extend_from_slice(&token_amount.to_be_bytes());
      encoded.extend_from_slice(sender.to_bytes().as_ref());
      encoded.extend_from_slice(&msg);
      encoded
  } else {
      let mut encoded = Vec::with_capacity(113); // 1 + 32 + 32 + 16 + 32
      encoded.extend_from_slice(&(vec![msg_type]));
      encoded.extend_from_slice(&meme_token_addr);
      encoded.extend_from_slice(&to_addr);
      encoded.extend_from_slice(&eth_amount.to_be_bytes());
      //32 bytes for token_amount
      encoded.extend_from_slice(&vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]);
      encoded.extend_from_slice(&token_amount.to_be_bytes());
      encoded
  }
}

pub fn is_buy_token(message:&[u8]) -> bool {
    message[0] == 1
}

pub fn get_meme_addr(message: &[u8]) -> [u8; 32] {
    let mut meme_addr = [0; 32];
    meme_addr.copy_from_slice(&message[MEME_TOKEN_ADDRESS_OFFSET..RECEIPT_ADDRESS_OFFSET]);
    meme_addr
}

pub fn get_receipt_addr(message: &[u8]) -> [u8; 32] {
    let mut receipt_addr = [0; 32];
    receipt_addr.copy_from_slice(&message[RECEIPT_ADDRESS_OFFSET..SOL_AMOUNT_OFFSET]);
    receipt_addr
}

pub fn get_sol_amount(message: &[u8]) -> u64 {
    let mut sol_amount = [0; 8];
    sol_amount.copy_from_slice(&message[SOL_AMOUNT_OFFSET+8..TOKEN_AMOUNT_OFFSET]);
    u64::from_be_bytes(sol_amount)
}

// pub fn get_token_amount(message: &[u8]) -> u64 {
//     let mut token_amount = [0; 8];
//     token_amount.copy_from_slice(&message[TOKEN_AMOUNT_OFFSET..]);
//     u64::from_be_bytes(token_amount)
// }