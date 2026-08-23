# Toby's Lib

A simple library that I will fill with functions, structs and macros
that I feel like using over and over again, especially across different crates.

Feel free to use this library, if you find anything that you might need.

## no-std support

This library strives to support no-std environments as much as possible.
To enable no-std, disable default features.
Some feature flags MAY require `alloc` crate, if they do they will enable `alloc` flag by itself.

## A few notes

- The MSRV will increase if need be. The only guarantee is that it will stay at least a few versions behind the newest rust version.
- Stability is not really guaranteed for now, but may be in the future. :)
- PRs are welcome!