//! Source-derived process-local one-use FFI gate, plus a disclosed callback adapter.
//!
//! The original extern C write surface consumes a handle but returns
//! CompatibilityHold (-8) for nonempty data. It never writes knowledge.
//! `with_borrowed_payload` is NEW POC009 glue: it uses the source gate and borrowed
//! slice, then lets the caller independently recompute and persist model evidence.
//! No native verdict, receipt, capability lineage, or knowledge authority is copied
//! into that caller. The callback's own checks determine its effects.
//!
//! Handles are process-local. Their generation resets when the process restarts;
//! never persist, restore, or transfer them across process lifetimes. Nonces are
//! deterministic labels, not secrets or authentication. The source registry has no
//! revocation operation and retains consumed slots; it is not a hostile in-process
//! sandbox. Pointer validity remains the unsafe caller's responsibility.

use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
const fn fnv(s: &[u8]) -> u64 {
    let mut h: u64 = 14695981039346656037;
    let mut i = 0;
    while i < s.len() {
        h ^= s[i] as u64;
        h = h.wrapping_mul(1099511628211);
        i += 1;
    }
    h
}
const fn seal(m: &[u8]) -> u64 {
    fnv(m) ^ fnv(b"GJATA_LEGACY")
}

// Compile-time nonce-ët — të prera, jo të negociueshme
pub const S_SHADOW_WRITE: u64 = seal(b"SHADOW_WRITE_KERNEL");
pub const S_SHADOW_VERIFY: u64 = seal(b"SHADOW_VERIFICATION_OUTPUT");
pub const S_LAB_TRL1: u64 = seal(b"LAB_TRL1_FFI");
pub const S_LAB_TRL2: u64 = seal(b"LAB_TRL2_FFI");
pub struct LgcToken(PhantomData<*const ()>);
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CapHandle {
    pub gen: u64,   // ID gjenerimi — identifikon CapSlot në Rust heap
    pub nonce: u64, // FNV seal e modulit — verifikim matematik
}
impl CapHandle {
    pub const INVALID: Self = Self { gen: 0, nonce: 0 };
}
struct CapSlot {
    valid: AtomicBool, // burned me CAS — JO me store()
    gen: u64,
    expected_nonce: u64,
}

#[derive(Debug, PartialEq)]
pub enum FfiErr {
    HandleNotFound,
    NonceMismatch,
    AlreadyConsumed, // ← KURTHI I SULMIT C: CAS i rënë në burned slot
}
struct SovereignGate {
    slots: HashMap<u64, Box<CapSlot>>,
    counter: u64,
}

impl SovereignGate {
    fn new() -> Self {
        Self {
            slots: HashMap::new(),
            counter: 1,
        }
    }

    /// Krijon CapSlot në heap Rust, kthen vetëm CapHandle (dy numra) tek C.
    fn issue(&mut self, module_nonce: u64) -> CapHandle {
        let gen = self.counter;
        self.counter += 1;
        self.slots.insert(
            gen,
            Box::new(CapSlot {
                valid: AtomicBool::new(true),
                gen,
                expected_nonce: module_nonce,
            }),
        );
        CapHandle {
            gen,
            nonce: module_nonce,
        }
    }

