use super::{types, WasiError};
use runtime_interface::ExecutionContext;
use std::{
    os::fd::{AsRawFd, IntoRawFd},
    path::PathBuf,
};

#[derive(Clone)]
pub(super) struct FileDescriptor {
    pub(super) stat: types::FdStat,
    pub(super) fd: types::FD,
    pub(super) should_close: bool,

    /// only given for file descriptors of files in filesystem
    pub(super) file_path: Option<String>,
}

impl FileDescriptor {
    pub(super) fn has_right_or_err(&self, rights: types::Rights) -> Result<(), types::Errno> {
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
    pub(super) execution_context: *mut ExecutionContext,
    pub(super) open_fds: Vec<FileDescriptor>,
    pub(super) args: Vec<String>,
    pub(super) env: Vec<(String, String)>,
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
