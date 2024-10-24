use crate::*;
use anchor_lang::solana_program;
use anchor_spl::{
    associated_token::{get_associated_token_address_with_program_id, ID as ASSOCIATED_TOKEN_ID},
    token_interface::Mint,
};
use oapp::endpoint_cpi::LzAccount;

#[derive(Accounts)]
pub struct LzReceiveTypes<'info> {
    #[account(
        seeds = [OAPP_SEED],
        bump = oapp_config.bump
    )]
    pub oapp_config: Box<Account<'info, OAppConfig>>,
}

// account structure
// account 0 - payer (executor)
// account 1 - peer
// account 2 - oft config
// // account 3 - token escrow (optional)
// account 4 - to address / wallet address
// account 5 - token dest
// account 6 - token mint
// account 7 - token program
// account 8 - associated token program
// account 9 - system program
// account 10 - event authority
// account 11 - this program
// account remaining accounts
//  0..9 - accounts for clear
//  9..16 - accounts for compose
impl LzReceiveTypes<'_> {
    pub fn apply(
        ctx: &Context<LzReceiveTypes>,
        params: &LzReceiveParams,
    ) -> Result<Vec<LzAccount>> {
        let oapp_info = &ctx.accounts.oapp_config;

        let (peer, _) = Pubkey::find_program_address(
            &[PEER_SEED, &oapp_info.key().to_bytes(), &params.src_eid.to_be_bytes()],
            ctx.program_id,
        );

        // account 0..1
        let mut accounts = vec![
            LzAccount { pubkey: Pubkey::default(), is_signer: true, is_writable: true }, // 0
            LzAccount { pubkey: peer, is_signer: false, is_writable: true },             // 1
        ];

        // account 2..3
        let (oapp_config, _) = Pubkey::find_program_address(
            &[OAPP_SEED],
            ctx.program_id,
        );
        // let token_escrow = if let OftConfigExt::Adapter(token_escrow) = oft.ext {
        //     LzAccount { pubkey: token_escrow, is_signer: false, is_writable: true }
        // } else {
        //     LzAccount { pubkey: ctx.program_id.key(), is_signer: false, is_writable: false }
        // };
        accounts.extend_from_slice(&[
            LzAccount { pubkey: oapp_config, is_signer: false, is_writable: false }, // 2
        ]);

        // account 4..8
        let token_mint = Pubkey::from(msg_codec::get_meme_addr(&params.message));
        let to_address = Pubkey::from(msg_codec::get_receipt_addr(&params.message));
        accounts.extend_from_slice(&[
            LzAccount { pubkey: token_mint, is_signer: false, is_writable: false }, // 4
            LzAccount { pubkey: to_address, is_signer: false, is_writable: true }, // 5
        ]);

        // account 9..11
        let (event_authority_account, _) =
            Pubkey::find_program_address(&[oapp::endpoint_cpi::EVENT_SEED], &ctx.program_id);
        accounts.extend_from_slice(&[
            LzAccount {
                pubkey: solana_program::system_program::ID,
                is_signer: false,
                is_writable: false,
            }, // 9
            LzAccount { pubkey: event_authority_account, is_signer: false, is_writable: false }, // 10
            LzAccount { pubkey: ctx.program_id.key(), is_signer: false, is_writable: false }, // 11
        ]);

        let endpoint_program = ctx.accounts.oapp_config.endpoint_program;
        // remaining accounts 0..9
        let accounts_for_clear = oapp::endpoint_cpi::get_accounts_for_clear(
            endpoint_program,
            &oapp_info.key(),
            params.src_eid,
            &params.sender,
            params.nonce,
        );
        accounts.extend(accounts_for_clear);


        Ok(accounts)
    }
}
