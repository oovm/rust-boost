//! Windows backend built on `ReadDirectoryChangesW`.
//!
//! Every subscription keeps its directory handle open for the whole subscription and reads
//! changes with overlapped I/O on a reader thread. The worker thread owns the handles, wakes a
//! reader with a per-root stop event, joins it, and only then closes anything. No event can be
//! delivered from a freed buffer.

use std::{
    collections::{BTreeMap, HashMap},
    ffi::{OsString, c_void},
    io,
    mem::size_of,
    os::windows::ffi::{OsStrExt, OsStringExt},
    path::{Path, PathBuf},
    ptr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, RecvTimeoutError},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use super::super::{
    RescanReason, WatchError, WatchEvent,
    emitter::Emitter,
    scan,
    worker::{Command, DeliveryState, RootId, lock},
};

/// Buffer handed to one `ReadDirectoryChangesW` call.
const BUFFER_SIZE: usize = 64 * 1024;

/// How often the worker flushes pending rescans and reaps finished readers.
const HOUSEKEEPING: Duration = Duration::from_millis(50);

/// Bytes before the variable-length name in `FILE_NOTIFY_INFORMATION`.
const RECORD_HEADER: usize = 12;

/// `BOOL` is an `i32` in the Win32 ABI.
const TRUE: i32 = 1;
const FALSE: i32 = 0;
const INFINITE: u32 = u32::MAX;

/// A kernel object handle.
///
/// The raw pointer is a kernel handle, not an address in this process. Moving one to the reader
/// thread is therefore sound, and the worker keeps sole ownership of closing it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct Handle(*mut c_void);

// SAFETY: the wrapped value is an opaque kernel handle that is valid in every thread of the
// process. Sharing happens through `Arc` fields that never mutate the handle.
unsafe impl Send for Handle {}
unsafe impl Sync for Handle {}

impl Handle {
    fn is_null(self) -> bool {
        self.0.is_null()
    }
}

const INVALID_HANDLE_VALUE: Handle = Handle(-1isize as *mut c_void);

const FILE_LIST_DIRECTORY: u32 = 0x0001;
const FILE_SHARE_READ: u32 = 0x0001;
const FILE_SHARE_WRITE: u32 = 0x0002;
const FILE_SHARE_DELETE: u32 = 0x0004;
const OPEN_EXISTING: u32 = 3;
const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
const FILE_FLAG_OVERLAPPED: u32 = 0x4000_0000;

const FILE_NOTIFY_CHANGE_FILE_NAME: u32 = 0x0000_0001;
const FILE_NOTIFY_CHANGE_DIR_NAME: u32 = 0x0000_0002;
const FILE_NOTIFY_CHANGE_ATTRIBUTES: u32 = 0x0000_0004;
const FILE_NOTIFY_CHANGE_SIZE: u32 = 0x0000_0008;
const FILE_NOTIFY_CHANGE_LAST_WRITE: u32 = 0x0000_0010;
const FILE_NOTIFY_CHANGE_CREATION: u32 = 0x0000_0040;
const FILE_NOTIFY_CHANGE_SECURITY: u32 = 0x0000_0100;

/// Access changes are excluded on purpose: reads, opens, and `mmap` are not events.
const NOTIFY_FILTER: u32 = FILE_NOTIFY_CHANGE_FILE_NAME
    | FILE_NOTIFY_CHANGE_DIR_NAME
    | FILE_NOTIFY_CHANGE_ATTRIBUTES
    | FILE_NOTIFY_CHANGE_SIZE
    | FILE_NOTIFY_CHANGE_LAST_WRITE
    | FILE_NOTIFY_CHANGE_CREATION
    | FILE_NOTIFY_CHANGE_SECURITY;

const FILE_ACTION_ADDED: u32 = 1;
const FILE_ACTION_REMOVED: u32 = 2;
const FILE_ACTION_MODIFIED: u32 = 3;
const FILE_ACTION_RENAMED_OLD_NAME: u32 = 4;
const FILE_ACTION_RENAMED_NEW_NAME: u32 = 5;

