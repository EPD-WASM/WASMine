use super::{
    context::{FileDescriptor, WasiContext},
    types::*,
    utils::errno,
};
use ir::structs::value::ValueRaw;
use std::{
    ffi::{CString, OsStr},
    mem::MaybeUninit,
    os::unix::ffi::OsStrExt,
    path::Path,
};

impl WasiContext {
    #[inline]
    fn check_fd(&self, fd: FD) -> Result<(), Errno> {
        if fd as usize >= self.open_fds.len() || fd.is_negative() {
            return Err(Errno::Badf);
        }
        Ok(())
    }
    #[inline]
    fn get_fd(&self, fd: FD) -> Result<&FileDescriptor, Errno> {
        self.check_fd(fd)?;
        Ok(&self.open_fds[fd as usize])
    }
    #[inline]
    fn get_fd_mut(&mut self, fd: FD) -> Result<&mut FileDescriptor, Errno> {
        self.check_fd(fd)?;
        Ok(&mut self.open_fds[fd as usize])
    }

    /// Read command-line argument data.
    #[inline]
    fn args_get_internal(&self, argv: Ptr<u32>, argv_buf: Ptr<u8>) -> Result<(), Errno> {
        log::debug!("wasi::args_get() -> [{}]", self.args.join(","));
        let arg_count = self.args.len();
        let args_contiguous_size = self
            .args
            .iter()
            .map(|arg| arg.len() + /* null terminator */ 1)
            .sum::<usize>();

        let args_heads = self.get_memory_slice(argv, arg_count)?;
        let args_storage = self.get_memory_slice(argv_buf, args_contiguous_size)?;

        let mut args_storage_offset = 0;
        for (i, arg) in self.args.iter().enumerate() {
            let arg_bytes = arg.as_bytes();
            let arg_len = arg_bytes.len();

            args_heads[i] = argv_buf.offset(args_storage_offset as u32).get();

            args_storage[args_storage_offset..args_storage_offset + arg_len]
                .copy_from_slice(arg_bytes);
            args_storage_offset += arg_len;
            args_storage[args_storage_offset] = b'\0';
            args_storage_offset += 1;
        }
        Ok(())
    }
    pub(super) unsafe extern "C" fn args_get(
        ctxt: *mut WasiContext,
        params: *const ValueRaw,
        returns: *mut ValueRaw,
    ) {
        let argv = (*params.offset(0)).into();
        let argv_buf = (*params.offset(1)).into();
        *returns = errno((*ctxt).args_get_internal(argv, argv_buf)).into();
    }

    /// Reads command-line
    ///
    /// #### Results
    /// - `Result<(Size, Size), Errno>`: Returns the number of arguments and the size of the argument string data, or an error.
    #[inline]
    fn args_sizes_get_internal(
        &self,
        num_args_out: Ptr<u32>,
        size_args_out: Ptr<u32>,
    ) -> Result<(), Errno> {
        log::debug!("wasi::args_sizes_get()");
        let num_args_out = self.get_memory_slice(num_args_out, 1)?;
        num_args_out[0] = self.args.len() as u32;

        let size_args_out = self.get_memory_slice(size_args_out, 1)?;
        size_args_out[0] = self.args.iter().map(|arg| arg.len() + 1).sum::<usize>() as u32;
        Ok(())
    }
    pub(super) unsafe extern "C" fn args_sizes_get(
        ctxt: *mut WasiContext,
        params: *const ValueRaw,
        returns: *mut ValueRaw,
    ) {
        let num_args_out = (*params.offset(0)).into();
        let size_args_out = (*params.offset(1)).into();
        *returns = errno((*ctxt).args_sizes_get_internal(num_args_out, size_args_out)).into();
    }

