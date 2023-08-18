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
    pub fn new(n: u64) -> Option<Self> {
        Self::with_rng(n, &mut rand::thread_rng())
    }

    pub fn with_rng<R: Rng>(n: u64, rng: &mut R) -> Option<Self> {
        let factored_n = FactoredInteger::new(n)?;
        let num_prime_powers = factored_n.factors.len();

        let mut order = (0..num_prime_powers).collect::<Vec<_>>();
        for a in 0..num_prime_powers {
            let b = rng.gen_range(a..num_prime_powers);
            order.swap(a, b);
        }

        let sub_perms = (0..num_prime_powers)
            .map(|i| {
                let (p, k) = factored_n.factors[order[i]];
                let pk = (p as u64).pow(k as u32);
                let mut vec = (0..pk).collect::<Vec<_>>();

                let pk = pk as usize;
                for a in 0..pk {
                    let b = rng.gen_range(a..pk);
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
            let pk = perm.len();
            rem.push(perm[n as usize % pk]);
            n /= pk as u64;
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
    pub fn new(perms: &'a [RandomPermutation]) -> Option<Self> {
        if perms.len() == 0 {
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
        self.perms.iter().fold(Some(n), |n, perm| perm.nth(n?))
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
            let mut rng = Xoshiro256StarStar::seed_from_u64(0);
            let p = RandomPermutation::with_rng((1..=20).product(), &mut rng).unwrap();

            assert_eq!(p.nth(0), Some(1021963400465470519));
            assert_eq!(p.nth(1), Some(1816380382727230519));
            assert_eq!(p.nth(2), Some(575103847943230519));
            assert_eq!(p.nth(1000000000000000000), Some(2281814948517154192));
            assert_eq!(p.nth(2262432606807922128), Some(0));
            assert_eq!(p.nth(2432902008176639999), Some(2366747676416260137));
            assert_eq!(p.nth(2432902008176640000), None);
            assert_eq!(p.nth(u64::MAX), None);
        }

        #[test]
        fn test_nth_2() {
            let mut rng = Xoshiro256StarStar::seed_from_u64(0);
            let p = RandomPermutation::with_rng(300, &mut rng).unwrap();
            let v = p.iter().collect::<Vec<_>>();

            assert_eq!(
                v,
                &[
                    18, 186, 114, 198, 6, 54, 258, 162, 126, 102, 294, 174, 270, 78, 90, 222, 42,
                    30, 210, 282, 234, 246, 150, 138, 66, 218, 86, 14, 98, 206, 254, 158, 62, 26,
                    2, 194, 74, 170, 278, 290, 122, 242, 230, 110, 182, 134, 146, 50, 38, 266, 118,
                    286, 214, 298, 106, 154, 58, 262, 226, 202, 94, 274, 70, 178, 190, 22, 142,
                    130, 10, 82, 34, 46, 250, 238, 166, 93, 261, 189, 273, 81, 129, 33, 237, 201,
                    177, 69, 249, 45, 153, 165, 297, 117, 105, 285, 57, 9, 21, 225, 213, 141, 293,
                    161, 89, 173, 281, 29, 233, 137, 101, 77, 269, 149, 245, 53, 65, 197, 17, 5,
                    185, 257, 209, 221, 125, 113, 41, 193, 61, 289, 73, 181, 229, 133, 37, 1, 277,
                    169, 49, 145, 253, 265, 97, 217, 205, 85, 157, 109, 121, 25, 13, 241, 243, 111,
                    39, 123, 231, 279, 183, 87, 51, 27, 219, 99, 195, 3, 15, 147, 267, 255, 135,
                    207, 159, 171, 75, 63, 291, 143, 11, 239, 23, 131, 179, 83, 287, 251, 227, 119,
                    299, 95, 203, 215, 47, 167, 155, 35, 107, 59, 71, 275, 263, 191, 43, 211, 139,
                    223, 31, 79, 283, 187, 151, 127, 19, 199, 295, 103, 115, 247, 67, 55, 235, 7,
                    259, 271, 175, 163, 91, 168, 36, 264, 48, 156, 204, 108, 12, 276, 252, 144, 24,
                    120, 228, 240, 72, 192, 180, 60, 132, 84, 96, 0, 288, 216, 68, 236, 164, 248,
                    56, 104, 8, 212, 176, 152, 44, 224, 20, 128, 140, 272, 92, 80, 260, 32, 284,
                    296, 200, 188, 116, 268, 136, 64, 148, 256, 4, 208, 112, 76, 52, 244, 124, 220,
                    28, 40, 172, 292, 280, 160, 232, 184, 196, 100, 88, 16
                ]
            );
        }
    }

    mod inverse {
        use super::*;

        #[test]
        fn test_nth_1() {
            let mut rng = Xoshiro256StarStar::seed_from_u64(0);
            let p = RandomPermutation::with_rng((1..=20).product(), &mut rng).unwrap();
            let inv = p.inverse();

            assert_eq!(inv.nth(1021963400465470519), Some(0));
            assert_eq!(inv.nth(1816380382727230519), Some(1));
            assert_eq!(inv.nth(575103847943230519), Some(2));
            assert_eq!(inv.nth(2281814948517154192), Some(1000000000000000000));
            assert_eq!(inv.nth(0), Some(2262432606807922128));
            assert_eq!(inv.nth(2366747676416260137), Some(2432902008176639999));
            assert_eq!(inv.nth(2432902008176640000), None);
            assert_eq!(inv.nth(u64::MAX), None);
        }

        #[test]
        fn test_nth_2() {
            let mut rng = Xoshiro256StarStar::seed_from_u64(0);
            let p = RandomPermutation::with_rng(300, &mut rng).unwrap();
            let inv = p.inverse();
            let v = inv.iter().collect::<Vec<_>>();

            assert_eq!(
                v,
                &[
                    247, 133, 34, 163, 280, 117, 4, 219, 256, 95, 68, 176, 232, 148, 27, 164, 299,
                    116, 0, 210, 262, 96, 65, 178, 236, 147, 33, 159, 288, 105, 17, 204, 269, 81,
                    70, 193, 226, 132, 48, 152, 289, 124, 16, 200, 260, 87, 71, 190, 228, 136, 47,
                    158, 284, 113, 5, 217, 254, 94, 56, 195, 243, 126, 32, 173, 277, 114, 24, 216,
                    250, 85, 62, 196, 240, 128, 36, 172, 283, 109, 13, 205, 267, 79, 69, 181, 245,
                    143, 26, 157, 298, 102, 14, 224, 266, 75, 60, 187, 246, 140, 28, 161, 297, 108,
                    9, 213, 255, 92, 54, 194, 231, 145, 43, 151, 282, 123, 2, 214, 274, 91, 50,
                    185, 237, 146, 40, 153, 286, 122, 8, 209, 263, 80, 67, 179, 244, 131, 45, 168,
                    276, 107, 23, 202, 264, 99, 66, 175, 235, 137, 46, 165, 278, 111, 22, 208, 259,
                    88, 55, 192, 229, 144, 31, 170, 293, 101, 7, 223, 252, 89, 74, 191, 225, 135,
                    37, 171, 290, 103, 11, 222, 258, 84, 63, 180, 242, 129, 44, 156, 295, 118, 1,
                    207, 273, 77, 64, 199, 241, 125, 35, 162, 296, 115, 3, 211, 272, 83, 59, 188,
                    230, 142, 29, 169, 281, 120, 18, 201, 257, 98, 52, 189, 249, 141, 25, 160, 287,
                    121, 15, 203, 261, 97, 58, 184, 238, 130, 42, 154, 294, 106, 20, 218, 251, 82,
                    73, 177, 239, 149, 41, 150, 285, 112, 21, 215, 253, 86, 72, 183, 234, 138, 30,
                    167, 279, 119, 6, 220, 268, 76, 57, 198, 227, 139, 49, 166, 275, 110, 12, 221,
                    265, 78, 61, 197, 233, 134, 38, 155, 292, 104, 19, 206, 270, 93, 51, 182, 248,
                    127, 39, 174, 291, 100, 10, 212, 271, 90, 53, 186
                ]
            );
        }

        #[test]
        fn test_nth_3() {
            let mut rng = Xoshiro256StarStar::seed_from_u64(0);
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