const ERROR_FILE_NOT_FOUND: i32 = 2;
const ERROR_PATH_NOT_FOUND: i32 = 3;
const ERROR_TOO_MANY_OPEN_FILES: i32 = 4;
const ERROR_ACCESS_DENIED: i32 = 5;
const ERROR_INVALID_HANDLE: i32 = 6;
const ERROR_NOT_ENOUGH_MEMORY: i32 = 8;
const ERROR_OPERATION_ABORTED: i32 = 995;
const ERROR_IO_PENDING: i32 = 997;
const ERROR_IO_INCOMPLETE: i32 = 996;
const ERROR_NOTIFY_ENUM_DIR: i32 = 1022;
const ERROR_NO_SYSTEM_RESOURCES: i32 = 1450;

const WAIT_OBJECT_0: u32 = 0;
const WAIT_FAILED: u32 = u32::MAX;

#[repr(C)]
struct Overlapped {
    internal: usize,
    internal_high: usize,
    offset: u32,
    offset_high: u32,
    event: Handle,
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn CreateFileW(
        file_name: *const u16,
        desired_access: u32,
        share_mode: u32,
        security_attributes: *mut c_void,
        creation_disposition: u32,
        flags_and_attributes: u32,
        template_file: Handle,
    ) -> Handle;
    fn ReadDirectoryChangesW(
        directory: Handle,
        buffer: *mut c_void,
        buffer_length: u32,
        watch_subtree: i32,
        notify_filter: u32,
        bytes_returned: *mut u32,
        overlapped: *mut Overlapped,
        completion_routine: *mut c_void,
    ) -> i32;
    fn GetOverlappedResult(file: Handle, overlapped: *mut Overlapped, transferred: *mut u32, wait: i32) -> i32;
    fn CancelIoEx(file: Handle, overlapped: *mut Overlapped) -> i32;
    fn CreateEventW(attributes: *mut c_void, manual_reset: i32, initial_state: i32, name: *const u16) -> Handle;
    fn SetEvent(event: Handle) -> i32;
    fn ResetEvent(event: Handle) -> i32;
    fn WaitForMultipleObjects(count: u32, handles: *const Handle, wait_all: i32, milliseconds: u32) -> u32;
    fn CloseHandle(object: Handle) -> i32;
}

/// One live subscription.
struct NativeRoot {
    root: PathBuf,
    directory: Handle,
    stop: Handle,
    change: Handle,
    reader: Option<JoinHandle<()>>,
}

/// Run the native backend until shutdown or until the watcher goes away.
pub(super) fn run(commands: Receiver<Command>, emitter: Arc<Emitter>, shared: Arc<Mutex<DeliveryState>>) {
    let mut backend =
        Backend { emitter, shared, fatal: Arc::new(AtomicBool::new(false)), roots: BTreeMap::new(), by_path: HashMap::new(), next_id: 0 };

    loop {
        let shutdown = match commands.recv_timeout(HOUSEKEEPING) {
            Ok(command) => backend.handle(command),
            Err(RecvTimeoutError::Timeout) => false,
            Err(RecvTimeoutError::Disconnected) => true,
        };

        backend.emitter.flush();
        backend.reap();

        // A fatal event is already queued; keeping the root active lets the delivery thread
        // deliver it before the backend releases everything.
        let fatal = backend.fatal.load(Ordering::Relaxed);
        if shutdown || fatal || backend.emitter.is_disconnected() {
            backend.close_all(!fatal);
            break;
        }
    }
}

struct Backend {
    emitter: Arc<Emitter>,
    shared: Arc<Mutex<DeliveryState>>,
    fatal: Arc<AtomicBool>,
    roots: BTreeMap<RootId, NativeRoot>,
    by_path: HashMap<PathBuf, RootId>,
    next_id: RootId,
}

impl Backend {
    /// Handle one command and report whether the backend must stop.
    fn handle(&mut self, command: Command) -> bool {
        match command {
            Command::Watch { path, ack } => {
                let result = self.watch(path);
                let _ = ack.send(result);
                false
            }
            Command::Unwatch { path, ack } => {
                let result = self.unwatch(&path);
                let _ = ack.send(result);
                false
            }
            Command::UnwatchAll { ack } => {
                self.unwatch_all();
                let _ = ack.send(Ok(()));
                false
            }
            Command::Shutdown { ack } => {
                let _ = ack.send(Ok(()));
                true
            }
        }
    }

