# Toby's Lib

A simple library that I will fill with functions, structs and macros
that I feel like using over and over again, especially across different crates.

Feel free to use this library, if you find anything that you might need.

## no-std support

This library strives to support no-std environments as much as possible.
To enable no-std, disable default features.
Some feature flags MAY require `alloc` crate, if they do they will enable `alloc` flag by itself.

- This will soon be opt-in behavior, and any feature that requires `alloc` will not enable it when that feature is enabled.

## All features

Here's the list of all current features;
- `alloc` will enable `alloc` crate, useful when in a `no_std` environment.
- `bigint` will enable functionality that requires the [`num-bigint`](https://crates.io/crates/num-bigint) crate.
- `cron` will enable the cron scheduler.
- `full` enables all feature flags, except `alloc`.
- `macros` will export some useful macros.
- `rand` will enable functionality that requires the [`rand`](https://crates.io/crates/rand) crate.
- `std` (enabled by default) will enable the standard library, and use these bindings over the `alloc` crate (even if both are enabled).

## A few notes

- The MSRV will increase if need be. The only guarantee is that it will stay at least a few versions behind the newest rust version.
- Stability is not really guaranteed for now, but may be in the future. :)
- PRs are welcome!