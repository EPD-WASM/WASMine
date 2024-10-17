use crate::{objects::execution_context::ExecutionContextWrapper, RuntimeError};
use nix::{
    errno::Errno,
    sys::signal::{self, SaFlags, SigAction, SigHandler, SigSet, Signal},
};
use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
    ffi::CStr,
    mem::MaybeUninit,
    ptr,
    sync::{
        atomic::{AtomicBool, AtomicI32},
        Mutex,
    },
};

// signal-handling is made on a per-process basis, so we can use a global static
static SIGNAL_HANDLER_FALLBACK_FUNCS: Lazy<Mutex<HashMap<libc::c_int, SigAction>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

static SIGNAL_HANDLER_REGISTER_COUNT: Lazy<AtomicI32> = Lazy::new(|| AtomicI32::new(0));

thread_local! {
    static THREAD_CURRENTLY_EXECUTES_WASM: AtomicBool = const {AtomicBool::new(false)};
    static SIGNAL_ALT_STACK: SigAltStack = SigAltStack::new();
}

pub(crate) struct SignalHandler;

fn is_stack_overflow(sig_code: i32, addr: *mut libc::c_void) -> bool {
    if sig_code != libc::SIGSEGV && sig_code != libc::SIGBUS {
        return false;
    }
    let thread_id = unsafe { libc::pthread_self() };
    let mut thread_attrs = MaybeUninit::uninit();
    assert!(0 == unsafe { libc::pthread_getattr_np(thread_id, thread_attrs.as_mut_ptr()) });
    let attr = unsafe { thread_attrs.assume_init() };

    let mut stack_addr = MaybeUninit::uninit();
    let mut stack_size = MaybeUninit::uninit();
    assert!(
        0 == unsafe {
            libc::pthread_attr_getstack(&attr, stack_addr.as_mut_ptr(), stack_size.as_mut_ptr())
        }
    );
    let stack_addr = unsafe { stack_addr.assume_init() };
    let stack_size = unsafe { stack_size.assume_init() };

    // check that fault address is within +-8 bytes of the stack boundary
    addr < unsafe { stack_addr.add(stack_size + 8) }
        || addr < unsafe { stack_addr.add(stack_size - 8) }
}

impl SignalHandler {
    extern "C" fn handle_signal(
        sig: libc::c_int,
        info: *mut libc::siginfo_t,
        ucontext: *mut libc::c_void,
    ) {
        // hic sunt dracones
        if THREAD_CURRENTLY_EXECUTES_WASM.with(|b| b.load(std::sync::atomic::Ordering::Relaxed)) {
            if is_stack_overflow(sig, unsafe { (*info).si_addr() }) {
                ExecutionContextWrapper::trap(RuntimeError::Exhaustion);
            } else {
                ExecutionContextWrapper::trap(RuntimeError::Trap(format!(
                    "execution triggered {:?} signal",
                    unsafe { CStr::from_ptr(libc::strsignal(sig)) }
                )))
            }
        }

        let signal = Signal::try_from(sig).unwrap();
        if let Some(sig_action) = SIGNAL_HANDLER_FALLBACK_FUNCS.lock().unwrap().get(&sig) {
            unsafe {
                signal::sigaction(Signal::try_from(sig).unwrap(), sig_action).unwrap();
            }
        }
        signal::raise(signal).unwrap();
    }

    fn register_signal(signal: Signal, sig_action: &SigAction) {
        SIGNAL_HANDLER_FALLBACK_FUNCS
            .lock()
            .unwrap()
            .insert(signal as i32, unsafe {
                signal::sigaction(signal, sig_action).unwrap()
            });
    }

