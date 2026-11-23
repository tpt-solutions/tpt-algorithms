// SPDX-License-Identifier: MIT OR Apache-2.0
//! Contract helpers: `requires!` and `ensures!`.
//!
//! Both expand to `debug_assert!`, so they are checked in debug and test builds and
//! compiled out of release builds. The failure message contains the condition text.
//! Use them for physical and logical limits (for example a state-of-charge in `[0, 1]`).
//! They are not a substitute for returning `Result` on bad external input.

/// Asserts a precondition. Checked only when debug assertions are enabled.
#[macro_export]
macro_rules! requires {
    ($cond:expr $(,)?) => {
        debug_assert!($cond, "precondition failed: {}", stringify!($cond))
    };
}

/// Asserts a postcondition. Checked only when debug assertions are enabled.
#[macro_export]
macro_rules! ensures {
    ($cond:expr $(,)?) => {
        debug_assert!($cond, "postcondition failed: {}", stringify!($cond))
    };
}

#[cfg(test)]
mod tests {
    fn clamp_unit(x: f64) -> f64 {
        let y = x.clamp(0.0, 1.0);
        crate::ensures!((0.0..=1.0).contains(&y));
        y
    }

    #[test]
    fn passing_contracts_are_silent() {
        crate::requires!(1 + 1 == 2);
        assert_eq!(clamp_unit(1.5), 1.0);
    }

    #[test]
    #[should_panic(expected = "precondition failed")]
    #[cfg(debug_assertions)]
    fn failing_precondition_panics_in_debug() {
        crate::requires!(1 > 2);
    }
}
