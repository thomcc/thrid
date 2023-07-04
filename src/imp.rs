#![allow(unused_macros)]
use core::num::NonZeroUsize;

// This is pretty fragile, but I can't come up with another way of doing it.
macro_rules! asm_maybe_with_pure {
    ($tmpl:expr, out(reg) $out:ident, options($($opts:tt),* $(,)?) $(,)?) => {
        #[cfg(feature = "unsound_pure_asm")]
        core::arch::asm!(
            $tmpl,
            out(reg) $out,
            options($($opts),*),
        );
        #[cfg(not(feature = "unsound_pure_asm"))]
        core::arch::asm!(
            $tmpl,
            out(reg) $out,
            options($($opts),*),
        );
    };
}

cfg_if::cfg_if! {
    if #[cfg(any(
        // If the user wants to opt in to yolo mode, then so be it.
        thrid_unsafely_assume_target_is_single_threaded,
        // Avoid pulling in `std` on single-threaded wasm targets.
        all(target_family = "wasm", not(target_feature = "atomics")),
    ))] {
        #[inline(always)]
        pub(super) fn tid_impl() -> NonZeroUsize {
            static BYTE: u8 = 0;
            let addr = core::ptr::addr_of!(BYTE) as usize;
            unsafe { NonZeroUsize::new_unchecked(addr) }
        }
    } else if #[cfg(all(
        not(any(
            miri,
            // Anything weird or from the future
            not(any(target_pointer_width = "32", target_pointer_width = "64")),
            // 32 bit ABI on 64 bit arch (x32, arm64_32, ...)
            all(
                not(target_pointer_width = "64"),
                any(target_arch = "x86_64", target_arch = "aarch64"),
            ),
            // 64 bit ABI on 32 bit arch (maybe someday CHERI...?)
            all(
                not(target_pointer_width = "32"),
                any(target_arch = "x86", target_arch = "arm"),
            ),
        )),
        any(
            // macOS x86, x86_64, aarch64.
            all(target_os = "macos", any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")),
            // iOS/tvOS aarch64. Note that watchOS is pretty cursed so it's excluded.
            all(any(target_os = "ios", target_os = "tvos"), target_arch = "aarch64"),
            // Windows x86, x86_64, aarch64.
            all(windows, any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")),
            // linux x86, x86_64, aarch64, but only glibc or musl (other libc's
            // are *probably* fine, but these two consider this ABI).
            all(
                target_os = "linux",
                any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"),
                any(target_env = "musl", target_env = "gnu"),
            ),
            // Probably some BSDs are reasonable...
        ),
    ))] {
        #[inline(always)]
        pub(crate) fn tid_impl() -> NonZeroUsize {
            unsafe {
                #[allow(unused_assignments)]
                let mut output = 0usize;

                cfg_if::cfg_if! {
                    if #[cfg(all(target_os = "macos", target_arch = "x86_64"))] {
                        // x86_64 macos uses gs, and starts with a pointer to TCB.
                        asm_maybe_with_pure!(
                            "mov {}, gs:0",
                            out(reg) output,
                            options(nostack, readonly, preserves_flags),
                        );
                    } else if #[cfg(all(target_os = "macos", target_arch = "x86"))] {
                        // As above, but with fs
                        asm_maybe_with_pure!(
                            "mov {}, fs:0",
                            out(reg) output,
                            options(nostack, readonly, preserves_flags),
                        );
                    } else if #[cfg(all(any(target_os = "macos", target_os = "ios"), target_arch = "aarch64"))] {
                        // `TPIDRRO_EL0` is the TLS base pointer on these targets.
                        asm_maybe_with_pure!(
                            "mrs {}, tpidrro_el0",
                            out(reg) output,
                            options(nostack, nomem, preserves_flags),
                        );
                    } else if #[cfg(all(windows, target_arch = "x86_64"))] {
                        // manual impl of NtCurrentTeb (64 bit ptrs).
                        asm_maybe_with_pure!(
                            "mov {}, gs:48",
                            out(reg) output,
                            options(nostack, readonly, preserves_flags),
                        );
                    } else if #[cfg(all(windows, target_arch = "x86"))] {
                        // manual impl of NtCurrentTeb (32 bit ptrs).
                        asm_maybe_with_pure!(
                            "mov {}, fs:24",
                            out(reg) output,
                            options(nostack, readonly, preserves_flags),
                        );
                    } else if #[cfg(all(windows, target_arch = "aarch64"))] {
                        // According to MSDN this is the TEB already, but it
                        // notes that it's only correct for user-mode.
                        asm_maybe_with_pure!(
                            // aka xpr
                            "mov {}, x18",
                            out(reg) output,
                            options(nostack, nomem, preserves_flags),
                        );
                    } else if #[cfg(all(windows, target_arch = "arm"))] {
                        // _MoveFromCoprocessor(CP15_TPIDRURW)
                        asm_maybe_with_pure!(
                            "mrc p15, #0, {}, c13, c0, #2",
                            out(reg) output,
                            options(nostack, nomem, preserves_flags),
                        );
                    } else if #[cfg(all(target_os = "linux", target_arch = "x86_64"))] {
                        // Pointer to TCB.
                        asm_maybe_with_pure!(
                            "mov {}, fs:0",
                            out(reg) output,
                            options(nostack, readonly, preserves_flags),
                        );
                    } else if #[cfg(all(target_os = "linux", target_arch = "x86"))] {
                        // Pointer to TCB.
                        asm_maybe_with_pure!(
                            "mov {}, gs:0",
                            out(reg) output,
                            options(nostack, readonly, preserves_flag),
                        );
                    } else if #[cfg(all(target_os = "linux", target_arch = "aarch64"))] {
                        // This is some thread-specific pointer. Not sure if TCB or TLS base.
                        asm_maybe_with_pure!(
                            "mrs {}, tpidr_el0",
                            out(reg) output,
                            options(nostack, nomem, preserves_flags),
                        );
                    } else {
                        compile_error!("bug: mismatch between `cfg_if`s");
                    }
                }
                debug_assert!(output != 0);
                core::num::NonZeroUsize::new_unchecked(output)
            }
        }
    } else if #[cfg(unix)] {
        #[inline(always)]
        pub(super) fn tid_impl() -> NonZeroUsize {
            #[cfg(feature = "libc")] use libc::pthread_self;
            #[cfg(not(feature = "libc"))] extern crate std;
            #[cfg(not(feature = "libc"))] extern "C" { fn pthread_self() -> usize; }
            let thread_id = unsafe { pthread_self() as usize };
            let thread_id = if thread_id == 0 {
                // It's legal for `pthread_self` to be 0, and it sometimes will
                // be.
                !0usize
            } else {
                // Technically it's probably legal for pthread_self to be
                // `!0usize` too, but it doesn't seem like can happen in actual
                // implementations (many use it internally as a sentinel). If
                // this assert actually can get hit in practice, we should just
                // use the `thread_local!` version instead.
                assert!(thread_id != !0usize);
                thread_id
            };
            // Safety: Already checked and excluded 0.
            unsafe { NonZeroUsize::new_unchecked(thread_id) }
        }
    } else if #[cfg(windows)] {
        #[inline]
        pub(super) fn tid_impl() -> NonZeroUsize {
            #[link(name = "kernel32")]
            extern "system" {
                fn GetCurrentThreadId() -> u32;
            }
            let thread_id = unsafe { GetCurrentThreadId() as usize };
            // https://learn.microsoft.com/en-us/windows/win32/procthread/thread-handles-and-identifiers,
            // says "no thread identifier will ever be 0", so we don't have to
            // do the dance we do on unix.
            debug_assert_ne!(thread_id, 0);
            unsafe { NonZeroUsize::new_unchecked(thread_id) }
        }
    } else {
        #[inline]
        pub(super) fn tid_impl() -> NonZeroUsize {
            extern crate std;
            std::thread_local!(static BYTE: u8 = const { 0 });
            let addr = BYTE.with(|b: &u8| b as *const u8 as usize);
            debug_assert_ne!(addr, 0);
            unsafe { NonZeroUsize::new_unchecked(addr) }
        }
    }
}
