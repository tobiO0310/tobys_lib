//! Prime number related functions.

#[cfg(all(feature = "bigint", feature = "rand"))]
mod rabin_miller {
    use num_bigint::{BigRng010, BigUint, ToBigUint};
    use num_traits::{One, Zero};
    use rand::Rng;
    #[cfg(feature = "rayon")]
    use rayon::iter::{ParallelBridge, ParallelIterator};

    macro_rules! biguint {
        ($e:expr) => {
            ($e).to_biguint().unwrap()
        };
    }

    fn miller_rabin_decompose(n: &BigUint) -> (u64, BigUint) {
        assert!(!n.is_zero() && !n.is_one());

        #[expect(clippy::arithmetic_side_effects, reason = "n is at least 2")]
        let n = n - 1u8; // n is at least 2 as per assertion -> n is now at least 1
        #[expect(clippy::unwrap_used, reason = "n is not zero")]
        let s = n.trailing_zeros().unwrap();

        #[expect(
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

        #[expect(clippy::arithmetic_side_effects, reason = "n is at least 1")]
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
    /// [`rayon`] parallelization can be enabled with the `rayon` feature flag.
    ///
    /// # None
    ///
    /// Returns [`None`] if `n` cannot become a [`BigUint`].
    pub fn is_probable_prime<T: ToBigUint, R: Rng + Send + Sync>(
        n: &T,
        k: usize,
        rng: &mut R,
    ) -> Option<bool> {
        use crate::alias::{repeat_with, vec};

        let n = &n.to_biguint()?;
        let two = &biguint!(2);

        if n <= &BigUint::one() {
            return Some(false);
        } else if n <= &biguint!(3) {
            return Some(true);
        } else if n <= &biguint!(0xFFFF_FFFF_FFFF_FFFFu64) {
            #[expect(
                clippy::arithmetic_side_effects,
                reason = "n is at least 3"
            )]
            let n_minus_one: BigUint = n - 1u8;
            let (s, d) = miller_rabin_decompose(n);

            // if n less than u64, simply use 16 small known primes
            let samples = vec![
                2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53,
            ];

            // rayon is not used here, as it is unnecessary (amount of numbers are limited to 16)

            return Some(
                samples
                    .into_iter()
                    .filter(|&m| biguint!(m) < n_minus_one)
                    .find(|&a| miller_rabin(&biguint!(a), n, s, &d))
                    .is_some(),
            );
        }

        #[expect(clippy::arithmetic_side_effects, reason = "n is at least 3")]
        let n_minus_one: BigUint = n - 1u8;
        let (s, d) = miller_rabin_decompose(n);

        #[expect(
            clippy::cast_possible_truncation,
            clippy::as_conversions,
            reason = "bits will never overflow??"
        )]
        let bits = n_minus_one.bits() as u32;
        #[expect(
            clippy::arithmetic_side_effects,
            reason = "bits should be at least 2"
        )]
        let min = two.pow(bits - 1);
        #[expect(
            clippy::arithmetic_side_effects,
            reason = "bits should be at least 2"
        )]
        let max = two.pow(bits) - 1u8;

        #[cfg(not(feature = "rayon"))]
        return Some(
            repeat_with(|| rng.random_biguint_range(&min, &max))
                .filter(|m| m < &n_minus_one)
                .take(k)
                .find(|a| miller_rabin(a, n, s, &d))
                .is_some(),
        );

        #[cfg(feature = "rayon")]
        Some(
            repeat_with(|| rng.random_biguint_range(&min, &max))
                .take(k)
                .par_bridge()
                .find_any(|a| miller_rabin(a, n, s, &d))
                .is_some(),
        )
    }

    #[cfg(test)]
    mod tests {
        use rand::rngs::StdRng;

        use super::*;

        #[test]
        fn simple_test() {
            let mut rng: StdRng = rand::make_rng();

            assert_eq!(is_probable_prime(&5u8, 10, &mut rng), Some(true));
            assert_eq!(is_probable_prime(&6u8, 10, &mut rng), Some(false));
            assert_eq!(is_probable_prime(&8u8, 10, &mut rng), Some(false));
            assert_eq!(is_probable_prime(&9u8, 10, &mut rng), Some(false));
            assert_eq!(
                is_probable_prime(&949_284_328_995u64, 10, &mut rng),
                Some(false)
            );
            assert_eq!(
                is_probable_prime(&949_284_328_996u64, 10, &mut rng),
                Some(false)
            );
            assert_eq!(
                is_probable_prime(&252_097_800_623u64, 10, &mut rng),
                Some(true)
            );
        }
    }
}

#[doc(inline)]
#[cfg(all(feature = "bigint", feature = "rand"))]
pub use rabin_miller::is_probable_prime;