    /// Read environment variable data.
    /// The sizes of the buffers should match that returned by `environ_sizes_get`.
    /// Key/value pairs are expected to be joined with `=`s, and terminated with `\0`s.
    ///
    /// #### Params
    /// - `environ`: Pointer to the environment variable data.
    /// - `environ_buf`: Pointer to the environment variable buffer.
    ///
    /// #### Results
    /// - `Result<(), Errno>`
    #[inline]
    fn environ_get_internal(&self, environ: Ptr<u32>, environ_buf: Ptr<u8>) -> Result<(), Errno> {
        log::debug!("wasi::environ_get()");
        let num_env_vars = self.env.len();
        let env_vars_size = self
            .env
            .iter()
            .map(|(k, v)| k.len() + v.len() + /* null terminator */ 1 + /* equals symbol */ 1)
            .sum::<usize>();

        let environ = self.get_memory_slice(environ, num_env_vars)?;
        let environ_buf = self.get_memory_slice(environ_buf, env_vars_size)?;

        let mut environ_buf_offset = 0;
        for (i, (key, value)) in self.env.iter().enumerate() {
            let key_bytes = key.as_bytes();
            let value_bytes = value.as_bytes();

            let key_len = key_bytes.len();
            let value_len = value_bytes.len();

            environ[i] = environ_buf_offset as u32;

            let environ_buf_slot =
                &mut environ_buf[environ_buf_offset..environ_buf_offset + key_len];
            environ_buf_slot.copy_from_slice(key_bytes);

            environ_buf[environ_buf_offset + key_len] = b'=';
            environ_buf
                [environ_buf_offset + key_len + 1..environ_buf_offset + key_len + 1 + value_len]
                .copy_from_slice(value_bytes);

            environ_buf[environ_buf_offset + key_len + 1 + value_len] = b'\0';
            environ_buf_offset += key_len + value_len + 2;
        }
        Ok(())
    }
    pub(super) unsafe extern "C" fn environ_get(
        ctxt: *mut WasiContext,
        params: *const ValueRaw,
        returns: *mut ValueRaw,
    ) {
        let environ = (*params.offset(0)).into();
        let environ_buf = (*params.offset(1)).into();
        *returns = errno((*ctxt).environ_get_internal(environ, environ_buf)).into();
    }

    /// Return environment variable data sizes.
    ///
    /// #### Results
    /// - `Result<(Size, Size), Errno>`: Returns the number of environment variable arguments and the size of the environment variable data.
    #[inline]
    fn environ_sizes_get_internal(
        &self,
        num_env_vars_out: Ptr<u32>,
        size_env_vars_out: Ptr<u32>,
    ) -> Result<(), Errno> {
        log::debug!("wasi::environ_sizes_get()");
        let num_env_vars_out = self.get_memory_slice(num_env_vars_out, 1)?;
        num_env_vars_out[0] = self.env.len() as u32;

        let size_env_vars_out = self.get_memory_slice(size_env_vars_out, 1)?;
        size_env_vars_out[0] = self
            .env
            .iter()
            .map(|(k, v)| k.len() + v.len() + /* null terminator */ 1 + /* equals symbol */ 1)
            .sum::<usize>() as u32;
        Ok(())
    }
    pub(super) unsafe extern "C" fn environ_sizes_get(
        ctxt: *mut WasiContext,
        params: *const ValueRaw,
        returns: *mut ValueRaw,
    ) {
        let num_env_vars_out = (*params.offset(0)).into();
        let size_env_vars_out = (*params.offset(1)).into();
        *returns =
            errno((*ctxt).environ_sizes_get_internal(num_env_vars_out, size_env_vars_out)).into();
    }

    /// Terminate the process normally. An exit code of 0 indicates successful termination of the program. The meanings of other values is dependent on the environment.
    ///
    /// #### Params
    /// - `rval`: `ExitCode`: The exit code returned by the process.
    #[inline]
    fn proc_exit_internal(&self, rval: ExitCode) -> ! {
        log::debug!("wasi::proc_exit(exit_val: {rval})");
        std::process::exit(rval as i32);
    }
    pub(super) unsafe extern "C" fn proc_exit(
        ctxt: *mut WasiContext,
        params: *const ValueRaw,
        _: *mut ValueRaw,
    ) {
        let rval = (*params.offset(0)).into();
        (*ctxt).proc_exit_internal(rval)
    }

    /// Write to a file descriptor.
    /// Note: This is similar to `writev` in POSIX.
    ///
    /// #### Params
    /// - `fd`: `FD`: The file descriptor.
    /// - `iovs`: `CIOVecArray`: List of scatter/gather vectors from which to retrieve data.
    ///
    /// #### Results
    /// - `Result<size, errno>`: Returns the number of bytes written if successful, or an error if one occurred.
    #[inline]
    fn fd_write_internal(
        &self,
        fd: FD,
        iovs: CIOVecArray,
        num_iovs: Size,
        written_bytes_out: Ptr<Size>,
    ) -> Result<(), Errno> {
        log::debug!("wasi::fd_write(fd: {fd}, num_data_vecs: {num_iovs})");
        let opened_fd = self.get_fd(fd)?;
        opened_fd.has_right_or_err(Rights::FdWrite)?;
        if num_iovs > 1024 {
            return Err(Errno::Inval);
        }
        let iovecs = self.get_memory_slice(iovs, num_iovs as usize)?;
        let out = self.get_memory_slice(written_bytes_out, 1)?;

        let mut c_iovecs = Vec::with_capacity(num_iovs as usize);
        for iov in iovecs {
            let buf = self.get_memory_slice(iov.buf, iov.buf_len as usize)?;
            c_iovecs.push(libc::iovec {
                iov_base: buf.as_ptr() as _,
                iov_len: buf.len(),
            })
        }
        let written = unsafe { libc::writev(opened_fd.fd, c_iovecs.as_ptr(), num_iovs as i32) };
        if written == -1 {
            return Err(Errno::Io);
        }
        out[0] = written as u32;
        Ok(())
    }
    pub(super) unsafe extern "C" fn fd_write(
        ctxt: *mut WasiContext,
        params: *const ValueRaw,
        returns: *mut ValueRaw,
    ) {
        let fd = (*params.offset(0)).into();
        let iovs = (*params.offset(1)).into();
        let num_iovs = (*params.offset(2)).into();
        let writte_bytes_out = (*params.offset(3)).into();
        *returns = errno((*ctxt).fd_write_internal(fd, iovs, num_iovs, writte_bytes_out)).into();
    }

