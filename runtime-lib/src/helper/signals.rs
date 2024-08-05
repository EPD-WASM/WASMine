use crate::{objects::execution_context::ExecutionContextWrapper, RuntimeError};
use nix::sys::signal::{self, SaFlags, SigAction, SigHandler, SigSet, Signal};
use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
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
}

pub(crate) struct SignalHandler;

impl SignalHandler {
    extern "C" fn handle_signal(
        sig: libc::c_int,
        info: *mut libc::siginfo_t,
        ucontext: *mut libc::c_void,
    ) {
        if THREAD_CURRENTLY_EXECUTES_WASM.with(|b| b.load(std::sync::atomic::Ordering::Relaxed)) {
            ExecutionContextWrapper::trap(RuntimeError::Trap("signal handler".to_string()))
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
                SaFlags::SA_NODEFER | SaFlags::SA_SIGINFO,
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
