use crate::{
    helpers::{
        AccountCheck, AccountClose, AssociatedTokenAccount, AssociatedTokenAccountCheck,
        AssociatedTokenAccountInit, MintInterface, ProgramAccount, SignerAccount,
    },
    state::Escrow,
};
use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    AccountView, Address,
};
use pinocchio_token::instructions::{CloseAccount, Transfer};

pub struct RefundAccounts<'a> {
    maker: &'a AccountView,
    escrow: &'a AccountView,
    mint_a: &'a AccountView,
    vault: &'a AccountView,
    maker_ata_a: &'a AccountView,
    system_program: &'a AccountView,
    token_program: &'a AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for RefundAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [maker, escrow, mint_a, vault, maker_ata_a, system_program, token_program, _] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        SignerAccount::check(maker)?;
        ProgramAccount::check(escrow)?;
        MintInterface::check(mint_a)?;
        AssociatedTokenAccount::check(vault, escrow, mint_a, token_program)?;

        Ok(Self {
            maker,
            escrow,
            mint_a,
            maker_ata_a,
            vault,
            token_program,
            system_program,
        })
    }
}

pub struct Refund<'a> {
    pub accounts: RefundAccounts<'a>,
}

impl<'a> TryFrom<&'a [AccountView]> for Refund<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let accounts = RefundAccounts::try_from(accounts)?;

        // 确保 maker_ata_a 账户存在, 没有则创建
        AssociatedTokenAccount::init_if_needed(
            accounts.maker_ata_a,
            accounts.mint_a,
            accounts.maker,
            accounts.maker,
            accounts.system_program,
            accounts.token_program,
        )?;

        Ok(Self { accounts })
    }
}

impl<'a> Refund<'a> {
    pub const DISCRIMINATOR: &'a u8 = &2;

    pub fn process(&self) -> Result<(), ProgramError> {
        // 利用 block 作用域限制借用的生命周期, 离开 block 后, escrow 的借用就会被释放, 避免了手动释放
        let (seed, bump) = {
            let data = self.accounts.escrow.try_borrow()?;
            let escrow = Escrow::load(&data)?;

            // 这里使用了 create_program_address, 因为不需要找到 bump
            let escrow_address = Address::create_program_address(
                &[
                    b"escrow",
                    self.accounts.maker.address().as_ref(),
                    &escrow.seed.to_le_bytes(),
                    &escrow.bump,
                ],
                &crate::ID,
            )?;

            // 判断 escrow 账户是否正确, 和 take 一样
            if self.accounts.escrow.address() != &escrow_address {
                return Err(ProgramError::InvalidAccountOwner);
            }

            (escrow.seed, escrow.bump)
        };

        let amount = {
            let vault_data = self.accounts.vault.try_borrow()?;
            // pinocchio-token/src/state/token.rs 中 amount 在结构体的第 64 位开始
            u64::from_le_bytes(vault_data[64..72].try_into().unwrap())
        };

        let seed_binding = seed.to_le_bytes();
        let escrow_seed = [
            Seed::from(b"escrow"),
            Seed::from(self.accounts.maker.address().as_ref()),
            Seed::from(&seed_binding),
            Seed::from(&bump),
        ];
        let signers = &[Signer::from(&escrow_seed)];

        // 从 vault 转账 token 到 maker_ata_a
        Transfer {
            from: self.accounts.vault,
            to: self.accounts.maker_ata_a,
            authority: self.accounts.escrow,
            amount,
        }
        .invoke_signed(signers)?;

        // 关闭 vault token account
        CloseAccount {
            account: self.accounts.vault,
            destination: self.accounts.maker,
            authority: self.accounts.escrow,
        }
        .invoke_signed(signers)?;

        // 关闭 escrow 账户
        ProgramAccount::close(self.accounts.escrow, self.accounts.maker)?;

        Ok(())
    }
}
