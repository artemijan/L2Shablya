use crate::crypt::constants::{KP0, KS0, KS1, KS2, KS3, P_SZ, SBOX_SK};
use encoding::all;
use rand::prelude::*;
use rand::thread_rng;

pub const BLOWFISH_KEY_SIZE: usize = 16;

pub fn generate_blowfish_key() -> [u8; BLOWFISH_KEY_SIZE] {
    let mut key = [0u8; BLOWFISH_KEY_SIZE];
    let mut rng = thread_rng();
    for item in key.iter_mut().take(BLOWFISH_KEY_SIZE) {
        *item = rng.gen();
    }
    key
}

pub static STATIC_BLOWFISH_KEY: [u8; 16] = [154, 125, 7, 25, 132, 212, 137, 240, 220, 37, 6, 180, 21, 131, 47, 197];

#[derive(Debug, PartialEq, Clone, Copy)]
enum BFTables {
    P,
    S0,
    S1,
    S2,
    S3,
}

#[derive(Debug, Clone)]
pub struct BlowfishEngine {
    s0: [i32; 256],
    s1: [i32; 256],
    s2: [i32; 256],
    s3: [i32; 256],
    p: [i32; 18],
    ks0: [i32; 256],
    ks1: [i32; 256],
    ks2: [i32; 256],
    ks3: [i32; 256],
    kp0: [i32; 18],
}
#[allow(clippy::similar_names)]
impl BlowfishEngine {
    pub fn new(key: &[u8]) -> BlowfishEngine {
        let ks0: [i32; 256] = KS0;
        let ks1: [i32; 256] = KS1;
        let ks2: [i32; 256] = KS2;
        let ks3: [i32; 256] = KS3;
        let kp0: [i32; 18] = KP0;
        let mut engine = BlowfishEngine {
            ks0,
            ks1,
            ks2,
            ks3,
            kp0,
            s0: ks0,
            s1: ks1,
            s2: ks2,
            s3: ks3,
            p: kp0,
        };
        engine.set_key(key);
        engine
    }

    #[allow(clippy::cast_sign_loss)]
    fn func(&self, x: i32) -> i32 {
        let index0 = (x >> 24) & 0xFF;
        let index1 = (x >> 16) & 0xFF;
        let index2 = (x >> 8) & 0xFF;
        let index3 = x & 0xFF;
        let mut result = self.s0[index0 as usize].wrapping_add(self.s1[index1 as usize]);
        result ^= self.s2[index2 as usize];
        result.wrapping_add(self.s3[index3 as usize])
    }

    fn process_table(&mut self, mut xl: i32, mut xr: i32, table_type: BFTables, size: usize) {
        for s in (0..size).step_by(2) {
            xl ^= self.p[0];
            xr ^= self.func(xl) ^ self.p[1];
            xl ^= self.func(xr) ^ self.p[2];
            xr ^= self.func(xl) ^ self.p[3];
            xl ^= self.func(xr) ^ self.p[4];
            xr ^= self.func(xl) ^ self.p[5];
            xl ^= self.func(xr) ^ self.p[6];
            xr ^= self.func(xl) ^ self.p[7];
            xl ^= self.func(xr) ^ self.p[8];
            xr ^= self.func(xl) ^ self.p[9];
            xl ^= self.func(xr) ^ self.p[10];
            xr ^= self.func(xl) ^ self.p[11];
            xl ^= self.func(xr) ^ self.p[12];
            xr ^= self.func(xl) ^ self.p[13];
            xl ^= self.func(xr) ^ self.p[14];
            xr ^= self.func(xl) ^ self.p[15];
            xl ^= self.func(xr) ^ self.p[16];
            xr ^= self.p[17];
            match table_type {
                BFTables::S0 => {
                    (self.s0[s], self.s0[s + 1]) = (xr, xl);
                    (xr, xl) = (xl, self.s0[s]);
                }
                BFTables::S1 => {
                    (self.s1[s], self.s1[s + 1]) = (xr, xl);
                    (xr, xl) = (xl, self.s1[s]);
                }
                BFTables::S2 => {
                    (self.s2[s], self.s2[s + 1]) = (xr, xl);
                    (xr, xl) = (xl, self.s2[s]);
                }
                BFTables::S3 => {
                    (self.s3[s], self.s3[s + 1]) = (xr, xl);
                    (xr, xl) = (xl, self.s3[s]);
                }
                BFTables::P => {
                    (self.p[s], self.p[s + 1]) = (xr, xl);
                    (xr, xl) = (xl, self.p[s]);
                }
            };
        }
    }
    fn process_p(&mut self, key: &[u8]) {
        let key_length = key.len();
        let mut key_index = 0;
        for i in 0..P_SZ {
            let mut data = 0x0000_0000;
            for _ in 0..4 {
                data = (data << 8) | (i32::from(key[key_index]) & 0xff);
                key_index = (key_index + 1) % key_length;
            }
            self.p[i] ^= data;
        }
    }
    fn set_key(&mut self, key: &[u8]) {
        self.process_p(key);
        self.set_p_key();
        self.set_s0_key();
        self.set_s1_key();
        self.set_s2_key();
        self.set_s3_key();
    }

