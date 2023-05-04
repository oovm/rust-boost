#![cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]

use std::{
    fs,
    path::PathBuf,
    sync::mpsc::{self, Receiver},
    thread,
    time::{Duration, Instant},
};

use fs_tools::watch::{RescanReason, WatchBackend, WatchEvent, WatchOptions, Watcher};

const INTERVAL: Duration = Duration::from_millis(20);
const TIMEOUT: Duration = Duration::from_secs(10);

fn options(capacity: usize) -> WatchOptions {
    WatchOptions::new().queue_capacity(capacity).backend(WatchBackend::Polling { interval: INTERVAL })
}

#[cfg(windows)]
fn native_options(capacity: usize) -> WatchOptions {
    WatchOptions::new().queue_capacity(capacity).backend(WatchBackend::Native)
}

fn root_of(folder: &fs_tools::temp::TempFolder) -> PathBuf {
    normalize(fs::canonicalize(folder.path()).unwrap())
}

#[cfg(windows)]
fn normalize(path: PathBuf) -> PathBuf {
    use std::path::Prefix;

    let mut components = path.components();
    let Some(std::path::Component::Prefix(prefix)) = components.next()
    else {
        return path;
    };
    let rest: PathBuf = components.collect();
    match prefix.kind() {
        Prefix::VerbatimDisk(drive) => {
            let mut normalized = PathBuf::from(format!("{}:\\", drive as char));
            normalized.push(rest);
            normalized
        }
        _ => path,
    }
}

#[cfg(not(windows))]
fn normalize(path: PathBuf) -> PathBuf {
    path
}

fn wait_for(rx: &Receiver<WatchEvent>, predicate: impl Fn(&WatchEvent) -> bool) -> bool {
    let deadline = Instant::now() + TIMEOUT;
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return false;
        }
        match rx.recv_timeout(remaining) {
            Ok(event) if predicate(&event) => return true,
            Ok(_) => {}
            Err(_) => return false,
        }
    }
}

#[test]
fn polling_reports_created_modified_and_removed() {
    let folder = fs_tools::temp::create_folder().unwrap();
    let root = root_of(&folder);
    let (events, received) = mpsc::channel();
    let mut watcher = Watcher::new(move |event| drop(events.send(event)), options(64)).expect("polling watcher");
    watcher.watch(&root).unwrap();

    let file = root.join("a.txt");
    fs::write(&file, b"one").unwrap();
    assert!(wait_for(&received, |event| matches!(event, WatchEvent::Created(path) if path == &file)));

    fs::write(&file, b"longer").unwrap();
    assert!(wait_for(&received, |event| matches!(event, WatchEvent::Modified(path) if path == &file)));

    fs::remove_file(&file).unwrap();
    assert!(wait_for(&received, |event| matches!(event, WatchEvent::Removed(path) if path == &file)));

    watcher.close().unwrap();
}

#[test]
fn unwatch_discards_queued_events_for_the_root() {
    let folder = fs_tools::temp::create_folder().unwrap();
    let root = root_of(&folder);
    let (events, received) = mpsc::channel();
    let mut watcher = Watcher::new(move |event| drop(events.send(event)), options(64)).unwrap();
    watcher.watch(&root).unwrap();

    let before = root.join("before.txt");
    fs::write(&before, b"one").unwrap();
    assert!(wait_for(&received, |event| matches!(event, WatchEvent::Created(path) if path == &before)));

    watcher.unwatch(&root).unwrap();
    assert!(matches!(watcher.unwatch(&root), Err(fs_tools::watch::WatchError::NotWatched(_))));

    let after = root.join("after.txt");
    fs::write(&after, b"two").unwrap();
    thread::sleep(INTERVAL * 5);
    while let Ok(event) = received.recv_timeout(INTERVAL) {
        assert!(!matches!(&event, WatchEvent::Created(path) if path == &after), "an event for an unwatched root was delivered: {event:?}");
    }

    watcher.close().unwrap();
}

#[test]
fn unwatch_all_allows_the_root_to_be_watched_again() {
    let folder = fs_tools::temp::create_folder().unwrap();
    let root = root_of(&folder);
    let (events, _received) = mpsc::channel();
    let mut watcher = Watcher::new(move |event| drop(events.send(event)), options(64)).unwrap();

    watcher.watch(&root).unwrap();
    assert!(matches!(watcher.watch(&root), Err(fs_tools::watch::WatchError::AlreadyWatched(_))));
    assert!(matches!(watcher.watch(root.join("missing")), Err(fs_tools::watch::WatchError::NotFound(_))));

    let file = root.join("file.txt");
    fs::write(&file, b"one").unwrap();
    assert!(matches!(watcher.watch(&file), Err(fs_tools::watch::WatchError::NotDirectory(_))));

    watcher.unwatch_all().unwrap();
    watcher.unwatch_all().unwrap();
    watcher.watch(&root).unwrap();

    watcher.close().unwrap();
}

