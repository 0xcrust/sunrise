use crate::{system, token, utils, BeamError, MintGsol};
use anchor_lang::prelude::*;

pub fn handler(ctx: Context<MintGsol>, amount: u64) -> Result<()> {
    let state = &mut ctx.accounts.state;
    let gsol_mint = &ctx.accounts.gsol_mint;

    let cpi_program = utils::get_cpi_program_id(&ctx.accounts.sysvar.to_account_info())?;
    system::check_beam_validity(state, &ctx.accounts.beam, &cpi_program)?;

    let pre_supply = state.pre_supply;
    let effective_supply = gsol_mint.supply.checked_sub(pre_supply).unwrap();

    let details = state
        .get_mut_beam_details(&ctx.accounts.beam.key())
        .ok_or(BeamError::UnidentifiedBeam)?;

    let mint_window = if effective_supply != 0 {
        (details.allocation as u64)
            .checked_mul(effective_supply)
            .unwrap()
            .checked_div(100)
            .unwrap()
    } else {
        // Let the first request mint without restrictions. The allocations will come into effect afterwards.
        amount
    };

    if details.minted > mint_window {
        return Err(BeamError::MintWindowExceeded.into());
    }

    details.minted = details.minted.checked_add(amount).unwrap();
    token::mint_to(
        amount,
        &ctx.accounts.gsol_mint.to_account_info(),
        &ctx.accounts.gsol_mint_authority,
        &ctx.accounts.mint_gsol_to.to_account_info(),
        &ctx.accounts.token_program,
        &ctx.accounts.state,
    )?;

    Ok(())
}