    pub fn encrypt_block(&self, raw: &mut [u8], index: usize) {
        let mut xl = Self::bytes_to_32bits(raw, index);
        let mut xr = Self::bytes_to_32bits(raw, index + 4);
        xl ^= self.p[0];
        xr ^= self.func(xl) ^ self.p[1];
        xl ^= self.func(xr) ^ self.p[2];
        xr ^= self.func(xl) ^ self.p[3];
        xl ^= self.func(xr) ^ self.p[4];
        xr ^= self.func(xl) ^ self.p[5];
        xl ^= self.func(xr) ^ self.p[6];
        xr ^= self.func(xl) ^ self.p[7];
        xl ^= self.func(xr) ^ self.p[8];
        xr ^= self.func(xl) ^ self.p[9];
        xl ^= self.func(xr) ^ self.p[10];
        xr ^= self.func(xl) ^ self.p[11];
        xl ^= self.func(xr) ^ self.p[12];
        xr ^= self.func(xl) ^ self.p[13];
        xl ^= self.func(xr) ^ self.p[14];
        xr ^= self.func(xl) ^ self.p[15];
        xl ^= self.func(xr) ^ self.p[16];
        xr ^= self.p[17];
        Self::bits32_to_bytes(xr, raw, index);
        Self::bits32_to_bytes(xl, raw, index + 4);
    }
    pub fn decrypt_block(&self, raw: &mut [u8], index: usize) {
        let mut xl = Self::bytes_to_32bits(raw, index);
        let mut xr = Self::bytes_to_32bits(raw, index + 4);
        xl ^= self.p[17];
        xr ^= self.func(xl) ^ self.p[16];
        xl ^= self.func(xr) ^ self.p[15];
        xr ^= self.func(xl) ^ self.p[14];
        xl ^= self.func(xr) ^ self.p[13];
        xr ^= self.func(xl) ^ self.p[12];
        xl ^= self.func(xr) ^ self.p[11];
        xr ^= self.func(xl) ^ self.p[10];
        xl ^= self.func(xr) ^ self.p[9];
        xr ^= self.func(xl) ^ self.p[8];
        xl ^= self.func(xr) ^ self.p[7];
        xr ^= self.func(xl) ^ self.p[6];
        xl ^= self.func(xr) ^ self.p[5];
        xr ^= self.func(xl) ^ self.p[4];
        xl ^= self.func(xr) ^ self.p[3];
        xr ^= self.func(xl) ^ self.p[2];
        xl ^= self.func(xr) ^ self.p[1];
        xr ^= self.p[0];
        Self::bits32_to_bytes(xr, raw, index);
        Self::bits32_to_bytes(xl, raw, index + 4);
    }

