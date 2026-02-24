// 存钱, 创建金库
use crate::{
    helpers::{
        AccountCheck, AssociatedTokenAccount, AssociatedTokenAccountCheck,
        AssociatedTokenAccountInit, MintInterface, ProgramAccount, ProgramAccountInit,
        SignerAccount,
    },
    state::Escrow,
};
use pinocchio::{cpi::Seed, error::ProgramError, AccountView, Address};
use pinocchio_token::instructions::Transfer;

// 定义账户列表的结构体
// 注意账户的顺序, 和调用指令时传入的账户顺序一致
pub struct MakeAccounts<'a> {
    // maker 账户 (签名账户, 地址存入 escrow 账户中)
    pub maker: &'a AccountView,
    // 托管的数据账户
    pub escrow: &'a AccountView,
    // 存入的 token a 的 mint 账户 (需要进行转账操作和存入地址到 escrow 账户中)
    pub mint_a: &'a AccountView,
    // 期望得到的 token b 的 mint 账户 (需要进行存入地址到 escrow 账户中)
    pub mint_b: &'a AccountView,
    // maker 账户的 token a 的 ata 账户 (需要进行转账操作)
    pub maker_ata_a: &'a AccountView,
    // 金库 ata 账户
    pub vault: &'a AccountView,
    // system program
    pub system_program: &'a AccountView,
    // token program
    pub token_program: &'a AccountView,
}

// 为账户列表实现 TryFrom trait
impl<'a> TryFrom<&'a [AccountView]> for MakeAccounts<'a> {
    type Error = ProgramError;

    // 校验账户
    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [maker, escrow, mint_a, mint_b, maker_ata_a, vault, system_program, token_program, _] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // 校验账户
        SignerAccount::check(maker)?;
        MintInterface::check(mint_a)?;
        MintInterface::check(mint_b)?;
        AssociatedTokenAccount::check(maker_ata_a, maker, mint_a, token_program)?;

        Ok(Self {
            maker,
            escrow,
            mint_a,
            mint_b,
            maker_ata_a,
            vault,
            token_program,
            system_program,
        })
    }
}

// 定义指令所需的数据结构体
pub struct MakeInstructionData {
    // maker 传入的 seed
    pub seed: u64,
    // 希望接收的 token b 的数量
    pub receive: u64,
    // maker 存入的 token a 的数量
    pub amount: u64,
}

// 为指令数据实现 TryFrom trait
impl<'a> TryFrom<&'a [u8]> for MakeInstructionData {
    type Error = ProgramError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        if data.len() != size_of::<u64>() * 3 {
            return Err(ProgramError::InvalidInstructionData);
        }

        let seed = u64::from_le_bytes(data[0..8].try_into().unwrap());
        let receive = u64::from_le_bytes(data[8..16].try_into().unwrap());
        let amount = u64::from_le_bytes(data[16..24].try_into().unwrap());

        // 存入的 token a 的数量不能为 0
        if amount == 0 {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(Self {
            seed,
            receive,
            amount,
        })
    }
}

pub struct Make<'a> {
    pub instruction_data: MakeInstructionData,
    pub accounts: MakeAccounts<'a>,
    // 缓存的 bump 值
    pub bump: u8,
}

// 为 Make 结构体实现 TryFrom trait
impl<'a> TryFrom<(&'a [u8], &'a [AccountView])> for Make<'a> {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'a [u8], &'a [AccountView])) -> Result<Self, Self::Error> {
        let accounts = MakeAccounts::try_from(accounts)?;
        let instruction_data = MakeInstructionData::try_from(data)?;

        // 计算 pda 以及 pda 签名种子
        let seed_binding = instruction_data.seed.to_le_bytes();
        let (_escrow_pda, bump) = Address::find_program_address(
            &[
                b"escrow",
                &accounts.maker.address().to_bytes(),
                &seed_binding,
            ],
            &crate::ID,
        );
        let bump_binding = [bump];
        let escrow_seeds = [
            Seed::from(b"escrow"),
            Seed::from(accounts.maker.address().as_ref()),
            Seed::from(&seed_binding),
            Seed::from(&bump_binding),
        ];

        // 创建 escrow PDA 数据账户
        ProgramAccount::init(
            &accounts.maker,
            &accounts.escrow,
            &escrow_seeds,
            Escrow::LEN,
        )?;

        // 创建 vault ATA 账户
        AssociatedTokenAccount::init(
            accounts.vault,
            accounts.mint_a,
            accounts.maker,
            accounts.escrow,
            accounts.system_program,
            accounts.token_program,
        )?;

        Ok(Self {
            instruction_data,
            accounts,
            bump,
        })
    }
}

// 实现 Make 的方法
impl<'a> Make<'a> {
    pub const DISCRIMINATOR: &'a u8 = &0;

    pub fn process(&self) -> Result<(), ProgramError> {
        // 1. 借用 escrow PDA 链上的数据账户的可变原始内存
        let mut data = self.accounts.escrow.try_borrow_mut()?;

        // 2. 将 escrow 原始内存映射为 Escrow 数据结构体, 只是以 Escrow 结构体的视角去读取这块内存
        // 因为是零拷贝的, 所以 escrow 和 data 此时指向的是同一快内存
        let escrow = Escrow::load_mut(data.as_mut())?;

        // 设置 escrow 数据等同于更改 escrow PDA 的内存, 也就是更改了 escrow PDA 链上的数据
        escrow.set_inner(
            self.instruction_data.seed,
            self.accounts.maker.address().clone(),
            self.accounts.mint_a.address().clone(),
            self.accounts.mint_b.address().clone(),
            self.instruction_data.receive,
            [self.bump],
        );

        // 转账 maker 的 token a 到 vault
        Transfer {
            from: self.accounts.maker_ata_a, // maker 的 token a 的 ATA 账户
            to: self.accounts.vault,
            authority: self.accounts.maker,
            amount: self.instruction_data.amount,
        }
        .invoke()?;

        Ok(())
    }
}