    /// Close a file descriptor.
    /// Note: This is similar to `close` in POSIX.
    ///
    /// #### Params
    /// - `fd`: `FD`: The file descriptor.
    ///
    /// #### Results
    /// - `Result<(), Errno>`: Returns `Ok(())` if the file descriptor is closed successfully, or an error if one occurred.
    #[inline]
    fn fd_close_internal(&self, fd: FD) -> Result<(), Errno> {
        log::debug!("wasi::fd_close(fd: {fd})");
        let opened_fd = self.get_fd(fd)?;
        if !opened_fd.should_close {
            return Err(Errno::Badf);
        }
        Ok(())
    }
    pub(super) unsafe extern "C" fn fd_close(
        ctxt: *mut WasiContext,
        params: *const ValueRaw,
        returns: *mut ValueRaw,
    ) {
        let fd = (*params.offset(0)).into();
        *returns = errno((*ctxt).fd_close_internal(fd)).into();
    }

    /// Get the attributes of a file descriptor.
    /// Note: This returns similar flags to `fcntl(fd, F_GETFL)` in POSIX, as well as additional fields.
    ///
    /// #### Params
    /// - `fd`: `FD`: The file descriptor.
    ///
    /// #### Results
    /// - `Result<fdstat, Errno>`: Returns the attributes of the file descriptor if successful, or an error if one occurred.

    #[inline]
    fn fd_fdstat_get_internal(&self, fd: FD, out: Ptr<FdStat>) -> Result<(), Errno> {
        log::debug!("wasi::fd_fdstat_get(fd: {fd})");
        let opened_fd = self.get_fd(fd)?;
        let out = self.get_memory_slice(out, 1)?;
        out[0] = opened_fd.stat;
        Ok(())
    }
    pub(super) unsafe extern "C" fn fd_fdstat_get(
        ctxt: *mut WasiContext,
        params: *const ValueRaw,
        returns: *mut ValueRaw,
    ) {
        let fd = (*params.offset(0)).into();
        let out = (*params.offset(1)).into();
        *returns = errno((*ctxt).fd_fdstat_get_internal(fd, out)).into();
    }

    /// Move the offset of a file descriptor.
    /// Note: This is similar to `lseek` in POSIX.
    ///
    ///
    /// #### Params
    /// - `fd`: `FD`: The file descriptor.
    /// - `offset`: `FileDelta`: The number of bytes to move.
    /// - `whence`: `Whence`: The base from which the offset is relative.
    ///
    /// #### Results
    /// - `Result<FileSize, Errno>`: The new offset of the file descriptor, relative to the start of the file.
    #[inline]
    fn fd_seek_internal(
        &self,
        fd: FD,
        offset: FileDelta,
        whence: Whence,
        out: Ptr<FileSize>,
    ) -> Result<(), Errno> {
        log::debug!("wasi::fd_seek(fd: {fd}, offset: {offset}, whence: {whence:?})");
        let opened_fd = self.get_fd(fd)?;
        opened_fd.has_right_or_err(Rights::FdSeek)?;
        let out = self.get_memory_slice(out, 1)?;
        debug_assert_eq!(Whence::Cur as u8, libc::SEEK_CUR as u8);
        debug_assert_eq!(Whence::End as u8, libc::SEEK_END as u8);
        debug_assert_eq!(Whence::Set as u8, libc::SEEK_SET as u8);
        let res = unsafe { libc::lseek(opened_fd.fd, offset, whence as i32) };
        if res == -1 {
            return Err(Errno::Io);
        }
        // to u64 is safe, as a currect offset is always non-negative
        out[0] = res as u64;
        Ok(())
    }
    pub(super) unsafe extern "C" fn fd_seek(
        ctxt: *mut WasiContext,
        params: *const ValueRaw,
        returns: *mut ValueRaw,
    ) {
        let fd = (*params.offset(0)).into();
        let offset = (*params.offset(1)).into();
        let whence = std::mem::transmute::<u8, Whence>((*params.offset(2)).as_u32() as u8);
        let out = (*params.offset(3)).into();
        *returns = errno((*ctxt).fd_seek_internal(fd, offset, whence, out)).into();
    }

