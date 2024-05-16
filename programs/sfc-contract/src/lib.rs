use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintTo, Burn, TokenAccount, Token, Transfer };

use anchor_lang::system_program;
use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;
declare_id!("DKPreu6SebHaxEWXDEvoX5vXc1wkWEjorvRba1HMrGXc");

#[program]
pub mod sfcvnd {
    use super::*;
    const ADMIN: &str = "Cy66eHC9PcTb9wqwU77vyVievDyzYSwdbFkysDKkBYii";
    const SOL_COUNT: u64 = LAMPORTS_PER_SOL;
    pub fn user_message_target(ctx: Context<MessageTarget>, message: String) -> Result<()> {
        if !message.is_empty() {
            msg!("From user name is {}", ctx.accounts.fromclient.account_name);
            msg!("Wallet address is {}", ctx.accounts.signer.key().to_string());
            msg!("Said {}", message);
            msg!("To user name is {}", ctx.accounts.toclient.account_name);
            msg!("Wallet address is {}", ctx.accounts.target.asset_target.to_string());
        }else {
            return Err(ErrorCode::MessageIsEmpty.into());
        };
        Ok(())
    }
    
    pub fn user_message(ctx: Context<Message>, message: String) -> Result<()> {
        if !message.is_empty() {
            msg!("User name is {}", ctx.accounts.client.account_name);
            msg!("Wallet address is {}", ctx.accounts.signer.key().to_string());
            msg!("Said {}", message);
        }else {
            return Err(ErrorCode::MessageIsEmpty.into());
        };
        Ok(())
    }

    pub fn change_name(ctx: Context<ChangeName>, name: String) -> Result<()> {
        if !name.is_empty() {
            ctx.accounts.client.account_name = name.clone();
            msg!("Your account name is {} right now", name);
        }else {
            return Err(ErrorCode::NameIsEmpty.into());
        };
        Ok(())
    }

    pub fn change_name_target(ctx: Context<NameTarget>, name: String) -> Result<()> {
        if ctx.accounts.signer.key().to_string() != ADMIN {
            return Err(ErrorCode::Unauthorized.into());
        }
        if !name.is_empty() {
            ctx.accounts.client.account_name = name.clone();
            msg!("Your account name is {} right now", name);
        }else {
            return Err(ErrorCode::NameIsEmpty.into());
        };
        Ok(())
    }

    pub fn lock_target(ctx: Context<TargetUser>, targetkey: Pubkey) -> Result<()> {
        if targetkey == Pubkey::default() {
            return Err(ErrorCode::InvalidTargetKey.into());
        };
        ctx.accounts.target.asset_target = targetkey;
        Ok(())
    }

    pub fn init_user(ctx: Context<CreateUser>) -> Result<()> {
        if ctx.accounts.client.asset_account != 0 {
            return Err(ErrorCode::AccountNotEmpty.into());
        };
        ctx.accounts.client.asset_account = 0;
        Ok(())
    }

    pub fn clear_user(ctx: Context<DeleteUser>) -> Result<()> {
        if ctx.accounts.client.asset_account != 0 {
            return Err(ErrorCode::AccountNotEmpty.into());
        } else {
            let _ = ctx.accounts.client.close(ctx.accounts.sol_destination.to_account_info());
        };
        Ok(())
    }

    pub fn withdraw_asset(ctx: Context<FixAccount>, amount: u64) -> Result<()> {
        if ctx.accounts.signer.key().to_string() != ADMIN {
            return Err(ErrorCode::Unauthorized.into());
        }
        if amount < 10_000 {
            return Err(ErrorCode::InvalidAmount.into());
        };
        if ctx.accounts.client.asset_account < amount {
            return Err(ErrorCode::NotEnoughVND.into());
        } else {
            ctx.accounts.client.asset_account -= amount;
            msg!("You withdraw {} VND. You have {} VND left.", amount, ctx.accounts.client.asset_account);
        };
        Ok(())
    }

    pub fn deposit_asset(ctx: Context<FixAccount>, amount: u64) -> Result<()> {
        if ctx.accounts.signer.key().to_string() != ADMIN {
            return Err(ErrorCode::Unauthorized.into());
        };
        if amount < 10_000 {
            return Err(ErrorCode::InvalidAmount.into());
        };
        ctx.accounts.client.asset_account += amount;
        msg!("You deposit {} VND. You now have {} VND.", amount, ctx.accounts.client.asset_account);
        Ok(())
    }

