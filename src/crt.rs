pub fn chinese_remainder(remainders: &[u64], moduli: &[u64]) -> Option<u64> {
    if remainders.len() != moduli.len() {
        return None;
    }

    let product_of_moduli: u64 = moduli.iter().product();
    let mut result = 0;

    for (&remainder, &modulus) in remainders.iter().zip(moduli) {
        let partial_product = product_of_moduli / modulus;
        let inverse = mod_inverse(partial_product, modulus)?;
        result += remainder * partial_product * inverse;
    }

    Some(result % product_of_moduli)
}

pub fn mod_inverse(a: u64, m: u64) -> Option<u64> {
    let mut mn = (m, a);
    let mut xy = (0, 1u64);

    while mn.1 != 0 {
        let quotient = mn.0 / mn.1;
        mn = (mn.1, mn.0 - quotient * mn.1);
        xy = (xy.1, xy.0 - quotient * xy.1);
    }

    if mn.0 > 1 {
        None
    } else {
        Some((xy.0 + m) % m)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chinese_remainder() {
        let result = chinese_remainder(&[2, 3, 2], &[3, 5, 7]);
        assert_eq!(result, Some(23));
    }

    #[test]
    fn test_mod_inverse() {
        assert_eq!(mod_inverse(3, 7), Some(5));
        assert_eq!(mod_inverse(4, 7), Some(2));
        assert_eq!(mod_inverse(2, 5), Some(3));
        assert_eq!(mod_inverse(3, 6), None);
    }
}