    /// Adjust the flags associated with a file descriptor.
    /// Note: This is similar to `fcntl(fd, F_SETFL, flags)` in POSIX.
    ///
    /// #### Params
    /// - `fd`: `FD`: The file descriptor.
    /// - `flags`: `fdflags`: The desired values of the file descriptor flags.
    ///
    /// #### Results
    /// - `Result<(), Errno>`: Returns `Ok(())` if the flags are successfully adjusted, or an error if one occurred.
    #[inline]
    fn fd_fdstat_set_flags_internal(&mut self, fd: FD, flags: FdFlags) -> Result<(), Errno> {
        log::debug!("wasi::fd_fdstat_set_flags(fd: {fd}, flags: {flags:?})");
        let opened_fd = self.get_fd_mut(fd)?;
        opened_fd.has_right_or_err(Rights::FdFdstatSetFlags)?;

        // on linux, changing of the sync flags is not possible and would be silently ignored by fcntl
        if flags & FdFlags::DSync != opened_fd.stat.fs_flags & FdFlags::DSync
            || flags & FdFlags::Sync != opened_fd.stat.fs_flags & FdFlags::Sync
            || flags & FdFlags::RSync != opened_fd.stat.fs_flags & FdFlags::RSync
        {
            return Err(Errno::Inval);
        }

        // TODO: do we actually need to sync this with the file system?
        debug_assert_eq!(FdFlags::Append.bits(), libc::O_APPEND as u16);
        debug_assert_eq!(FdFlags::Nonblock.bits(), libc::O_NONBLOCK as u16);
        let res = unsafe { libc::fcntl(opened_fd.fd, libc::F_SETFL, flags.bits() as libc::c_int) };
        if res == -1 {
            return Err(Errno::Io);
        }
        opened_fd.stat.fs_flags = flags;
        Ok(())
    }
    pub(super) unsafe extern "C" fn fd_fdstat_set_flags(
        ctxt: *mut WasiContext,
        params: *const ValueRaw,
        returns: *mut ValueRaw,
    ) {
        let fd = (*params.offset(0)).into();
        let flags = FdFlags::from_bits_truncate((*params.offset(2)).as_u32() as u16);
        *returns = errno((*ctxt).fd_fdstat_set_flags_internal(fd, flags)).into();
    }

    /// Return a description of the given preopened file descriptor.
    ///
    /// #### Params
    /// - `fd`: `FD`: The file descriptor.
    ///
    /// #### Results
    /// - `Result<prestat, errno>`: Returns the description of the preopened file descriptor if successful, or an error if one occurred.
    #[inline]
    fn fd_prestat_get_internal(&self, fd: FD, out: Ptr<PreStat>) -> Result<(), Errno> {
        log::debug!("wasi::fd_prestat_get(fd: {fd})");
        let opened_fd = self.get_fd(fd)?;
        match opened_fd.file_path {
            None => Err(Errno::NotSup),
            Some(ref path) => {
                let out = self.get_memory_slice(out, 1)?;
                out[0] = PreStat::Dir(PreStatDir {
                    pr_name_len: path.len() as u32,
                });
                Ok(())
            }
        }
    }
    pub(super) unsafe extern "C" fn fd_prestat_get(
        ctxt: *mut WasiContext,
        params: *const ValueRaw,
        returns: *mut ValueRaw,
    ) {
        let fd = (*params.offset(0)).into();
        let out = (*params.offset(1)).into();
        *returns = errno((*ctxt).fd_prestat_get_internal(fd, out)).into();
    }

    /// Return a description of the given preopened file descriptor.
    ///
    /// #### Params
    /// - `fd`: `FD`: The file descriptor.
    /// - `path`: `Ptr`: A buffer into which to write the preopened directory name.
    /// - `path_len`: `Size`: The length of the buffer.
    ///
    /// #### Results
    /// - `Result<(), Errno>`: Returns `Ok(())` if the description is successfully written to the buffer, or an error if one occurred.
    #[inline]
    fn fd_prestat_dir_name_internal(
        &self,
        fd: FD,
        out_path: Ptr<u8>,
        out_path_len: Size,
    ) -> Result<(), Errno> {
        log::debug!("wasi::fd_prestat_dir_name(fd: {fd})");
        let opened_fd = self.get_fd(fd)?;
        match opened_fd.file_path {
            None => Err(Errno::NotSup),
            Some(ref stored_path) => {
                if stored_path.len() > out_path_len as usize {
                    return Err(Errno::NameTooLong);
                }
                // use stored_path.len() to prevent panic in subsequent copy_from_slice
                let out_path = self.get_memory_slice(out_path, stored_path.len())?;
                out_path.copy_from_slice(stored_path.as_bytes());
                log::debug!("wasi::fd_prestat_dir_name -> {}", stored_path);
                Ok(())
            }
        }
    }
    pub(super) unsafe extern "C" fn fd_prestat_dir_name(
        ctxt: *mut WasiContext,
        params: *const ValueRaw,
        returns: *mut ValueRaw,
    ) {
        let fd = (*params.offset(0)).into();
        let path = (*params.offset(1)).into();
        let path_len = (*params.offset(2)).into();
        *returns = errno((*ctxt).fd_prestat_dir_name_internal(fd, path, path_len)).into();
    }

