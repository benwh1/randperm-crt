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

    pub fn num_points(&self) -> u64 {
        self.num_points
    }

    pub fn nth(&self, mut n: u64) -> Option<u64> {
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

    pub fn iter(&self) -> RandomPermutationIter<'_> {
        RandomPermutationIter { perm: self, idx: 0 }
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
