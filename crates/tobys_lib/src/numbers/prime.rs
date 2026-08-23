//! Prime number related functions.

#[cfg(all(feature = "bigint", feature = "rand"))]
mod rabin_miller {
    use num_bigint::{BigRng010, BigUint, ToBigUint};
    use num_traits::{One, Zero};
    use rand::Rng;

    macro_rules! biguint {
        ($e:expr) => {
            ($e).to_biguint().unwrap()
        };
    }

    #[cfg(all(feature = "bigint", feature = "rand"))]
    fn miller_rabin_decompose(n: &BigUint) -> (u64, BigUint) {
        assert!(!n.is_zero() && !n.is_one());

        #[allow(clippy::arithmetic_side_effects, reason = "n is at least 2")]
        let n = n - 1u8; // n is not zero as per assertion
        let s = n.trailing_zeros().unwrap();

        #[allow(
            clippy::arithmetic_side_effects,
            reason = "causes no overflow/underflow"
        )]
        (s, n >> s)
    }
    fn miller_rabin(
        base: &BigUint,
        num: &BigUint,
        s: u64,
        d: &BigUint,
    ) -> bool {
        assert!(!num.is_zero());

        #[allow(clippy::arithmetic_side_effects, reason = "n is at least 1")]
        let n_minus_1 = num - 1u8;
        let one = BigUint::one();
        let two = biguint!(2);

        let mut x = base.modpow(d, num);
        let mut y = one.clone();

        for _ in 0..s {
            y = x.modpow(&two, num);
            if y == one && x != one && x != n_minus_1 {
                return false;
            }
            x.clone_from(&y);
        }
        y == one
    }

    /// Return `true` if `n` is a probable prime.
    ///
    /// Uses the Miller-Rabin primality test, testing `k` times.
    /// The false positive risk is bounded by 4^<sup>-*k*</sup>.
    ///
    /// You must supply some form of struct that implements [`Rng`],
    /// that is also *uniformly* random.
    ///
    /// # Panics
    ///
    /// Panics if `n` cannot become a [`BigUint`].
    #[cfg(all(feature = "bigint", feature = "rand"))]
    #[cfg_attr(docsrs, doc(cfg(all(feature = "bigint", feature = "rand"))))]
    pub fn is_probable_prime<T: ToBigUint, R: Rng>(
        n: &T,
        k: usize,
        rng: &mut R,
    ) -> bool {
        use crate::alias::{Vec, repeat_with, vec};

        let n = &n.to_biguint().unwrap();
        let two = &biguint!(2);

        if n <= &BigUint::one() {
            return false;
        } else if n <= &biguint!(3) {
            return true;
        } else if n <= &biguint!(0xFFFF_FFFF_FFFF_FFFFu64) {
            #[allow(
                clippy::arithmetic_side_effects,
                reason = "n is at least 3"
            )]
            let n_minus_one: BigUint = n - 1u8;
            let (s, d) = miller_rabin_decompose(n);

            // if n less than u64, simply use 16 small known primes
            let samples = vec![
                2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53,
            ];

            return samples
                .iter()
                .filter(|&&m| biguint!(m) < n_minus_one)
                .find(|&&a| miller_rabin(&biguint!(a), n, s, &d))
                .is_none();
        }

        #[allow(clippy::arithmetic_side_effects, reason = "n is at least 3")]
        let n_minus_one: BigUint = n - 1u8;
        let (s, d) = miller_rabin_decompose(n);

        #[allow(
            clippy::cast_possible_truncation,
            reason = "bits will never overflow??"
        )]
        let bits = n_minus_one.bits() as u32;
        #[allow(
            clippy::arithmetic_side_effects,
            reason = "bits should be at least 2"
        )]
        let min = two.pow(bits - 1);
        #[allow(
            clippy::arithmetic_side_effects,
            reason = "bits should be at least 2"
        )]
        let max = two.pow(bits) - 1u8;
        let samples: Vec<_> =
            repeat_with(|| rng.random_biguint_range(&min, &max))
                .filter(|m| m < &n_minus_one)
                .take(k)
                .collect();

        samples
            .iter()
            .find(|&a| miller_rabin(a, n, s, &d))
            .is_none()
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn simple_test() {
            let mut rng = rand::rng();

            assert!(is_probable_prime(&5u8, 10, &mut rng));
            assert!(!is_probable_prime(&6u8, 10, &mut rng));
            assert!(!is_probable_prime(&949_284_328_996u64, 10, &mut rng));
            assert!(!is_probable_prime(&252_097_800_623u64, 10, &mut rng));
        }
    }
}

#[doc(inline)]
#[cfg(all(feature = "bigint", feature = "rand"))]
pub use rabin_miller::is_probable_prime;
