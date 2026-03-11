use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

/// On-Chain API Key Management System
/// Reframes Solana as a distributed state-machine backend for access control.
///
/// Web2 equivalent: A REST API that issues opaque tokens, stores them in a
/// relational DB (e.g. Postgres), validates on every request, and enforces
/// rate limits + expiry via cron jobs.
///
/// On-chain model: Each API key is a Program Derived Address (PDA) account
/// owned by the issuer. Validation is trustless — any program or client can
/// verify a key without calling a centralised service.

#[program]
pub mod api_key_manager {
    use super::*;

    /// Issue a new API key for a given service.
    /// Seeds: [b"api_key", issuer, holder, service_name]
    pub fn issue_key(
        ctx: Context<IssueKey>,
        service_name: String,
        permissions: Vec<String>,
        expires_in_slots: Option<u64>,
    ) -> Result<()> {
        require!(service_name.len() <= 32, ApiKeyError::ServiceNameTooLong);
        require!(permissions.len() <= 8, ApiKeyError::TooManyPermissions);

        let key_account = &mut ctx.accounts.key_account;
        let clock = Clock::get()?;

        key_account.issuer = ctx.accounts.issuer.key();
        key_account.holder = ctx.accounts.holder.key();
        key_account.service_name = service_name;
        key_account.permissions = permissions;
        key_account.issued_at = clock.slot;
        key_account.issued_epoch = clock.epoch;
        key_account.expires_at = expires_in_slots.map(|s| clock.slot + s);
        key_account.is_revoked = false;
        key_account.call_count = 0;
        key_account.rate_limit = 1000;
        key_account.bump = ctx.bumps.key_account;

        emit!(KeyIssued {
            issuer: key_account.issuer,
            holder: key_account.holder,
            service_name: key_account.service_name.clone(),
            issued_at: clock.slot,
        });

        Ok(())
    }

    /// Validate an API key — increments call_count and checks:
    ///   1. Not revoked
    ///   2. Not expired
    ///   3. Rate limit not exceeded
    ///   4. Permission present (if required)
    pub fn validate_key(
        ctx: Context<ValidateKey>,
        required_permission: Option<String>,
    ) -> Result<()> {
        let key_account = &mut ctx.accounts.key_account;
        let clock = Clock::get()?;

        require!(!key_account.is_revoked, ApiKeyError::KeyRevoked);

        if let Some(exp) = key_account.expires_at {
            require!(clock.slot <= exp, ApiKeyError::KeyExpired);
        }

        // Rate-limit: reset counter at each new Solana epoch boundary.
        // Uses clock.epoch (true epoch counter) rather than slot arithmetic
        // so that resets align precisely with actual epoch transitions.
        if clock.epoch > key_account.issued_epoch {
            key_account.call_count = 0;
            key_account.issued_epoch = clock.epoch;
        }
        require!(
            key_account.call_count < key_account.rate_limit,
            ApiKeyError::RateLimitExceeded
        );

        if let Some(perm) = required_permission {
            require!(
                key_account.permissions.contains(&perm),
                ApiKeyError::PermissionDenied
            );
        }

        key_account.call_count += 1;

        emit!(KeyValidated {
            holder: key_account.holder,
            service_name: key_account.service_name.clone(),
            call_count: key_account.call_count,
        });

        Ok(())
    }

    /// Revoke an API key. Only the original issuer can revoke.
    pub fn revoke_key(ctx: Context<RevokeKey>) -> Result<()> {
        let key_account = &mut ctx.accounts.key_account;
        key_account.is_revoked = true;

        emit!(KeyRevoked {
            issuer: key_account.issuer,
            holder: key_account.holder,
            service_name: key_account.service_name.clone(),
        });

        Ok(())
    }

    /// Update permissions on an existing key (issuer only).
    pub fn update_permissions(
        ctx: Context<UpdatePermissions>,
        new_permissions: Vec<String>,
    ) -> Result<()> {
        require!(new_permissions.len() <= 8, ApiKeyError::TooManyPermissions);
        let key_account = &mut ctx.accounts.key_account;
        key_account.permissions = new_permissions;
        Ok(())
    }

    /// Close the key account and reclaim rent (issuer only).
    pub fn close_key(_ctx: Context<CloseKey>) -> Result<()> {
        Ok(())
    }
}

// === Account Structs ===

#[account]
pub struct ApiKeyAccount {
    pub issuer: Pubkey,
    pub holder: Pubkey,
    pub service_name: String,
    pub permissions: Vec<String>,
    pub issued_at: u64,
    /// The Solana epoch when this key was last reset.
    /// Used for accurate epoch-boundary rate limit resets.
    pub issued_epoch: u64,
    pub expires_at: Option<u64>,
    pub is_revoked: bool,
    pub call_count: u64,
    pub rate_limit: u64,
    pub bump: u8,
}

impl ApiKeyAccount {
    pub const MAX_SIZE: usize = 32 + 32 + (4 + 32) + (4 + 8 * (4 + 16)) + 8 + 8 + 9 + 1 + 8 + 8 + 1 + 8;
}

// === Contexts ===

#[derive(Accounts)]
#[instruction(service_name: String)]
pub struct IssueKey<'info> {
    #[account(
        init,
        payer = issuer,
        space = 8 + ApiKeyAccount::MAX_SIZE,
        seeds = [b"api_key", issuer.key().as_ref(), holder.key().as_ref(), service_name.as_bytes()],
        bump
    )]
    pub key_account: Account<'info, ApiKeyAccount>,
    #[account(mut)]
    pub issuer: Signer<'info>,
    /// CHECK: Holder is the recipient of the key
    pub holder: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ValidateKey<'info> {
    #[account(mut)]
    pub key_account: Account<'info, ApiKeyAccount>,
    /// CHECK: Caller presents the key for validation
    pub caller: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct RevokeKey<'info> {
    #[account(mut, has_one = issuer)]
    pub key_account: Account<'info, ApiKeyAccount>,
    pub issuer: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdatePermissions<'info> {
    #[account(mut, has_one = issuer)]
    pub key_account: Account<'info, ApiKeyAccount>,
    pub issuer: Signer<'info>,
}

#[derive(Accounts)]
pub struct CloseKey<'info> {
    #[account(mut, has_one = issuer, close = issuer)]
    pub key_account: Account<'info, ApiKeyAccount>,
    #[account(mut)]
    pub issuer: Signer<'info>,
}

// === Events ===

#[event]
pub struct KeyIssued {
    pub issuer: Pubkey,
    pub holder: Pubkey,
    pub service_name: String,
    pub issued_at: u64,
}

#[event]
pub struct KeyValidated {
    pub holder: Pubkey,
    pub service_name: String,
    pub call_count: u64,
}

#[event]
pub struct KeyRevoked {
    pub issuer: Pubkey,
    pub holder: Pubkey,
    pub service_name: String,
}

// === Errors ===

#[error_code]
pub enum ApiKeyError {
    #[msg("Service name exceeds 32 characters")]
    ServiceNameTooLong,
    #[msg("Maximum 8 permissions per key")]
    TooManyPermissions,
    #[msg("This API key has been revoked")]
    KeyRevoked,
    #[msg("This API key has expired")]
    KeyExpired,
    #[msg("Rate limit exceeded for this epoch")]
    RateLimitExceeded,
    #[msg("Permission not granted for this key")]
    PermissionDenied,
}
