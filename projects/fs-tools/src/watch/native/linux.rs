//! Linux backend built on `inotify`.
//!
//! inotify watches one directory at a time, so a subscription owns one inotify instance per root
//! and registers every directory below it. The worker drains those instances without blocking,
//! which makes cancellation a single `close`: no reader thread, buffer, or watch can outlive a
//! subscription.

use std::{
    collections::{BTreeMap, HashMap},
    ffi::{CString, OsStr, c_char, c_void},
    fs, io,
    os::{fd::RawFd, unix::ffi::OsStrExt},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, RecvTimeoutError},
    },
    time::Duration,
};

use super::super::{
    RescanReason, WatchError, WatchEvent,
    emitter::Emitter,
    scan,
    worker::{Command, DeliveryState, RootId, lock},
};

/// Bytes read from one inotify instance per `read` call.
const BUFFER_SIZE: usize = 64 * 1024;

/// How often the worker drains the instances when no command arrives.
const HOUSEKEEPING: Duration = Duration::from_millis(50);

/// Bytes in the fixed part of `inotify_event`, before its variable-length name.
const EVENT_HEADER: usize = 16;

/// `inotify_init1` flags; they share their values with `O_NONBLOCK` and `O_CLOEXEC`.
const IN_NONBLOCK: i32 = 0x800;
const IN_CLOEXEC: i32 = 0x8_0000;

const IN_MODIFY: u32 = 0x0000_0002;
const IN_ATTRIB: u32 = 0x0000_0004;
const IN_MOVED_FROM: u32 = 0x0000_0040;
const IN_MOVED_TO: u32 = 0x0000_0080;
const IN_CREATE: u32 = 0x0000_0100;
const IN_DELETE: u32 = 0x0000_0200;
const IN_DELETE_SELF: u32 = 0x0000_0400;
const IN_MOVE_SELF: u32 = 0x0000_0800;
const IN_UNMOUNT: u32 = 0x0000_2000;
const IN_Q_OVERFLOW: u32 = 0x0000_4000;
const IN_IGNORED: u32 = 0x0000_8000;
const IN_ONLYDIR: u32 = 0x0100_0000;
const IN_DONT_FOLLOW: u32 = 0x0200_0000;
const IN_ISDIR: u32 = 0x4000_0000;

/// Access, open, and close events are excluded on purpose: reads are not changes.
const WATCH_MASK: u32 =
    IN_CREATE | IN_DELETE | IN_MODIFY | IN_ATTRIB | IN_MOVED_FROM | IN_MOVED_TO | IN_DELETE_SELF | IN_MOVE_SELF | IN_ONLYDIR | IN_DONT_FOLLOW;

const ENOENT: i32 = 2;
const EINTR: i32 = 4;
const EAGAIN: i32 = 11;
const ENOMEM: i32 = 12;
const EACCES: i32 = 13;
const ENOTDIR: i32 = 20;
const ENFILE: i32 = 23;
const EMFILE: i32 = 24;
const ENOSPC: i32 = 28;

#[link(name = "c")]
unsafe extern "C" {
    fn inotify_init1(flags: i32) -> RawFd;
    fn inotify_add_watch(fd: RawFd, path: *const c_char, mask: u32) -> i32;
    fn inotify_rm_watch(fd: RawFd, wd: i32) -> i32;
    fn read(fd: RawFd, buffer: *mut c_void, length: usize) -> isize;
    fn close(fd: RawFd) -> i32;
}

/// One live subscription.
struct LinuxRoot {
    root: PathBuf,
    instance: RawFd,
    /// Maps every watch descriptor back to the directory it covers.
    directories: HashMap<i32, PathBuf>,
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

        backend.drain();
        backend.emitter.flush();

        // A fatal event is already queued; keeping the roots active lets the delivery thread
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
    roots: BTreeMap<RootId, LinuxRoot>,
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
        let instance = open_instance()?;

        let id = self.next_id;
        self.next_id += 1;
        lock(&self.shared).active.insert(id);
        self.emitter.register(id, root.clone());
        self.by_path.insert(root.clone(), id);

        // Registering is the subscription: once the root watch exists, later changes queue in the
        // kernel, so nothing is lost between the acknowledgement and the first read.
        let mut directories = HashMap::new();
        let registered =
            add_watch(instance, &root, &mut directories).and_then(|()| register_tree(&self.emitter, id, instance, &root, &mut directories));
        if let Err(error) = registered {
            self.rollback(id, &root, instance);
            return Err(error);
        }