    #[allow(arithmetic_overflow)]
    #[allow(overflowing_literals)]
    fn bytes_to_32bits(src: &[u8], index: usize) -> i32 {
        let mut k = (i32::from(src[index + 3]) & 0xff) << 24;
        k |= (i32::from(src[index + 2]) & 0xff) << 16;
        k |= (i32::from(src[index + 1]) & 0xff) << 8;
        k | i32::from(src[index]) & 0xff
    }
    #[allow(clippy::cast_sign_loss)]
    fn bits32_to_bytes(in_value: i32, dst: &mut [u8], dst_index: usize) {
        dst[dst_index] = (in_value & 0xFF) as u8;
        dst[dst_index + 1] = ((in_value >> 8) & 0xFF) as u8;
        dst[dst_index + 2] = ((in_value >> 16) & 0xFF) as u8;
        dst[dst_index + 3] = ((in_value >> 24) & 0xFF) as u8;
    }
    fn set_p_key(&mut self) {
        self.process_table(0, 0, BFTables::P, self.p.len());
    }
    fn set_s0_key(&mut self) {
        self.process_table(self.p[P_SZ - 2], self.p[P_SZ - 1], BFTables::S0, self.s0.len());
    }
    fn set_s1_key(&mut self) {
        self.process_table(self.s0[SBOX_SK - 2], self.s0[SBOX_SK - 1], BFTables::S1, self.s1.len());
    }
    fn set_s2_key(&mut self) {
        self.process_table(self.s1[SBOX_SK - 2], self.s1[SBOX_SK - 1], BFTables::S2, self.s2.len());
    }
    fn set_s3_key(&mut self) {
        self.process_table(self.s2[SBOX_SK - 2], self.s2[SBOX_SK - 1], BFTables::S3, self.s3.len());
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;
    use crate::crypt::constants;

    #[test]
    fn test_bytes_to_32_bits() {
        let (engine, _) = gen_bf_engine_without_key_being_set();
        let num = BlowfishEngine::bytes_to_32bits(&[255, 246, 87, 7], 0); // [-1, -10, 87, 7] / [255, 246, 87, 7]
        let expected = 123_205_375;
        assert_eq!(expected, num, "Conversion is not okay");
    }

    #[test]
    fn test_32_bits_to_bytes() {
        let (engine, _) = gen_bf_engine_without_key_being_set();
        let mut res = [0; 4];
        BlowfishEngine::bits32_to_bytes(123_205_375, &mut res, 0); // [-1, -10, 87, 7] / [255, 246, 87, 7]
        assert_eq!(&res, &[255, 246, 87, 7], "Conversion is not okay");
    }

    #[test]
    fn test_process_p() {
        let (mut engine, key) = gen_bf_engine_without_key_being_set();
        engine.process_p(&key);
        let expected = [
            1_331_667_411,
            124_622_946,
            -550_312_325,
            1_864_113_960,
            -815_139_975,
            -1_420_713_631,
            -1_006_266_675,
            -2_145_255_195,
            776_530_621,
            -1_172_405_306,
            1_920_928_410,
            1_485_135_872,
            -1_412_635_924,
            1_270_005_868,
            -206_587_424,
            -651_467_397,
            -109_699_454,
            196_570_026,
        ];
        assert_eq!(engine.p, expected, "Blowfish P key must equal");
    }

    #[test]
    #[allow(clippy::similar_names)]
    fn test_func() {
        let mut xl: i32 = 0;
        let mut xr: i32 = 0;
        let (mut engine, key) = gen_bf_engine_without_key_being_set();
        engine.process_p(&key);
        let expected_xr = -1_582_570_961;
        let expected_xl = 1_331_667_411;
        xl ^= engine.p[0];
        xr ^= engine.func(xl) ^ engine.p[1];
        assert_eq!(xl, expected_xl, "func is incorrect");
        assert_eq!(xr, expected_xr, "func is incorrect");
    }

    #[test]
    fn test_set_p_key() {
        let (mut engine, key) = gen_bf_engine_without_key_being_set();
        let expected1 = [
            1_331_667_411,
            124_622_946,
            -550_312_325,
            1_864_113_960,
            -815_139_975,
            -1_420_713_631,
            -1_006_266_675,
            -2_145_255_195,
            776_530_621,
            -1_172_405_306,
            1_920_928_410,
            1_485_135_872,
            -1_412_635_924,
            1_270_005_868,
            -206_587_424,
            -651_467_397,
            -109_699_454,
            196_570_026,
        ];
        engine.process_p(&key);
        assert_eq!(engine.p, expected1, "P key must be correct");
        engine.set_p_key();
        let expected = [
            1_380_425_361,
            818_284_978,
            1_094_506_548,
            982_513_019,
            -2_036_939_528,
            1_971_259_599,
            2_077_564_926,
            -1_256_875_534,
            -1_570_502_760,
            -986_638_966,
            -1_128_515_323,
            -1_248_297_172,
            -539_348_144,
            -1_640_291_793,
            1_015_029_596,
            -1_497_303_223,
            1_368_507_002,
            -1_560_842_836,
        ];
        assert_eq!(engine.p, expected, "Expect P key to be correct");
    }

    fn csv_column_to_vector<T>(data: &str) -> Vec<T>
    where
        T: FromStr,
        T::Err: std::fmt::Debug, // To handle parsing errors with expect
    {
        data.trim()
            .replace('_', "")
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| s.parse::<T>().unwrap_or_else(|_| panic!("Can't parse value {s}")))
            .collect()
    }

