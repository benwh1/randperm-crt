pub mod crt;

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

pub trait Permutation {
    fn num_points(&self) -> u64;
    fn nth(&self, n: u64) -> Option<u64>;
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

    pub fn iter(&self) -> RandomPermutationIter<'_> {
        RandomPermutationIter { perm: self, idx: 0 }
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

pub struct RandomPermutationIter<'a> {
    perm: &'a RandomPermutation,
    idx: u64,
}

impl Iterator for RandomPermutationIter<'_> {
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

pub struct Inverse<'a> {
    perm: &'a RandomPermutation,
}

impl<'a> Permutation for Inverse<'a> {
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

#[cfg(test)]
mod tests {
    mod factored_integer {
        use crate::*;

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
        use rand::SeedableRng;
        use rand_xoshiro::Xoshiro256StarStar;

        use crate::*;

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
    }

    mod inverse {
        use rand::SeedableRng;
        use rand_xoshiro::Xoshiro256StarStar;

        use crate::*;

        #[test]
        fn test_nth_1() {
            let mut rng = Xoshiro256StarStar::seed_from_u64(0);
            let p = RandomPermutation::with_rng((1..=20).product(), &mut rng).unwrap();
            let inv = p.inverse();

            assert_eq!(inv.nth(0), Some(2262432606807922128));
        }

        #[test]
        fn test_nth_2() {
            let mut rng = Xoshiro256StarStar::seed_from_u64(0);
            let p = RandomPermutation::with_rng((1..=20).product(), &mut rng).unwrap();
            let inv = p.inverse();

            for i in 0..1000 {
                assert_eq!(p.nth(inv.nth(i).unwrap()), Some(i));
            }
        }
    }
}
