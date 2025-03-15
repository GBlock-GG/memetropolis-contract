use crate::*;
use oapp::endpoint::{instructions::SendParams as EndpointSendParams, MessagingReceipt};

#[event_cpi]
#[derive(Accounts)]
#[instruction(params: SendParams)]
pub struct Send<'info> {
  pub signer: Signer<'info>,
  #[account(
    mut,
    seeds = [
      PEER_SEED,
      oapp_config.key().as_ref(),
      &params.dst_eid.to_be_bytes()
    ],
    bump = peer.bump
  )]
  pub peer: Account<'info, Peer>,
  #[account(
    mut,
    seeds = [OAPP_SEED],
    bump = oapp_config.bump
  )]
  pub oapp_config: Account<'info, OAppConfig>,
}

impl Send<'_> {
  pub fn apply(
      ctx: &mut Context<Send>,
      params: &SendParams,
  ) -> Result<MessagingReceipt> {
    // send message to endpoint
    require!(
      ctx.accounts.oapp_config.key() == ctx.remaining_accounts[1].key(),
      OftError::InvalidSender
    );
    // let amount_sd = ctx.accounts.oft_store.ld2sd(amount_received_ld);
    let msg_receipt = oapp::endpoint_cpi::send(
      ctx.accounts.oapp_config.endpoint_program,
      ctx.accounts.oapp_config.key(),
      ctx.remaining_accounts,
      &[OAPP_SEED, &[ctx.accounts.oapp_config.bump]],
      EndpointSendParams {
        dst_eid: params.dst_eid,
        receiver: ctx.accounts.peer.address,
        message: msg_codec::encode(
          params.msg_type,
          params.token_addr,
          params.to_addr,
          params.eth_amount,
          params.token_amount,
          ctx.accounts.signer.key(),
          &params.compose_msg,
        ),
        options: ctx
          .accounts
          .peer
          .enforced_options
          .combine_options(&params.compose_msg, &params.options)?,
        native_fee: params.native_fee,
        lz_token_fee: params.lz_token_fee,
      },
    )?;
    Ok(msg_receipt)
  }
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct SendParams {
  pub dst_eid: u32,
  pub msg_type: u8,
  pub token_addr: [u8;32],
  pub to_addr: [u8; 32],
  pub eth_amount: u128,  //16 bytes
  pub token_amount: u128, //32 bytes
  pub options: Vec<u8>,
  pub compose_msg: Option<Vec<u8>>,
  pub native_fee: u64,
  pub lz_token_fee: u64,
}