    /// Read from a file descriptor.
    /// Note: This is similar to `readv` in POSIX.
    ///
    /// #### Params
    /// - `fd`: `FD`: The file descriptor.
    /// - `iovs`: `IOVecArray`: The array of iovec structures specifying the buffers to read into.
    ///
    /// #### Results
    /// - `Result<size, errno>`: Returns the number of bytes read if successful, or an error if one occurred.
    #[inline]
    fn fd_read_internal(
        &self,
        fd: FD,
        iovs: IOVecArray,
        num_iovs: Size,
        read_bytes_out: Ptr<Size>,
    ) -> Result<(), Errno> {
        log::debug!("wasi::fd_read(fd: {fd}, num_data_vecs: {num_iovs})");
        let opened_fd: &FileDescriptor = self.get_fd(fd)?;
        // harmonize with wasmtime, as we would normally return NotCapable
        opened_fd
            .has_right_or_err(Rights::FdRead)
            .map_err(|_| Errno::Badf)?;
        let input_iovecs = self.get_memory_slice(iovs, num_iovs as usize)?;
        let read_bytes_out = self.get_memory_slice(read_bytes_out, 1)?;

        let mut iovecs = Vec::with_capacity(num_iovs as usize);
        for iov in input_iovecs {
            let buf = self.get_memory_slice(iov.buf, iov.buf_len as usize)?;
            iovecs.push(libc::iovec {
                iov_base: buf.as_ptr() as _,
                iov_len: buf.len(),
            })
        }
        let read = unsafe { libc::readv(opened_fd.fd, iovecs.as_ptr(), num_iovs as i32) };
        if read == -1 {
            return Err(Errno::Io);
        }
        read_bytes_out[0] = read as u32;
        Ok(())
    }
    pub(super) unsafe extern "C" fn fd_read(
        ctxt: *mut WasiContext,
        params: *const ValueRaw,
        returns: *mut ValueRaw,
    ) {
        let fd = (*params.offset(0)).into();
        let iovs = (*params.offset(1)).into();
        let num_iovs = (*params.offset(2)).into();
        let read_bytes_out = (*params.offset(3)).into();
        *returns = errno((*ctxt).fd_read_internal(fd, iovs, num_iovs, read_bytes_out)).into();
    }

