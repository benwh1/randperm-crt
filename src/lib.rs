#![warn(clippy::must_use_candidate)]
#![deny(clippy::use_self)]
#![deny(clippy::double_must_use)]
#![deny(clippy::if_not_else)]
#![deny(clippy::inconsistent_struct_constructor)]
#![deny(clippy::iter_not_returning_iterator)]
#![deny(clippy::map_unwrap_or)]
#![deny(clippy::mod_module_files)]
#![deny(clippy::semicolon_if_nothing_returned)]

mod crt;

use rand::Rng;

#[derive(Debug, Clone, PartialEq, Eq)]
struct FactoredInteger {
    factors: Vec<(u8, u8)>,
}

impl FactoredInteger {
    fn new(mut n: u64) -> Option<Self> {
        let mut factors = Vec::new();

        let pow2 = n.trailing_zeros() as u8;
        if pow2 != 0 {
            n >>= pow2;
            factors.push((2, pow2));
        }

        for p in (3..u8::MAX).step_by(2) {
            let q = p as u64;

            let mut counter = 0;
            while n % q == 0 {
                counter += 1;
                n /= q;
            }

            if counter > 0 {
                factors.push((p, counter));
            }

            if n == 1 {
                break;
            }
        }

        if n == 1 {
            Some(Self { factors })
        } else {
            None
        }
    }
}

pub trait Permutation: Sized {
    fn num_points(&self) -> u64;
    fn nth(&self, n: u64) -> Option<u64>;

    fn iter(&self) -> PermutationIter<'_, Self> {
        PermutationIter { perm: self, idx: 0 }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RandomPermutation {
    num_points: u64,
    sub_perms: Vec<Vec<u64>>,
}

impl RandomPermutation {
    #[must_use]
    pub fn new(n: u64) -> Option<Self> {
        Self::with_rng(n, &mut rand::rng())
    }

    pub fn with_rng<R: Rng>(n: u64, rng: &mut R) -> Option<Self> {
        let factored_n = FactoredInteger::new(n)?;
        let num_prime_powers = factored_n.factors.len();

        let mut order = (0..num_prime_powers).collect::<Vec<_>>();
        for a in 0..num_prime_powers {
            let b = rng.random_range(a..num_prime_powers);
            order.swap(a, b);
        }

        let sub_perms = (0..num_prime_powers)
            .map(|i| {
                let (p, k) = factored_n.factors[order[i]];
                let pk = (p as u64).pow(k as u32);
                let mut vec = (0..pk).collect::<Vec<_>>();

                let pk = pk as usize;
                for a in 0..pk {
                    let b = rng.random_range(a..pk);
                    vec.swap(a, b);
                }

                vec
            })
            .collect();

        Some(Self {
            num_points: n,
            sub_perms,
        })
    }

    #[must_use]
    pub fn inverse(&self) -> Inverse<'_> {
        Inverse { perm: self }
    }
}

impl Permutation for RandomPermutation {
    fn num_points(&self) -> u64 {
        self.num_points
    }

    fn nth(&self, mut n: u64) -> Option<u64> {
        if n >= self.num_points {
            return None;
        }

        let remainders = self.sub_perms.iter().fold(Vec::new(), |mut rem, perm| {
            let pk = perm.len() as u64;
            rem.push(perm[(n % pk) as usize]);
            n /= pk;
            rem
        });

        let moduli = self
            .sub_perms
            .iter()
            .map(|perm| perm.len() as u64)
            .collect::<Vec<_>>();

        Some(crt::chinese_remainder(&remainders, &moduli).unwrap())
    }
}

pub struct Inverse<'a> {
    perm: &'a RandomPermutation,
}

impl Permutation for Inverse<'_> {
    fn num_points(&self) -> u64 {
        self.perm.num_points()
    }

    fn nth(&self, n: u64) -> Option<u64> {
        if n >= self.num_points() {
            None
        } else {
            Some(self.perm.sub_perms.iter().rev().fold(0, |idx, perm| {
                let pk = perm.len() as u64;
                let pos = perm.iter().position(|&a| a == n % pk).unwrap() as u64;
                idx * pk + pos
            }))
        }
    }
}

