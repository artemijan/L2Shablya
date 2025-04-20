#[derive(Debug, Clone)]
pub struct BitMask {
    flags: u32  ,
}

impl Default for BitMask {
    fn default() -> Self {
        Self::new()
    }
}

impl BitMask {
    #[must_use]
    pub fn new() -> Self {
        Self {
            flags: 0
        }
    }

    pub fn add_mask<T>(&mut self, mask: T)
    where
        T: Into<u32>, // Allow u8, u16, u32, i32
    {
        let val = mask.into();
        self.flags |= 1 << val;
    }

    #[must_use]
    pub fn contains_mask<K>(&self, mask: K) -> bool
    where
        K: Into<u32>, // Allow u8, u16, u32, i32
    {
        let mask = mask.into();
        (self.flags & (1 << mask)) != 0
    }

    #[must_use]
    pub fn flags(&self) -> u32 {
        self.flags
    }
}
