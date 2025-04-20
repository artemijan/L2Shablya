use num_traits::ToPrimitive;
use sqlx::types::Decimal;

pub trait ToU32Rounded {
    fn to_u32_rounded(self) -> anyhow::Result<u32>;
}
impl ToU32Rounded for f64 {
    fn to_u32_rounded(self) -> anyhow::Result<u32> {
        let rounded = self.round();
        let i = rounded
            .to_u32()
            .ok_or_else(|| anyhow::anyhow!("Value {} is too big for u32", self))?;
        Ok(i)
    }
}
impl ToU32Rounded for Decimal {
    fn to_u32_rounded(self) -> anyhow::Result<u32> {
        let rounded = self.round();
        let i = rounded
            .to_u32()
            .ok_or_else(|| anyhow::anyhow!("Value {} is too big for u32", self))?;
        Ok(i)
    }
}

pub trait ToU16Rounded {
    fn to_u16_rounded(self) -> anyhow::Result<u16>;
}

impl ToU16Rounded for f64 {
    fn to_u16_rounded(self) -> anyhow::Result<u16> {
        let rounded = self.round();
        let i = rounded
            .to_u16()
            .ok_or_else(|| anyhow::anyhow!("Value {} is too big for u16", self))?;
        Ok(i)
    }
}