    // ═══ MURI MATEMATIK ═══════════════════════════════════════════════════
    //   compare_exchange(true→false, AcqRel, Acquire)
    //     valid==true  → atomikisht false → Ok  → LgcToken (1× konsum)
    //     valid==false → s'ndryshon       → Err → AlreadyConsumed (burned)
    fn validate_and_burn(&self, h: CapHandle) -> Result<LgcToken, FfiErr> {
        let slot = self.slots.get(&h.gen).ok_or(FfiErr::HandleNotFound)?;
        // Gjenerata në handle duhet të jetë po ajo që mban slot-i real.
        match slot.gen != h.gen {
            true => return Err(FfiErr::HandleNotFound),
            false => {}
        }
        // LAW_5: NK-check (nonce matematik) PARA operacionit.
        match slot.expected_nonce != h.nonce {
            true => return Err(FfiErr::NonceMismatch),
            false => {}
        }
        slot.valid
            .compare_exchange(true, false, Ordering::AcqRel, Ordering::Acquire)
            .map(|_| LgcToken(PhantomData))
            .map_err(|_| FfiErr::AlreadyConsumed)
    }
}
static GATE: OnceLock<Mutex<SovereignGate>> = OnceLock::new();
fn gate() -> &'static Mutex<SovereignGate> {
    GATE.get_or_init(|| Mutex::new(SovereignGate::new()))
}
/// C thërret: merr CapHandle(gen, nonce).
#[no_mangle]
pub extern "C" fn sovereign_issue_capability(module_seal: u64) -> CapHandle {
    // PJESA C: lock-u SOVRAN (autoriteti i token-ave) — nëse helmohet, njofto PARA
    // rikuperimit (gjendja duhet audituar). Rikuperimi mbetet (fail-safe), por jo i heshtur.
    gate().lock().unwrap_or_else(|p| {
        eprintln!("[SOVEREIGN] ANOMALY: gate lock poisoned te issue() — recovering, gjendja sovrane duhet audituar");
        p.into_inner()
    }).issue(module_seal)
}
/// # Safety
/// Kur `data_len > 0`, `data_ptr` duhet të tregojë te të paktën `data_len` byte
/// të lexueshëm dhe të mbetet valid gjatë gjithë thirrjes. Pointeri mund të
/// lexohet edhe në rrugën `AlreadyConsumed` për gjurmën e sulmit të klonimit.
#[no_mangle]
pub unsafe extern "C" fn sovereign_validate_and_write(
    handle: CapHandle,
    data_ptr: *const u8,
    data_len: usize,
) -> i32 {
    match gate().lock().unwrap_or_else(|p| {
        eprintln!("[SOVEREIGN] ANOMALY: gate lock poisoned te validate_and_burn() — recovering, gjendja sovrane duhet audituar");
        p.into_inner()
    }).validate_and_burn(handle) {
        Ok(token) => {
            // SAFETY: përcillet kontrata e pointerit nga kufiri publik unsafe.
            unsafe { sovereign_commit(token, handle, data_ptr, data_len) }
        }
        Err(FfiErr::AlreadyConsumed) => {
            // ANOMALI: handle i klonuar/ri-përdorur nga C — sulm i dokumentuar.
            eprintln!("[SOVEREIGN] ANOMALY: gen={} AlreadyConsumed — clone attack?", handle.gen);
            -1
        }
        Err(FfiErr::HandleNotFound) => -2,
        Err(FfiErr::NonceMismatch)  => -3,
    }
}
/// LAW_3: ABI legacy mbetet i ekspozueshëm vetëm me feature të shprehur,
/// por nuk mund të materializojë dije pa një verdict të komituar. Payload-i
/// lexohet si borrowed view mbi të njëjtin pointer dhe nuk kopjohet.
unsafe fn sovereign_commit(
    _token: LgcToken,
    handle: CapHandle,
    data_ptr: *const u8,
    data_len: usize,
) -> i32 {
    let data = match unsafe { borrow_ffi_payload(data_ptr, data_len) } {
        Ok(data) => data,
        Err(code) => return code,
    };
    let input_id = fnv(data);
    let proof = [handle.gen, handle.nonce];
    let lineage = capability_lineage::Lineage::derive_capability(input_id, &proof);
    match lineage.is_traceable() {
        false => -4,
        true => {
            eprintln!("[SOVEREIGN] CompatibilityHold: direct FFI write kërkon committed verdict");
            -8
        }
    }
}

/// # Safety
/// Caller-i garanton pointer të lexueshëm për `data_len` dhe lifetime deri në
/// kthimin e thirrjes. View-i ruan adresën bazë; Rust nuk e liron memorien.
unsafe fn borrow_ffi_payload<'a>(data_ptr: *const u8, data_len: usize) -> Result<&'a [u8], i32> {
    match data_ptr.is_null() || data_len == 0 {
        true => Err(-5),
        false => {
            // SAFETY: kontrata e caller-it garanton pointer, gjatësi, alignment
            // byte-wise, immutable aliasing dhe lifetime për gjithë thirrjen.
            Ok(unsafe { std::slice::from_raw_parts(data_ptr, data_len) })
        }
    }
}

