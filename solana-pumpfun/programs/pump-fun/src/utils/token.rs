use anchor_lang::{
  prelude::*,
  solana_program::{program::{invoke, invoke_signed}, system_instruction},
};

use anchor_spl::token_2022;

pub fn transfer_sol<'info>(
  sender: AccountInfo<'info>,
  from: AccountInfo<'info>,
  transfer_lamports: u64,
) -> Result<()> {
  let transfer_ins = system_instruction::transfer(
    &sender.key,
    &from.key,
    transfer_lamports,
  );
  invoke(&transfer_ins, &[
    sender.to_account_info(),
    from.to_account_info(),
  ])?;
  Ok(())
}

pub fn transfer_sol_from_vault_to_user<'info>(
  sender: AccountInfo<'info>,
  to: AccountInfo<'info>,
  transfer_lamports: u64,
  signer_seeds: &[&[&[u8]]]
) -> Result<()> {
  let transfer_ins = system_instruction::transfer(
    &sender.key,
    &to.key,
    transfer_lamports,
  );
  invoke_signed(
    &transfer_ins,
    &[
      sender.to_account_info(),
      to.to_account_info(),
    ],
    signer_seeds,
  )?;
  Ok(())
}

pub fn transfer_token_from_user_to_vault<'info>(
  authority: AccountInfo<'info>,
  sender: AccountInfo<'info>,
  to: AccountInfo<'info>,
  mint: AccountInfo<'info>,
  token_program: AccountInfo<'info>,
  amount: u64,
  decimals: u8  
) -> Result<()> {
  token_2022::transfer_checked(
    CpiContext::new(
        token_program.to_account_info(),
        token_2022::TransferChecked {
            from: sender,
            mint,
            to,
            authority,            
        },
    ),
    amount,
    decimals,
  )
}