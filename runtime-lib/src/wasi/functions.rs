use super::types::*;

/// Read command-line argument data.
#[no_mangle]
pub extern "C" fn args_get(argv: Ptr<Ptr<u8>>, argv_buf: Ptr<u8>) -> Result<(), Errno> {
    unimplemented!()
}

/// Reads command-line
///
/// #### Results
/// - `Result<(Size, Size), Errno>`: Returns the number of arguments and the size of the argument string data, or an error.
#[no_mangle]
pub extern "C" fn args_sizes_get() -> Result<(Size, Size), Errno> {
    unimplemented!()
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
#[no_mangle]
pub extern "C" fn environ_get(environ: Ptr<Ptr<u8>>, environ_buf: Ptr<u8>) -> Result<(), Errno> {
    unimplemented!()
}

/// Return environment variable data sizes.
///
/// #### Results
/// - `Result<(Size, Size), Errno>`: Returns the number of environment variable arguments and the size of the environment variable data.
#[no_mangle]
pub extern "C" fn environ_sizes_get() -> Result<(Size, Size), Errno> {
    unimplemented!()
}

/// Return the resolution of a clock.
/// Implementations are required to provide a non-zero value for supported clocks. For unsupported clocks, return `Errno::Inval`.
/// Note: This is similar to `clock_getres` in POSIX.
///
/// #### Params
/// - `id`: `ClockID`: The clock for which to return the resolution.
///
/// #### Results
/// - `Result<TimeStamp, Errno>`: The resolution of the clock, or an error if one happened.
///
///
#[no_mangle]
pub extern "C" fn clock_res_get(id: ClockID) -> Result<TimeStamp, Errno> {
    unimplemented!()
}

/// Return the time value of a clock.
/// Note: This is similar to `clock_gettime` in POSIX.
///
/// #### Params
/// - `id`: `ClockID`: The clock for which to return the time.
/// - `precision`: `TimeStamp`: The maximum lag (exclusive) that the returned time value may have, compared to its actual value.
///
/// #### Results
/// - `Result<TimeStamp, Errno>`: The time value of the clock.
#[no_mangle]
pub extern "C" fn clock_time_get(id: ClockID, precision: TimeStamp) -> Result<TimeStamp, Errno> {
    unimplemented!()
}

/// Provide file advisory information on a file descriptor.
/// Note: This is similar to `posix_fadvise` in POSIX.
///
/// #### Params
/// - `fd`: `FD`: The file descriptor.
/// - `offset`: `FileSize`: The offset within the file to which the advisory applies.
/// - `len`: `FileSize`: The length of the region to which the advisory applies.
/// - `advice`: `Advice`: The advice.
///
/// #### Results
/// - `Result<(), Errno>`
#[no_mangle]
pub extern "C" fn fd_advise(
    fd: FD,
    offset: FileSize,
    len: FileSize,
    advice: Advice,
) -> Result<(), Errno> {
    unimplemented!()
}

/// Force the allocation of space in a file.
/// Note: This is similar to `posix_fallocate` in POSIX.
///
/// #### Params
/// - `fd`: `FD`: The file descriptor.
/// - `offset`: `FileSize`: The offset at which to start the allocation.
/// - `len`: `FileSize`: The length of the area that is allocated.
///
/// #### Results
/// - `Result<(), Errno>`
#[no_mangle]
pub extern "C" fn fd_allocate(fd: FD, offset: FileSize, len: FileSize) -> Result<(), Errno> {
    unimplemented!()
}

/// Close a file descriptor.
/// Note: This is similar to `close` in POSIX.
///
/// #### Params
/// - `fd`: `FD`: The file descriptor.
///
/// #### Results
/// - `Result<(), Errno>`: Returns `Ok(())` if the file descriptor is closed successfully, or an error if one occurred.
#[no_mangle]
pub extern "C" fn fd_close(fd: FD) -> Result<(), Errno> {
    unimplemented!()
}

/// Synchronize the data of a file to disk.
/// Note: This is similar to `fdatasync` in POSIX.
///
/// #### Params
/// - `fd`: `FD`: The file descriptor.
///
/// #### Results
/// - `Result<(), Errno>`: Returns `Ok(())` if the data is successfully synchronized to disk, or an error if one occurred.
#[no_mangle]
pub extern "C" fn fd_datasync(fd: FD) -> Result<(), Errno> {
    unimplemented!()
}

/// Get the attributes of a file descriptor.
/// Note: This returns similar flags to `fcntl(fd, F_GETFL)` in POSIX, as well as additional fields.
///
/// #### Params
/// - `fd`: `FD`: The file descriptor.
///
/// #### Results
/// - `Result<fdstat, Errno>`: Returns the attributes of the file descriptor if successful, or an error if one occurred.
#[no_mangle]
pub extern "C" fn fd_fdstat_get(fd: FD) -> Result<FdStat, Errno> {
    unimplemented!()
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
#[no_mangle]
pub extern "C" fn fd_fdstat_set_flags(fd: FD, flags: FdFlags) -> Result<(), Errno> {
    unimplemented!()
}

/// Adjust the rights associated with a file descriptor.
/// This can only be used to remove rights, and returns `Errno::NotCapable` if called in a way that would attempt to add rights.
///
/// #### Params
/// - `fd`: `FD`: The file descriptor.
/// - `fs_rights_base`: `rights`: The desired rights of the file descriptor.
/// - `fs_rights_inheriting`: `rights`: The desired inheriting rights of the file descriptor.
///
/// #### Results
/// - `Result<(), Errno>`: Returns `Ok(())` if the rights are successfully adjusted, or an error if one occurred.
#[no_mangle]
pub extern "C" fn fd_fdstat_set_rights(
    fd: FD,
    fs_rights_base: Rights,
    fs_rights_inheriting: Rights,
) -> Result<(), Errno> {
    unimplemented!()
}

/// Return the attributes of an open file.
///
/// #### Params
/// - `fd`: `FD`: The file descriptor.
///
/// #### Results
/// - `Result<filestat, errno>`: Returns the attributes of the file descriptor if successful, or an error if one occurred.
#[no_mangle]
pub extern "C" fn fd_filestat_get(fd: FD, out: Ptr<FileStat>) -> Errno {
    unimplemented!()
}

/// Adjust the size of an open file. If this increases the file's size, the extra bytes are filled with zeros.
/// Note: This is similar to `ftruncate` in POSIX.
///
/// #### Params
/// - `fd`: `FD`: The file descriptor.
/// - `size`: `filesize`: The desired file size.
///
/// #### Results
/// - `Result<(), errno>`
#[no_mangle]
pub extern "C" fn fd_filestat_set_size(fd: FD, size: FileSize) -> Result<(), Errno> {
    unimplemented!()
}

/// Adjust the timestamps of an open file or directory.
/// Note: This is similar to `futimens` in POSIX.
///
/// #### Params
/// - `fd`: `FD`: The file descriptor.
/// - `atim`: `timestamp`: The desired values of the data access timestamp.
/// - `mtim`: `timestamp`: The desired values of the data modification timestamp.
/// - `fst_flags`: `fstflags`: A bitmask indicating which timestamps to adjust.
///
/// #### Results
/// - `Result<(), errno>`
#[no_mangle]
pub extern "C" fn fd_filestat_set_times(
    fd: FD,
    atim: TimeStamp,
    mtim: TimeStamp,
    fst_flags: FstFlags,
) -> Result<(), Errno> {
    unimplemented!()
}

/// Read from a file descriptor, without using and updating the file descriptor's offset.
///
/// #### Params
/// - `fd`: `FD`: The file descriptor.
/// - `iovs`: `IOVecArray`: The array of iovec structures specifying the buffers to read into.
/// - `offset`: `FileSize`: The offset within the file at which to read.
///
/// #### Results
/// - `Result<size, errno>`: Returns the number of bytes read if successful, or an error if one occurred.
#[no_mangle]
pub extern "C" fn fd_pread(fd: FD, iovs: IOVecArray, offset: FileSize) -> Result<Size, Errno> {
    unimplemented!()
}

/// Return a description of the given preopened file descriptor.
///
/// #### Params
/// - `fd`: `FD`: The file descriptor.
///
/// #### Results
/// - `Result<prestat, errno>`: Returns the description of the preopened file descriptor if successful, or an error if one occurred.
#[no_mangle]
pub extern "C" fn fd_prestat_get(fd: FD) -> Result<PreStat, Errno> {
    unimplemented!()
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
#[no_mangle]
pub extern "C" fn fd_prestat_dir_name(
    fd: FD,
    path: Ptr<libc::c_char>,
    path_len: Size,
) -> Result<(), Errno> {
    unimplemented!()
}

/// Write to a file descriptor, without using and updating the file descriptor's offset.
///
/// #### Params
/// - `fd`: `FD`: The file descriptor.
/// - `iovs`: `CIOVecArray`: List of scatter/gather vectors from which to retrieve data.
/// - `offset`: `FileSize`: The offset within the file at which to write.
///
/// #### Results
/// - `Result<size, errno>`: Returns the number of bytes written if successful, or an error if one occurred.
#[no_mangle]
pub extern "C" fn fd_pwrite(fd: FD, iovs: CIOVecArray, offset: FileSize) -> Result<Size, Errno> {
    unimplemented!()
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
#[no_mangle]
pub extern "C" fn fd_read(fd: FD, iovs: IOVecArray) -> Result<Size, Errno> {
    unimplemented!()
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
#[no_mangle]
pub extern "C" fn fd_seek(fd: FD, offset: FileDelta, whence: Whence) -> Result<FileSize, Errno> {
    unimplemented!()
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
#[no_mangle]
pub extern "C" fn fd_write(fd: FD, iovs: CIOVecArray) -> Result<Size, Errno> {
    unimplemented!()
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
#[no_mangle]
pub extern "C" fn path_filestat_get(
    fd: FD,
    flags: LookupFlags,
    path: String,
) -> Result<FileStat, Errno> {
    unimplemented!()
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
#[no_mangle]
pub extern "C" fn path_open(
    fd: FD,
    dirflags: LookupFlags,
    path: String,
    oflags: OpenFlags,
    fs_rights_base: Rights,
    fs_rights_inheriting: Rights,
    fdflags: FdFlags,
) -> Result<FD, Errno> {
    unimplemented!()
}

/// Terminate the process normally. An exit code of 0 indicates successful termination of the program. The meanings of other values is dependent on the environment.
///
/// #### Params
/// - `rval`: `ExitCode`: The exit code returned by the process.
#[no_mangle]
pub extern "C" fn proc_exit(rval: ExitCode) {
    unimplemented!()
}

/// Temporarily yield execution of the calling thread.
/// Note: This is similar to `sched_yield` in POSIX.
///
/// #### Results
/// - `Result<(), Errno>`
#[no_mangle]
pub extern "C" fn sched_yield() -> Result<(), Errno> {
    unimplemented!()
}

// ---

// #### <a href="#random_get" name="random_get"></a> `random_get(buf: Pointer<u8>, buf_len: size) -> Result<(), errno>`
// Write high-quality random data into a buffer.
// This function blocks when the implementation is unable to immediately
// provide sufficient high-quality random data.
// This function may execute slowly, so when large mounts of random data are
// required, it's advisable to use this function to seed a pseudo-random
// number generator, rather than to provide the random data directly.

// ##### Params
// - <a href="#random_get.buf" name="random_get.buf"></a> `buf`: `Pointer<u8>`
// The buffer to fill with random data.

// - <a href="#random_get.buf_len" name="random_get.buf_len"></a> `buf_len`: [`size`](#size)

// ##### Results
// - <a href="#random_get.error" name="random_get.error"></a> `error`: `Result<(), errno>`

// ###### Variant Layout
// - size: 8
// - align: 4
// - tag_size: 4
// ###### Variant cases
// - <a href="#random_get.error.ok" name="random_get.error.ok"></a> `ok`

// - <a href="#random_get.error.err" name="random_get.error.err"></a> `err`: [`errno`](#errno)

// ---

// #### <a href="#sock_accept" name="sock_accept"></a> `sock_accept(fd: fd, flags: fdflags) -> Result<fd, errno>`
// Accept a new incoming connection.
// Note: This is similar to `accept` in POSIX.

// ##### Params
// - <a href="#sock_accept.fd" name="sock_accept.fd"></a> `fd`: [`fd`](#fd)
// The listening socket.

// - <a href="#sock_accept.flags" name="sock_accept.flags"></a> `flags`: [`fdflags`](#fdflags)
// The desired values of the file descriptor flags.

// ##### Results
// - <a href="#sock_accept.error" name="sock_accept.error"></a> `error`: `Result<fd, errno>`
// New socket connection

// ###### Variant Layout
// - size: 8
// - align: 4
// - tag_size: 4
// ###### Variant cases
// - <a href="#sock_accept.error.ok" name="sock_accept.error.ok"></a> `ok`: [`fd`](#fd)

// - <a href="#sock_accept.error.err" name="sock_accept.error.err"></a> `err`: [`errno`](#errno)

// ---

// #### <a href="#sock_recv" name="sock_recv"></a> `sock_recv(fd: fd, ri_data: iovec_array, ri_flags: riflags) -> Result<(size, roflags), errno>`
// Receive a message from a socket.
// Note: This is similar to `recv` in POSIX, though it also supports reading
// the data into multiple buffers in the manner of `readv`.

// ##### Params
// - <a href="#sock_recv.fd" name="sock_recv.fd"></a> `fd`: [`fd`](#fd)

// - <a href="#sock_recv.ri_data" name="sock_recv.ri_data"></a> `ri_data`: [`iovec_array`](#iovec_array)
// List of scatter/gather vectors to which to store data.

// - <a href="#sock_recv.ri_flags" name="sock_recv.ri_flags"></a> `ri_flags`: [`riflags`](#riflags)
// Message flags.

// ##### Results
// - <a href="#sock_recv.error" name="sock_recv.error"></a> `error`: `Result<(size, roflags), errno>`
// Number of bytes stored in ri_data and message flags.

// ###### Variant Layout
// - size: 12
// - align: 4
// - tag_size: 4
// ###### Variant cases
// - <a href="#sock_recv.error.ok" name="sock_recv.error.ok"></a> `ok`: `(size, roflags)`

// ####### Record members
// - <a href="#sock_recv.error.ok.0" name="sock_recv.error.ok.0"></a> `0`: [`size`](#size)

// Offset: 0

// - <a href="#sock_recv.error.ok.1" name="sock_recv.error.ok.1"></a> `1`: [`roflags`](#roflags)

// Offset: 4

// - <a href="#sock_recv.error.err" name="sock_recv.error.err"></a> `err`: [`errno`](#errno)

// ---

// #### <a href="#sock_send" name="sock_send"></a> `sock_send(fd: fd, si_data: ciovec_array, si_flags: siflags) -> Result<size, errno>`
// Send a message on a socket.
// Note: This is similar to `send` in POSIX, though it also supports writing
// the data from multiple buffers in the manner of `writev`.

// ##### Params
// - <a href="#sock_send.fd" name="sock_send.fd"></a> `fd`: [`fd`](#fd)

// - <a href="#sock_send.si_data" name="sock_send.si_data"></a> `si_data`: [`ciovec_array`](#ciovec_array)
// List of scatter/gather vectors to which to retrieve data

// - <a href="#sock_send.si_flags" name="sock_send.si_flags"></a> `si_flags`: [`siflags`](#siflags)
// Message flags.

// ##### Results
// - <a href="#sock_send.error" name="sock_send.error"></a> `error`: `Result<size, errno>`
// Number of bytes transmitted.

// ###### Variant Layout
// - size: 8
// - align: 4
// - tag_size: 4
// ###### Variant cases
// - <a href="#sock_send.error.ok" name="sock_send.error.ok"></a> `ok`: [`size`](#size)

// - <a href="#sock_send.error.err" name="sock_send.error.err"></a> `err`: [`errno`](#errno)

// ---

// #### <a href="#sock_shutdown" name="sock_shutdown"></a> `sock_shutdown(fd: fd, how: sdflags) -> Result<(), errno>`
// Shut down socket send and receive channels.
// Note: This is similar to `shutdown` in POSIX.

// ##### Params
// - <a href="#sock_shutdown.fd" name="sock_shutdown.fd"></a> `fd`: [`fd`](#fd)

// - <a href="#sock_shutdown.how" name="sock_shutdown.how"></a> `how`: [`sdflags`](#sdflags)
// Which channels on the socket to shut down.

// ##### Results
// - <a href="#sock_shutdown.error" name="sock_shutdown.error"></a> `error`: `Result<(), errno>`

// ###### Variant Layout
// - size: 8
// - align: 4
// - tag_size: 4
// ###### Variant cases
// - <a href="#sock_shutdown.error.ok" name="sock_shutdown.error.ok"></a> `ok`

// - <a href="#sock_shutdown.error.err" name="sock_shutdown.error.err"></a> `err`: [`errno`](#errno)

// ---

// #### <a href="#fd_prestat_dir_name" name="fd_prestat_dir_name"></a> `fd_prestat_dir_name(fd: fd, path: Pointer<u8>, path_len: size) -> Result<(), errno>`
// Return a description of the given preopened file descriptor.

// ##### Params
// - <a href="#fd_prestat_dir_name.fd" name="fd_prestat_dir_name.fd"></a> `fd`: [`fd`](#fd)

// - <a href="#fd_prestat_dir_name.path" name="fd_prestat_dir_name.path"></a> `path`: `Pointer<u8>`
// A buffer into which to write the preopened directory name.

// - <a href="#fd_prestat_dir_name.path_len" name="fd_prestat_dir_name.path_len"></a> `path_len`: [`size`](#size)

// ##### Results
// - <a href="#fd_prestat_dir_name.error" name="fd_prestat_dir_name.error"></a> `error`: `Result<(), errno>`

// ###### Variant Layout
// - size: 8
// - align: 4
// - tag_size: 4
// ###### Variant cases
// - <a href="#fd_prestat_dir_name.error.ok" name="fd_prestat_dir_name.error.ok"></a> `ok`

// - <a href="#fd_prestat_dir_name.error.err" name="fd_prestat_dir_name.error.err"></a> `err`: [`errno`](#errno)

// ---

// #### <a href="#fd_pwrite" name="fd_pwrite"></a> `fd_pwrite(fd: fd, iovs: ciovec_array, offset: filesize) -> Result<size, errno>`
// Write to a file descriptor, without using and updating the file descriptor's offset.
// Note: This is similar to `pwritev` in Linux (and other Unix-es).

// Like Linux (and other Unix-es), any calls of `pwrite` (and other
// functions to read or write) for a regular file by other threads in the
// WASI process should not be interleaved while `pwrite` is executed.

// ##### Params
// - <a href="#fd_pwrite.fd" name="fd_pwrite.fd"></a> `fd`: [`fd`](#fd)

// - <a href="#fd_pwrite.iovs" name="fd_pwrite.iovs"></a> `iovs`: [`ciovec_array`](#ciovec_array)
// List of scatter/gather vectors from which to retrieve data.

// - <a href="#fd_pwrite.offset" name="fd_pwrite.offset"></a> `offset`: [`filesize`](#filesize)
// The offset within the file at which to write.

// ##### Results
// - <a href="#fd_pwrite.error" name="fd_pwrite.error"></a> `error`: `Result<size, errno>`
// The number of bytes written.

// ###### Variant Layout
// - size: 8
// - align: 4
// - tag_size: 4
// ###### Variant cases
// - <a href="#fd_pwrite.error.ok" name="fd_pwrite.error.ok"></a> `ok`: [`size`](#size)

// - <a href="#fd_pwrite.error.err" name="fd_pwrite.error.err"></a> `err`: [`errno`](#errno)

// ---
// ---

// #### <a href="#fd_readdir" name="fd_readdir"></a> `fd_readdir(fd: fd, buf: Pointer<u8>, buf_len: size, cookie: dircookie) -> Result<size, errno>`
// Read directory entries from a directory.
// When successful, the contents of the output buffer consist of a sequence of
// directory entries. Each directory entry consists of a [`dirent`](#dirent) object,
// followed by [`dirent::d_namlen`](#dirent.d_namlen) bytes holding the name of the directory
// entry.
// This function fills the output buffer as much as possible, potentially
// truncating the last directory entry. This allows the caller to grow its
// read buffer size in case it's too small to fit a single large directory
// entry, or skip the oversized directory entry.

// ##### Params
// - <a href="#fd_readdir.fd" name="fd_readdir.fd"></a> `fd`: [`fd`](#fd)

// - <a href="#fd_readdir.buf" name="fd_readdir.buf"></a> `buf`: `Pointer<u8>`
// The buffer where directory entries are stored

// - <a href="#fd_readdir.buf_len" name="fd_readdir.buf_len"></a> `buf_len`: [`size`](#size)

// - <a href="#fd_readdir.cookie" name="fd_readdir.cookie"></a> `cookie`: [`dircookie`](#dircookie)
// The location within the directory to start reading

// ##### Results
// - <a href="#fd_readdir.error" name="fd_readdir.error"></a> `error`: `Result<size, errno>`
// The number of bytes stored in the read buffer. If less than the size of the read buffer, the end of the directory has been reached.

// ###### Variant Layout
// - size: 8
// - align: 4
// - tag_size: 4
// ###### Variant cases
// - <a href="#fd_readdir.error.ok" name="fd_readdir.error.ok"></a> `ok`: [`size`](#size)

// - <a href="#fd_readdir.error.err" name="fd_readdir.error.err"></a> `err`: [`errno`](#errno)

// ---

// #### <a href="#fd_renumber" name="fd_renumber"></a> `fd_renumber(fd: fd, to: fd) -> Result<(), errno>`
// Atomically replace a file descriptor by renumbering another file descriptor.
// Due to the strong focus on thread safety, this environment does not provide
// a mechanism to duplicate or renumber a file descriptor to an arbitrary
// number, like `dup2()`. This would be prone to race conditions, as an actual
// file descriptor with the same number could be allocated by a different
// thread at the same time.
// This function provides a way to atomically renumber file descriptors, which
// would disappear if `dup2()` were to be removed entirely.

// ##### Params
// - <a href="#fd_renumber.fd" name="fd_renumber.fd"></a> `fd`: [`fd`](#fd)

// - <a href="#fd_renumber.to" name="fd_renumber.to"></a> `to`: [`fd`](#fd)
// The file descriptor to overwrite.

// ##### Results
// - <a href="#fd_renumber.error" name="fd_renumber.error"></a> `error`: `Result<(), errno>`

// ###### Variant Layout
// - size: 8
// - align: 4
// - tag_size: 4
// ###### Variant cases
// - <a href="#fd_renumber.error.ok" name="fd_renumber.error.ok"></a> `ok`

// - <a href="#fd_renumber.error.err" name="fd_renumber.error.err"></a> `err`: [`errno`](#errno)

// ---

// #### <a href="#path_filestat_set_times" name="path_filestat_set_times"></a> `path_filestat_set_times(fd: fd, flags: lookupflags, path: string, atim: timestamp, mtim: timestamp, fst_flags: fstflags) -> Result<(), errno>`
// Adjust the timestamps of a file or directory.
// Note: This is similar to `utimensat` in POSIX.

// ##### Params
// - <a href="#path_filestat_set_times.fd" name="path_filestat_set_times.fd"></a> `fd`: [`fd`](#fd)

// - <a href="#path_filestat_set_times.flags" name="path_filestat_set_times.flags"></a> `flags`: [`lookupflags`](#lookupflags)
// Flags determining the method of how the path is resolved.

// - <a href="#path_filestat_set_times.path" name="path_filestat_set_times.path"></a> `path`: `string`
// The path of the file or directory to operate on.

// - <a href="#path_filestat_set_times.atim" name="path_filestat_set_times.atim"></a> `atim`: [`timestamp`](#timestamp)
// The desired values of the data access timestamp.

// - <a href="#path_filestat_set_times.mtim" name="path_filestat_set_times.mtim"></a> `mtim`: [`timestamp`](#timestamp)
// The desired values of the data modification timestamp.

// - <a href="#path_filestat_set_times.fst_flags" name="path_filestat_set_times.fst_flags"></a> `fst_flags`: [`fstflags`](#fstflags)
// A bitmask indicating which timestamps to adjust.

// ##### Results
// - <a href="#path_filestat_set_times.error" name="path_filestat_set_times.error"></a> `error`: `Result<(), errno>`

// ###### Variant Layout
// - size: 8
// - align: 4
// - tag_size: 4
// ###### Variant cases
// - <a href="#path_filestat_set_times.error.ok" name="path_filestat_set_times.error.ok"></a> `ok`

// - <a href="#path_filestat_set_times.error.err" name="path_filestat_set_times.error.err"></a> `err`: [`errno`](#errno)

// ---

// #### <a href="#path_link" name="path_link"></a> `path_link(old_fd: fd, old_flags: lookupflags, old_path: string, new_fd: fd, new_path: string) -> Result<(), errno>`
// Create a hard link.
// Note: This is similar to `linkat` in POSIX.

// ##### Params
// - <a href="#path_link.old_fd" name="path_link.old_fd"></a> `old_fd`: [`fd`](#fd)

// - <a href="#path_link.old_flags" name="path_link.old_flags"></a> `old_flags`: [`lookupflags`](#lookupflags)
// Flags determining the method of how the path is resolved.

// - <a href="#path_link.old_path" name="path_link.old_path"></a> `old_path`: `string`
// The source path from which to link.

// - <a href="#path_link.new_fd" name="path_link.new_fd"></a> `new_fd`: [`fd`](#fd)
// The working directory at which the resolution of the new path starts.

// - <a href="#path_link.new_path" name="path_link.new_path"></a> `new_path`: `string`
// The destination path at which to create the hard link.

// ##### Results
// - <a href="#path_link.error" name="path_link.error"></a> `error`: `Result<(), errno>`

// ###### Variant Layout
// - size: 8
// - align: 4
// - tag_size: 4
// ###### Variant cases
// - <a href="#path_link.error.ok" name="path_link.error.ok"></a> `ok`

// - <a href="#path_link.error.err" name="path_link.error.err"></a> `err`: [`errno`](#errno)

// ---

// ---

// #### <a href="#path_readlink" name="path_readlink"></a> `path_readlink(fd: fd, path: string, buf: Pointer<u8>, buf_len: size) -> Result<size, errno>`
// Read the contents of a symbolic link.
// Note: This is similar to `readlinkat` in POSIX.

// ##### Params
// - <a href="#path_readlink.fd" name="path_readlink.fd"></a> `fd`: [`fd`](#fd)

// - <a href="#path_readlink.path" name="path_readlink.path"></a> `path`: `string`
// The path of the symbolic link from which to read.

// - <a href="#path_readlink.buf" name="path_readlink.buf"></a> `buf`: `Pointer<u8>`
// The buffer to which to write the contents of the symbolic link.

// - <a href="#path_readlink.buf_len" name="path_readlink.buf_len"></a> `buf_len`: [`size`](#size)

// ##### Results
// - <a href="#path_readlink.error" name="path_readlink.error"></a> `error`: `Result<size, errno>`
// The number of bytes placed in the buffer.

// ###### Variant Layout
// - size: 8
// - align: 4
// - tag_size: 4
// ###### Variant cases
// - <a href="#path_readlink.error.ok" name="path_readlink.error.ok"></a> `ok`: [`size`](#size)

// - <a href="#path_readlink.error.err" name="path_readlink.error.err"></a> `err`: [`errno`](#errno)

// ---

// #### <a href="#path_remove_directory" name="path_remove_directory"></a> `path_remove_directory(fd: fd, path: string) -> Result<(), errno>`
// Remove a directory.
// Return [`errno::notempty`](#errno.notempty) if the directory is not empty.
// Note: This is similar to `unlinkat(fd, path, AT_REMOVEDIR)` in POSIX.

// ##### Params
// - <a href="#path_remove_directory.fd" name="path_remove_directory.fd"></a> `fd`: [`fd`](#fd)

// - <a href="#path_remove_directory.path" name="path_remove_directory.path"></a> `path`: `string`
// The path to a directory to remove.

// ##### Results
// - <a href="#path_remove_directory.error" name="path_remove_directory.error"></a> `error`: `Result<(), errno>`

// ###### Variant Layout
// - size: 8
// - align: 4
// - tag_size: 4
// ###### Variant cases
// - <a href="#path_remove_directory.error.ok" name="path_remove_directory.error.ok"></a> `ok`

// - <a href="#path_remove_directory.error.err" name="path_remove_directory.error.err"></a> `err`: [`errno`](#errno)

// ---

// #### <a href="#path_rename" name="path_rename"></a> `path_rename(fd: fd, old_path: string, new_fd: fd, new_path: string) -> Result<(), errno>`
// Rename a file or directory.
// Note: This is similar to `renameat` in POSIX.

// ##### Params
// - <a href="#path_rename.fd" name="path_rename.fd"></a> `fd`: [`fd`](#fd)

// - <a href="#path_rename.old_path" name="path_rename.old_path"></a> `old_path`: `string`
// The source path of the file or directory to rename.

// - <a href="#path_rename.new_fd" name="path_rename.new_fd"></a> `new_fd`: [`fd`](#fd)
// The working directory at which the resolution of the new path starts.

// - <a href="#path_rename.new_path" name="path_rename.new_path"></a> `new_path`: `string`
// The destination path to which to rename the file or directory.

// ##### Results
// - <a href="#path_rename.error" name="path_rename.error"></a> `error`: `Result<(), errno>`

// ###### Variant Layout
// - size: 8
// - align: 4
// - tag_size: 4
// ###### Variant cases
// - <a href="#path_rename.error.ok" name="path_rename.error.ok"></a> `ok`

// - <a href="#path_rename.error.err" name="path_rename.error.err"></a> `err`: [`errno`](#errno)

// ---

// #### <a href="#path_symlink" name="path_symlink"></a> `path_symlink(old_path: string, fd: fd, new_path: string) -> Result<(), errno>`
// Create a symbolic link.
// Note: This is similar to `symlinkat` in POSIX.

// ##### Params
// - <a href="#path_symlink.old_path" name="path_symlink.old_path"></a> `old_path`: `string`
// The contents of the symbolic link.

// - <a href="#path_symlink.fd" name="path_symlink.fd"></a> `fd`: [`fd`](#fd)

// - <a href="#path_symlink.new_path" name="path_symlink.new_path"></a> `new_path`: `string`
// The destination path at which to create the symbolic link.

// ##### Results
// - <a href="#path_symlink.error" name="path_symlink.error"></a> `error`: `Result<(), errno>`

// ###### Variant Layout
// - size: 8
// - align: 4
// - tag_size: 4
// ###### Variant cases
// - <a href="#path_symlink.error.ok" name="path_symlink.error.ok"></a> `ok`

// - <a href="#path_symlink.error.err" name="path_symlink.error.err"></a> `err`: [`errno`](#errno)

// ---

// #### <a href="#path_unlink_file" name="path_unlink_file"></a> `path_unlink_file(fd: fd, path: string) -> Result<(), errno>`
// Unlink a file.
// Return [`errno::isdir`](#errno.isdir) if the path refers to a directory.
// Note: This is similar to `unlinkat(fd, path, 0)` in POSIX.

// ##### Params
// - <a href="#path_unlink_file.fd" name="path_unlink_file.fd"></a> `fd`: [`fd`](#fd)

// - <a href="#path_unlink_file.path" name="path_unlink_file.path"></a> `path`: `string`
// The path to a file to unlink.

// ##### Results
// - <a href="#path_unlink_file.error" name="path_unlink_file.error"></a> `error`: `Result<(), errno>`

// ###### Variant Layout
// - size: 8
// - align: 4
// - tag_size: 4
// ###### Variant cases
// - <a href="#path_unlink_file.error.ok" name="path_unlink_file.error.ok"></a> `ok`

// - <a href="#path_unlink_file.error.err" name="path_unlink_file.error.err"></a> `err`: [`errno`](#errno)

// ---

// #### <a href="#poll_oneoff" name="poll_oneoff"></a> `poll_oneoff(in: ConstPointer<subscription>, out: Pointer<event>, nsubscriptions: size) -> Result<size, errno>`
// Concurrently poll for the occurrence of a set of events.

// If `nsubscriptions` is 0, returns [`errno::inval`](#errno.inval).

// ##### Params
// - <a href="#poll_oneoff.in" name="poll_oneoff.in"></a> `in`: `ConstPointer<subscription>`
// The events to which to subscribe.

// - <a href="#poll_oneoff.out" name="poll_oneoff.out"></a> `out`: `Pointer<event>`
// The events that have occurred.

// - <a href="#poll_oneoff.nsubscriptions" name="poll_oneoff.nsubscriptions"></a> `nsubscriptions`: [`size`](#size)
// Both the number of subscriptions and events.

// ##### Results
// - <a href="#poll_oneoff.error" name="poll_oneoff.error"></a> `error`: `Result<size, errno>`
// The number of events stored.

// ###### Variant Layout
// - size: 8
// - align: 4
// - tag_size: 4
// ###### Variant cases
// - <a href="#poll_oneoff.error.ok" name="poll_oneoff.error.ok"></a> `ok`: [`size`](#size)

// - <a href="#poll_oneoff.error.err" name="poll_oneoff.error.err"></a> `err`: [`errno`](#errno)

// ---

// ---

// #### <a href="#fd_sync" name="fd_sync"></a> `fd_sync(fd: fd) -> Result<(), errno>`
// Synchronize the data and metadata of a file to disk.
// Note: This is similar to `fsync` in POSIX.

// ##### Params
// - <a href="#fd_sync.fd" name="fd_sync.fd"></a> `fd`: [`fd`](#fd)

// ##### Results
// - <a href="#fd_sync.error" name="fd_sync.error"></a> `error`: `Result<(), errno>`

// ###### Variant Layout
// - size: 8
// - align: 4
// - tag_size: 4
// ###### Variant cases
// - <a href="#fd_sync.error.ok" name="fd_sync.error.ok"></a> `ok`

// - <a href="#fd_sync.error.err" name="fd_sync.error.err"></a> `err`: [`errno`](#errno)

// ---

// #### <a href="#fd_tell" name="fd_tell"></a> `fd_tell(fd: fd) -> Result<filesize, errno>`
// Return the current offset of a file descriptor.
// Note: This is similar to `lseek(fd, 0, SEEK_CUR)` in POSIX.

// ##### Params
// - <a href="#fd_tell.fd" name="fd_tell.fd"></a> `fd`: [`fd`](#fd)

// ##### Results
// - <a href="#fd_tell.error" name="fd_tell.error"></a> `error`: `Result<filesize, errno>`
// The current offset of the file descriptor, relative to the start of the file.

// ###### Variant Layout
// - size: 16
// - align: 8
// - tag_size: 4
// ###### Variant cases
// - <a href="#fd_tell.error.ok" name="fd_tell.error.ok"></a> `ok`: [`filesize`](#filesize)

// - <a href="#fd_tell.error.err" name="fd_tell.error.err"></a> `err`: [`errno`](#errno)

// ---

// ---

// #### <a href="#path_create_directory" name="path_create_directory"></a> `path_create_directory(fd: fd, path: string) -> Result<(), errno>`
// Create a directory.
// Note: This is similar to `mkdirat` in POSIX.

// ##### Params
// - <a href="#path_create_directory.fd" name="path_create_directory.fd"></a> `fd`: [`fd`](#fd)

// - <a href="#path_create_directory.path" name="path_create_directory.path"></a> `path`: `string`
// The path at which to create the directory.

// ##### Results
// - <a href="#path_create_directory.error" name="path_create_directory.error"></a> `error`: `Result<(), errno>`

// ###### Variant Layout
// - size: 8
// - align: 4
// - tag_size: 4
// ###### Variant cases
// - <a href="#path_create_directory.error.ok" name="path_create_directory.error.ok"></a> `ok`

// - <a href="#path_create_directory.error.err" name="path_create_directory.error.err"></a> `err`: [`errno`](#errno)
