use crate::*;
use anchor_lang::solana_program;
use anchor_spl::{
    associated_token::{get_associated_token_address_with_program_id, ID as ASSOCIATED_TOKEN_ID},
    token::ID as TOKEN_ID,
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
// account 2 - oapp config
// account 3 - buy token_mint
// account 4 - to address / wallet address
// account 5 - bonding_curve vault
// account 6 - bonding_curve tokenAccount
// account 7 - to_address's tokenAccount
// account 8 - token program
// account 9 - associate token program
// account 10 - system program

// account 11 - event authority
// account 12 - this program
// account remaining accounts
//  0..9 - accounts for clear

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

        accounts.extend_from_slice(&[
            LzAccount { pubkey: oapp_config, is_signer: false, is_writable: false }, // 2
        ]);

        // account 3..10
        let token_mint = Pubkey::from(msg_codec::get_meme_addr(&params.message));
        let to_address = Pubkey::from(msg_codec::get_receipt_addr(&params.message));
        let (bonding_curve, _) = Pubkey::find_program_address(
            &[BONDING_CURVE_SEED, &token_mint.to_bytes()],
            ctx.program_id,
        );
        let associted_bonding_curve = get_associated_token_address_with_program_id(
            &bonding_curve,
            &token_mint,
            &TOKEN_ID,
        );
        let associted_user_token_account = get_associated_token_address_with_program_id(
            &to_address,
            &token_mint,
            &TOKEN_ID,
        );
        let (user_conf, _) = Pubkey::find_program_address(
            &[USER_CONF_SEED, &token_mint.to_bytes(), &to_address.to_bytes()],
            ctx.program_id,
        );
        accounts.extend_from_slice(&[
            LzAccount { pubkey: token_mint, is_signer: false, is_writable: false }, // 3
            LzAccount { pubkey: to_address, is_signer: false, is_writable: false }, // 4
            LzAccount { pubkey: bonding_curve, is_signer: false, is_writable: true }, // 5
            LzAccount { pubkey: associted_bonding_curve, is_signer: false, is_writable: true}, // 6
            LzAccount { pubkey: associted_user_token_account, is_signer: false, is_writable: true}, // 7
            LzAccount { pubkey: user_conf, is_signer: false, is_writable: true},
            LzAccount { pubkey: TOKEN_ID, is_signer: false, is_writable: false }, // 8
            LzAccount { pubkey: ASSOCIATED_TOKEN_ID, is_signer: false, is_writable: false }, // 9
            LzAccount { pubkey: solana_program::system_program::ID, is_signer: false, is_writable: false }, // 10
        ]);

        // account 11..
        let (event_authority_account, _) =
            Pubkey::find_program_address(&[oapp::endpoint_cpi::EVENT_SEED], &ctx.program_id);
        accounts.extend_from_slice(&[
            LzAccount { pubkey: event_authority_account, is_signer: false, is_writable: false }, // 11
            LzAccount { pubkey: ctx.program_id.key(), is_signer: false, is_writable: false }, // 12
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
