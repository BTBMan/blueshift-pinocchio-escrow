use core::fmt;
use pinocchio::error::ProgramError;

// 自定义错误类型
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EscrowError {
    // 租金不足
    NotEnoughRentExempt,
    // 账户不是 signer
    NotSigner,
    // 账户不是预期的所有者
    InvalidOwner,
    // 账户数据无效
    InvalidAccountData,
    // 地址无效
    InvalidAddress,
}

// 为 ProgramError 实现 From trait
// 可以将 EscrowError 转换为 ProgramError::custom(EscrowError::Xxx as u32) 类型
impl From<EscrowError> for ProgramError {
    fn from(err: EscrowError) -> Self {
        ProgramError::Custom(err as u32)
    }
}

// 为 EscrowError 实现自定义 Display
impl fmt::Display for EscrowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EscrowError::NotEnoughRentExempt => {
                write!(f, "Lamports balance below rent-exempt threshold")
            }
            EscrowError::NotSigner => write!(f, "没有签名"),
            EscrowError::InvalidOwner => write!(f, "非法的所有者"),
            EscrowError::InvalidAccountData => write!(f, "非法的账户数据"),
            EscrowError::InvalidAddress => write!(f, "非法的地址"),
        }
    }
}