    /// Open a file or directory.
    /// The returned file descriptor is not guaranteed to be the lowest-numbered file descriptor not currently open; it is randomized to prevent applications from depending on making assumptions about indexes, since this is error-prone in multi-threaded contexts. The returned file descriptor is guaranteed to be less than 2**31.
    /// Note: This is similar to `openat` in POSIX.
    ///
    /// #### Params
    /// - `fd`: `FD`: The file descriptor.
    /// - `dirflags`: `LookupFlags`: Flags determining the method of how the path is resolved.
    /// - `path`: `String`: The relative path of the file or directory to open, relative to the `fd` directory.
    /// - `oflags`: `OFlags`: The method by which to open the file.
    /// - `fs_rights_base`: `Rights`: The initial rights of the newly created file descriptor. The implementation is allowed to return a file descriptor with fewer rights than specified, if and only if those rights do not apply to the type of file being opened. The *base* rights are rights that will apply to operations using the file descriptor itself, while the *inheriting* rights are rights that apply to file descriptors derived from it.
    /// - `fs_rights_inheriting`: `Rights`
    /// - `fdflags`: `FdFlags`
    ///
    /// #### Results
    /// - `Result<FD, Errno>`: The file descriptor of the file that has been opened.
    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn path_open_internal(
        &mut self,
        fd: FD,
        dirflags: LookupFlags,
        path: Ptr<u8>,
        path_len: Size,
        oflags: OpenFlags,
        fs_rights_base: Rights,
        fs_rights_inheriting: Rights,
        fdflags: FdFlags,
        out_fd: Ptr<FD>,
    ) -> Result<(), Errno> {
        log::debug!("wasi::path_open(fd: {fd}...");
        let opened_fd = self.get_fd(fd)?;
        if opened_fd.stat.fs_filetype != FileType::Directory {
            return Err(Errno::NotDir);
        }
        let path = self.get_memory_slice(path, path_len as usize)?;
        let null_terminated_path = CString::new(path).map_err(|_| Errno::Inval)?;
        let os_path = OsStr::from_bytes(null_terminated_path.as_bytes());
        let path = Path::new(os_path);
        if !path.is_relative() {
            log::debug!("...path not relative");
            return Err(Errno::Inval);
        }
        log::debug!("...path: {:?})wasi::path_open", path);

        let flags = oflags.to_libc()
            | dirflags.to_libc()
            | fs_rights_base.to_libc_open_flags()?
            | fdflags.to_libc_open_flags();

        let fd_res = unsafe {
            libc::openat(
                opened_fd.fd,
                null_terminated_path.into_bytes_with_nul().as_ptr() as *const i8,
                flags,
                fs_rights_base.to_libc_mode(),
            )
        };
        if fd_res == -1 {
            return Err(Errno::Io);
        }
        let new_host_fd = FileDescriptor {
            fd: fd_res,
            stat: FdStat {
                fs_filetype: FileType::RegularFile,
                fs_rights_base,
                fs_rights_inheriting,
                fs_flags: fdflags,
            },
            file_path: None,
            should_close: true,
        };
        let out_fd = self.get_memory_slice(out_fd, 1)?;
        // TODO: check for shared memory to prevent race condition
        out_fd[0] = self.open_fds.len() as i32;
        self.open_fds.push(new_host_fd);
        Ok(())
    }
    pub(super) unsafe extern "C" fn path_open(
        ctxt: *mut WasiContext,
        params: *const ValueRaw,
        returns: *mut ValueRaw,
    ) {
        let fd = (*params.offset(0)).into();
        let dirflags = LookupFlags::from_bits_truncate((*params.offset(1)).as_u32());
        let path = (*params.offset(2)).into();
        let path_len = (*params.offset(3)).into();
        let oflags = OpenFlags::from_bits_truncate((*params.offset(4)).as_u32() as u16);
        let fs_rights_base = Rights::from_bits_truncate((*params.offset(5)).as_u64());
        let fs_rights_inheriting = Rights::from_bits_truncate((*params.offset(6)).as_u64());
        let fdflags = FdFlags::from_bits_truncate((*params.offset(7)).as_u32() as u16);
        let out_fd = (*params.offset(8)).into();
        *returns = errno((*ctxt).path_open_internal(
            fd,
            dirflags,
            path,
            path_len,
            oflags,
            fs_rights_base,
            fs_rights_inheriting,
            fdflags,
            out_fd,
        ))
        .into();
    }

    /// Return the attributes of a file or directory.
    /// Note: This is similar to `stat` in POSIX.
    ///
    /// #### Params
    /// - `fd`: `FD`: The file descriptor.
    /// - `flags`: `LookupFlags`: Flags determining the method of how the path is resolved.
    /// - `path`: `String`: The path of the file or directory to inspect.
    ///
    /// #### Results
    /// - `Result<FileStat, Errno>`: The buffer where the file's attributes are stored.
    #[inline]
    fn path_filestat_get_internal(
        &self,
        fd: FD,
        flags: LookupFlags,
        path: Ptr<u8>,
        path_len: Size,
        out: Ptr<FileStat>,
    ) -> Result<(), Errno> {
        log::debug!("wasi::path_filestat_get(fd: {fd})");
        let opened_fd = self.get_fd(fd)?;
        if opened_fd.stat.fs_filetype != FileType::Directory {
            return Err(Errno::NotDir);
        }
        let path = self.get_memory_slice(path, path_len as usize)?;
        let null_terminated_path = CString::new(path).map_err(|_| Errno::Inval)?;
        let os_path = OsStr::from_bytes(null_terminated_path.as_bytes());
        let path = Path::new(os_path);
        if !path.is_relative() {
            return Err(Errno::Inval);
        }

        let mut stat = MaybeUninit::<libc::stat>::uninit();
        let stat_res = unsafe {
            libc::fstatat(
                opened_fd.fd,
                null_terminated_path.as_ptr(),
                stat.as_mut_ptr(),
                // fstatat by default follows symlinks
                if flags.contains(LookupFlags::SymlinkFollow) {
                    0
                } else {
                    libc::AT_SYMLINK_NOFOLLOW
                },
            )
        };
        if stat_res == -1 {
            // TODO: map real errno to wasi errno
            log::debug!("fstatat failed: {}", std::io::Error::last_os_error());
            return Err(Errno::Io);
        }
        let stat = unsafe { stat.assume_init() };
        let filetype = FileType::from_libc(stat.st_mode);
        let out = self.get_memory_slice(out, 1)?;
        out[0] = FileStat {
            dev: stat.st_dev,
            ino: stat.st_ino,
            filetype,
            nlink: stat.st_nlink,
            size: stat.st_size as FileSize,
            atim: stat.st_atime as TimeStamp,
            mtim: stat.st_mtime as TimeStamp,
            ctim: stat.st_ctime as TimeStamp,
        };
        Ok(())
    }
    pub(super) unsafe extern "C" fn path_filestat_get(
        ctxt: *mut WasiContext,
        params: *const ValueRaw,
        returns: *mut ValueRaw,
    ) {
        let fd = (*params.offset(0)).into();
        let flags = LookupFlags::from_bits_truncate((*params.offset(1)).as_u32());
        let path = (*params.offset(2)).into();
        let path_len = (*params.offset(3)).into();
        let out = (*params.offset(4)).into();
        *returns = errno((*ctxt).path_filestat_get_internal(fd, flags, path, path_len, out)).into();
    }