    fn watch(&mut self, path: PathBuf) -> Result<(), WatchError> {
        let root = scan::canonical_root(&path)?;
        if self.by_path.contains_key(&root) {
            return Err(WatchError::AlreadyWatched(path));
        }

        let directory = open_directory(&root)?;
        let events = match EventPair::new() {
            Ok(events) => events,
            Err(error) => {
                close(directory);
                return Err(error);
            }
        };

        // The reader may deliver as soon as it starts, so bookkeeping happens first.
        let id = self.next_id;
        self.next_id += 1;
        lock(&self.shared).active.insert(id);
        self.emitter.register(id, root.clone());
        self.by_path.insert(root.clone(), id);

        let (ready, started) = std::sync::mpsc::channel();
        let reader = Reader {
            id,
            root: root.clone(),
            directory,
            stop: events.stop,
            change: events.change,
            emitter: Arc::clone(&self.emitter),
            fatal: Arc::clone(&self.fatal),
        };
        let reader = match thread::Builder::new().name(format!("fs-tools-watch-{id}")).spawn(move || reader.run(ready)) {
            Ok(reader) => reader,
            Err(error) => {
                self.rollback(id, &root, events, directory);
                return Err(WatchError::Backend(error));
            }
        };

        // The first read is already pending when this returns, so no change can slip between the
        // acknowledgement and the subscription becoming effective.
        match started.recv() {
            Ok(Ok(())) => {
                self.roots.insert(id, NativeRoot { root, directory, stop: events.stop, change: events.change, reader: Some(reader) });
                Ok(())
            }
            Ok(Err(error)) => {
                let _ = reader.join();
                self.rollback(id, &root, events, directory);
                Err(error)
            }
            Err(_) => {
                let _ = reader.join();
                self.rollback(id, &root, events, directory);
                Err(WatchError::Backend(io::Error::new(io::ErrorKind::BrokenPipe, "the watcher reader did not start")))
            }
        }
    }

    fn rollback(&mut self, id: RootId, root: &Path, events: EventPair, directory: Handle) {
        self.by_path.remove(root);
        self.emitter.unregister(id);
        lock(&self.shared).active.remove(&id);
        close(events.change);
        close(events.stop);
        close(directory);
    }

    fn unwatch(&mut self, path: &Path) -> Result<(), WatchError> {
        let key = scan::subscription_key(path);
        let id = self.by_path.remove(&key).ok_or_else(|| WatchError::NotWatched(path.to_path_buf()))?;
        self.stop(id, true);
        Ok(())
    }

    fn unwatch_all(&mut self) {
        let ids: Vec<RootId> = self.roots.keys().copied().collect();
        for id in ids {
            self.stop(id, true);
        }
        self.emitter.clear();
        lock(&self.shared).active.clear();
    }

    fn close_all(&mut self, discard: bool) {
        let ids: Vec<RootId> = self.roots.keys().copied().collect();
        for id in ids {
            self.stop(id, discard);
        }
        self.emitter.clear();
        if discard {
            lock(&self.shared).active.clear();
        }
    }

    /// Stop readers that terminated on their own and release their handles.
    ///
    /// The subscription stays in the delivery set so the terminal event already queued for it is
    /// still delivered; the backend produces nothing else for that root.
    fn reap(&mut self) {
        let finished: Vec<RootId> =
            self.roots.iter().filter(|(_, root)| root.reader.as_ref().is_none_or(JoinHandle::is_finished)).map(|(id, _)| *id).collect();
        for id in finished {
            self.stop(id, false);
        }
    }

    fn stop(&mut self, id: RootId, discard: bool) {
        let Some(root) = self.roots.remove(&id)
        else {
            return;
        };
        self.by_path.remove(&root.root);
        self.emitter.unregister(id);
        if discard {
            lock(&self.shared).active.remove(&id);
        }

        // The reader waits on this event, cancels a pending read, and returns before the join.
        unsafe { SetEvent(root.stop) };
        if let Some(reader) = root.reader {
            let _ = reader.join();
        }
        close(root.change);
        close(root.stop);
        close(root.directory);
    }
}

struct EventPair {
    stop: Handle,
    change: Handle,
}

impl EventPair {
    fn new() -> Result<Self, WatchError> {
        let stop = create_event()?;
        match create_event() {
            Ok(change) => Ok(Self { stop, change }),
            Err(error) => {
                close(stop);
                Err(error)
            }
        }
    }
}