    pub fn tranfer_asset(ctx: Context<TranferAccount>, amount: u64) -> Result<()> {
        if ctx.accounts.fromclient.asset_account < amount {
            return Err(ErrorCode::NotEnoughVND.into());
        } else {
            ctx.accounts.fromclient.asset_account -= amount;
            ctx.accounts.toclient.asset_account += amount;
            msg!("You tranfer {} VND. ", amount);
            msg!("To Pubkey: {}. ", ctx.accounts.target.asset_target);
        };
        Ok(())
    }
    
    pub fn tranfer_sol(ctx: Context<TransferSol>, amount: u64) -> Result<()> {
        if ctx.accounts.signer.to_account_info().lamports() < amount {
            return Err(ErrorCode::YouNotEnoughSol.into());
        } else {
            let cpi_context = CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.signer.to_account_info(),
                    to: ctx.accounts.target.clone(),
                },
            );
            system_program::transfer(cpi_context, amount)?;
            msg!("You tranfer {} Sol. ", amount / SOL_COUNT);
            msg!("To Pubkey: {}. ", ctx.accounts.target.key());
        };
        Ok(())
    }

    pub fn tranfer_token(ctx: Context<TranferToken>, amount: u64, choose: bool) -> Result<()> {
        if ctx.accounts.fromtoken.amount < amount {
            if choose {
                return Err(ErrorCode::YouNotEnoughSFCVND.into());
            } else {
                return Err(ErrorCode::YouNotEnoughLPSFC.into());
            };  
        } else {
            let cpi_ctx = CpiContext::new(
                    ctx.accounts.token.to_account_info(),
                    Transfer {
                        from: ctx.accounts.fromtoken.to_account_info(),
                        to: ctx.accounts.totoken.to_account_info(),
                        authority: ctx.accounts.signer.to_account_info(),
                    },
                );
            token::transfer(cpi_ctx, amount)?;
            if choose {
                msg!("You tranfer {} SFC - VND. ", amount / SOL_COUNT);
            } else {
                msg!("You tranfer {} LPSFC. ", amount / SOL_COUNT);
            };
            msg!("To Pubkey: {}. ", ctx.accounts.totoken.owner);
        };
        Ok(())
    }
    
    pub fn buy_sol(ctx: Context<VaultAccountAMMDEX>, amountin: u64) -> Result<()> {
        if ctx.accounts.vaultsol.to_account_info().lamports() < amountin {
            return Err(ErrorCode::VaultNotEnoughSol.into());
        } else {
            let t1 = ctx.accounts.vaultsol.to_account_info().lamports();
            let t2 = ctx.accounts.vaultsfc.amount;
            let k1 = ctx.accounts.vaultsol.k_value;
            let k2 = t1 - amountin;
            let k3: f64 = k1 as f64 / k2 as f64 * SOL_COUNT as f64;
            let k4 = k3 - t2 as f64;
            let amountout = k4;
            let realamountout = amountout as u64;
            if ctx.accounts.donator.amount < realamountout{
                return Err(ErrorCode::YouNotEnoughSFCVND.into());
            } else {
                let cpi_ctx = CpiContext::new(
                    ctx.accounts.token.to_account_info(),
                    Transfer {
                        from: ctx.accounts.donator.to_account_info(),
                        to: ctx.accounts.vaultsfc.to_account_info(),
                        authority: ctx.accounts.signer.to_account_info(),
                    },
                );
                token::transfer(cpi_ctx, realamountout)?;
                **ctx
                    .accounts
                    .vaultsol
                    .to_account_info()
                    .try_borrow_mut_lamports()? -= amountin;
                **ctx
                    .accounts
                    .signer
                    .to_account_info()
                    .try_borrow_mut_lamports()? += amountin;
                msg!("You buy {} Sol. ", amountin / SOL_COUNT);
                msg!("From the vault");
                msg!("By {} SFC - VND", realamountout / SOL_COUNT);
            };  
        };
        Ok(())
    }
    
    pub fn sell_sol(ctx: Context<VaultAccountAMMDEX>, amountin: u64, bump: u64) -> Result<()> {
        if ctx.accounts.signer.lamports() < amountin {
            return Err(ErrorCode::YouNotEnoughSol.into());
        } else {
            let t1 = ctx.accounts.vaultsol.to_account_info().lamports();
            let t2 = ctx.accounts.vaultsfc.amount;
            let k1 = ctx.accounts.vaultsol.k_value;
            let k2 = t1 + amountin;
            let k3: f64 = k1 as f64 / k2 as f64 * SOL_COUNT as f64;
            let k4 = t2 as f64 - k3;
            let amountout = k4;
            let realamountout = amountout as u64;
            if ctx.accounts.vaultsfc.amount < realamountout{
                return Err(ErrorCode::VaultNotEnoughSFCVND.into());
            } else {
                let cpi_context = CpiContext::new(
                    ctx.accounts.system_program.to_account_info(),
                    system_program::Transfer {
                        from: ctx.accounts.signer.to_account_info(),
                        to: ctx.accounts.vaultsol.to_account_info(),
                    },
                );
                system_program::transfer(cpi_context, amountin)?;
                let seeds = &[b"vault".as_ref(), &[bump.try_into().unwrap()]];
                let signer_seeds = &[&seeds[..]];
                let cpi_ctx = CpiContext::new_with_signer(
                    ctx.accounts.token.to_account_info(),
                    Transfer {
                        from: ctx.accounts.vaultsfc.to_account_info(),
                        to: ctx.accounts.donator.to_account_info(),
                        authority: ctx.accounts.vaultsol.to_account_info(),
                    },
                    signer_seeds,
                );
                token::transfer(cpi_ctx, realamountout)?;
                msg!("You sell {} Sol. ", amountin / SOL_COUNT);
                msg!("To the vault");
                msg!("For {} SFC - VND", realamountout / SOL_COUNT);
            };  
        };
        Ok(())
    }
    
    pub fn provide_liquidity(ctx: Context<VaultAccountAMMLP>, amount: u64, ratio: u64, bump: u64) -> Result<()> {
        let t0 = ctx.accounts.mint.supply;
        let t1 = ctx.accounts.vaultsol.to_account_info().lamports();
        let amount_sol = (amount as f64 / t0 as f64 * t1 as f64) as u64;
        if ctx.accounts.signer.lamports() < amount_sol {
            return Err(ErrorCode::YouNotEnoughSol.into());
        };
        let t2 = ctx.accounts.vaultsfc.amount;
        let k1: f64 = amount_sol as f64 / SOL_COUNT as f64;
        let k2: f64 = ratio as f64 / SOL_COUNT as f64;
        let k3 = k1 * k2;
        let k4 = k3 * SOL_COUNT as f64;
        let k5 =  t1 + amount_sol;
        let k6 = t2 as f64 + k4;
        let amountout = k4;
        let amount_sfc = amountout as u64;
        if ctx.accounts.donatorsfc.amount < amount_sfc {
            return Err(ErrorCode::YouNotEnoughSFCVND.into());
        };
        ctx.accounts.vaultsol.k_value = ((k5 as f64 / SOL_COUNT as f64) * (k6 / SOL_COUNT as f64)) as u64 * SOL_COUNT;
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info().clone(),
            system_program::Transfer {
                from: ctx.accounts.signer.to_account_info().clone(),
                to: ctx.accounts.vaultsol.to_account_info().clone(),
            },
        );
        system_program::transfer(cpi_context, amount_sol)?;
        let cpi_ctx_sfc = CpiContext::new(
            ctx.accounts.token.to_account_info(),
            Transfer {
                from: ctx.accounts.donatorsfc.to_account_info().clone(),
                to: ctx.accounts.vaultsfc.to_account_info().clone(),
                authority: ctx.accounts.signer.to_account_info().clone(),
            },
        );
        token::transfer(cpi_ctx_sfc, amount_sfc)?;
        let seeds = &[b"vault".as_ref(), &[bump.try_into().unwrap()]];
        let signer_seeds = &[&seeds[..]];
        let cpi_ctx_lp = CpiContext::new_with_signer(
            ctx.accounts.token.to_account_info().clone(),
            MintTo {
                mint: ctx.accounts.mint.to_account_info().clone(),
                to: ctx.accounts.donatorlp.to_account_info().clone(),
                authority: ctx.accounts.vaultsol.to_account_info().clone(),
            },
            signer_seeds,
        );
        token::mint_to(cpi_ctx_lp, amount)?;
        msg!("You provided {} Dev Sol to the vault", amount_sfc / SOL_COUNT);
        msg!("And provided {} SFC - VND to the vault", amount_sfc / SOL_COUNT);
        msg!("You earn {} LPSFC from the vault", amount / SOL_COUNT);
        Ok(())
    }

    pub fn withdraw_liquidity(ctx: Context<VaultAccountAMMLP>, amount: u64, ratio: u64, bump: u64) -> Result<()> {
        let t0 = ctx.accounts.mint.supply;
        let t1 = ctx.accounts.vaultsol.to_account_info().lamports();
        let amount_sol = (amount as f64 / t0 as f64 * t1 as f64) as u64;
        if t1 < amount_sol {
            return Err(ErrorCode::VaultNotEnoughSol.into());
        };
        let t2 = ctx.accounts.vaultsfc.amount;
        let k1: f64 = amount_sol as f64 / SOL_COUNT as f64;
        let k2: f64 = ratio as f64 / SOL_COUNT as f64;
        let k3 = k1 * k2;
        let k4 = k3 * SOL_COUNT as f64;
        let k5 = t1 - amount_sol;
        let k6 = t2 as f64 - k4;
        let amountout = k4;
        let amount_sfc = amountout as u64;
        if t2 < amount_sfc {
            return Err(ErrorCode::VaultNotEnoughSFCVND.into());
        };
        ctx.accounts.vaultsol.k_value = ((k5 as f64 / SOL_COUNT as f64) * (k6 / SOL_COUNT as f64)) as u64 * SOL_COUNT;
        let seeds = &[b"vault".as_ref(), &[bump.try_into().unwrap()]];
        let signer_seeds = &[&seeds[..]];
        let cpi_ctx_sfc = CpiContext::new_with_signer(
            ctx.accounts.token.to_account_info(),
            Transfer {
                from: ctx.accounts.vaultsfc.to_account_info().clone(),
                to: ctx.accounts.donatorsfc.to_account_info().clone(),
                authority: ctx.accounts.vaultsol.to_account_info().clone(),
            },
            signer_seeds,
        );
        token::transfer(cpi_ctx_sfc, amount_sfc)?;
        let cpi_ctx_lp = CpiContext::new(
            ctx.accounts.token.to_account_info().clone(),
            Burn {
                mint: ctx.accounts.mint.to_account_info().clone(),
                from: ctx.accounts.donatorlp.to_account_info().clone(),
                authority: ctx.accounts.signer.to_account_info().clone(),
            },
        );
        token::burn(cpi_ctx_lp, amount)?;
        **ctx
            .accounts
            .vaultsol
            .to_account_info()
            .try_borrow_mut_lamports()? -= amount_sol;
        **ctx
            .accounts
            .signer
            .to_account_info()
            .try_borrow_mut_lamports()? += amount_sol;
        msg!("You withdrew {} Dev Sol from the vault", amount_sol / SOL_COUNT);
        msg!("And withdrew {} SFC - VND from the vault", amount_sfc / SOL_COUNT);
        msg!("You pay {} LPSFC to the vault", amount / SOL_COUNT);
        Ok(())
    }


    pub fn tribute_asset(ctx: Context<VaultAccountTribute>, amount: u64, bump: u64) -> Result<()> {
        if ctx.accounts.donator.asset_account < amount / 100_000 {
            return Err(ErrorCode::NotEnoughVND.into());
        } else {
            ctx.accounts.donator.asset_account -= amount / 100_000;
            ctx.accounts.vault.asset_account += amount / 100_000;
            msg!("You tribute {} VND. ", amount / 100_000);
            msg!("To the vault, for mint {} SFC - VND", amount / SOL_COUNT);
            let seeds = &[b"vault".as_ref(), &[bump.try_into().unwrap()]];
            let signer_seeds = &[&seeds[..]];
            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.token.to_account_info(),
                MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.tk.to_account_info(),
                    authority: ctx.accounts.authority.clone(),
                },
                signer_seeds,
            );
            token::mint_to(cpi_ctx, amount)?;
        };
        Ok(())
    }
    
    pub fn summon_asset(ctx: Context<VaultAccountSummonAsset>, amount: u64) -> Result<()> {
        if ctx.accounts.tk.amount < amount {
            return Err(ErrorCode::NotEnoughSFCVND.into());
        } else {
            let cpi_ctx = CpiContext::new(
                ctx.accounts.token.to_account_info(),
                Burn {
                    mint: ctx.accounts.mint.to_account_info(),
                    from: ctx.accounts.tk.to_account_info(),
                    authority: ctx.accounts.signer.to_account_info(),
                },
            );
            token::burn(cpi_ctx, amount)?;
            ctx.accounts.vault.asset_account -= amount / 100_000;
            ctx.accounts.donator.asset_account += amount / 100_000;
            msg!("You earn {} VND. ", amount / 100_000);
            msg!("From the vault, by you burn {} SFC - VND", amount / SOL_COUNT);
        };
        Ok(())
    }

}
#[derive(Accounts)]
pub struct TransferSol<'info> {
    #[account(mut)]
    /// CHECK: This field represents the target account for some operation. It is marked unsafe
    ///        because it involves interactions with external accounts, but no additional checks
    ///        through types are necessary as it's validated elsewhere in the program logic.
    pub target: AccountInfo<'info>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct MessageTarget<'info> {
    #[account( 
        mut,
        seeds = [b"target", signer.key().as_ref()],
        bump,
    )]
    pub target: Account<'info, UserTarget>,
    #[account(
        mut,
        seeds = [b"client", target.asset_target.to_bytes().as_ref()],
        bump,
    )]
    pub toclient: Account<'info, UserInfor>,
    #[account(
        mut,
        seeds = [b"client", signer.key().as_ref()],
        bump,
    )]
    pub fromclient: Account<'info, UserInfor>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct Message<'info> {
    #[account( 
        mut,
        seeds = [b"client", signer.key().as_ref()],
        bump,
    )]
    pub client: Account<'info, UserInfor>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct NameTarget<'info> {
    #[account( 
        mut,
        seeds = [b"target", signer.key().as_ref()],
        bump,
    )]
    pub target: Account<'info, UserTarget>,
    #[account(
        mut,
        seeds = [b"client", target.asset_target.to_bytes().as_ref()],
        bump,
    )]
    pub client: Account<'info, UserInfor>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct ChangeName<'info> {
    #[account( 
        mut,
        seeds = [b"client", signer.key().as_ref()],
        bump,
    )]
    pub client: Account<'info, UserInfor>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct TargetUser<'info> {
    #[account( 
        init_if_needed, 
        payer = signer,
        space = 1000,
        seeds = [b"target", signer.key().as_ref()],
        bump,
    )]
    pub target: Account<'info, UserTarget>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct CreateUser<'info> {
    #[account( 
        init_if_needed, 
        payer = signer,
        space = 1000,
        seeds = [b"client", signer.key().as_ref()],
        bump,
    )]
    pub client: Account<'info, UserInfor>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct DeleteUser<'info> {
    #[account(
        mut,
        close = sol_destination,
        seeds = [b"client", signer.key().as_ref()],
        bump,
    )]
    pub client: Account<'info, UserInfor>,
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub sol_destination: Signer<'info>,
}