        self.roots.insert(id, LinuxRoot { root, instance, directories });
        Ok(())
    }

    fn rollback(&mut self, id: RootId, root: &Path, instance: RawFd) {
        self.by_path.remove(root);
        self.emitter.unregister(id);
        lock(&self.shared).active.remove(&id);
        release(instance);
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
        release(root.instance);
    }

    fn drain(&mut self) {
        let ids: Vec<RootId> = self.roots.keys().copied().collect();
        for id in ids {
            if self.fatal {
                return;
            }
            self.drain_root(id);
        }
    }

    fn drain_root(&mut self, id: RootId) {
        let mut buffer = vec![0u8; BUFFER_SIZE];
        let mut moves: HashMap<u32, (PathBuf, bool)> = HashMap::new();

        loop {
            let Some(root) = self.roots.get(&id)
            else {
                return;
            };
            let length = unsafe { read(root.instance, buffer.as_mut_ptr().cast(), BUFFER_SIZE) };
            if length < 0 {
                let error = io::Error::last_os_error();
                match error.raw_os_error() {
                    Some(EINTR) => continue,
                    Some(EAGAIN) => break,
                    _ => {
                        self.emitter.send(id, WatchEvent::Error(WatchError::Fatal(error)));
                        self.fatal = true;
                        return;
                    }
                }
            }
            if length == 0 {
                break;
            }
            self.dispatch(id, &buffer[..length as usize], &mut moves);
            if !self.roots.contains_key(&id) {
                return;
            }
        }

        // A move whose other side never arrived: the destination is outside every subscription.
        let mut unmatched: Vec<(PathBuf, bool)> = moves.into_values().collect();
        unmatched.sort();
        for (from, is_dir) in unmatched {
            if is_dir {
                self.forget_subtree(id, &from);
            }
            self.emitter.send(id, WatchEvent::Removed(from));
            self.emitter.mark(id, RescanReason::UnpairedRename);
        }
    }

    /// Split one `read` result into `inotify_event` records.
    fn dispatch(&mut self, id: RootId, bytes: &[u8], moves: &mut HashMap<u32, (PathBuf, bool)>) {
        let mut offset = 0;
        while offset + EVENT_HEADER <= bytes.len() {
            let wd = i32::from_ne_bytes([bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]]);
            let mask = read_u32(bytes, offset + 4);
            let cookie = read_u32(bytes, offset + 8);
            let length = read_u32(bytes, offset + 12) as usize;
            if offset + EVENT_HEADER + length > bytes.len() {
                break;
            }
            let name = &bytes[offset + EVENT_HEADER..offset + EVENT_HEADER + length];
            let name = match name.iter().position(|byte| *byte == 0) {
                Some(end) => &name[..end],
                None => name,
            };
            offset += EVENT_HEADER + length;

            self.record(id, wd, mask, cookie, OsStr::from_bytes(name), moves);
        }
    }

    fn record(&mut self, id: RootId, wd: i32, mask: u32, cookie: u32, name: &OsStr, moves: &mut HashMap<u32, (PathBuf, bool)>) {
        if mask & IN_Q_OVERFLOW != 0 {
            // The kernel queue dropped events for this root, not for every subscription.
            self.emitter.mark(id, RescanReason::Overflow);
            return;
        }

        let Some(directory) = self.directory_of(id, wd)
        else {
            return;
        };
        let path = if name.is_empty() { directory } else { directory.join(name) };

        if mask & IN_IGNORED != 0 {
            if let Some(root) = self.roots.get_mut(&id) {
                root.directories.remove(&wd);
            }
            return;
        }
        if mask & (IN_DELETE_SELF | IN_MOVE_SELF | IN_UNMOUNT) != 0 {
            // Only the subscription root ends here; a subdirectory is still covered by its parent.
            if self.is_root(id, &path) {
                self.emitter.send(id, WatchEvent::Removed(path));
                self.stop(id, false);
            }
            return;
        }

        let is_dir = mask & IN_ISDIR != 0;
        if mask & IN_MOVED_FROM != 0 {
            moves.insert(cookie, (path, is_dir));
            return;
        }
        if mask & IN_MOVED_TO != 0 {
            match moves.remove(&cookie) {
                Some((from, from_is_dir)) => {
                    self.emitter.send(id, WatchEvent::Renamed { from: from.clone(), to: path.clone() });
                    if from_is_dir {
                        self.forget_subtree(id, &from);
                    }
                    if is_dir {
                        self.follow(id, &path);
                    }
                }
                None => self.created(id, &path, is_dir),
            }
            return;
        }
        if mask & IN_CREATE != 0 {
            self.created(id, &path, is_dir);
            return;
        }
        if mask & IN_DELETE != 0 {
            self.emitter.send(id, WatchEvent::Removed(path));
            return;
        }
        if mask & (IN_MODIFY | IN_ATTRIB) != 0 {
            self.emitter.send(id, WatchEvent::Modified(path));
        }
    }

    fn created(&mut self, id: RootId, path: &Path, is_dir: bool) {
        self.emitter.send(id, WatchEvent::Created(path.to_path_buf()));
        if is_dir {
            self.follow(id, path);
        }
    }

    /// Register `path` and its subdirectories, then cover them with a rescan.
    ///
    /// The registration may race contents created between the directory appearing and its watch
    /// becoming effective, so the subtree is always marked as uncertain.
    fn follow(&mut self, id: RootId, path: &Path) {
        let Some(root) = self.roots.get_mut(&id)
        else {
            return;
        };
        let instance = root.instance;
        if let Err(error) = add_watch(instance, path, &mut root.directories) {
            self.emitter.send(id, WatchEvent::Error(error));
        }
        let _ = register_tree(&self.emitter, id, instance, path, &mut root.directories);
        self.emitter.mark(id, RescanReason::NewDirectory);
    }

    fn forget_subtree(&mut self, id: RootId, prefix: &Path) {
        let Some(root) = self.roots.get_mut(&id)
        else {
            return;
        };
        let stale: Vec<i32> = root.directories.iter().filter(|(_, path)| path.starts_with(prefix)).map(|(wd, _)| *wd).collect();
        for wd in stale {
            unsafe { inotify_rm_watch(root.instance, wd) };
            root.directories.remove(&wd);
        }
    }

    fn directory_of(&self, id: RootId, wd: i32) -> Option<PathBuf> {
        self.roots.get(&id)?.directories.get(&wd).cloned()
    }

    fn is_root(&self, id: RootId, path: &Path) -> bool {
        self.roots.get(&id).is_some_and(|root| root.root == path)
    }
}

