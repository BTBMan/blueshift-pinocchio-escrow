use pinocchio::{error::ProgramError, Address};

// Pinocchio 中的 instruction data 是连续的
// 而结构体的总大小必须是其最大字段的对齐要求的倍数
// 所以须要手动定义结构体字段的顺序, 从大到小依次往下排列.
// #[repr(C)] 的作用就是按照字段的声明顺序排列
#[repr(C)]
pub struct Escrow {
    // maker 传入的 seed
    pub seed: u64,
    // 托管程序的创建者
    pub maker: Address,
    // token a 的 mint 地址
    pub mint_a: Address,
    // token b 的 mint 地址
    pub mint_b: Address,
    // 希望接收的 token b 的数量
    pub receive: u64,
    // 缓存的 bump (bumps 更合适, 但是这里和 blueshift 官方教程保持一致吧)
    pub bump: [u8; 1],
}

// 实现 Escrow 结构体, 自定义一些方法
impl Escrow {
    // 计算 Escrow 结构体的大小 bytes
    pub const LEN: usize = size_of::<u64>() // 8 bytes (seed)
        + size_of::<Address>() // 32 bytes (maker)
        + size_of::<Address>() // 32 bytes (mint_a)
        + size_of::<Address>() // 32 bytes (mint_b)
        + size_of::<u64>() // 8 bytes (receive)
        + size_of::<[u8; 1]>(); // 1 bytes (bump)

    // inline(always) 用于在调用处展开函数代码块, 减少 CU 的消耗
    // 将原始字节指针转换为 Escrow 结构体的可变引用
    #[inline(always)]
    pub fn load_mut(bytes: &mut [u8]) -> Result<&mut Self, ProgramError> {
        if bytes.len() != Escrow::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(unsafe { &mut *core::mem::transmute::<*mut u8, *mut Self>(bytes.as_mut_ptr()) })
    }

    // 功能E-Business load_mut 一样, 只是得到的是不可变引用
    #[inline(always)]
    pub fn load(bytes: &[u8]) -> Result<&Self, ProgramError> {
        if bytes.len() != Escrow::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(unsafe { &*core::mem::transmute::<*const u8, *const Self>(bytes.as_ptr()) })
    }

    // 设置 seed 字段
    #[inline(always)]
    pub fn set_seed(&mut self, seed: u64) {
        self.seed = seed;
    }

    // 设置 maker 字段
    #[inline(always)]
    pub fn set_maker(&mut self, maker: Address) {
        self.maker = maker;
    }

    // 设置 mint_a 字段
    #[inline(always)]
    pub fn set_mint_a(&mut self, mint_a: Address) {
        self.mint_a = mint_a;
    }

    // 设置 mint_b 字段
    #[inline(always)]
    pub fn set_mint_b(&mut self, mint_b: Address) {
        self.mint_b = mint_b;
    }

    // 设置 receive 字段
    #[inline(always)]
    pub fn set_receive(&mut self, receive: u64) {
        self.receive = receive;
    }

    // 设置 bump 字段
    #[inline(always)]
    pub fn set_bump(&mut self, bump: [u8; 1]) {
        self.bump = bump;
    }

    // 设置所有字段
    #[inline(always)]
    pub fn set_inner(
        &mut self,
        seed: u64,
        maker: Address,
        mint_a: Address,
        mint_b: Address,
        receive: u64,
        bump: [u8; 1],
    ) {
        self.seed = seed;
        self.maker = maker;
        self.mint_a = mint_a;
        self.mint_b = mint_b;
        self.receive = receive;
        self.bump = bump;
    }
}
