#![doc = include_str!("README.md")]
#![no_std]
#![expect(clippy::inline_always, reason = "This is used rationally.")]

#[cfg(target_arch = "x86")]
use core::arch::x86;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64 as x86;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use core::mem::transmute;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use core::sync::atomic::{AtomicPtr, Ordering};

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use self::x86::{__cpuid, CpuidResult};

/// Returns `true` if the CPU supports fast PDEP/PEXT instructions.
///
/// In this context, "fast" means that the instructions are available in the
/// CPU's instruction set and is not implemented in microcode. This is worth
/// checking in performance-critical code, as the microcode implementation in
/// Zen, Zen+, and Zen2 CPUs is significantly slower than fallback options.
///
/// For the sake of simplicity, if the CPU is neither x86 nor x86-64, this
/// function will return `false`.
///
/// # Examples
///
/// Basic usage:
///
/// ```no_run
/// use has_fast_pdep::has_fast_pdep;
///
/// pub fn exposed_fn(value: u64) -> u64 {
///     if has_fast_pdep() {
///         // SAFETY: This is safe because we know that the CPU supports BMI2 and that
///         // the PDEP/PEXT instructions are not implemented in microcode.
///         unsafe { uses_pdep(value) }
///     } else {
///         fallback(value)
///     }
/// }
///
/// #[target_feature(enable = "bmi2")]
/// fn uses_pdep(value: u64) -> u64 {
///     // some algorithm that uses PDEP/PEXT
///     # todo!()
/// }
///
/// fn fallback(value: u64) -> u64 {
///     // some fallback algorithm
///     # todo!()
/// }
/// ```
#[must_use]
#[inline(always)]
pub fn has_fast_pdep() -> bool {
    inner()
}

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
#[must_use]
#[inline(always)]
fn inner() -> bool {
    false
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
static SELECTED_FN: AtomicPtr<()> = AtomicPtr::new(inner_bootstrap as *mut ());

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[must_use]
#[inline(always)]
fn inner() -> bool {
    let fn_ptr = SELECTED_FN.load(Ordering::Acquire);
    let selected_fn = unsafe { transmute::<*mut (), fn() -> bool>(fn_ptr) };
    selected_fn()
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[must_use]
const fn inner_true() -> bool {
    true
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[must_use]
const fn inner_false() -> bool {
    false
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[must_use]
fn inner_bootstrap() -> bool {
    const AMD: [u8; 12] = *b"AuthenticAMD";
    const HYGON: [u8; 12] = *b"HygonGenuine";

    // SAFETY: CPUID is given to exist in practice due to the constraints of
    // `i686-unknown-linux-gnu` and similar targets.
    let first_call = unsafe { __cpuid(0x00) };
    let max_parameter = first_call.eax;

    let manufacturer_id = unsafe {
        let CpuidResult { ebx, ecx, edx, .. } = first_call;
        transmute::<[u32; 3], [u8; 12]>([ebx, edx, ecx])
    };

    let slow_hint = if manufacturer_id == AMD || manufacturer_id == HYGON {
        const ZEN: u32 = 0x17;
        const HYGON_DHYANA: u32 = 0x18;
        // SAFETY: In practice, every CPU that supports CPUID will have a max
        // parameter of at least 0x01. So, we can safely assume that CPUID can
        // be called with 0x01.
        let leaf1 = unsafe { __cpuid(0x01) };
        let family = (leaf1.eax >> 8) & 0x0F;
        let extended_family = (leaf1.eax >> 20) & 0xFF;
        // SAFETY: Even in the freak case that every bit is set, at most this
        // will be equal to 0xFF + 0x0F; that sum won't overflow a u32.
        let sum = unsafe { family.unchecked_add(extended_family) };
        matches!(sum, ZEN | HYGON_DHYANA)
    } else {
        false
    };

    let supports_bmi2 = if max_parameter >= 0x07 {
        let leaf = unsafe { __cpuid(0x07) };
        (leaf.ebx >> 8) & 0x01 == 1
    } else {
        false
    };

    let fast_pdep = supports_bmi2 && !slow_hint;

    let selected_fn = if fast_pdep { inner_true } else { inner_false };
    SELECTED_FN.store(selected_fn as *mut (), Ordering::Release);

    fast_pdep
}
