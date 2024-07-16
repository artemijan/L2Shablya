pub trait ToU16Array {
    fn to_u16_array(&self) -> Vec<u16>;
}

impl ToU16Array for [u8] {
    fn to_u16_array(&self) -> Vec<u16> {
        let mut utf16_vec = Vec::with_capacity(self.len() / 2);
        for i in (0..self.len()).step_by(2) {
            let code_unit = u16::from_le_bytes([self[i], self[i + 1]]);
            utf16_vec.push(code_unit);
        }
        utf16_vec
    }
}
