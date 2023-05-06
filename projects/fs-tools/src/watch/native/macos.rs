//! macOS backend built on FSEvents.
//!
//! FSEvents already covers a directory recursively, so a subscription only needs one stream. The
//! callback runs on a reader thread's run loop; stopping sets a flag and wakes that loop, and the
//! reader releases the stream before the worker joins it. Paths arrive as C strings so non-UTF-8
//! names survive.

use std::{
    collections::{BTreeMap, HashMap},
    ffi::{CStr, CString, OsStr, c_char, c_void},
    io,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
    ptr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, RecvTimeoutError, Sender},
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

/// How often the worker flushes pending rescans and reaps finished readers.
const HOUSEKEEPING: Duration = Duration::from_millis(50);

/// How often the reader checks whether the subscription was stopped.
const RUN_LOOP_POLL: f64 = 0.2;

/// Coalescing window handed to FSEvents.
const LATENCY: f64 = 0.05;

/// `kFSEventStreamEventIdSinceNow`: only report changes made after the stream starts.
const SINCE_NOW: u64 = 0;

const CREATE_FILE_EVENTS: u32 = 0x10;
const CREATE_NO_DEFER: u32 = 0x02;
const CREATE_WATCH_ROOT: u32 = 0x04;

/// Events the kernel coalesced or dropped: the stream is no longer complete.
const MUST_SCAN_SUB_DIRS: u32 = 0x0000_0001;
const USER_DROPPED: u32 = 0x0000_0002;
const KERNEL_DROPPED: u32 = 0x0000_0004;
const EVENT_IDS_WRAPPED: u32 = 0x0000_0008;
const ROOT_CHANGED: u32 = 0x0000_0020;
const UNMOUNT: u32 = 0x0000_0080;
const ITEM_CREATED: u32 = 0x0000_0100;
const ITEM_REMOVED: u32 = 0x0000_0200;
const ITEM_INODE_META_MOD: u32 = 0x0000_0400;
const ITEM_RENAMED: u32 = 0x0000_0800;
const ITEM_MODIFIED: u32 = 0x0000_1000;
const ITEM_FINDER_INFO_MOD: u32 = 0x0000_2000;
const ITEM_CHANGE_OWNER: u32 = 0x0000_4000;
const ITEM_XATTR_MOD: u32 = 0x0000_8000;

/// Metadata-only changes still count as [`WatchEvent::Modified`].
const ITEM_METADATA: u32 = ITEM_INODE_META_MOD | ITEM_FINDER_INFO_MOD | ITEM_CHANGE_OWNER | ITEM_XATTR_MOD;

type CFAllocatorRef = *const c_void;
type CFArrayRef = *const c_void;
type CFIndex = isize;
type CFStringRef = *const c_void;
type CFRunLoopRef = *mut c_void;
type FSEventStreamRef = *mut c_void;

#[repr(C)]
struct CFArrayCallBacks {
    version: CFIndex,
    retain: *const c_void,
    release: *const c_void,
    copy_description: *const c_void,
    equal: *const c_void,
}

#[repr(C)]
struct FSEventStreamContext {
    version: CFIndex,
    info: *mut c_void,
    retain: *const c_void,
    release: *const c_void,
    copy_description: *const c_void,
}

type FSEventStreamCallback = unsafe extern "C" fn(FSEventStreamRef, *mut c_void, usize, *mut c_void, *const u32, *const u64);

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    static kCFTypeArrayCallBacks: CFArrayCallBacks;
    static kCFRunLoopDefaultMode: CFStringRef;

    fn CFStringCreateWithFileSystemRepresentation(allocator: CFAllocatorRef, buffer: *const c_char) -> CFStringRef;
    fn CFArrayCreate(allocator: CFAllocatorRef, values: *const *const c_void, count: CFIndex, callbacks: *const CFArrayCallBacks)
    -> CFArrayRef;
    fn CFRelease(object: *const c_void);
    fn CFRunLoopGetCurrent() -> CFRunLoopRef;
    fn CFRunLoopRunInMode(mode: CFStringRef, seconds: f64, return_after_source_handled: u8) -> i32;
    fn CFRunLoopStop(run_loop: CFRunLoopRef);
}

#[link(name = "CoreServices", kind = "framework")]
unsafe extern "C" {
    fn FSEventStreamCreate(
        allocator: CFAllocatorRef,
        callback: FSEventStreamCallback,
        context: *mut FSEventStreamContext,
        paths_to_watch: CFArrayRef,
        since_when: u64,
        latency: f64,
        flags: u32,
    ) -> FSEventStreamRef;
    fn FSEventStreamScheduleWithRunLoop(stream: FSEventStreamRef, run_loop: CFRunLoopRef, mode: CFStringRef);
    fn FSEventStreamStart(stream: FSEventStreamRef) -> u8;
    fn FSEventStreamStop(stream: FSEventStreamRef);
    fn FSEventStreamInvalidate(stream: FSEventStreamRef);
    fn FSEventStreamRelease(stream: FSEventStreamRef);
}

