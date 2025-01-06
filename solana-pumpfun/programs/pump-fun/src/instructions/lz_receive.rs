use crate::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};
use anchor_lang::prelude::Rent;
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
    pub bonding_curve: Box<Account<'info, BondingCurve>>,

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
    #[account(
        init_if_needed,
        seeds = [
            USER_CONF_SEED,
            token_mint.key().as_ref(),
            to_address.key().as_ref(),
        ],
        payer = payer,
        bump,
        space = 8 + UserConf::INIT_SPACE
    )]
    pub user_conf: Box<Account<'info, UserConf>>,
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
            require!(
                ctx.accounts.bonding_curve.launch_date <= Clock::get()?.unix_timestamp.try_into().unwrap(),
                PumpFunError::TokenNotLaunched,
            );
            // check to ensure funding goal is not met
            let init_supply = ctx.accounts.bonding_curve.liquidity_pool_ratio * ctx.accounts.bonding_curve.max_supply / 10000;
            require!(
                ctx.accounts.associted_bonding_curve.amount > init_supply,
                PumpFunError::AlreadyRaised
            );
            let token_total_supply = (1000 - ctx.accounts.bonding_curve.reserved_ratio) * ctx.accounts.bonding_curve.max_supply / 10000;
            let current_supply = token_total_supply - ctx.accounts.associted_bonding_curve.amount;

            let rent = Rent::get()?;
            let token_account_size = 165; // SPL Token account size in bytes
            let rent_exemption = rent.minimum_balance(token_account_size);

            let sol = sol_amount - rent_exemption;  //fee to create tokenAccount
            let token_amount_to_purchased = calculate_token_amount(current_supply, sol, decimals, ctx.accounts.bonding_curve.k, ctx.accounts.bonding_curve.initial_price);
            let available_qty =
                ctx.accounts.associted_bonding_curve.amount - init_supply;

            require!(token_amount_to_purchased <= available_qty, PumpFunError::NotEnoughSuppply);
            if ctx.accounts.bonding_curve.maximum_per_user > 0 {
                require!(
                  ctx.accounts.user_conf.bought_amount + sol <= ctx.accounts.bonding_curve.maximum_per_user,
                  PumpFunError::UserBuyLimitExceed
                )
            }

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
            ctx.accounts.user_conf.bought_amount += sol;
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