use once_cell::sync::Lazy;
use runtime_interface::ExecutionContext;
use std::{
    collections::HashSet,
    os::fd::{AsRawFd, IntoRawFd},
    path::PathBuf,
};

use crate::{objects::functions::Function, Cluster};

/// Reference: https://github.com/WebAssembly/WASI/blob/89646e96b8f61fc57ae4c7d510d2dce68620e6a4/legacy/preview1/docs.md
mod functions;
mod types;
mod utils;

// static I32: ValType = ValType::Number(NumType::I32);

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
    ])
});

impl WasiContext {
    #[allow(clippy::borrow_interior_mutable_const)]
    pub(crate) fn get_func_by_name(module: &str, name: &str) -> Function {
        debug_assert!(WASI_FUNCS.contains(&(module, name)));
        match name {
            "args_get" => Function::from_wasi_func(WasiContext::args_get),
            "args_sizes_get" => Function::from_wasi_func(WasiContext::args_sizes_get),
            "environ_get" => Function::from_wasi_func(WasiContext::environ_get),
            "environ_sizes_get" => Function::from_wasi_func(WasiContext::environ_sizes_get),
            "fd_write" => Function::from_wasi_func(WasiContext::fd_write),
            "proc_exit" => Function::from_wasi_func(WasiContext::proc_exit),
            "fd_close" => Function::from_wasi_func(WasiContext::fd_close),
            "fd_fdstat_get" => Function::from_wasi_func(WasiContext::fd_fdstat_get),
            "fd_seek" => Function::from_wasi_func(WasiContext::fd_seek),
            "fd_fdstat_set_flags" => Function::from_wasi_func(WasiContext::fd_fdstat_set_flags),
            "fd_prestat_get" => Function::from_wasi_func(WasiContext::fd_prestat_get),
            "fd_prestat_dir_name" => Function::from_wasi_func(WasiContext::fd_prestat_dir_name),
            "fd_read" => Function::from_wasi_func(WasiContext::fd_read),
            "path_open" => Function::from_wasi_func(WasiContext::path_open),
            _ => unreachable!(),
        }
    }
}

#[derive(Clone)]
pub(crate) struct FileDescriptor {
    stat: types::FdStat,
    fd: types::FD,
    should_close: bool,

    /// only given for file descriptors of files in filesystem
    file_path: Option<String>,
}

impl FileDescriptor {
    pub(crate) fn has_right_or_err(&self, rights: types::Rights) -> Result<(), types::Errno> {
        if self.stat.fs_rights_base & rights == rights {
            Ok(())
        } else {
            Err(types::Errno::NotCapable)
        }
    }
}

pub struct WasiContext {
    execution_context: *mut ExecutionContext,
    open_fds: Vec<FileDescriptor>,
    args: Vec<String>,
}

impl WasiContext {
    pub(crate) fn register_new(
        cluster: &Cluster,
        execution_context: *mut ExecutionContext,
    ) -> Result<Option<&mut Self>, WasiError> {
        if !cluster.config.enable_wasi {
            return Ok(None);
        }
        let mut ctxt = Self {
            execution_context,
            open_fds: Vec::new(),
            args: cluster.config.wasi_args.clone(),
        };
        ctxt.open_fds.push(FileDescriptor {
            stat: types::FdStat {
                fs_filetype: types::FileType::CharacterDevice,
                fs_flags: types::FdFlags::empty(),
                fs_rights_base: types::Rights::FdRead,
                fs_rights_inheriting: types::Rights::FdRead,
            },
            fd: std::io::stdin().as_raw_fd(),
            should_close: false,
            file_path: None,
        });
        ctxt.open_fds.push(FileDescriptor {
            stat: types::FdStat {
                fs_filetype: types::FileType::CharacterDevice,
                fs_flags: types::FdFlags::empty(),
                fs_rights_base: types::Rights::FdWrite,
                fs_rights_inheriting: types::Rights::FdWrite,
            },
            fd: std::io::stdout().as_raw_fd(),
            should_close: false,
            file_path: None,
        });
        ctxt.open_fds.push(FileDescriptor {
            stat: types::FdStat {
                fs_filetype: types::FileType::CharacterDevice,
                fs_flags: types::FdFlags::empty(),
                fs_rights_base: types::Rights::FdWrite,
                fs_rights_inheriting: types::Rights::FdWrite,
            },
            fd: std::io::stderr().as_raw_fd(),
            should_close: false,
            file_path: None,
        });
        for path in &cluster.config.wasi_dirs {
            let dir = std::fs::File::open(path)
                .map_err(|e| WasiError::InvalidPreopenDir(PathBuf::from(path), e))?;
            ctxt.open_fds.push(FileDescriptor {
                stat: types::FdStat {
                    fs_filetype: types::FileType::Directory,
                    fs_flags: types::FdFlags::empty(),
                    fs_rights_base: types::Rights::FdRead,
                    fs_rights_inheriting: types::Rights::FdRead,
                },
                fd: dir.into_raw_fd(),
                should_close: true,
                file_path: Some(path.to_string_lossy().to_string()),
            });
        }
        Ok(Some(cluster.alloc_wasi_context(ctxt)))
    }
}

impl Drop for WasiContext {
    fn drop(&mut self) {
        for fd in self.open_fds.iter() {
            if fd.should_close {
                log::debug!("Closing file descriptor: {:?}", fd.fd);
                unsafe {
                    libc::close(fd.fd);
                }
            }
        }
    }
}
