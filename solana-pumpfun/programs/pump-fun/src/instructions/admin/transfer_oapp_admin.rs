use crate::*;

#[derive(Accounts)]
pub struct TransferOAppAdmin<'info> {
    pub admin: Signer<'info>,
    #[account(
        mut,
        seeds = [OAPP_SEED],
        bump = oapp_config.bump,
        has_one = admin @OftError::Unauthorized
    )]
    pub oapp_config: Account<'info, OAppConfig>,
}

impl TransferOAppAdmin<'_> {
    pub fn apply(ctx: &mut Context<TransferOAppAdmin>, params: &TransferOAppAdminParams) -> Result<()> {
        ctx.accounts.oapp_config.admin = params.admin;
        Ok(())
    }
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct TransferOAppAdminParams {
    pub admin: Pubkey,
}
