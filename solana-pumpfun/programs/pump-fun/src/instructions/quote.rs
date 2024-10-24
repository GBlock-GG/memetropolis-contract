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
            &oapp_config.key().to_bytes(),
            &params.dst_eid.to_be_bytes()
        ],
        bump = peer.bump
    )]
    pub peer: Account<'info, Peer>,
    #[account(
        seeds = [
            ENFORCED_OPTIONS_SEED,
            &oapp_config.key().to_bytes(),
            &params.dst_eid.to_be_bytes()
        ],
        bump = enforced_options.bump
    )]
    pub enforced_options: Account<'info, EnforcedOptions>,
    // #[account(address = oapp_config.token_mint)]
    // pub token_mint: InterfaceAccount<'info, anchor_spl::token_interface::Mint>,
}

impl Quote<'_> {
    pub fn apply(ctx: &Context<Quote>, params: &QuoteParams) -> Result<MessagingFee> {
        // 1. Quote the amount with token2022 fee and dedust it
        // let amount_received_ld = ctx.accounts.oapp_config.remove_dust(params.amount_ld);
        // require!(amount_received_ld >= params.min_amount_ld, OftError::SlippageExceeded);

        // calling endpoint cpi
        oapp::endpoint_cpi::quote(
            ctx.accounts.oapp_config.endpoint_program,
            ctx.remaining_accounts,
            EndpointQuoteParams {
                sender: ctx.accounts.oapp_config.key(),
                dst_eid: params.dst_eid,
                receiver: ctx.accounts.peer.address,
                message: msg_codec::encode(
                    params.to,
                    u64::default(),
                    Pubkey::default(),
                    &params.compose_msg,
                ),
                pay_in_lz_token: params.pay_in_lz_token,
                options: ctx
                    .accounts
                    .enforced_options
                    .combine_options(&params.compose_msg, &params.options)?,
            },
        )
    }
}


#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct QuoteParams {
    pub dst_eid: u32,
    pub to: [u8; 32],
    pub amount_ld: u64,
    pub min_amount_ld: u64,
    pub options: Vec<u8>,
    pub compose_msg: Option<Vec<u8>>,
    pub pay_in_lz_token: bool,
}