#[test]
fn overflow_is_reported_as_a_rescan() {
    let folder = fs_tools::temp::create_folder().unwrap();
    let root = root_of(&folder);
    let (events, received) = mpsc::channel();
    let (started, blocked) = mpsc::channel();
    let (release, resume) = mpsc::channel();
    let mut first = true;
    let mut watcher = Watcher::new(
        move |event| {
            if first {
                first = false;
                let _ = started.send(());
                let _ = resume.recv();
            }
            drop(events.send(event));
        },
        options(1),
    )
    .unwrap();
    watcher.watch(&root).unwrap();

    fs::write(root.join("first.txt"), b"one").unwrap();
    blocked.recv_timeout(TIMEOUT).expect("the callback starts");

    for index in 0..8 {
        fs::write(root.join(format!("burst-{index}.txt")), b"one").unwrap();
    }
    release.send(()).unwrap();

    assert!(wait_for(&received, |event| matches!(event, WatchEvent::Rescan { reason: RescanReason::Overflow, .. })));

    watcher.close().unwrap();
}

#[test]
fn new_folder_is_followed_and_rescanned() {
    let folder = fs_tools::temp::create_folder().unwrap();
    let root = root_of(&folder);
    let (events, received) = mpsc::channel();
    let mut watcher = Watcher::new(move |event| drop(events.send(event)), options(64)).unwrap();
    watcher.watch(&root).unwrap();

    let nested = root.join("nested");
    fs::create_dir(&nested).unwrap();
    assert!(wait_for(&received, |event| matches!(event, WatchEvent::Created(path) if path == &nested)));
    assert!(wait_for(&received, |event| matches!(
        event,
        WatchEvent::Rescan { root: path, reason: RescanReason::NewDirectory } if path == &nested
    )));

    let file = nested.join("inside.txt");
    fs::write(&file, b"one").unwrap();
    assert!(wait_for(&received, |event| matches!(event, WatchEvent::Created(path) if path == &file)));

    watcher.close().unwrap();
}

#[test]
fn polling_reports_a_rename_as_two_events() {
    let folder = fs_tools::temp::create_folder().unwrap();
    let root = root_of(&folder);
    let (events, received) = mpsc::channel();
    let mut watcher = Watcher::new(move |event| drop(events.send(event)), options(64)).unwrap();
    watcher.watch(&root).unwrap();

    let old = root.join("old.txt");
    let new = root.join("new.txt");
    fs::write(&old, b"one").unwrap();
    assert!(wait_for(&received, |event| matches!(event, WatchEvent::Created(path) if path == &old)));

    fs::rename(&old, &new).unwrap();
    assert!(wait_for(&received, |event| matches!(event, WatchEvent::Removed(path) if path == &old)));
    assert!(wait_for(&received, |event| matches!(event, WatchEvent::Created(path) if path == &new)));

    thread::sleep(INTERVAL * 3);
    while let Ok(event) = received.recv_timeout(INTERVAL) {
        assert!(!matches!(event, WatchEvent::Renamed { .. }), "polling must not report a paired rename: {event:?}");
    }

    watcher.close().unwrap();
}

#[test]
fn close_discards_queued_events() {
    let folder = fs_tools::temp::create_folder().unwrap();
    let root = root_of(&folder);
    let (events, received) = mpsc::channel();
    let mut watcher = Watcher::new(move |event| drop(events.send(event)), options(64)).unwrap();
    watcher.watch(&root).unwrap();

    let file = root.join("a.txt");
    fs::write(&file, b"one").unwrap();
    assert!(wait_for(&received, |event| matches!(event, WatchEvent::Created(path) if path == &file)));

    let later = root.join("b.txt");
    fs::write(&later, b"two").unwrap();
    thread::sleep(INTERVAL * 3);
    while received.recv_timeout(INTERVAL).is_ok() {}
    watcher.close().unwrap();

    assert!(received.recv_timeout(INTERVAL * 3).is_err(), "a queued event survived close");
}

#[test]
fn invalid_options_are_rejected() {
    let (events, _received) = mpsc::channel();
    assert!(matches!(Watcher::new(move |event| drop(events.send(event)), options(0)), Err(fs_tools::watch::WatchError::InvalidOptions(_))));

    let (events, _received) = mpsc::channel();
    assert!(matches!(
        Watcher::new(
            move |event| drop(events.send(event)),
            WatchOptions::new().backend(WatchBackend::Polling { interval: Duration::from_millis(1) })
        ),
        Err(fs_tools::watch::WatchError::InvalidOptions(_))
    ));
}

#[cfg(not(windows))]
#[test]
fn native_is_rejected_where_no_backend_exists() {
    let (events, _received) = mpsc::channel();
    assert!(matches!(Watcher::new(move |event| drop(events.send(event)), WatchOptions::new()), Err(fs_tools::watch::WatchError::Unsupported)));
}

