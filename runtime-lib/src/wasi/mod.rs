#![allow(non_upper_case_globals)] // required because we use double underscores

/// Reference: https://github.com/WebAssembly/WASI/blob/89646e96b8f61fc57ae4c7d510d2dce68620e6a4/legacy/preview1/docs.md
use crate::objects::functions::Function;
use once_cell::sync::Lazy;
use std::{collections::HashSet, path::PathBuf};
use wasm_types::{FuncTypeBuilder, ValType};

pub use context::{PreopenDirInheritPerms, PreopenDirPerms, WasiContext, WasiContextBuilder};

mod context;
mod functions;
mod types;
mod utils;

#[derive(Debug, thiserror::Error)]
pub enum WasiError {
    #[error("Could not open supplied directory '{0}': {1}")]
    InvalidPreopenDir(PathBuf, std::io::Error),
}

#[allow(clippy::declare_interior_mutable_const)]
pub(crate) const WASI_FUNCS: Lazy<HashSet<(&str, &str)>> = Lazy::new(|| {
    HashSet::from([
        ("wasi_snapshot_preview1", "args_get"),
        ("wasi_snapshot_preview1", "args_sizes_get"),
        ("wasi_snapshot_preview1", "environ_get"),
        ("wasi_snapshot_preview1", "environ_sizes_get"),
        ("wasi_snapshot_preview1", "fd_write"),
        ("wasi_snapshot_preview1", "proc_exit"),
        ("wasi_snapshot_preview1", "fd_close"),
        ("wasi_snapshot_preview1", "fd_fdstat_get"),
        ("wasi_snapshot_preview1", "fd_seek"),
        ("wasi_snapshot_preview1", "fd_fdstat_set_flags"),
        ("wasi_snapshot_preview1", "fd_prestat_get"),
        ("wasi_snapshot_preview1", "fd_prestat_dir_name"),
        ("wasi_snapshot_preview1", "fd_read"),
        ("wasi_snapshot_preview1", "path_open"),
        ("wasi_snapshot_preview1", "path_filestat_get"),
        ("wasi_snapshot_preview1", "fd_filestat_get"),
    ])
});

impl WasiContext {
    #[allow(clippy::borrow_interior_mutable_const)]
    pub(crate) fn get_func_by_name(module: &str, name: &str) -> &'static Function {
        debug_assert!(WASI_FUNCS.contains(&(module, name)));
        match name {
            "args_get" => &WASI_SNAPSHOT_PREVIEW1__ARGS_GET,
            "args_sizes_get" => &WASI_SNAPSHOT_PREVIEW1__ARGS_SIZES_GET,
            "environ_get" => &WASI_SNAPSHOT_PREVIEW1__ENVIRON_GET,
            "environ_sizes_get" => &WASI_SNAPSHOT_PREVIEW1__ENVIRON_SIZES_GET,
            "fd_write" => &WASI_SNAPSHOT_PREVIEW1__FD_WRITE,
            "proc_exit" => &WASI_SNAPSHOT_PREVIEW1__PROC_EXIT,
            "fd_close" => &WASI_SNAPSHOT_PREVIEW1__FD_CLOSE,
            "fd_fdstat_get" => &WASI_SNAPSHOT_PREVIEW1__FD_FDSTAT_GET,
            "fd_seek" => &WASI_SNAPSHOT_PREVIEW1__FD_SEEK,
            "fd_fdstat_set_flags" => &WASI_SNAPSHOT_PREVIEW1__FD_FDSTAT_SET_FLAGS,
            "fd_prestat_get" => &WASI_SNAPSHOT_PREVIEW1__FD_PRESTAT_GET,
            "fd_prestat_dir_name" => &WASI_SNAPSHOT_PREVIEW1__FD_PRESTAT_DIR_NAME,
            "fd_read" => &WASI_SNAPSHOT_PREVIEW1__FD_READ,
            "path_open" => &WASI_SNAPSHOT_PREVIEW1__PATH_OPEN,
            "path_filestat_get" => &WASI_SNAPSHOT_PREVIEW1__PATH_FILESTAT_GET,
            "fd_filestat_get" => &WASI_SNAPSHOT_PREVIEW1__FD_FILESTAT_GET,
            _ => unreachable!(),
        }
    }
}

static WASI_SNAPSHOT_PREVIEW1__ARGS_GET: Lazy<Function> = Lazy::new(|| {
    Function::from_wasi_func(
        WasiContext::args_get,
        FuncTypeBuilder::create(
            // argv, argv_buf
            &[ValType::i32(), ValType::i32()],
            // errno
            &[ValType::i32()],
        ),
    )
});

static WASI_SNAPSHOT_PREVIEW1__ARGS_SIZES_GET: Lazy<Function> = Lazy::new(|| {
    Function::from_wasi_func(
        WasiContext::args_sizes_get,
        FuncTypeBuilder::create(
            // num_args_out, size_args_out
            &[ValType::i32(), ValType::i32()],
            // errno
            &[ValType::i32()],
        ),
    )
});

static WASI_SNAPSHOT_PREVIEW1__ENVIRON_GET: Lazy<Function> = Lazy::new(|| {
    Function::from_wasi_func(
        WasiContext::environ_get,
        FuncTypeBuilder::create(
            // environ, environ_buf
            &[ValType::i32(), ValType::i32()],
            // errno
            &[ValType::i32()],
        ),
    )
});

