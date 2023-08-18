# randperm-crt

Small library for generating random permutations of the set {0, ..., n-1} where n is a product of small prime powers, with much less than O(n) memory usage.

Thinking of a permutation as a function `σ` from {0, ..., n-1} to itself, this library also allows for computation of `σ(i)` and `σ^(-1)(i)` in constant time (independent of `i`).

# How it works

First `n` is factored into prime powers, and random permutations of {0, ..., q-1} are generated for each prime power `q` in the factorization of `n`. Then the Chinese Remainder Theorem is used to combine each combination of elements from these "sub-permutations" into a permutation of {0, ..., n-1}.

# When not to use this

Don't use this if you need any of the following:

- Any level of randomness beyond "it looks kind of random to the user". The permutations generated are very much *not* "patternless", for example there can (and will) be long streaks of numbers that are all equal modulo a prime power factor of `n`. You can use the `Composition` struct to compose multiple permutations which can reduce the chance of this happening.
- Random permutations on n points where n is not the product of small prime powers.

# Example

```rust
// Create a permutation on 11! points.
let factorial_11 = (1..=11).product();
let perm = RandomPermutation::new(factorial_11).unwrap();

// Calculate the image of 0, 1, 2, ..., 99 under the permutation.
let image = perm.iter().take(100).collect::<Vec<_>>();
println!("{image:?}");

// Find `i` such that the image of `i` is 0.
let i = perm.inverse().nth(0).unwrap();
assert_eq!(perm.nth(i), Some(0));
```