    /// Return the attributes of an open file.
    ///
    /// #### Params
    /// - `fd`: `FD`: The file descriptor.
    ///
    /// #### Results
    /// - `Result<FileStat, errno>`: Returns the attributes of the file descriptor if successful, or an error if one occurred.
    #[inline]
    fn fd_filestat_get_internal(&self, fd: FD, out: Ptr<FileStat>) -> Result<(), Errno> {
        log::debug!("wasi::fd_filestat_get(fd: {fd})");
        let opened_fd = self.get_fd(fd)?;
        let out = self.get_memory_slice(out, 1)?;

        let mut stat = MaybeUninit::<libc::stat>::uninit();
        let stat_res = unsafe { libc::fstat(opened_fd.fd, stat.as_mut_ptr()) };
        if stat_res == -1 {
            // TODO: map real errno to wasi errno
            log::debug!("fstat failed: {}", std::io::Error::last_os_error());
            return Err(Errno::Io);
        }
        let stat = unsafe { stat.assume_init() };
        let filetype = FileType::from_libc(stat.st_mode);
        out[0] = FileStat {
            dev: stat.st_dev,
            ino: stat.st_ino,
            filetype,
            nlink: stat.st_nlink,
            size: stat.st_size as FileSize,
            atim: stat.st_atime as TimeStamp,
            mtim: stat.st_mtime as TimeStamp,
            ctim: stat.st_ctime as TimeStamp,
        };
        Ok(())
    }
    pub(super) unsafe extern "C" fn fd_filestat_get(
        ctxt: *mut WasiContext,
        params: *const ValueRaw,
        returns: *mut ValueRaw,
    ) {
        let fd = (*params.offset(0)).into();
        let out = (*params.offset(1)).into();
        *returns = errno((*ctxt).fd_filestat_get_internal(fd, out)).into();
    }
}

// /// Return the resolution of a clock.
// /// Implementations are required to provide a non-zero value for supported clocks. For unsupported clocks, return `Errno::Inval`.
// /// Note: This is similar to `clock_getres` in POSIX.
// ///
// /// #### Params
// /// - `id`: `ClockID`: The clock for which to return the resolution.
// ///
// /// #### Results
// /// - `Result<TimeStamp, Errno>`: The resolution of the clock, or an error if one happened.
// ///
// ///
// pub extern "C" fn clock_res_get(id: ClockID) -> Result<TimeStamp, Errno> {
//     unimplemented!()
// }

// /// Return the time value of a clock.
// /// Note: This is similar to `clock_gettime` in POSIX.
// ///
// /// #### Params
// /// - `id`: `ClockID`: The clock for which to return the time.
// /// - `precision`: `TimeStamp`: The maximum lag (exclusive) that the returned time value may have, compared to its actual value.
// ///
// /// #### Results
// /// - `Result<TimeStamp, Errno>`: The time value of the clock.
// pub extern "C" fn clock_time_get(id: ClockID, precision: TimeStamp) -> Result<TimeStamp, Errno> {
//     unimplemented!()
// }

// /// Provide file advisory information on a file descriptor.
// /// Note: This is similar to `posix_fadvise` in POSIX.
// ///
// /// #### Params
// /// - `fd`: `FD`: The file descriptor.
// /// - `offset`: `FileSize`: The offset within the file to which the advisory applies.
// /// - `len`: `FileSize`: The length of the region to which the advisory applies.
// /// - `advice`: `Advice`: The advice.
// ///
// /// #### Results
// /// - `Result<(), Errno>`
// pub extern "C" fn fd_advise(
//     fd: FD,
//     offset: FileSize,
//     len: FileSize,
//     advice: Advice,
// ) -> Result<(), Errno> {
//     unimplemented!()
// }

