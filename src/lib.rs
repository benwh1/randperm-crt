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
    #[cfg(feature = "thread_rng")]
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

    const SEED: [u8; 32] = [
        144, 115, 104, 224, 226, 59, 231, 208, 100, 18, 137, 138, 234, 236, 129, 82, 184, 196, 19,
        43, 145, 94, 60, 77, 184, 198, 244, 164, 174, 224, 59, 152,
    ];

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
            let mut rng = Xoshiro256StarStar::from_seed(SEED);

            let p = RandomPermutation::with_rng(362880, &mut rng).unwrap();
            let mut vec = p.iter().collect::<Vec<_>>();
            vec.sort();

            assert!(vec.iter().copied().eq(0..362880));
        }

        #[test]
        fn test_nth() {
            let mut rng = Xoshiro256StarStar::from_seed(SEED);
            let p = RandomPermutation::with_rng((1..=20).product(), &mut rng).unwrap();

            assert_eq!(p.nth(0), Some(1319942158626894223));
            assert_eq!(p.nth(1), Some(2412265509236814223));
            assert_eq!(p.nth(2), Some(2263312325062734223));
            assert_eq!(p.nth(1000000000000000000), Some(320224176323398978));
            assert_eq!(p.nth(1146506666437230775), Some(0));
            assert_eq!(p.nth(2432902008176639999), Some(725036512928904481));
            assert_eq!(p.nth(2432902008176640000), None);
            assert_eq!(p.nth(u64::MAX), None);
        }

        #[test]
        fn test_nth_2() {
            let mut rng = Xoshiro256StarStar::from_seed(SEED);
            let p = RandomPermutation::with_rng(300, &mut rng).unwrap();
            let v = p.iter().collect::<Vec<_>>();

            assert_eq!(
                v,
                &[
                    176, 276, 76, 101, 201, 1, 26, 126, 226, 251, 51, 151, 200, 0, 100, 125, 225,
                    25, 50, 150, 250, 275, 75, 175, 152, 252, 52, 77, 177, 277, 2, 102, 202, 227,
                    27, 127, 224, 24, 124, 149, 249, 49, 74, 174, 274, 299, 99, 199, 8, 108, 208,
                    233, 33, 133, 158, 258, 58, 83, 183, 283, 284, 84, 184, 209, 9, 109, 134, 234,
                    34, 59, 159, 259, 236, 36, 136, 161, 261, 61, 86, 186, 286, 11, 111, 211, 104,
                    204, 4, 29, 129, 229, 254, 54, 154, 179, 279, 79, 212, 12, 112, 137, 237, 37,
                    62, 162, 262, 287, 87, 187, 272, 72, 172, 197, 297, 97, 122, 222, 22, 47, 147,
                    247, 68, 168, 268, 293, 93, 193, 218, 18, 118, 143, 243, 43, 56, 156, 256, 281,
                    81, 181, 206, 6, 106, 131, 231, 31, 80, 180, 280, 5, 105, 205, 230, 30, 130,
                    155, 255, 55, 44, 144, 244, 269, 69, 169, 194, 294, 94, 119, 219, 19, 128, 228,
                    28, 53, 153, 253, 278, 78, 178, 203, 3, 103, 20, 120, 220, 245, 45, 145, 170,
                    270, 70, 95, 195, 295, 164, 264, 64, 89, 189, 289, 14, 114, 214, 239, 39, 139,
                    296, 96, 196, 221, 21, 121, 146, 246, 46, 71, 171, 271, 260, 60, 160, 185, 285,
                    85, 110, 210, 10, 35, 135, 235, 116, 216, 16, 41, 141, 241, 266, 66, 166, 191,
                    291, 91, 92, 192, 292, 17, 117, 217, 242, 42, 142, 167, 267, 67, 248, 48, 148,
                    173, 273, 73, 98, 198, 298, 23, 123, 223, 32, 132, 232, 257, 57, 157, 182, 282,
                    82, 107, 207, 7, 140, 240, 40, 65, 165, 265, 290, 90, 190, 215, 15, 115, 188,
                    288, 88, 113, 213, 13, 38, 138, 238, 263, 63, 163
                ]
            );
        }
    }

    mod inverse {
        use super::*;

        #[test]
        fn test_nth_1() {
            let mut rng = Xoshiro256StarStar::from_seed(SEED);
            let p = RandomPermutation::with_rng((1..=20).product(), &mut rng).unwrap();
            let inv = p.inverse();

            assert_eq!(inv.nth(1319942158626894223), Some(0));
            assert_eq!(inv.nth(2412265509236814223), Some(1));
            assert_eq!(inv.nth(2263312325062734223), Some(2));
            assert_eq!(inv.nth(320224176323398978), Some(1000000000000000000));
            assert_eq!(inv.nth(0), Some(1146506666437230775));
            assert_eq!(inv.nth(725036512928904481), Some(2432902008176639999));
            assert_eq!(inv.nth(2432902008176640000), None);
            assert_eq!(inv.nth(u64::MAX), None);
        }

        #[test]
        fn test_nth_2() {
            let mut rng = Xoshiro256StarStar::from_seed(SEED);
            let p = RandomPermutation::with_rng(300, &mut rng).unwrap();
            let inv = p.inverse();
            let v = inv.iter().collect::<Vec<_>>();

            assert_eq!(
                v,
                &[
                    13, 5, 30, 178, 86, 147, 139, 275, 48, 64, 224, 81, 97, 293, 198, 286, 230,
                    243, 127, 167, 180, 208, 116, 261, 37, 17, 6, 34, 170, 87, 151, 143, 264, 52,
                    68, 225, 73, 101, 294, 202, 278, 231, 247, 131, 156, 184, 212, 117, 253, 41,
                    18, 10, 26, 171, 91, 155, 132, 268, 56, 69, 217, 77, 102, 298, 194, 279, 235,
                    251, 120, 160, 188, 213, 109, 257, 42, 22, 2, 27, 175, 95, 144, 136, 272, 57,
                    61, 221, 78, 106, 290, 195, 283, 239, 240, 124, 164, 189, 205, 113, 258, 46,
                    14, 3, 31, 179, 84, 148, 140, 273, 49, 65, 222, 82, 98, 291, 199, 287, 228,
                    244, 128, 165, 181, 209, 114, 262, 38, 15, 7, 35, 168, 88, 152, 141, 265, 53,
                    66, 226, 74, 99, 295, 203, 276, 232, 248, 129, 157, 185, 210, 118, 254, 39, 19,
                    11, 24, 172, 92, 153, 133, 269, 54, 70, 218, 75, 103, 299, 192, 280, 236, 249,
                    121, 161, 186, 214, 110, 255, 43, 23, 0, 28, 176, 93, 145, 137, 270, 58, 62,
                    219, 79, 107, 288, 196, 284, 237, 241, 125, 162, 190, 206, 111, 259, 47, 12, 4,
                    32, 177, 85, 149, 138, 274, 50, 63, 223, 83, 96, 292, 200, 285, 229, 245, 126,
                    166, 182, 207, 115, 263, 36, 16, 8, 33, 169, 89, 150, 142, 266, 51, 67, 227,
                    72, 100, 296, 201, 277, 233, 246, 130, 158, 183, 211, 119, 252, 40, 20, 9, 25,
                    173, 90, 154, 134, 267, 55, 71, 216, 76, 104, 297, 193, 281, 234, 250, 122,
                    159, 187, 215, 108, 256, 44, 21, 1, 29, 174, 94, 146, 135, 271, 59, 60, 220,
                    80, 105, 289, 197, 282, 238, 242, 123, 163, 191, 204, 112, 260, 45
                ]
            );
        }

        #[test]
        fn test_nth_3() {
            let mut rng = Xoshiro256StarStar::from_seed(SEED);
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
            let mut rng = Xoshiro256StarStar::from_seed(SEED);
            let p = RandomPermutation::with_rng(3113510400, &mut rng).unwrap();
            let mut iter = p.iter();

            for i in 0..1000 {
                assert_eq!(iter.next(), p.nth(i));
            }
        }

        #[test]
        fn test_nth() {
            let mut rng = Xoshiro256StarStar::from_seed(SEED);
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
            let mut rng = Xoshiro256StarStar::from_seed(SEED);
            let p1 = RandomPermutation::with_rng(300, &mut rng).unwrap();
            let p2 = RandomPermutation::with_rng(400, &mut rng).unwrap();

            let v = vec![p1, p2];
            let comp = Composition::new(&v);

            assert!(comp.is_none());
        }

        #[test]
        fn test_nth() {
            let mut rng = Xoshiro256StarStar::from_seed(SEED);
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