/// Source capability-lineage primitives used only by the held compatibility path.
pub mod capability_lineage {
    const fn fnv1a(seed: u64, bytes: &[u8]) -> u64 {
        let mut h = seed;
        let mut i = 0;
        while i < bytes.len() {
            h ^= bytes[i] as u64;
            h = h.wrapping_mul(1099511628211);
            i += 1;
        }
        h
    }
    const FNV_OFFSET: u64 = 14695981039346656037;

    #[inline]
    fn mix_u64(h: u64, v: u64) -> u64 {
        fnv1a(h, &v.to_le_bytes())
    }

    /// Gjatësia minimale e zinxhirit për gjurmë të vlefshme (Router+Matrix+Judiciary).
    pub const MIN_CHAIN: u8 = 3;
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Lineage {
        /// Identifikuesi i inputit (stable_id).
        pub input_id: u64,
        /// Bit i ndezur për çdo nod që dha verdikt == 1 (index = `ShadowNode as u16`).
        pub chain_mask: u16,
        /// Sa verifikues prodhuan verdikt (gjatësia n e zinxhirit).
        pub chain_count: u8,
        /// Gjatësia e proof_chain (zinxhiri i provave algoritmike nga upstream).
        pub proof_len: u32,
        /// Nënshkrimi i derivimit — FNV mbi (input_id ∥ verdiktet ∥ proof_chain).
        pub derivation: u64,
        /// E vulosur nga një kapacitet sovran single-use (rruga FFI). Shih
        /// `sovereign_ffi_gate`: vula atomike = autorizim, vlen si gjurmë.
        pub cap_sealed: bool,
    }
    impl Lineage {
        /// Ndërton gjurmën për një shkrim të autorizuar nga kapaciteti sovran (FFI).
        /// proof_chain këtu = lineage i kapacitetit (p.sh. [gen, nonce]).
        pub fn derive_capability(input_id: u64, proof_chain: &[u64]) -> Self {
            let mut h = mix_u64(FNV_OFFSET, input_id);
            h = mix_u64(h, 0xCA9A_B111_7E57_5EA1); // domain-tag "capability"
            for p in proof_chain {
                h = mix_u64(h, *p);
            }
            Self {
                input_id,
                chain_mask: 0,
                chain_count: 0,
                proof_len: proof_chain.len() as u32,
                derivation: h,
                cap_sealed: true,
            }
        }
        /// LIGJI I GJURMUESHMËRISË — a e trajton sistemi këtë si DIJE?
        /// Pa gjurmë algoritmike (proof_chain bosh, nënshkrim zero, ose zinxhir i
        /// pamjaftueshëm pa vulë kapaciteti) → NUK është dije → do të fshihet.
        pub fn is_traceable(&self) -> bool {
            self.derivation != 0
                && self.proof_len > 0
                && (self.cap_sealed || self.chain_count >= MIN_CHAIN)
        }
    }
}