/// A run loop address owned by a reader thread.
///
/// It is a kernel-side object address, not a pointer into this thread's stack, so storing it in the
/// worker is sound; only the reader outlives it.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
struct Handle(CFRunLoopRef);

// SAFETY: the wrapped value is a CoreFoundation run loop that stays alive until the reader thread
// exits, and the worker only passes it back to `CFRunLoopStop`.
unsafe impl Send for Handle {}
unsafe impl Sync for Handle {}

/// One live subscription.
struct MacRoot {
    root: PathBuf,
    stop: Arc<AtomicBool>,
    run_loop: Handle,
    reader: Option<JoinHandle<()>>,
}

/// Run the native backend until shutdown or until the watcher goes away.
pub(super) fn run(commands: Receiver<Command>, emitter: Arc<Emitter>, shared: Arc<Mutex<DeliveryState>>) {
    let mut backend = Backend { emitter, shared, fatal: false, roots: BTreeMap::new(), by_path: HashMap::new(), next_id: 0 };

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
        let fatal = backend.fatal;
        if shutdown || fatal || backend.emitter.is_disconnected() {
            backend.close_all(!fatal);
            break;
        }
    }
}

struct Backend {
    emitter: Arc<Emitter>,
    shared: Arc<Mutex<DeliveryState>>,
    fatal: bool,
    roots: BTreeMap<RootId, MacRoot>,
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
                self.close_all(true);
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

        // The reader may deliver as soon as it starts, so bookkeeping happens first.
        let id = self.next_id;
        self.next_id += 1;
        lock(&self.shared).active.insert(id);
        self.emitter.register(id, root.clone());
        self.by_path.insert(root.clone(), id);

        let stop = Arc::new(AtomicBool::new(false));
        let (ready, started) = std::sync::mpsc::channel();
        let reader = Reader { id, root: root.clone(), emitter: Arc::clone(&self.emitter), stop: Arc::clone(&stop) };
        let reader = match thread::Builder::new().name(format!("fs-tools-watch-{id}")).spawn(move || reader.run(ready)) {
            Ok(reader) => reader,
            Err(error) => {
                self.rollback(id, &root);
                return Err(WatchError::Backend(error));
            }
        };

        // `FSEventStreamStart` already returned, so no change can slip between the acknowledgement
        // and the stream becoming effective.
        match started.recv() {
            Ok(Ok(run_loop)) => {
                self.roots.insert(id, MacRoot { root, stop, run_loop, reader: Some(reader) });
                Ok(())
            }
            Ok(Err(error)) => {
                let _ = reader.join();
                self.rollback(id, &root);
                Err(error)
            }
            Err(_) => {
                let _ = reader.join();
                self.rollback(id, &root);
                Err(WatchError::Backend(io::Error::new(io::ErrorKind::BrokenPipe, "the watcher reader did not start")))
            }
        }
    }

    fn rollback(&mut self, id: RootId, root: &Path) {
        self.by_path.remove(root);
        self.emitter.unregister(id);
        lock(&self.shared).active.remove(&id);
    }

    fn unwatch(&mut self, path: &Path) -> Result<(), WatchError> {
        let key = scan::subscription_key(path);
        let id = self.by_path.remove(&key).ok_or_else(|| WatchError::NotWatched(path.to_path_buf()))?;
        self.stop(id, true);
        Ok(())
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

    /// Stop readers that terminated on their own and release their streams.
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

        // The flag covers a stop that arrives before the reader enters its run loop; the call wakes
        // a loop that is already waiting.
        root.stop.store(true, Ordering::Relaxed);
        unsafe { CFRunLoopStop(root.run_loop.0) };
        if let Some(reader) = root.reader {
            let _ = reader.join();
        }
    }
}

/// Per-root reader that owns the stream and its run loop.
struct Reader {
    id: RootId,
    root: PathBuf,
    emitter: Arc<Emitter>,
    stop: Arc<AtomicBool>,
}

