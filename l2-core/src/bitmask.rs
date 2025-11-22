const BIT_POSITIONS: [u8; 8] = [
    0x80, // bit 7
    0x40, // bit 6
    0x20, // bit 5
    0x10, // bit 4
    0x08, // bit 3
    0x04, // bit 2
    0x02, // bit 1
    0x01, // bit 0
];

#[derive(Debug, Clone)]
pub struct BitMask {
    flags: Vec<u8>, // 1 byte = 8 flags
}

impl BitMask {
    /// Creates a new bitmask with enough space for `num_flags` bits.
    #[must_use] 
    pub fn new(num_flags: i32) -> Self {
        let num_bytes = (num_flags + 7) / 8;
        Self {
            flags: vec![0u8; num_bytes as usize],
        }
    }

    /// Sets the bit at the given mask index.
    pub fn add_mask<T>(&mut self, mask: T)
    where T: Into<u32>,
    {
        let mask_val = mask.into();
        let byte_index = mask_val >> 3;
        let bit_index = mask_val & 7;
        if let Some(byte) = self.flags.get_mut(byte_index as usize) {
            *byte |= BIT_POSITIONS[bit_index as usize];
        }
    }

    /// Returns true if the bit at the given mask index is set.
    pub fn contains_mask<T>(&self, mask: T) -> bool 
    where T: Into<u32>,
    {
        let mask_val = mask.into();
        let byte_index = mask_val >> 3;
        let bit_index = mask_val & 7;
        self.flags
            .get(byte_index as usize)
            .is_some_and(|&b| (b & BIT_POSITIONS[bit_index as usize]) != 0)
    }

    /// Returns the raw flags array (for debugging or export).
    #[must_use] 
    pub fn flags(&self) -> &[u8] {
        &self.flags
    }
}