#[derive(Accounts)]
pub struct FixAccount<'info> {
    #[account( 
        mut,
        seeds = [b"target", signer.key().as_ref()],
        bump,
    )]
    pub target: Account<'info, UserTarget>,
    #[account(
        mut,
        seeds = [b"client", target.asset_target.to_bytes().as_ref()],
        bump,
    )]
    pub client: Account<'info, UserInfor>,
    #[account(mut)]
    pub signer: Signer<'info>,
}

#[derive(Accounts)]
pub struct VaultAccountAMMDEX<'info> {
    #[account(
        mut,
        seeds = [b"vault"],
        bump,
    )]
    pub vaultsol: Account<'info, UserInfor>,
    #[account(mut)]
    pub vaultsfc: Account<'info, TokenAccount>,
    #[account(mut)]
    pub donator: Account<'info, TokenAccount>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub token: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct VaultAccountAMMLP<'info> {
    #[account(
        mut,
        seeds = [b"vault"],
        bump,
    )]
    pub vaultsol: Account<'info, UserInfor>,
    #[account(mut)]
    pub vaultsfc: Account<'info, TokenAccount>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub donatorsfc: Account<'info, TokenAccount>,
    #[account(mut)]
    pub donatorlp: Account<'info, TokenAccount>,
    pub token: Program<'info, Token>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct VaultAccountTribute<'info> {
    #[account(
        mut,
        seeds = [b"vault"],
        bump,
    )]
    pub vault: Account<'info, UserInfor>,
    #[account(
        mut,
        seeds = [b"client", signer.key().as_ref()],
        bump,
    )]
    pub donator: Account<'info, UserInfor>,
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub tk: Account<'info, TokenAccount>,
    /// CHECK: This field represents the target account for some operation. It is marked unsafe
    ///        because it involves interactions with external accounts, but no additional checks
    ///        through types are necessary as it's validated elsewhere in the program logic.
    pub authority: AccountInfo<'info>,
    pub token: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct VaultAccountSummonAsset<'info> {
    #[account(
        mut,
        seeds = [b"vault"],
        bump,
    )]
    pub vault: Account<'info, UserInfor>,
    #[account(
        mut,
        seeds = [b"client", signer.key().as_ref()],
        bump,
    )]
    pub donator: Account<'info, UserInfor>,
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub tk: Account<'info, TokenAccount>,
    pub token: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct TranferAccount<'info> {
    #[account( 
        mut,
        seeds = [b"target", signer.key().as_ref()],
        bump,
    )]
    pub target: Account<'info, UserTarget>,
    #[account(
        mut,
        seeds = [b"client", target.asset_target.to_bytes().as_ref()],
        bump,
    )]
    pub toclient: Account<'info, UserInfor>,
    #[account(
        mut,
        seeds = [b"client", signer.key().as_ref()],
        bump,
    )]
    pub fromclient: Account<'info, UserInfor>,
    #[account(mut)]
    pub signer: Signer<'info>,
}
#[derive(Accounts)]
pub struct TranferToken<'info> {
    #[account(mut)]
    pub fromtoken: Account<'info, TokenAccount>,
    #[account(mut)]
    pub totoken: Account<'info, TokenAccount>,
    pub token: Program<'info, Token>,
    #[account(mut)]
    pub signer: Signer<'info>,
}

