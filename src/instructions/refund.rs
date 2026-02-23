use pinocchio::AccountView;

pub struct RefundAccounts<'a> {
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

pub struct Refund<'a> {
    pub accounts: RefundAccounts<'a>,
    pub bump: u8,
}
