//! Morton encoding functions.
//!
//! Includes a Intel BMI2 version for ~10x speed.
//! Use `RUSTFLAGS="-C target-cpu=native"` when building to possibly
//! get the machine-dependent version

#![cfg_attr(all(feature = "nightly", test), feature(test))]
#![cfg_attr(feature = "nightly", feature(cfg_target_feature))]
#![cfg_attr(feature = "nightly", feature(link_llvm_intrinsics))]

#[cfg(not(all(feature = "nightly", target_feature = "bmi2")))]
pub use portable::{morton_encode, morton_decode};
#[cfg(all(feature = "nightly", target_feature = "bmi2"))]
pub use bmi::{morton_encode, morton_decode};

#[cfg(test)]
const INPUT: (u32, u32) = (0x123456, 0x456789);
#[cfg(test)]
const OUTPUT: u64 = 0x21262d3a9196;

#[cfg(all(feature = "nightly", target_feature = "bmi2"))]
pub mod bmi {

    mod x86 {
        extern "C" {
            #[link_name = "llvm.x86.bmi.pdep.64"]
            pub fn bmi_pdep_64(a: i64, b: i64) -> i64;

            #[link_name = "llvm.x86.bmi.pext.64"]
            pub fn bmi_pext_64(a: i64, b: i64) -> i64;
        }
    }

    const PATTERN: i64 = 0x5555555555555555;

    pub fn morton_encode(x: u32, y: u32) -> u64 {
        unsafe {
            ((x86::bmi_pdep_64(y as i64, PATTERN) << 1) |
             x86::bmi_pdep_64(x as i64, PATTERN)) as u64
        }
    }

    pub fn morton_decode(a: u64) -> (u32, u32) {
        unsafe {
            (x86::bmi_pext_64(a as i64, PATTERN) as u32,
             x86::bmi_pext_64(a as i64 >> 1, PATTERN) as u32)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::super::{INPUT, OUTPUT};

        #[test]
        fn test_morton_encode() {
            let (x, y) = INPUT;
            let encoded = super::morton_encode(x, y);
            println!("bmi::morton_encode({}, {}) -> {}", x, y, encoded);
            assert_eq!(OUTPUT, encoded);
        }

        #[test]
        fn test_morton_decode() {
            let decoded = super::morton_decode(OUTPUT);
            println!("bmi::morton_decode({}) -> {:?}", OUTPUT, decoded);

            assert_eq!(INPUT, decoded);
        }

        extern crate test;

        #[bench]
        fn bench_1k_morton_decode(b: &mut test::Bencher) {
            let x = test::black_box(0x5555555555555555);
            b.iter(|| {
                for _ in 0..1_000 {
                    let coords = super::morton_decode(x);
                    test::black_box(coords);
                }
            });
        }

        #[bench]
        fn bench_1k_morton_encode(b: &mut test::Bencher) {
            let (x, y) = test::black_box(INPUT);
            b.iter(|| {
                for _ in 0..1_000 {
                    let encoded = super::morton_encode(x, y);
                    test::black_box(encoded);
                }
            });
        }
    }
}

pub mod portable {

    fn part1by1(x: u32) -> u64 {
        let mut x = x as u64;
        x &= 0x00000000ffffffff;
        x = (x ^ (x << 16)) & 0x0000ffff0000ffff;
        x = (x ^ (x << 8)) & 0x00ff00ff00ff00ff;
        x = (x ^ (x << 4)) & 0x0f0f0f0f0f0f0f0f;
        x = (x ^ (x << 2)) & 0x3333333333333333;
        x = (x ^ (x << 1)) & 0x5555555555555555;
        x
    }

    pub fn morton_encode(x: u32, y: u32) -> u64 {
        (part1by1(y) << 1) + part1by1(x)
    }


    fn compact1by1(mut x: u64) -> u32 {
        x &= 0x5555555555555555;
        x = (x ^ (x >> 1)) & 0x3333333333333333;
        x = (x ^ (x >> 2)) & 0x0f0f0f0f0f0f0f0f;
        x = (x ^ (x >> 4)) & 0x00ff00ff00ff00ff;
        x = (x ^ (x >> 8)) & 0x0000ffff0000ffff;
        x = (x ^ (x >> 16)) & 0x00000000ffffffff;
        x as u32
    }

    pub fn morton_decode(x: u64) -> (u32, u32) {
        (compact1by1(x), compact1by1(x >> 1))
    }

    #[cfg(test)]
    mod tests {
        use super::super::{INPUT, OUTPUT};

        #[test]
        fn test_morton_encode() {
            let (x, y) = INPUT;
            let encoded = super::morton_encode(x, y);
            println!("bmi::morton_encode({}, {}) -> {}", x, y, encoded);
            assert_eq!(OUTPUT, encoded);
        }

        #[test]
        fn test_morton_decode() {
            let decoded = super::morton_decode(OUTPUT);
            println!("bmi::morton_decode({}) -> {:?}", OUTPUT, decoded);

            assert_eq!(INPUT, decoded);
        }

        #[cfg(feature = "nightly")]
        extern crate test;

        #[cfg(feature = "nightly")]
        #[bench]
        fn bench_1k_morton_decode(b: &mut test::Bencher) {
            let x = test::black_box(0x5555555555555555);
            b.iter(|| {
                for _ in 0..1_000 {
                    let coords = super::morton_decode(x);
                    test::black_box(coords);
                }
            });
        }

        #[cfg(feature = "nightly")]
        #[bench]
        fn bench_1k_morton_encode(b: &mut test::Bencher) {
            let (x, y) = test::black_box(INPUT);
            b.iter(|| {
                for _ in 0..1_000 {
                    let encoded = super::morton_encode(x, y);
                    test::black_box(encoded);
                }
            });
        }
    }
}
