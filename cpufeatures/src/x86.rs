//! x86/x86-64 CPU feature detection support.
//!
//! Portable, `no_std`-friendly implementation that relies on the x86 `CPUID`
//! instruction for feature detection.

// Evaluate the given `$body` expression any of the supplied target features
// are not enabled. Otherwise returns true.
//
// The `$body` expression is not evaluated on SGX targets, and returns false
// on these targets unless *all* supplied target features are enabled.
#[macro_export]
#[doc(hidden)]
macro_rules! __unless_target_features {
    ($($tf:tt),+ => $body:expr ) => {{
        false
    }};
}

// Use CPUID to detect the presence of all supplied target features.
#[macro_export]
#[doc(hidden)]
macro_rules! __detect_target_features {
    ($($tf:tt),+) => {{
        #[cfg(target_arch = "x86")]
        use core::arch::x86::{__cpuid, __cpuid_count, CpuidResult};
        #[cfg(target_arch = "x86_64")]
        use core::arch::x86_64::{__cpuid, __cpuid_count, CpuidResult};

        // These wrappers are workarounds around
        // https://github.com/rust-lang/rust/issues/101346
        //
        // DO NOT remove it until MSRV is bumped to a version
        // with the issue fix (at least 1.64).
        #[inline(never)]
        unsafe fn cpuid(leaf: u32) -> CpuidResult {
            __cpuid(leaf)
        }

        #[inline(never)]
        unsafe fn cpuid_count(leaf: u32, sub_leaf: u32) -> CpuidResult {
            __cpuid_count(leaf, sub_leaf)
        }

        let cr = unsafe {
            [cpuid(1), cpuid_count(7, 0)]
        };

        $($crate::check!(cr, $tf) & )+ true
    }};
}

macro_rules! __expand_check_macro {
    ($(($name:tt $(, $i:expr, $reg:ident, $offset:expr)*)),* $(,)?) => {
        #[macro_export]
        #[doc(hidden)]
        macro_rules! check {
            $(
                ($cr:expr, $name) => {
                    true
                    $(
                        & ($cr[$i].$reg & (1 << $offset) != 0)
                    )*
                };
            )*
        }
    };
}

// Note that according to the [Intel manual][0] AVX2 and FMA require
// that we check availability of AVX before using them.
//
// [0]: https://www.intel.com/content/dam/develop/external/us/en/documents/36945
__expand_check_macro! {
    ("mmx", 0, edx, 23),
    ("sse", 0, edx, 25),
    ("sse2", 0, edx, 26),
    ("sse3", 0, ecx, 0),
    ("pclmulqdq", 0, ecx, 1),
    ("ssse3", 0, ecx, 9),
    ("fma", 0, ecx, 28, 0, ecx, 12),
    ("sse4.1", 0, ecx, 19),
    ("sse4.2", 0, ecx, 20),
    ("popcnt", 0, ecx, 23),
    ("aes", 0, ecx, 25),
    ("avx", 0, ecx, 28),
    ("rdrand", 0, ecx, 30),
    ("sgx", 1, ebx, 2),
    ("bmi1", 1, ebx, 3),
    ("avx2", 0, ecx, 28, 1, ebx, 5),
    ("bmi2", 1, ebx, 8),
    ("rdseed", 1, ebx, 18),
    ("adx", 1, ebx, 19),
    ("sha", 1, ebx, 29),
}