/// Per-root reader.
struct Reader {
    id: RootId,
    root: PathBuf,
    directory: Handle,
    stop: Handle,
    change: Handle,
    emitter: Arc<Emitter>,
    fatal: Arc<AtomicBool>,
}

impl Reader {
    fn run(self, ready: std::sync::mpsc::Sender<Result<(), WatchError>>) {
        // `ReadDirectoryChangesW` writes `FILE_NOTIFY_INFORMATION` records, so the buffer stays
        // 4-byte aligned instead of being a plain byte vector.
        let mut buffer = vec![0u32; BUFFER_SIZE / size_of::<u32>()];
        let mut first = true;

        loop {
            let mut overlapped = Overlapped { internal: 0, internal_high: 0, offset: 0, offset_high: 0, event: self.change };
            let issued = unsafe {
                ReadDirectoryChangesW(
                    self.directory,
                    buffer.as_mut_ptr().cast(),
                    BUFFER_SIZE as u32,
                    TRUE,
                    NOTIFY_FILTER,
                    ptr::null_mut(),
                    &mut overlapped,
                    ptr::null_mut(),
                )
            } != 0;

            if !issued {
                let error = io::Error::last_os_error();
                if error.raw_os_error() != Some(ERROR_IO_PENDING) {
                    if first {
                        let _ = ready.send(Err(startup_error(&self.root, &error)));
                        return;
                    }
                    if self.fail(error) {
                        return;
                    }
                    continue;
                }
            }

            // `watch` only returns once the first read is pending.
            if first {
                first = false;
                let _ = ready.send(Ok(()));
            }

            let mut transferred = 0;
            if unsafe { GetOverlappedResult(self.directory, &mut overlapped, &mut transferred, FALSE) } != 0 {
                unsafe { ResetEvent(self.change) };
                self.process(transferred, &buffer);
                continue;
            }

            let error = io::Error::last_os_error();
            if error.raw_os_error() != Some(ERROR_IO_INCOMPLETE) {
                if self.fail(error) {
                    return;
                }
                continue;
            }

            let handles = [self.stop, self.change];
            match unsafe { WaitForMultipleObjects(2, handles.as_ptr(), FALSE, INFINITE) } {
                WAIT_OBJECT_0 => {
                    // A stop wins over a simultaneous completion; the queued batch is discarded.
                    unsafe { CancelIoEx(self.directory, &mut overlapped) };
                    let mut transferred = 0;
                    unsafe { GetOverlappedResult(self.directory, &mut overlapped, &mut transferred, TRUE) };
                    return;
                }
                index if index == WAIT_OBJECT_0 + 1 => {
                    unsafe { ResetEvent(self.change) };
                    let mut transferred = 0;
                    if unsafe { GetOverlappedResult(self.directory, &mut overlapped, &mut transferred, FALSE) } == 0 {
                        let error = io::Error::last_os_error();
                        if self.fail(error) {
                            return;
                        }
                        continue;
                    }
                    self.process(transferred, &buffer);
                }
                WAIT_FAILED => {
                    let error = io::Error::last_os_error();
                    self.emitter.send(self.id, WatchEvent::Error(WatchError::Fatal(error)));
                    self.fatal.store(true, Ordering::Relaxed);
                    return;
                }
                _ => return,
            }
        }
    }

    /// Classify a failed read and report whether the subscription ends.
    fn fail(&self, error: io::Error) -> bool {
        match error.raw_os_error() {
            Some(ERROR_OPERATION_ABORTED | ERROR_INVALID_HANDLE) => true,
            Some(ERROR_NOTIFY_ENUM_DIR) => {
                self.emitter.mark(self.id, RescanReason::Overflow);
                false
            }
            _ => {
                if !self.root.is_dir() {
                    self.emitter.send(self.id, WatchEvent::Removed(self.root.clone()));
                }
                else {
                    self.emitter.send(self.id, WatchEvent::Error(WatchError::Fatal(error)));
                    self.fatal.store(true, Ordering::Relaxed);
                }
                true
            }
        }
    }

    /// Turn one completed batch into events.
    fn process(&self, transferred: u32, buffer: &[u32]) {
        if transferred == 0 {
            // The system discarded changes because its internal buffer overflowed.
            self.emitter.mark(self.id, RescanReason::Overflow);
            return;
        }
        let bytes = unsafe { std::slice::from_raw_parts(buffer.as_ptr().cast::<u8>(), transferred as usize) };
        self.dispatch(bytes);
    }