// /// Force the allocation of space in a file.
// /// Note: This is similar to `posix_fallocate` in POSIX.
// ///
// /// #### Params
// /// - `fd`: `FD`: The file descriptor.
// /// - `offset`: `FileSize`: The offset at which to start the allocation.
// /// - `len`: `FileSize`: The length of the area that is allocated.
// ///
// /// #### Results
// /// - `Result<(), Errno>`
// pub extern "C" fn fd_allocate(fd: FD, offset: FileSize, len: FileSize) -> Result<(), Errno> {
//     unimplemented!()
// }

// /// Synchronize the data of a file to disk.
// /// Note: This is similar to `fdatasync` in POSIX.
// ///
// /// #### Params
// /// - `fd`: `FD`: The file descriptor.
// ///
// /// #### Results
// /// - `Result<(), Errno>`: Returns `Ok(())` if the data is successfully synchronized to disk, or an error if one occurred.
// pub extern "C" fn fd_datasync(fd: FD) -> Result<(), Errno> {
//     unimplemented!()
// }

// /// Adjust the rights associated with a file descriptor.
// /// This can only be used to remove rights, and returns `Errno::NotCapable` if called in a way that would attempt to add rights.
// ///
// /// #### Params
// /// - `fd`: `FD`: The file descriptor.
// /// - `fs_rights_base`: `rights`: The desired rights of the file descriptor.
// /// - `fs_rights_inheriting`: `rights`: The desired inheriting rights of the file descriptor.
// ///
// /// #### Results
// /// - `Result<(), Errno>`: Returns `Ok(())` if the rights are successfully adjusted, or an error if one occurred.
// pub extern "C" fn fd_fdstat_set_rights(
//     fd: FD,
//     fs_rights_base: Rights,
//     fs_rights_inheriting: Rights,
// ) -> Result<(), Errno> {
//     unimplemented!()
// }

// /// Adjust the size of an open file. If this increases the file's size, the extra bytes are filled with zeros.
// /// Note: This is similar to `ftruncate` in POSIX.
// ///
// /// #### Params
// /// - `fd`: `FD`: The file descriptor.
// /// - `size`: `filesize`: The desired file size.
// ///
// /// #### Results
// /// - `Result<(), errno>`
// pub extern "C" fn fd_filestat_set_size(fd: FD, size: FileSize) -> Result<(), Errno> {
//     unimplemented!()
// }

// /// Adjust the timestamps of an open file or directory.
// /// Note: This is similar to `futimens` in POSIX.
// ///
// /// #### Params
// /// - `fd`: `FD`: The file descriptor.
// /// - `atim`: `timestamp`: The desired values of the data access timestamp.
// /// - `mtim`: `timestamp`: The desired values of the data modification timestamp.
// /// - `fst_flags`: `fstflags`: A bitmask indicating which timestamps to adjust.
// ///
// /// #### Results
// /// - `Result<(), errno>`
// pub extern "C" fn fd_filestat_set_times(
//     fd: FD,
//     atim: TimeStamp,
//     mtim: TimeStamp,
//     fst_flags: FstFlags,
// ) -> Result<(), Errno> {
//     unimplemented!()
// }

// /// Read from a file descriptor, without using and updating the file descriptor's offset.
// ///
// /// #### Params
// /// - `fd`: `FD`: The file descriptor.
// /// - `iovs`: `IOVecArray`: The array of iovec structures specifying the buffers to read into.
// /// - `offset`: `FileSize`: The offset within the file at which to read.
// ///
// /// #### Results
// /// - `Result<size, errno>`: Returns the number of bytes read if successful, or an error if one occurred.
// pub extern "C" fn fd_pread(fd: FD, iovs: IOVecArray, offset: FileSize) -> Result<Size, Errno> {
//     unimplemented!()
// }

// /// Write to a file descriptor, without using and updating the file descriptor's offset.
// ///
// /// #### Params
// /// - `fd`: `FD`: The file descriptor.
// /// - `iovs`: `CIOVecArray`: List of scatter/gather vectors from which to retrieve data.
// /// - `offset`: `FileSize`: The offset within the file at which to write.
// ///
// /// #### Results
// /// - `Result<size, errno>`: Returns the number of bytes written if successful, or an error if one occurred.
// pub extern "C" fn fd_pwrite(fd: FD, iovs: CIOVecArray, offset: FileSize) -> Result<Size, Errno> {
//     unimplemented!()
// }

// /// Temporarily yield execution of the calling thread.
// /// Note: This is similar to `sched_yield` in POSIX.
// ///
// /// #### Results
// /// - `Result<(), Errno>`
// pub extern "C" fn sched_yield() -> Result<(), Errno> {
//     unimplemented!()
// }