static WASI_SNAPSHOT_PREVIEW1__ENVIRON_SIZES_GET: Lazy<Function> = Lazy::new(|| {
    Function::from_wasi_func(
        WasiContext::environ_sizes_get,
        FuncTypeBuilder::create(
            // num_env_vars_out, size_env_vars_out
            &[ValType::i32(), ValType::i32()],
            // errno
            &[ValType::i32()],
        ),
    )
});

static WASI_SNAPSHOT_PREVIEW1__PROC_EXIT: Lazy<Function> = Lazy::new(|| {
    Function::from_wasi_func(
        WasiContext::proc_exit,
        FuncTypeBuilder::create(
            // rval
            &[ValType::i32()],
            &[],
        ),
    )
});

static WASI_SNAPSHOT_PREVIEW1__FD_WRITE: Lazy<Function> = Lazy::new(|| {
    Function::from_wasi_func(
        WasiContext::fd_write,
        FuncTypeBuilder::create(
            // fd, iovs, num_iovs, written_bytes_out
            &[
                ValType::i32(),
                ValType::i32(),
                ValType::i32(),
                ValType::i32(),
            ],
            // errno
            &[ValType::i32()],
        ),
    )
});

static WASI_SNAPSHOT_PREVIEW1__FD_CLOSE: Lazy<Function> = Lazy::new(|| {
    Function::from_wasi_func(
        WasiContext::fd_close,
        FuncTypeBuilder::create(
            // fd
            &[ValType::i32()],
            // errno
            &[ValType::i32()],
        ),
    )
});

static WASI_SNAPSHOT_PREVIEW1__FD_FDSTAT_GET: Lazy<Function> = Lazy::new(|| {
    Function::from_wasi_func(
        WasiContext::fd_fdstat_get,
        FuncTypeBuilder::create(
            // fd, buf
            &[ValType::i32(), ValType::i32()],
            // errno
            &[ValType::i32()],
        ),
    )
});

static WASI_SNAPSHOT_PREVIEW1__FD_SEEK: Lazy<Function> = Lazy::new(|| {
    Function::from_wasi_func(
        WasiContext::fd_seek,
        FuncTypeBuilder::create(
            // fd, offset, whence, newoffset_out
            &[
                ValType::i32(),
                ValType::i64(),
                ValType::i32(),
                ValType::i32(),
            ],
            // errno
            &[ValType::i32()],
        ),
    )
});

static WASI_SNAPSHOT_PREVIEW1__FD_FDSTAT_SET_FLAGS: Lazy<Function> = Lazy::new(|| {
    Function::from_wasi_func(
        WasiContext::fd_fdstat_set_flags,
        FuncTypeBuilder::create(
            // fd, flags
            &[ValType::i32(), ValType::i32()],
            // errno
            &[ValType::i32()],
        ),
    )
});

static WASI_SNAPSHOT_PREVIEW1__FD_PRESTAT_GET: Lazy<Function> = Lazy::new(|| {
    Function::from_wasi_func(
        WasiContext::fd_prestat_get,
        FuncTypeBuilder::create(
            // fd, buf
            &[ValType::i32(), ValType::i32()],
            // errno
            &[ValType::i32()],
        ),
    )
});

static WASI_SNAPSHOT_PREVIEW1__FD_PRESTAT_DIR_NAME: Lazy<Function> = Lazy::new(|| {
    Function::from_wasi_func(
        WasiContext::fd_prestat_dir_name,
        FuncTypeBuilder::create(
            // fd, path, path_len
            &[ValType::i32(), ValType::i32(), ValType::i32()],
            // errno
            &[ValType::i32()],
        ),
    )
});

static WASI_SNAPSHOT_PREVIEW1__FD_READ: Lazy<Function> = Lazy::new(|| {
    Function::from_wasi_func(
        WasiContext::fd_read,
        FuncTypeBuilder::create(
            // fd, iovs, num_iovs, nread_out
            &[
                ValType::i32(),
                ValType::i32(),
                ValType::i32(),
                ValType::i32(),
            ],
            // errno
            &[ValType::i32()],
        ),
    )
});

static WASI_SNAPSHOT_PREVIEW1__PATH_OPEN: Lazy<Function> = Lazy::new(|| {
    Function::from_wasi_func(
        WasiContext::path_open,
        FuncTypeBuilder::create(
            // dirfd, dirflags, path, path_len, oflags, fs_rights_base, fs_rights_inheriting, fs_flags, fd_out
            &[
                ValType::i32(),
                ValType::i32(),
                ValType::i32(),
                ValType::i32(),
                ValType::i32(),
                ValType::i64(),
                ValType::i64(),
                ValType::i32(),
                ValType::i32(),
            ],
            // errno
            &[ValType::i32()],
        ),
    )
});

static WASI_SNAPSHOT_PREVIEW1__PATH_FILESTAT_GET: Lazy<Function> = Lazy::new(|| {
    Function::from_wasi_func(
        WasiContext::path_filestat_get,
        FuncTypeBuilder::create(
            // fd, flags, path, path_len, buf
            &[
                ValType::i32(),
                ValType::i32(),
                ValType::i32(),
                ValType::i32(),
                ValType::i32(),
            ],
            // errno
            &[ValType::i32()],
        ),
    )
});

static WASI_SNAPSHOT_PREVIEW1__FD_FILESTAT_GET: Lazy<Function> = Lazy::new(|| {
    Function::from_wasi_func(
        WasiContext::fd_filestat_get,
        FuncTypeBuilder::create(
            // fd, buf
            &[ValType::i32(), ValType::i32()],
            // errno
            &[ValType::i32()],
        ),
    )
});
