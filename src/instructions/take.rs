use pinocchio::{AccountView, Address, cpi::Seed, error::ProgramError};
use pinocchio_token::instructions::Transfer;

use crate::{
    helpers::{
        AccountChecker, AssociatedTokenAccount, AssociatedTokenAccountCheck,
        AssociatedTokenAccountInit, MintInterface, ProgramAccount, SignerAccount,
    },
    state::Escrow,
};

pub struct TakeAccounts<'a> {
    maker: &'a AccountView,
    taker: &'a AccountView,
    escrow: &'a AccountView,
    mint_a: &'a AccountView,
    mint_b: &'a AccountView,
    maker_ata_b: &'a AccountView, // 从 taker 账户转账到 maker 的 token b 的 ata 账户
    taker_ata_b: &'a AccountView, // 账户给 maker 的 token b 的 ata 账户转账
    taker_ata_a: &'a AccountView, // 从 vault 转账到 taker 的 token a 的 ata 账户
    vault: &'a AccountView,       // vault 账户
    token_program: &'a AccountView,
    system_program: &'a AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for TakeAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [maker, taker, escrow, mint_a, mint_b, maker_ata_b, taker_ata_b, taker_ata_a, vault, token_program, system_program, _] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        SignerAccount::check(taker)?;
        MintInterface::check(mint_a)?;
        MintInterface::check(mint_b)?;
        ProgramAccount::check(escrow)?;
        AssociatedTokenAccount::check(taker_ata_b, taker, mint_b, token_program)?;
        AssociatedTokenAccount::check(vault, escrow, mint_a, token_program)?;

        Ok(Self {
            maker,
            taker,
            escrow,
            mint_a,
            mint_b,
            maker_ata_b,
            taker_ata_b,
            taker_ata_a,
            vault,
            token_program,
            system_program,
        })
    }
}

pub struct Take<'a> {
    pub accounts: TakeAccounts<'a>,
}

impl<'a> TryFrom<&'a [AccountView]> for Take<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let accounts = TakeAccounts::try_from(accounts)?;

        // 为 taker 创建 token a 的 ata 账户(如果不存在)
        AssociatedTokenAccount::init_if_needed(
            accounts.taker_ata_a,
            accounts.mint_a,
            accounts.taker,
            accounts.taker,
            accounts.system_program,
            accounts.token_program,
        )?;

        // 为 maker 创建 token b 的 ata 账户(如果不存在)
        AssociatedTokenAccount::init_if_needed(
            accounts.maker_ata_b,
            accounts.mint_b,
            accounts.taker,
            accounts.maker,
            accounts.system_program,
            accounts.token_program,
        )?;

        Ok(Self { accounts })
    }
}

impl<'a> Take<'a> {
    pub fn process(&self) -> Result<(), ProgramError> {
        let data = self.accounts.escrow.try_borrow()?;
        let escrow = Escrow::load(data.as_ref())?;

        // 判断 escrow 账户是否正确
        // 用调用指令所传入的账户中的 maker 账户和保存在 escrow 中的 seed 和 bump 了计算 escrow pda 地址
        // 通过计算出来的地址和指令账户列表中的 escrow 账户进行比较
        let (escrow_address,_) = Address::find_program_address(&[b"escrow", self.accounts.maker.address().as_ref(), &escrow.seed.to_le_bytes(), &escrow.bump], &crate::ID);
        if self.accounts.escrow.address().clone() != escrow_address {
            return Err(ProgramError::InvalidAccountOwner);
        }

        // 从 vault 转账 token a 到 taker
        let signers = [Seed::]
        Transfer {
            from: self.accounts.vault,
            to: self.accounts.taker_ata_a,
            authority: self.accounts.escrow,
            amount: escrow.receive,
        }
        .invoke_signed(signers)?;

        // 从 taker 转账 token b 到 maker

        Ok(())
    }
}