#[cfg(windows)]
#[test]
fn native_reports_created_modified_and_removed() {
    let folder = fs_tools::temp::create_folder().unwrap();
    let root = root_of(&folder);
    let (events, received) = mpsc::channel();
    let mut watcher = Watcher::new(move |event| drop(events.send(event)), native_options(256)).expect("native watcher");
    watcher.watch(&root).unwrap();

    let file = root.join("a.txt");
    fs::write(&file, b"one").unwrap();
    assert!(wait_for(&received, |event| matches!(event, WatchEvent::Created(path) if path == &file)));

    fs::write(&file, b"longer contents").unwrap();
    assert!(wait_for(&received, |event| matches!(event, WatchEvent::Modified(path) if path == &file)));

    fs::remove_file(&file).unwrap();
    assert!(wait_for(&received, |event| matches!(event, WatchEvent::Removed(path) if path == &file)));

    watcher.close().unwrap();
}

#[cfg(windows)]
#[test]
fn native_pairs_a_rename() {
    let folder = fs_tools::temp::create_folder().unwrap();
    let root = root_of(&folder);
    let (events, received) = mpsc::channel();
    let mut watcher = Watcher::new(move |event| drop(events.send(event)), native_options(256)).unwrap();
    watcher.watch(&root).unwrap();

    let old = root.join("old.txt");
    let new = root.join("new.txt");
    fs::write(&old, b"one").unwrap();
    assert!(wait_for(&received, |event| matches!(event, WatchEvent::Created(path) if path == &old)));

    fs::rename(&old, &new).unwrap();
    assert!(wait_for(&received, |event| matches!(
        event,
        WatchEvent::Renamed { from, to } if from == &old && to == &new
    )));

    watcher.close().unwrap();
}

#[cfg(windows)]
#[test]
fn native_follows_a_new_folder() {
    let folder = fs_tools::temp::create_folder().unwrap();
    let root = root_of(&folder);
    let (events, received) = mpsc::channel();
    let mut watcher = Watcher::new(move |event| drop(events.send(event)), native_options(256)).unwrap();
    watcher.watch(&root).unwrap();

    let nested = root.join("nested");
    fs::create_dir(&nested).unwrap();
    assert!(wait_for(&received, |event| matches!(event, WatchEvent::Created(path) if path == &nested)));
    assert!(wait_for(&received, |event| matches!(
        event,
        WatchEvent::Rescan { root: path, reason: RescanReason::NewDirectory } if path == &root
    )));

    let file = nested.join("inside.txt");
    fs::write(&file, b"one").unwrap();
    assert!(wait_for(&received, |event| matches!(event, WatchEvent::Created(path) if path == &file)));

    watcher.close().unwrap();
}

#[cfg(windows)]
#[test]
fn native_reports_root_removal() {
    let folder = fs_tools::temp::create_folder().unwrap();
    let root = root_of(&folder);
    let (events, received) = mpsc::channel();
    let mut watcher = Watcher::new(move |event| drop(events.send(event)), native_options(64)).unwrap();
    watcher.watch(&root).unwrap();

    fs::remove_dir(&root).unwrap();
    assert!(wait_for(&received, |event| matches!(event, WatchEvent::Removed(path) if path == &root)));

    watcher.close().unwrap();
}

#[cfg(windows)]
#[test]
fn native_unwatch_discards_events_for_the_root() {
    let folder = fs_tools::temp::create_folder().unwrap();
    let root = root_of(&folder);
    let (events, received) = mpsc::channel();
    let mut watcher = Watcher::new(move |event| drop(events.send(event)), native_options(256)).unwrap();
    watcher.watch(&root).unwrap();

    let before = root.join("before.txt");
    fs::write(&before, b"one").unwrap();
    assert!(wait_for(&received, |event| matches!(event, WatchEvent::Created(path) if path == &before)));

    watcher.unwatch(&root).unwrap();
    assert!(matches!(watcher.unwatch(&root), Err(fs_tools::watch::WatchError::NotWatched(_))));

    let after = root.join("after.txt");
    fs::write(&after, b"two").unwrap();
    thread::sleep(Duration::from_millis(100));
    while let Ok(event) = received.recv_timeout(Duration::from_millis(50)) {
        assert!(!matches!(&event, WatchEvent::Created(path) if path == &after), "an event for an unwatched root was delivered: {event:?}");
    }

    watcher.close().unwrap();
}

#[cfg(windows)]
#[test]
fn native_overflow_is_reported_as_a_rescan() {
    let folder = fs_tools::temp::create_folder().unwrap();
    let root = root_of(&folder);
    let (events, received) = mpsc::channel();
    let (started, blocked) = mpsc::channel();
    let (release, resume) = mpsc::channel();
    let mut first = true;
    let mut watcher = Watcher::new(
        move |event| {
            if first {
                first = false;
                let _ = started.send(());
                let _ = resume.recv();
            }
            drop(events.send(event));
        },
        native_options(1),
    )
    .unwrap();
    watcher.watch(&root).unwrap();

    fs::write(root.join("first.txt"), b"one").unwrap();
    blocked.recv_timeout(TIMEOUT).expect("the callback starts");

    for index in 0..8 {
        fs::write(root.join(format!("burst-{index}.txt")), b"one").unwrap();
    }
    release.send(()).unwrap();

    assert!(wait_for(&received, |event| matches!(event, WatchEvent::Rescan { reason: RescanReason::Overflow, .. })));

    watcher.close().unwrap();
}