    #[test]
    #[allow(clippy::cast_possible_truncation)]
    fn test_set_key_works() {
        let s0 = csv_column_to_vector::<i64>(include_str!("../test_data/s0.csv"));
        let s1 = csv_column_to_vector::<i64>(include_str!("../test_data/s1.csv"));
        let s2 = csv_column_to_vector::<i64>(include_str!("../test_data/s2.csv"));
        let s3 = csv_column_to_vector::<i64>(include_str!("../test_data/s3.csv"));
        let p = csv_column_to_vector::<i64>(include_str!("../test_data/p.csv"));
        let key = [107, 96, 203, 91, 130, 206, 144, 177, 204, 43, 108, 85, 108, 108, 108, 108];
        let decryptor = BlowfishEngine::new(&key);
        assert_eq!(
            decryptor.p,
            p.into_iter().map(|x| x as i32).collect::<Vec<i32>>().as_slice(),
            "P not equal"
        );
        assert_eq!(
            decryptor.s0,
            s0.into_iter().map(|x| x as i32).collect::<Vec<i32>>().as_slice(),
            "S0 not equal"
        );
        assert_eq!(
            decryptor.s1,
            s1.into_iter().map(|x| x as i32).collect::<Vec<i32>>().as_slice(),
            "S1 not equal"
        );
        assert_eq!(
            decryptor.s2,
            s2.into_iter().map(|x| x as i32).collect::<Vec<i32>>().as_slice(),
            "S2 not equal"
        );
        assert_eq!(
            decryptor.s3,
            s3.into_iter().map(|x| x as i32).collect::<Vec<i32>>().as_slice(),
            "S3 not equal"
        );
    }

    #[allow(clippy::similar_names)]
    fn gen_bf_engine_without_key_being_set() -> (BlowfishEngine, Vec<u8>) {
        let key = [107, 96, 203, 91, 130, 206, 144, 177, 204, 43, 108, 85, 108, 108, 108, 108];
        let ks0 = KS0;
        let ks1 = KS1;
        let ks2 = KS2;
        let ks3 = KS3;
        let kp0 = KP0;
        (
            BlowfishEngine {
                ks0,
                ks1,
                ks2,
                ks3,
                kp0,
                s0: ks0,
                s1: ks1,
                s2: ks2,
                s3: ks3,
                p: kp0,
            },
            key.to_vec(),
        )
    }
}
