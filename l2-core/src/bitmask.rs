#[derive(Debug, Clone)]
pub struct BitMask {
    masks: Vec<u8>,
}

impl BitMask {
    #[must_use]
    pub fn new(size: usize) -> Self {
        Self {
            masks: vec![0; size],
        }
    }

    pub fn add_mask(&mut self, mask: u32) {
        let byte_index = (mask / 8) as usize;
        let bit_position = mask % 8;
        self.masks[byte_index] |= 1 << bit_position;
    }

    #[must_use]
    pub fn contains_mask(&self, mask: u32) -> bool {
        let byte_index = (mask / 8) as usize;
        let bit_position = mask % 8;
        (self.masks[byte_index] & (1 << bit_position)) != 0
    }
}
