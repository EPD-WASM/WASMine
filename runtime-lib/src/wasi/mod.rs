use crate::objects::functions::Function;
use once_cell::sync::Lazy;
use runtime_interface::ExecutionContext;
use std::{
    collections::HashSet,
    os::fd::{AsRawFd, IntoRawFd},
    path::PathBuf,
};

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
        ("wasi_snapshot_preview1", "path_filestat_get"),
        ("wasi_snapshot_preview1", "fd_filestat_get"),
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
            "path_filestat_get" => Function::from_wasi_func(WasiContext::path_filestat_get),
            "fd_filestat_get" => Function::from_wasi_func(WasiContext::fd_filestat_get),
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

impl Default for FileDescriptor {
    fn default() -> Self {
        Self {
            stat: types::FdStat {
                fs_filetype: types::FileType::Unknown,
                fs_flags: types::FdFlags::empty(),
                fs_rights_base: types::Rights::empty(),
                fs_rights_inheriting: types::Rights::empty(),
            },
            fd: -1,
            should_close: false,
            file_path: None,
        }
    }
}

pub struct WasiContext {
    execution_context: *mut ExecutionContext,
    open_fds: Vec<FileDescriptor>,
    args: Vec<String>,
    env: Vec<(String, String)>,
}

impl WasiContext {
    pub(crate) fn set_execution_context(&mut self, execution_context: *mut ExecutionContext) {
        self.execution_context = execution_context;
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

impl Default for WasiContext {
    fn default() -> Self {
        Self {
            execution_context: std::ptr::null_mut(),
            open_fds: vec![FileDescriptor::default(); 3],
            args: Vec::new(),
            env: Vec::new(),
        }
    }
}

#[derive(Default)]
pub struct WasiContextBuilder {
    ctxt: WasiContext,
}

impl WasiContextBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn arg(&mut self, arg: String) -> &mut Self {
        self.ctxt.args.push(arg);
        self
    }

    pub fn args(&mut self, args: Vec<String>) -> &mut Self {
        self.ctxt.args.extend(args);
        self
    }

    pub fn env(&mut self, key: String, val: String) -> &mut Self {
        self.ctxt.env.push((key, val));
        self
    }

    pub fn envs(&mut self, envs: Vec<(String, String)>) -> &mut Self {
        self.ctxt.env.extend(envs);
        self
    }

    pub fn inherit_stdio(&mut self) -> &mut Self {
        self.inherit_stdin();
        self.inherit_stdout();
        self.inherit_stderr();
        self
    }

    pub fn inherit_stdout(&mut self) -> &mut Self {
        self.set_stdout(std::io::stdout().as_raw_fd(), false);
        self
    }

    pub fn inherit_stdin(&mut self) -> &mut Self {
        self.set_stdin(std::io::stdin().as_raw_fd(), false);
        self
    }

    pub fn inherit_stderr(&mut self) -> &mut Self {
        self.set_stderr(std::io::stderr().as_raw_fd(), false);
        self
    }

    pub fn set_stdout(&mut self, fd: i32, should_close: bool) -> &mut Self {
        self.ctxt.open_fds[libc::STDOUT_FILENO as usize] = FileDescriptor {
            stat: types::FdStat {
                fs_filetype: types::FileType::CharacterDevice,
                fs_flags: types::FdFlags::empty(),
                fs_rights_base: types::Rights::FdWrite,
                fs_rights_inheriting: types::Rights::FdWrite,
            },
            fd,
            should_close,
            file_path: None,
        };
        self
    }

    pub fn set_stdin(&mut self, fd: i32, should_close: bool) -> &mut Self {
        self.ctxt.open_fds[libc::STDIN_FILENO as usize] = FileDescriptor {
            stat: types::FdStat {
                fs_filetype: types::FileType::CharacterDevice,
                fs_flags: types::FdFlags::empty(),
                fs_rights_base: types::Rights::FdRead,
                fs_rights_inheriting: types::Rights::FdRead,
            },
            fd,
            should_close,
            file_path: None,
        };
        self
    }

    pub fn set_stderr(&mut self, fd: i32, should_close: bool) -> &mut Self {
        self.ctxt.open_fds[libc::STDERR_FILENO as usize] = FileDescriptor {
            stat: types::FdStat {
                fs_filetype: types::FileType::CharacterDevice,
                fs_flags: types::FdFlags::empty(),
                fs_rights_base: types::Rights::FdWrite,
                fs_rights_inheriting: types::Rights::FdWrite,
            },
            fd,
            should_close,
            file_path: None,
        };
        self
    }

    pub fn inherit_host_env(&mut self) -> &mut Self {
        self.ctxt.env = std::env::vars().collect();
        self
    }

    pub fn inherit_args(&mut self) -> &mut Self {
        self.ctxt.args = std::env::args().collect();
        self
    }

    pub fn preopen_dir(
        &mut self,
        host_path: impl AsRef<std::path::Path>,
        guest_path: impl AsRef<str>,
        dir_perms: PreopenDirPerms,
        dir_inherit_perms: PreopenDirInheritPerms,
    ) -> Result<&mut Self, WasiError> {
        self.ctxt.open_fds.push(FileDescriptor {
            stat: types::FdStat {
                fs_filetype: types::FileType::Directory,
                fs_flags: types::FdFlags::empty(),
                fs_rights_base: types::Rights::from_bits_truncate(dir_perms.bits()),
                fs_rights_inheriting: types::Rights::from_bits_truncate(dir_inherit_perms.bits()),
            },
            fd: std::fs::File::open(host_path.as_ref())
                .map_err(|e| WasiError::InvalidPreopenDir(PathBuf::from(host_path.as_ref()), e))?
                .into_raw_fd(),
            should_close: true,
            file_path: if guest_path.as_ref() == "." || guest_path.as_ref() == "./" {
                Some(String::default())
            } else {
                Some(guest_path.as_ref().to_string())
            },
        });
        Ok(self)
    }

    pub fn finish(self) -> WasiContext {
        self.ctxt
    }
}

bitflags::bitflags! {
    pub struct PreopenDirPerms: u64 {
        const READ = types::Rights::FdRead.bits();
        const WRITE = types::Rights::FdWrite.bits();
    }
}

bitflags::bitflags! {
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub struct PreopenDirInheritPerms: u64 {
        const READ = types::Rights::FdRead.bits();
        const WRITE = types::Rights::FdWrite.bits();
    }
}
