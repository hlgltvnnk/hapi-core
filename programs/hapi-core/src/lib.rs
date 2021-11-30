use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

mod context;
mod error;
mod state;

use context::*;
use error::ErrorCode;
use state::{
    address::Category,
    case::CaseStatus,
    reporter::{ReporterStatus, ReporterRole},
};

#[program]
pub mod hapi_core {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        stake_unlock_epochs: u64,
        confirmation_threshold: u32,
    ) -> ProgramResult {
        let community = &mut ctx.accounts.community;

        community.authority = *ctx.accounts.authority.key;
        community.cases = 0;
        community.stake_unlock_epochs = stake_unlock_epochs;
        community.confirmation_threshold = confirmation_threshold;

        Ok(())
    }

    pub fn create_network(ctx: Context<CreateNetwork>, name: [u8; 32], bump: u8) -> ProgramResult {
        let network = &mut ctx.accounts.network;

        network.community = ctx.accounts.community.key();
        network.bump = bump;

        network.name = name;

        Ok(())
    }

    pub fn create_reporter(
        ctx: Context<CreateReporter>,
        role: ReporterRole,
        name: [u8; 32],
        bump: u8,
    ) -> ProgramResult {
        let reporter = &mut ctx.accounts.reporter;

        reporter.community = ctx.accounts.community.key();
        reporter.pubkey = *ctx.accounts.pubkey.key;
        reporter.bump = bump;

        reporter.role = role;
        reporter.status = ReporterStatus::Inactive;
        reporter.name = name;

        Ok(())
    }

    pub fn create_case(
        ctx: Context<CreateCase>,
        case_id: u64,
        name: [u8; 32],
        bump: u8,
    ) -> ProgramResult {
        let community = &mut ctx.accounts.community;

        if case_id != community.cases + 1 {
            return Err(ErrorCode::NonSequentialCaseId.into());
        } else {
            community.cases = case_id;
        }

        let case = &mut ctx.accounts.case;

        case.community = ctx.accounts.community.key();
        case.id = case_id;
        case.bump = bump;

        case.name = name;
        case.status = CaseStatus::Open;
        case.reporter = ctx.accounts.reporter.key();

        Ok(())
    }

    pub fn create_address(
        ctx: Context<CreateAddress>,
        pubkey: Pubkey,
        category: Category,
        risk: u8,
        bump: u8,
    ) -> ProgramResult {
        let address = &mut ctx.accounts.address;

        address.network = ctx.accounts.network.key();
        address.address = pubkey;
        address.bump = bump;

        address.community = ctx.accounts.community.key();
        address.reporter = ctx.accounts.reporter.key();
        address.case_id = ctx.accounts.case.id;
        address.category = category;
        address.risk = risk;
        address.confirmations = 0;

        Ok(())
    }

    pub fn create_asset(
        ctx: Context<CreateAsset>,
        mint: Pubkey,
        asset_id: [u8; 32],
        category: Category,
        risk: u8,
        bump: u8,
    ) -> ProgramResult {
        let asset = &mut ctx.accounts.asset;

        asset.network = ctx.accounts.network.key();
        asset.mint = mint;
        asset.asset_id = asset_id;
        asset.bump = bump;

        asset.community = ctx.accounts.community.key();
        asset.reporter = ctx.accounts.reporter.key();
        asset.case_id = ctx.accounts.case.id;
        asset.category = category;
        asset.risk = risk;
        asset.confirmations = 0;

        Ok(())
    }

    pub fn activate_reporter(ctx: Context<ActivateReporter>) -> ProgramResult {
        let reporter = &mut ctx.accounts.reporter;

        // TODO: transfer stake tokens from reporter token account to program token account

        reporter.status = ReporterStatus::Active;

        Ok(())
    }

    pub fn deactivate_reporter(ctx: Context<DeactivateReporter>) -> ProgramResult {
        let community = &ctx.accounts.community;

        let reporter = &mut ctx.accounts.reporter;

        reporter.status = ReporterStatus::Unstaking;
        reporter.unlock_epoch = Clock::get()?.epoch + community.stake_unlock_epochs;

        Ok(())
    }

    pub fn release_reporter(ctx: Context<ReleaseReporter>) -> ProgramResult {
        let reporter = &mut ctx.accounts.reporter;

        if reporter.unlock_epoch < Clock::get()?.epoch {
            return Err(ErrorCode::ReleaseEpochInFuture.into());
        }

        // TODO: transfer stake tokens from program token account to reporter token account

        reporter.status = ReporterStatus::Inactive;
        reporter.unlock_epoch = 0;

        Ok(())
    }

    // pub fn confirm_address(ctx: Context<ConfirmAddress>) -> ProgramResult {
    //     Ok(())
    // }
}
