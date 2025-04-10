use crate::*;
use oapp::endpoint::{instructions::QuoteParams as EndpointQuoteParams, MessagingFee};

#[derive(Accounts)]
#[instruction(params: QuoteParams)]
pub struct Quote<'info> {
  #[account(
    seeds = [OAPP_SEED],
    bump = oapp_config.bump
  )]
  pub oapp_config: Account<'info, OAppConfig>,
  #[account(
    seeds = [
      PEER_SEED,
      oapp_config.key().as_ref(),
      &params.dst_eid.to_be_bytes()
    ],
    bump = peer.bump
  )]
  pub peer: Account<'info, Peer>,
}

impl Quote<'_> {
  pub fn apply(ctx: &Context<Quote>, params: &QuoteParams) -> Result<MessagingFee> {
    // calling endpoint cpi
    oapp::endpoint_cpi::quote(
      ctx.accounts.oapp_config.endpoint_program,
      ctx.remaining_accounts,
      EndpointQuoteParams {
        sender: ctx.accounts.oapp_config.key(),
        dst_eid: params.dst_eid,
        receiver: ctx.accounts.peer.address,
        message: msg_codec::encode(
          params.msg_type,
          params.token_addr,
          params.to_addr,
          params.eth_amount,
          params.token_amount,
          Pubkey::default(),
          &params.compose_msg,
        ),
        pay_in_lz_token: params.pay_in_lz_token,
        options: ctx
          .accounts
          .peer
          .enforced_options
          .combine_options(&params.compose_msg, &params.options)?,
      },
    )
  }
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct QuoteParams {
  pub dst_eid: u32,
  pub msg_type: u8, // 1: buy, 2: sell
  pub token_addr: [u8;32],
  pub to_addr: [u8; 32],
  pub eth_amount: u128,  //16 bytes
  pub token_amount: u128, //32 bytes
  pub options: Vec<u8>,
  pub compose_msg: Option<Vec<u8>>,
  pub pay_in_lz_token: bool,
}