    fn dispatch(&self, bytes: &[u8]) {
        let mut records: Vec<(u32, PathBuf)> = Vec::new();
        let mut offset = 0;
        while offset + RECORD_HEADER <= bytes.len() {
            let next = read_u32(bytes, offset) as usize;
            let action = read_u32(bytes, offset + 4);
            let length = read_u32(bytes, offset + 8) as usize;
            if offset + RECORD_HEADER + length > bytes.len() {
                break;
            }
            let name = &bytes[offset + RECORD_HEADER..offset + RECORD_HEADER + length];
            let (units, _) = name.as_chunks::<2>();
            let wide: Vec<u16> = units.iter().map(|unit| u16::from_ne_bytes(*unit)).collect();
            records.push((action, self.root.join(OsString::from_wide(&wide))));
            if next == 0 {
                break;
            }
            offset += next;
        }

        let mut index = 0;
        while index < records.len() {
            let (action, path) = &records[index];
            match *action {
                FILE_ACTION_ADDED => self.created(path),
                FILE_ACTION_REMOVED => self.emitter.send(self.id, WatchEvent::Removed(path.clone())),
                FILE_ACTION_MODIFIED => self.emitter.send(self.id, WatchEvent::Modified(path.clone())),
                FILE_ACTION_RENAMED_OLD_NAME => {
                    // Both sides are only paired inside one batch; a split is never guessed.
                    if let Some((FILE_ACTION_RENAMED_NEW_NAME, to)) = records.get(index + 1) {
                        self.emitter.send(self.id, WatchEvent::Renamed { from: path.clone(), to: to.clone() });
                        index += 1;
                    }
                    else {
                        self.emitter.send(self.id, WatchEvent::Removed(path.clone()));
                        self.emitter.mark(self.id, RescanReason::UnpairedRename);
                    }
                }
                FILE_ACTION_RENAMED_NEW_NAME => {
                    self.created(path);
                    self.emitter.mark(self.id, RescanReason::UnpairedRename);
                }
                _ => {}
            }
            index += 1;
        }
    }

    fn created(&self, path: &Path) {
        self.emitter.send(self.id, WatchEvent::Created(path.to_path_buf()));
        if path.is_dir() {
            // The subtree was created before the recursive read covered it.
            self.emitter.mark(self.id, RescanReason::NewDirectory);
        }
    }
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_ne_bytes([bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]])
}

fn open_directory(root: &Path) -> Result<Handle, WatchError> {
    let mut wide: Vec<u16> = root.as_os_str().encode_wide().collect();
    wide.push(0);
    let handle = unsafe {
        CreateFileW(
            wide.as_ptr(),
            FILE_LIST_DIRECTORY,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            ptr::null_mut(),
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OVERLAPPED,
            Handle(ptr::null_mut()),
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        return Err(startup_error(root, &io::Error::last_os_error()));
    }
    Ok(handle)
}

/// Map a failure that prevented the subscription from becoming effective.
fn startup_error(root: &Path, error: &io::Error) -> WatchError {
    match error.raw_os_error() {
        Some(ERROR_FILE_NOT_FOUND | ERROR_PATH_NOT_FOUND) => WatchError::NotFound(root.to_path_buf()),
        Some(ERROR_ACCESS_DENIED) => WatchError::PermissionDenied(root.to_path_buf()),
        Some(ERROR_TOO_MANY_OPEN_FILES | ERROR_NO_SYSTEM_RESOURCES | ERROR_NOT_ENOUGH_MEMORY) => WatchError::LimitReached,
        Some(code) => WatchError::Backend(io::Error::from_raw_os_error(code)),
        None => WatchError::Backend(io::Error::new(error.kind(), error.to_string())),
    }
}

fn create_event() -> Result<Handle, WatchError> {
    let event = unsafe { CreateEventW(ptr::null_mut(), TRUE, FALSE, ptr::null()) };
    if event.is_null() || event == INVALID_HANDLE_VALUE {
        return Err(WatchError::Backend(io::Error::last_os_error()));
    }
    Ok(event)
}

fn close(handle: Handle) {
    if !handle.is_null() && handle != INVALID_HANDLE_VALUE {
        unsafe { CloseHandle(handle) };
    }
}
