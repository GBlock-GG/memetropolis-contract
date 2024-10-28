use crate::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};
use oapp::endpoint::{
    cpi::accounts::Clear,
    instructions::ClearParams,
    ConstructCPIContext,
};

#[event_cpi]
#[derive(Accounts)]
#[instruction(params: LzReceiveParams)]
pub struct LzReceive<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        seeds = [
            PEER_SEED,
            &oapp_config.key().to_bytes(),
            &params.src_eid.to_be_bytes()
        ],
        bump = peer.bump,
        constraint = peer.address == params.sender @OftError::InvalidSender
    )]
    pub peer: Box<Account<'info, Peer>>,
    #[account(
        seeds = [OAPP_SEED],
        bump = oapp_config.bump
    )]
    pub oapp_config: Box<Account<'info, OAppConfig>>,

    #[account(address = Pubkey::from(msg_codec::get_meme_addr(&params.message)) @OftError::InvalidTokenMint)]
    pub token_mint: InterfaceAccount<'info, Mint>,

    /// CHECK: the wallet address to receive the token
    #[account(address = Pubkey::from(msg_codec::get_receipt_addr(&params.message)) @OftError::InvalidReceiver)]
    pub to_address: AccountInfo<'info>,

    /// CHECK: token vault address
    #[account(
        mut,
        seeds = [
            BONDING_CURVE_SEED,
            token_mint.key().as_ref()
        ],
        bump,
    )]
    pub bonding_curve: UncheckedAccount<'info>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = bonding_curve,
        token::token_program = token_program,
    )]
    pub associted_bonding_curve: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = token_mint,
        associated_token::authority = to_address,
        token::token_program = token_program,
      )]
      pub associted_user_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl LzReceive<'_> {
    pub fn apply(ctx: &mut Context<LzReceive>, params: &LzReceiveParams) -> Result<()> {
        let seeds: &[&[u8]] =
            &[OAPP_SEED, &[ctx.accounts.oapp_config.bump]];

        let accounts_for_clear = &ctx.remaining_accounts[0..Clear::MIN_ACCOUNTS_LEN];
        let _ = oapp::endpoint_cpi::clear(
            ctx.accounts.oapp_config.endpoint_program,
            ctx.accounts.oapp_config.key(),
            accounts_for_clear,
            seeds,
            ClearParams {
                receiver: ctx.accounts.oapp_config.key(),
                src_eid: params.src_eid,
                sender: params.sender,
                nonce: params.nonce,
                guid: params.guid,
                message: params.message.clone(),
            },
        )?;

        let is_buy = msg_codec::is_buy_token(&params.message);
        let sol_amount = msg_codec::get_sol_amount(&params.message);
        let decimals = 9;
        if is_buy {
            // check to ensure funding goal is not met
            require!(
                ctx.accounts.associted_bonding_curve.amount > INIT_SUPPLY,
                PumpFunError::AlreadyRaised
            );
            let current_supply =
                MAX_SUPPLY - ctx.accounts.associted_bonding_curve.amount;
            let sol = sol_amount - 4000000;  //fee to create tokenAccount
            let token_amount_to_purchased = calculate_token_amount(current_supply, sol, decimals);
            let available_qty =
                ctx.accounts.associted_bonding_curve.amount - INIT_SUPPLY;

            require!(token_amount_to_purchased <= available_qty, PumpFunError::NotEnoughSuppply);

            //transfer sol to vault
            transfer_sol(
                ctx.accounts.payer.to_account_info(),
                ctx.accounts.bonding_curve.to_account_info(),
                sol,
            )?;
            //transfer fee

            //transfer token from vault to user
            let token_mint = ctx.accounts.token_mint.key();
            let vault_seeds = &[
                BONDING_CURVE_SEED,
                token_mint.as_ref(),
                &[ctx.bumps.bonding_curve],
            ];
            let vault_signer_seeds = &[&vault_seeds[..]];

            transfer_token_from_vault_to_user(
                ctx.accounts.bonding_curve.to_account_info(),
                ctx.accounts.associted_bonding_curve.to_account_info(),
                ctx.accounts.associted_user_token_account.to_account_info(),
                ctx.accounts.token_mint.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
                token_amount_to_purchased,
                decimals,
                vault_signer_seeds,
            )?;
            emit!(BuyEvent {
                mint: ctx.accounts.token_mint.key(),
                token_output: token_amount_to_purchased,
                sol_input: sol,
                buyer: ctx.accounts.to_address.key()
            });
        }
        Ok(())
    }
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct LzReceiveParams {
    pub src_eid: u32,
    pub sender: [u8; 32],
    pub nonce: u64,
    pub guid: [u8; 32],
    pub message: Vec<u8>,
    pub extra_data: Vec<u8>,
}