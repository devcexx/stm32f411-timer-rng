# stm32f411-timer-rng

This library allows to generate "random" numbers from the jitter generated
between the STM32F411 core clock and its LSI. This is done thanks to the
capability of the TIM5 to use the LSI clock signal as source signal when working
in input capture mode.

This library will use the differences in oscillation between the core clock and
the LSI as entropy source. When the TIM5 is working using the core clock for
upcounting, and the LSI signal is used as input capture mode source, the timer
will latch the counter value that was available every time LSI clock signal
raises. Due to the fact that the LSI and the core clock are independent and
neither are fully accurate, this capture won't happen exactly periodically.
Those drifts will be used to generate random numbers.

The method used by this library is far to be perfect. This method doesn't work
for generating TRNG. Indeed, this library won't go through the [NIST Statistical
Test
Suite](https://csrc.nist.gov/projects/random-bit-generation/documentation-and-software).
That said, it can be perfectly useful for applications where you just need to
provide some appearance of randomness (e.g ID generation, random delay
generation, etc). Indeed, probably a good way for using this library is for
generating a single u64 value that can be used as seed for a PRNG, like the one
provided by the [rand](https://crates.io/crates/rand) crate. With it, you could
just do something like:

```rust
    let seed = stm32f411_timer_rng::gen_random_u64(&mut dp.RCC, &mut dp.TIM5);
    let rng = StdRng::seed_from_u64(seed);
    // ...
    let my_rand_num = rng.random_range(0..42);
```