/// NEW POC009 adapter over the exact source gate and source borrowed-payload view.
///
/// The source handle is consumed before payload validation and before the callback.
/// Copying its numeric fields does not copy the registry's one-use state. The
/// callback receives bytes only; it must perform its own model/scope/write checks.
/// A callback error or panic does not restore the consumed handle.
///
/// # Safety
/// For a nonnull, nonempty payload, `data_ptr` must identify one live allocation
/// readable for `data_len` bytes, with `data_len <= isize::MAX`. It must remain
/// valid and unmodified for the whole callback. The caller retains ownership.
/// Null or empty payloads are rejected without dereferencing the pointer.
pub unsafe fn with_borrowed_payload<T>(
    handle: CapHandle,
    data_ptr: *const u8,
    data_len: usize,
    use_bytes: impl FnOnce(&[u8]) -> T,
) -> Result<T, i32> {
    let _token = gate()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .validate_and_burn(handle)
        .map_err(|error| match error {
            FfiErr::AlreadyConsumed => -1,
            FfiErr::HandleNotFound => -2,
            FfiErr::NonceMismatch => -3,
        })?;
    // SAFETY: the caller provides the allocation and lifetime required above.
    let bytes = unsafe { borrow_ffi_payload(data_ptr, data_len) }?;
    Ok(use_bytes(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty_payload_is_rejected_after_capability_validation() {
        let h = sovereign_issue_capability(S_SHADOW_WRITE);
        assert_ne!(h, CapHandle::INVALID);
        // SAFETY: len=0 lejon pointer null sipas kontratës FFI.
        let r = unsafe { sovereign_validate_and_write(h, std::ptr::null(), 0) };
        assert_eq!(r, -5); // token konsumohet, por payload bosh nuk maskohet si sukses
    }

    #[test]
    fn clone_attack_second_use_is_refused() {
        let h_orig = sovereign_issue_capability(S_SHADOW_WRITE);
        let h_copy = h_orig; // memcpy perfekt — çdo bit identik
        assert_eq!(h_orig, h_copy);

        // SAFETY: len=0 lejon pointer null; përdorimi i parë djeg token-in edhe pse kthen -5.
        let r1 = unsafe { sovereign_validate_and_write(h_orig, std::ptr::null(), 0) };
        assert_eq!(r1, -5); // AtomicBool: true → false; payload bosh refuzohet

        // SAFETY: len=0 lejon pointer null; handle-i i kopjuar duhet të refuzohet.
        let r2 = unsafe { sovereign_validate_and_write(h_copy, std::ptr::null(), 0) };
        assert_eq!(r2, -1); // CAS(false→false) = refuzim hardware
    }

    #[test]
    fn forged_nonce_is_rejected() {
        let h_real = sovereign_issue_capability(S_SHADOW_WRITE);
        let h_fake = CapHandle {
            gen: h_real.gen,
            nonce: h_real.nonce ^ 0xDEAD_BEEF,
        };
        // SAFETY: len=0 lejon pointer null.
        assert_eq!(
            unsafe { sovereign_validate_and_write(h_fake, std::ptr::null(), 0) },
            -3
        );
    }

    #[test]
    fn unknown_handle_is_not_found() {
        let h = CapHandle {
            gen: 9_999_999,
            nonce: S_SHADOW_WRITE,
        };
        // SAFETY: len=0 lejon pointer null.
        assert_eq!(
            unsafe { sovereign_validate_and_write(h, std::ptr::null(), 0) },
            -2
        );
    }
    #[test]
    fn replay_third_time_still_burned() {
        let h = sovereign_issue_capability(S_SHADOW_WRITE);
        // SAFETY: len=0 lejon pointer null. Përdorimi i parë djeg token-in dhe kthen -5.
        assert_eq!(
            unsafe { sovereign_validate_and_write(h, std::ptr::null(), 0) },
            -5
        );
        // SAFETY: i njëjti kusht pointeri; replay duhet të mbetet i djegur.
        assert_eq!(
            unsafe { sovereign_validate_and_write(h, std::ptr::null(), 0) },
            -1
        );
        // SAFETY: i njëjti kusht pointeri; përdorimi i tretë nuk ringjall token-in.
        assert_eq!(
            unsafe { sovereign_validate_and_write(h, std::ptr::null(), 0) },
            -1
        );
    }

    #[test]
    fn nonempty_direct_write_is_compatibility_hold() {
        let data = b"borrowed-ffi-payload";
        let handle = sovereign_issue_capability(S_SHADOW_WRITE);
        // SAFETY: `data` është i lexueshëm për gjithë thirrjen.
        let result = unsafe { sovereign_validate_and_write(handle, data.as_ptr(), data.len()) };
        assert_eq!(result, -8);
    }

    #[test]
    fn ffi_view_preserves_pointer_identity_without_allocation() {
        let data = b"zero-copy-ffi-view";
        // SAFETY: `data` është i lexueshëm për gjithë view-n.
        let view =
            unsafe { borrow_ffi_payload(data.as_ptr(), data.len()) }.expect("borrowed FFI view");
        assert_eq!(view.as_ptr(), data.as_ptr());
        assert_eq!(view.len(), data.len());
    }

    // NEW adapter tests; the tests above are extracted from the private runtime.
    #[test]
    fn callback_preserves_every_byte_and_pointer_then_denies_reuse() {
        let data = (0..=u8::MAX).collect::<Vec<_>>();
        let handle = sovereign_issue_capability(S_SHADOW_WRITE);
        // SAFETY: data remains live and immutable throughout both calls.
        let result = unsafe {
            with_borrowed_payload(handle, data.as_ptr(), data.len(), |view| {
                assert_eq!(view.as_ptr(), data.as_ptr());
                assert_eq!(view, data.as_slice());
                view.len()
            })
        };
        assert_eq!(result, Ok(256));
        assert_eq!(
            unsafe {
                with_borrowed_payload::<()>(handle, data.as_ptr(), data.len(), |_| {
                    panic!("a consumed handle must not reach the callback")
                })
            },
            Err(-1)
        );
    }

    #[test]
    fn empty_and_null_calls_consume_the_grant_without_callback() {
        for (pointer, length) in [
            (std::ptr::null(), 0),
            (std::ptr::null(), 7),
            (b"x".as_ptr(), 0),
        ] {
            let handle = sovereign_issue_capability(S_SHADOW_WRITE);
            // SAFETY: these invalid/empty shapes are rejected before any borrow.
            assert_eq!(
                unsafe {
                    with_borrowed_payload::<()>(handle, pointer, length, |_| {
                        panic!("empty input must not reach the callback")
                    })
                },
                Err(-5)
            );
            assert_eq!(
                unsafe { sovereign_validate_and_write(handle, std::ptr::null(), 0) },
                -1
            );
        }
    }

    #[test]
    fn wrong_nonce_does_not_consume_the_real_handle() {
        let data = b"source-bound";
        let handle = sovereign_issue_capability(S_SHADOW_WRITE);
        let forged = CapHandle {
            gen: handle.gen,
            nonce: handle.nonce ^ 1,
        };
        // SAFETY: data remains live and immutable throughout both calls.
        assert_eq!(
            unsafe { with_borrowed_payload(forged, data.as_ptr(), data.len(), |_| ()) },
            Err(-3)
        );
        assert_eq!(
            unsafe { with_borrowed_payload(handle, data.as_ptr(), data.len(), |_| ()) },
            Ok(())
        );
    }

    #[test]
    fn callback_error_and_panic_do_not_restore_consumed_grants() {
        let data = b"source-bound";
        let first = sovereign_issue_capability(S_SHADOW_WRITE);
        // SAFETY: data remains live and immutable throughout each callback.
        let result = unsafe {
            with_borrowed_payload(first, data.as_ptr(), data.len(), |_| {
                Err::<(), _>("model rejected")
            })
        };
        assert_eq!(result, Ok(Err("model rejected")));
        assert_eq!(
            unsafe { sovereign_validate_and_write(first, data.as_ptr(), data.len()) },
            -1
        );
        let second = sovereign_issue_capability(S_SHADOW_WRITE);
        let panicked = std::panic::catch_unwind(|| unsafe {
            with_borrowed_payload::<()>(second, data.as_ptr(), data.len(), |_| {
                panic!("callback failed")
            })
        });
        assert!(panicked.is_err());
        assert_eq!(
            unsafe { sovereign_validate_and_write(second, data.as_ptr(), data.len()) },
            -1
        );
    }

    #[test]
    fn copied_handles_racing_in_one_process_reach_exactly_one_callback() {
        let handle = sovereign_issue_capability(S_SHADOW_WRITE);
        let results = std::thread::scope(|scope| {
            let threads = (0..16)
                .map(|_| {
                    scope.spawn(move || {
                        let bytes = b"same-process";
                        // SAFETY: static immutable bytes outlive the call.
                        unsafe {
                            with_borrowed_payload(handle, bytes.as_ptr(), bytes.len(), |_| ())
                        }
                    })
                })
                .collect::<Vec<_>>();
            threads
                .into_iter()
                .map(|thread| thread.join().unwrap())
                .collect::<Vec<_>>()
        });
        assert_eq!(
            results.iter().filter(|result| **result == Ok(())).count(),
            1
        );
        assert_eq!(
            results.iter().filter(|result| **result == Err(-1)).count(),
            15
        );
    }
}
