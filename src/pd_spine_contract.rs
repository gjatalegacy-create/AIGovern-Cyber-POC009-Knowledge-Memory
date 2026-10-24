pub const MASS_SCALE: u32 = 10_000;
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FixedMass(pub u32);
impl FixedMass {
    pub const ZERO: Self = Self(0);
    pub const FULL: Self = Self(MASS_SCALE);
    pub fn new(raw: u32) -> Self {
        Self(raw.min(MASS_SCALE))
    }
    pub fn from_ratio(positive: u32, total: u32) -> Self {
        match total {
            0 => Self::ZERO,
            _ => Self::new(positive.saturating_mul(MASS_SCALE) / total),
        }
    }
    pub fn raw(self) -> u32 {
        self.0
    }
    pub fn decimal_4(self) -> String {
        format!("{}.{:04}", self.0 / MASS_SCALE, self.0 % MASS_SCALE)
    }
}