#[account]
pub struct UserInfor {
    pub asset_account: u64,
    pub account_name: String,
    pub k_value: u64,
}

#[account]
pub struct UserTarget {
    /// CHECK: This field represents the target account for some operation. It is marked unsafe
    ///        because it involves interactions with external accounts, but no additional checks
    ///        through types are necessary as it's validated elsewhere in the program logic.
    pub asset_target: Pubkey,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Dev Sol price under slippage")]
    PriceUnderSlippage,
    #[msg("Dev Sol price over slippage")]
    PriceOverSlippage,
    #[msg("Not enough VND")]
    NotEnoughVND,
    #[msg("Not enough SFC - VND")]
    NotEnoughSFCVND,
    #[msg("Vault not enough SFC - VND")]
    VaultNotEnoughSFCVND,
    #[msg("You not enough SFC - VND")]
    YouNotEnoughSFCVND,
    #[msg("You not enough LPSFC")]
    YouNotEnoughLPSFC,
    #[msg("Not enough Sol")]
    NotEnoughSol,
    #[msg("Vault not enough Sol")]
    VaultNotEnoughSol,
    #[msg("You not enough Sol")]
    YouNotEnoughSol,
    #[msg("Account not empty")]
    AccountNotEmpty,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Invalid target key")]
    InvalidTargetKey,
    #[msg("Account already initialized")]
    AccountAlreadyInitialized,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("InvalidAuthority")]
    InvalidAuthority,
    #[msg("Name is empty")]
    NameIsEmpty,
    #[msg("Message is empty")]
    MessageIsEmpty,
}