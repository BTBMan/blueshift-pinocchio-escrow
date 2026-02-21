use pinocchio::Address;

// Pinocchio 中的 instruction data 是连续的, 解构体中的每个字段都必须满足 8 个字节, 所以须要手动定义结构体字段的顺序, 从大到小依次往下排列.
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
    // 缓存的 bumps
    pub bumps: [u8; 1],
}
