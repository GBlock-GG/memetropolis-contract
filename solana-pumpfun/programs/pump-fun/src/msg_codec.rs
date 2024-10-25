
const MEME_TOKEN_ADDRESS_OFFSET: usize = 1;
const RECEIPT_ADDRESS_OFFSET: usize = 33;
const SOL_AMOUNT_OFFSET: usize = 65;
const TOKEN_AMOUNT_OFFSET: usize = 73;

// pub fn encode(
//     send_to: [u8; 32],
//     amount_sd: u64,
//     sender: Pubkey,
//     compose_msg: &Option<Vec<u8>>,
// ) -> Vec<u8> {
//     if let Some(msg) = compose_msg {
//         let mut encoded = Vec::with_capacity(72 + msg.len()); // 32 + 8 + 32
//         encoded.extend_from_slice(&send_to);
//         encoded.extend_from_slice(&amount_sd.to_be_bytes());
//         encoded.extend_from_slice(sender.to_bytes().as_ref());
//         encoded.extend_from_slice(&msg);
//         encoded
//     } else {
//         let mut encoded = Vec::with_capacity(40); // 32 + 8
//         encoded.extend_from_slice(&send_to);
//         encoded.extend_from_slice(&amount_sd.to_be_bytes());
//         encoded
//     }
// }

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