/// Register every subdirectory of `directory`.
///
/// Returns [`WatchError::LimitReached`] so a subscription is never left partially registered;
/// individual failures are reported as non-fatal events and covered by a rescan.
fn register_tree(
    emitter: &Emitter,
    id: RootId,
    instance: RawFd,
    directory: &Path,
    directories: &mut HashMap<i32, PathBuf>,
) -> Result<(), WatchError> {
    let mut folders = vec![directory.to_path_buf()];
    while let Some(folder) = folders.pop() {
        let entries = match fs::read_dir(&folder) {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let Ok(file_type) = entry.file_type()
            else {
                continue;
            };
            if !file_type.is_dir() {
                continue;
            }
            let path = entry.path();
            match add_watch(instance, &path, directories) {
                Ok(()) => folders.push(path),
                Err(WatchError::LimitReached) => return Err(WatchError::LimitReached),
                Err(error) => {
                    emitter.send(id, WatchEvent::Error(error));
                    emitter.mark(id, RescanReason::NewDirectory);
                }
            }
        }
    }
    Ok(())
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_ne_bytes([bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]])
}

fn open_instance() -> Result<RawFd, WatchError> {
    let instance = unsafe { inotify_init1(IN_NONBLOCK | IN_CLOEXEC) };
    if instance < 0 {
        let error = io::Error::last_os_error();
        return Err(match error.raw_os_error() {
            Some(ENFILE | EMFILE | ENOMEM) => WatchError::LimitReached,
            _ => WatchError::Backend(error),
        });
    }
    Ok(instance)
}

fn add_watch(instance: RawFd, directory: &Path, directories: &mut HashMap<i32, PathBuf>) -> Result<(), WatchError> {
    let name = CString::new(directory.as_os_str().as_bytes())
        .map_err(|_| WatchError::Backend(io::Error::new(io::ErrorKind::InvalidInput, "watch path contains a NUL byte")))?;
    let wd = unsafe { inotify_add_watch(instance, name.as_ptr(), WATCH_MASK) };
    if wd < 0 {
        return Err(watch_error(directory, io::Error::last_os_error()));
    }
    directories.insert(wd, directory.to_path_buf());
    Ok(())
}

/// Map a failure that prevented a directory watch from becoming effective.
fn watch_error(directory: &Path, error: io::Error) -> WatchError {
    match error.raw_os_error() {
        Some(ENOENT) => WatchError::NotFound(directory.to_path_buf()),
        Some(ENOTDIR) => WatchError::NotDirectory(directory.to_path_buf()),
        Some(EACCES) => WatchError::PermissionDenied(directory.to_path_buf()),
        Some(ENOSPC | ENFILE | EMFILE) => WatchError::LimitReached,
        _ => WatchError::Backend(error),
    }
}

fn release(instance: RawFd) {
    if instance >= 0 {
        unsafe { close(instance) };
    }
}