    /// Register signal handlers for wasm trap detection
    ///
    /// This function together with deregister_globally behaves like a reentrant lock.
    /// Only the first and last matching pairs of calls to these functions actually register / deregister the signal handlers.
    /// All intermittend calls are ignored.
    pub(crate) fn register_globally() {
        if SIGNAL_HANDLER_REGISTER_COUNT
            .compare_exchange(
                0,
                1,
                std::sync::atomic::Ordering::Relaxed,
                std::sync::atomic::Ordering::Relaxed,
            )
            .is_ok()
        {
            let handle_signal = SigHandler::SigAction(Self::handle_signal);

            let sig_action = SigAction::new(
                handle_signal,
                // SA_SIGINFO is set by nix
                // SA_ONSTACK: the sigaltstack is allocated as a thread_local, so we are guaranteed that it is always available
                SaFlags::SA_NODEFER | SaFlags::SA_ONSTACK,
                SigSet::empty(),
            );

            Self::register_signal(signal::SIGSEGV, &sig_action);
            Self::register_signal(signal::SIGBUS, &sig_action);
            Self::register_signal(signal::SIGFPE, &sig_action);
            Self::register_signal(signal::SIGABRT, &sig_action);
            Self::register_signal(signal::SIGILL, &sig_action);
        } else {
            SIGNAL_HANDLER_REGISTER_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
    }

    /// Deregister signal handlers for wasm trap detection
    ///
    /// This function together with register_globally behaves like a reentrant lock.
    /// Only the first and last matching pairs of calls to these functions actually register / deregister the signal handlers.
    /// All intermittend calls are ignored.
    pub(crate) fn deregister_globally() {
        if SIGNAL_HANDLER_REGISTER_COUNT
            .compare_exchange(
                1,
                0,
                std::sync::atomic::Ordering::Relaxed,
                std::sync::atomic::Ordering::Relaxed,
            )
            .is_ok()
        {
            for (signal, sig_action) in SIGNAL_HANDLER_FALLBACK_FUNCS.lock().unwrap().drain() {
                unsafe {
                    signal::sigaction(Signal::try_from(signal).unwrap(), &sig_action).unwrap();
                }
            }
        } else if SIGNAL_HANDLER_REGISTER_COUNT
            .compare_exchange(
                0,
                0,
                std::sync::atomic::Ordering::Relaxed,
                std::sync::atomic::Ordering::Relaxed,
            )
            .is_err()
        {
            SIGNAL_HANDLER_REGISTER_COUNT.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        }
    }

    /// Enable catching of signals in the current thread.
    ///
    /// If not enabled, the signals are redirect to previously installed signal handlers.
    pub(crate) fn set_thread_executing_wasm() {
        THREAD_CURRENTLY_EXECUTES_WASM
            .with(|b| b.store(true, std::sync::atomic::Ordering::Relaxed));
    }

    /// Disable catching of signals in the current thread.
    pub(crate) fn unset_thread_executing_wasm() {
        THREAD_CURRENTLY_EXECUTES_WASM
            .with(|b| b.store(false, std::sync::atomic::Ordering::Relaxed));
    }
}

struct SigAltStack {
    addr: *mut libc::c_void,
    size: usize,
}

impl SigAltStack {
    fn new() -> Self {
        const ALT_STACK_SIZE: usize = libc::SIGSTKSZ;
        let alt_stack = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                ALT_STACK_SIZE,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANON,
                -1,
                0,
            )
        };
        assert!(
            alt_stack != libc::MAP_FAILED,
            "failed to allocate signal stack: {}",
            Errno::last()
        );
        let instance = Self {
            addr: alt_stack,
            size: ALT_STACK_SIZE,
        };
        instance.register_for_current_thread();
        instance
    }

    fn register_for_current_thread(&self) {
        let sig_stack = libc::stack_t {
            ss_sp: self.addr,
            ss_flags: 0,
            ss_size: self.size,
        };
        let res = unsafe { libc::sigaltstack(&sig_stack, ptr::null_mut()) };
        assert_eq!(res, 0, "failed to register signal stack: {}", Errno::last());
    }
}

impl Drop for SigAltStack {
    fn drop(&mut self) {
        let res = unsafe { libc::munmap(self.addr, self.size) };
        assert_eq!(
            res,
            0,
            "failed to deallocate signal stack: {}",
            Errno::last()
        );
    }
}