impl Reader {
    fn run(self, ready: Sender<Result<Handle, WatchError>>) {
        let context =
            Box::new(Context { id: self.id, root: self.root.clone(), emitter: Arc::clone(&self.emitter), stop: Arc::clone(&self.stop) });

        let Some(stream) = (unsafe { create_stream(&self.root, &*context as *const Context as *mut Context) })
        else {
            let _ = ready.send(Err(WatchError::Backend(io::Error::other("the native event stream could not be created"))));
            return;
        };

        let run_loop = unsafe { CFRunLoopGetCurrent() };
        unsafe { FSEventStreamScheduleWithRunLoop(stream, run_loop, kCFRunLoopDefaultMode) };
        if unsafe { FSEventStreamStart(stream) } == 0 {
            unsafe {
                FSEventStreamInvalidate(stream);
                FSEventStreamRelease(stream);
            }
            let _ = ready.send(Err(WatchError::Backend(io::Error::other("the native event stream could not start"))));
            return;
        }

        if ready.send(Ok(Handle(run_loop))).is_err() {
            // The worker is gone; release the stream without entering the loop.
            unsafe {
                FSEventStreamStop(stream);
                FSEventStreamInvalidate(stream);
                FSEventStreamRelease(stream);
            }
            return;
        }

        // A plain `CFRunLoopRun` would hang when the stop arrives before the loop starts.
        while !self.stop.load(Ordering::Relaxed) {
            unsafe { CFRunLoopRunInMode(kCFRunLoopDefaultMode, RUN_LOOP_POLL, 0) };
        }

        unsafe {
            FSEventStreamStop(stream);
            FSEventStreamInvalidate(stream);
            FSEventStreamRelease(stream);
        }
    }
}

/// State handed to the FSEvents callback.
struct Context {
    id: RootId,
    root: PathBuf,
    emitter: Arc<Emitter>,
    stop: Arc<AtomicBool>,
}

/// Build one stream that reports file-level events for `root` and its subdirectories.
unsafe fn create_stream(root: &Path, info: *mut Context) -> Option<FSEventStreamRef> {
    let name = CString::new(root.as_os_str().as_bytes()).ok()?;
    let path = unsafe { CFStringCreateWithFileSystemRepresentation(ptr::null(), name.as_ptr()) };
    if path.is_null() {
        return None;
    }

    let values = [path];
    let paths = unsafe { CFArrayCreate(ptr::null(), values.as_ptr(), 1, &raw const kCFTypeArrayCallBacks) };
    if paths.is_null() {
        unsafe { CFRelease(path) };
        return None;
    }

    let mut context =
        FSEventStreamContext { version: 0, info: info.cast(), retain: ptr::null(), release: ptr::null(), copy_description: ptr::null() };
    let stream = unsafe {
        FSEventStreamCreate(
            ptr::null(),
            callback,
            &mut context,
            paths,
            SINCE_NOW,
            LATENCY,
            CREATE_FILE_EVENTS | CREATE_NO_DEFER | CREATE_WATCH_ROOT,
        )
    };

    // The stream copies the paths it needs, so the CoreFoundation wrappers can go.
    unsafe {
        CFRelease(paths);
        CFRelease(path);
    }
    if stream.is_null() { None } else { Some(stream) }
}

/// Turn one FSEvents callback into watcher events.
unsafe extern "C" fn callback(
    _stream: FSEventStreamRef,
    info: *mut c_void,
    count: usize,
    paths: *mut c_void,
    flags: *const u32,
    _ids: *const u64,
) {
    if info.is_null() || paths.is_null() || flags.is_null() {
        return;
    }
    let context = unsafe { &*info.cast::<Context>() };
    let paths = paths.cast::<*const c_char>();
    let mut overflow = false;

    for index in 0..count {
        let flag = unsafe { *flags.add(index) };

        if flag & (MUST_SCAN_SUB_DIRS | USER_DROPPED | KERNEL_DROPPED | EVENT_IDS_WRAPPED) != 0 {
            overflow = true;
            continue;
        }
        if flag & (ROOT_CHANGED | UNMOUNT) != 0 {
            // The root left the namespace; the subscription ends after this event.
            context.emitter.send(context.id, WatchEvent::Removed(context.root.clone()));
            context.stop.store(true, Ordering::Relaxed);
            unsafe { CFRunLoopStop(CFRunLoopGetCurrent()) };
            continue;
        }
        let Some(path) = (unsafe { path_at(paths, index) })
        else {
            continue;
        };

        if flag & ITEM_RENAMED != 0 {
            // FSEvents reports only one side of a rename, so the pair is never guessed.
            context.emitter.mark(context.id, RescanReason::UnpairedRename);
        }
        // FSEvents coalesces every change to one path inside the latency window into a single
        // record, so one record can carry several actions. Dropping all but the first would lose
        // the modification that followed a creation.
        if flag & ITEM_REMOVED != 0 {
            context.emitter.send(context.id, WatchEvent::Removed(path.clone()));
        }
        if flag & ITEM_CREATED != 0 {
            context.emitter.send(context.id, WatchEvent::Created(path.clone()));
        }
        if flag & (ITEM_MODIFIED | ITEM_METADATA) != 0 {
            context.emitter.send(context.id, WatchEvent::Modified(path));
        }
    }

    if overflow {
        context.emitter.mark(context.id, RescanReason::Overflow);
    }
}

unsafe fn path_at(paths: *const *const c_char, index: usize) -> Option<PathBuf> {
    let pointer = unsafe { *paths.add(index) };
    if pointer.is_null() {
        return None;
    }
    Some(PathBuf::from(OsStr::from_bytes(unsafe { CStr::from_ptr(pointer) }.to_bytes())))
}