pub struct PermutationIter<'a, P: Permutation> {
    perm: &'a P,
    idx: u64,
}

impl<P: Permutation> Iterator for PermutationIter<'_, P> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        let a = self.perm.nth(self.idx);
        self.idx += 1;
        a
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.idx += n as u64;
        self.perm.nth(self.idx)
    }
}

pub struct Composition<'a> {
    perms: &'a [RandomPermutation],
}

impl<'a> Composition<'a> {
    #[must_use]
    pub fn new(perms: &'a [RandomPermutation]) -> Option<Self> {
        if perms.is_empty() {
            return None;
        }

        if !perms
            .iter()
            .all(|p| p.num_points() == perms[0].num_points())
        {
            return None;
        }

        Some(Self { perms })
    }
}

impl Permutation for Composition<'_> {
    fn num_points(&self) -> u64 {
        self.perms[0].num_points()
    }

    fn nth(&self, n: u64) -> Option<u64> {
        self.perms.iter().try_fold(n, |n, perm| perm.nth(n))
    }
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand_xoshiro::Xoshiro256StarStar;

    use crate::*;

    mod factored_integer {
        use super::*;

        #[test]
        fn test_new_1() {
            let n = FactoredInteger::new(14237396402848819200);
            assert_eq!(
                n,
                Some(FactoredInteger {
                    factors: vec![
                        (2, 11),
                        (3, 3),
                        (5, 2),
                        (7, 3),
                        (11, 4),
                        (13, 1),
                        (19, 3),
                        (23, 1)
                    ]
                })
            );
        }

        #[test]
        fn test_new_2() {
            let n = FactoredInteger::new(8929777156897433877);
            assert_eq!(
                n,
                Some(FactoredInteger {
                    factors: vec![(3, 25), (199, 1), (211, 1), (251, 1)]
                })
            );
        }

        #[test]
        fn test_new_3() {
            let n = FactoredInteger::new(2u64.pow(63));
            assert_eq!(
                n,
                Some(FactoredInteger {
                    factors: vec![(2, 63)]
                })
            );
        }

        #[test]
        fn test_new_4() {
            let n = FactoredInteger::new(257);
            assert_eq!(n, None);
        }

        #[test]
        fn test_new_5() {
            let n = FactoredInteger::new(1297068779 * 3196491187);
            assert_eq!(n, None);
        }
    }

    mod random_permutation {
        use super::*;

        #[test]
        fn test_random_permutation() {
            for seed in 0..10 {
                let mut rng = Xoshiro256StarStar::seed_from_u64(seed);

                let p = RandomPermutation::with_rng(362880, &mut rng).unwrap();
                p.nth(3);

                let mut vec = p.iter().collect::<Vec<_>>();
                vec.sort();

                assert!(vec.iter().copied().eq(0..362880));
            }
        }

        #[test]
        fn test_nth() {
            let mut rng = Xoshiro256StarStar::seed_from_u64(123456789);
            let p = RandomPermutation::with_rng((1..=20).product(), &mut rng).unwrap();

            assert_eq!(p.nth(0), Some(1541651892799010901));
            assert_eq!(p.nth(1), Some(682980595795490901));
            assert_eq!(p.nth(2), Some(2257211306968610901));
            assert_eq!(p.nth(285630477487272138), Some(0));
            assert_eq!(p.nth(1000000000000000000), Some(302819703353838471));
            assert_eq!(p.nth(2432902008176639999), Some(444231164144350210));
            assert_eq!(p.nth(2432902008176640000), None);
            assert_eq!(p.nth(u64::MAX), None);
        }

        #[test]
        fn test_nth_2() {
            let mut rng = Xoshiro256StarStar::seed_from_u64(123456789);
            let p = RandomPermutation::with_rng(300, &mut rng).unwrap();
            let v = p.iter().collect::<Vec<_>>();

            assert_eq!(
                v,
                &[
                    163, 271, 175, 235, 67, 7, 259, 187, 91, 103, 211, 199, 151, 43, 295, 247, 127,
                    115, 19, 31, 79, 223, 139, 55, 283, 88, 196, 100, 160, 292, 232, 184, 112, 16,
                    28, 136, 124, 76, 268, 220, 172, 52, 40, 244, 256, 4, 148, 64, 280, 208, 13,
                    121, 25, 85, 217, 157, 109, 37, 241, 253, 61, 49, 1, 193, 145, 97, 277, 265,
                    169, 181, 229, 73, 289, 205, 133, 238, 46, 250, 10, 142, 82, 34, 262, 166, 178,
                    286, 274, 226, 118, 70, 22, 202, 190, 94, 106, 154, 298, 214, 130, 58, 63, 171,
                    75, 135, 267, 207, 159, 87, 291, 3, 111, 99, 51, 243, 195, 147, 27, 15, 219,
                    231, 279, 123, 39, 255, 183, 288, 96, 0, 60, 192, 132, 84, 12, 216, 228, 36,
                    24, 276, 168, 120, 72, 252, 240, 144, 156, 204, 48, 264, 180, 108, 213, 21,
                    225, 285, 117, 57, 9, 237, 141, 153, 261, 249, 201, 93, 45, 297, 177, 165, 69,
                    81, 129, 273, 189, 105, 33, 138, 246, 150, 210, 42, 282, 234, 162, 66, 78, 186,
                    174, 126, 18, 270, 222, 102, 90, 294, 6, 54, 198, 114, 30, 258, 263, 71, 275,
                    35, 167, 107, 59, 287, 191, 203, 11, 299, 251, 143, 95, 47, 227, 215, 119, 131,
                    179, 23, 239, 155, 83, 188, 296, 200, 260, 92, 32, 284, 212, 116, 128, 236,
                    224, 176, 68, 20, 272, 152, 140, 44, 56, 104, 248, 164, 80, 8, 113, 221, 125,
                    185, 17, 257, 209, 137, 41, 53, 161, 149, 101, 293, 245, 197, 77, 65, 269, 281,
                    29, 173, 89, 5, 233, 38, 146, 50, 110, 242, 182, 134, 62, 266, 278, 86, 74, 26,
                    218, 170, 122, 2, 290, 194, 206, 254, 98, 14, 230, 158
                ]
            );
        }
    }

    mod inverse {
        use super::*;

        #[test]
        fn test_nth_1() {
            let mut rng = Xoshiro256StarStar::seed_from_u64(123456789);
            let p = RandomPermutation::with_rng((1..=20).product(), &mut rng).unwrap();
            let inv = p.inverse();

            assert_eq!(inv.nth(1541651892799010901), Some(0));
            assert_eq!(inv.nth(682980595795490901), Some(1));
            assert_eq!(inv.nth(2257211306968610901), Some(2));
            assert_eq!(inv.nth(0), Some(285630477487272138));
            assert_eq!(inv.nth(302819703353838471), Some(1000000000000000000));
            assert_eq!(inv.nth(444231164144350210), Some(2432902008176639999));
            assert_eq!(inv.nth(2432902008176640000), None);
            assert_eq!(inv.nth(u64::MAX), None);
        }

        #[test]
        fn test_nth_2() {
            let mut rng = Xoshiro256StarStar::seed_from_u64(123456789);
            let p = RandomPermutation::with_rng(300, &mut rng).unwrap();
            let inv = p.inverse();
            let v = inv.iter().collect::<Vec<_>>();

            assert_eq!(
                v,
                &[
                    127, 62, 291, 109, 45, 273, 194, 5, 249, 156, 78, 210, 132, 50, 297, 117, 33,
                    254, 188, 18, 239, 151, 90, 221, 136, 52, 287, 116, 34, 270, 198, 19, 230, 174,
                    81, 203, 135, 57, 275, 122, 42, 258, 179, 13, 243, 164, 76, 215, 146, 61, 277,
                    112, 41, 259, 195, 23, 244, 155, 99, 206, 128, 60, 282, 100, 47, 267, 183, 4,
                    238, 168, 89, 201, 140, 71, 286, 102, 37, 266, 184, 20, 248, 169, 80, 224, 131,
                    53, 285, 107, 25, 272, 192, 8, 229, 163, 93, 214, 126, 65, 296, 111, 27, 262,
                    191, 9, 245, 173, 94, 205, 149, 56, 278, 110, 32, 250, 197, 17, 233, 154, 88,
                    218, 139, 51, 290, 121, 36, 252, 187, 16, 234, 170, 98, 219, 130, 74, 281, 103,
                    35, 257, 175, 22, 242, 158, 79, 213, 143, 64, 276, 115, 46, 261, 177, 12, 241,
                    159, 95, 223, 144, 55, 299, 106, 28, 260, 182, 0, 247, 167, 83, 204, 138, 68,
                    289, 101, 40, 271, 186, 2, 237, 166, 84, 220, 148, 69, 280, 124, 31, 253, 185,
                    7, 225, 172, 92, 208, 129, 63, 293, 114, 26, 265, 196, 11, 227, 162, 91, 209,
                    145, 73, 294, 105, 49, 256, 178, 10, 232, 150, 97, 217, 133, 54, 288, 118, 39,
                    251, 190, 21, 236, 152, 87, 216, 134, 70, 298, 119, 30, 274, 181, 3, 235, 157,
                    75, 222, 142, 58, 279, 113, 43, 264, 176, 15, 246, 161, 77, 212, 141, 59, 295,
                    123, 44, 255, 199, 6, 228, 160, 82, 200, 147, 67, 283, 104, 38, 268, 189, 1,
                    240, 171, 86, 202, 137, 66, 284, 120, 48, 269, 180, 24, 231, 153, 85, 207, 125,
                    72, 292, 108, 29, 263, 193, 14, 226, 165, 96, 211
                ]
            );
        }

        #[test]
        fn test_nth_3() {
            let mut rng = Xoshiro256StarStar::seed_from_u64(123456789);
            let p = RandomPermutation::with_rng((1..=20).product(), &mut rng).unwrap();
            let inv = p.inverse();

            for i in 0..1000 {
                assert_eq!(p.nth(inv.nth(i).unwrap()), Some(i));
            }
        }
    }

    mod iterator {
        use super::*;

        #[test]
        fn test_next() {
            let mut rng = Xoshiro256StarStar::seed_from_u64(123456789);
            let p = RandomPermutation::with_rng(3113510400, &mut rng).unwrap();
            let mut iter = p.iter();

            for i in 0..1000 {
                assert_eq!(iter.next(), p.nth(i));
            }
        }

        #[test]
        fn test_nth() {
            let mut rng = Xoshiro256StarStar::seed_from_u64(123456789);
            let p = RandomPermutation::with_rng(3113510400, &mut rng).unwrap();
            let mut iter = p.iter();

            for i in 0..1000 {
                assert_eq!(iter.nth(1000000), p.nth((i + 1) * 1000000));
            }
        }
    }

    mod composition {
        use super::*;

        #[test]
        fn test_new_1() {
            let comp = Composition::new(&[]);
            assert!(comp.is_none());
        }

        #[test]
        fn test_new_2() {
            let mut rng = Xoshiro256StarStar::seed_from_u64(7777777);
            let p1 = RandomPermutation::with_rng(300, &mut rng).unwrap();
            let p2 = RandomPermutation::with_rng(400, &mut rng).unwrap();

            let v = vec![p1, p2];
            let comp = Composition::new(&v);

            assert!(comp.is_none());
        }

        #[test]
        fn test_nth() {
            let mut rng = Xoshiro256StarStar::seed_from_u64(7777777);
            let p1 = RandomPermutation::with_rng(300, &mut rng).unwrap();
            let p2 = RandomPermutation::with_rng(300, &mut rng).unwrap();

            let v = vec![p1.clone(), p2.clone()];
            let comp = Composition::new(&v).unwrap();

            for i in 0..300 {
                assert_eq!(comp.nth(i), p2.nth(p1.nth(i).unwrap()));
            }
        }
    }
}
