use std::path::{Path, PathBuf};

use fs_tools::{
    temp::create_folder,
    walk::{Entry, ErrorPolicy, Filter, Walker},
    write,
};

#[cfg(feature = "async")]
use fs_tools::async_walk::Walker as AsyncWalker;

/// Collect the paths below `root` relative to `root`, and fail the calling test on a walk error.
///
/// Relative paths keep the tests independent of the host separator.
fn relative_paths<I>(root: &Path, entries: I) -> Vec<PathBuf>
where
    I: IntoIterator<Item = Result<Entry, fs_tools::walk::Error>>,
{
    entries
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("walk error")
        .into_iter()
        .map(|entry| entry.path().strip_prefix(root).expect("root prefix").to_path_buf())
        .collect()
}

#[test]
fn sync_walker_visits_nested_files() {
    let root = create_folder().unwrap();
    fs_tools::create_folder_all(root.path().join("nested")).unwrap();
    write(root.path().join("nested").join("child.txt"), b"ok").unwrap();

    let paths = relative_paths(root.path(), Walker::new(root.path()));

    assert!(paths.contains(&PathBuf::from("nested")));
    assert!(paths.contains(&Path::new("nested").join("child.txt")));
}

#[test]
fn walker_prunes_folders_that_the_filter_rejects() {
    let root = create_folder().unwrap();
    fs_tools::create_folder_all(root.path().join("skipped").join("inner")).unwrap();
    write(root.path().join("skipped").join("inner").join("hidden.txt"), b"hidden").unwrap();
    write(root.path().join("skipped").join("also-hidden.txt"), b"hidden").unwrap();
    write(root.path().join("kept.txt"), b"kept").unwrap();

    let predicate = |entry: &Entry| {
        if entry.file_name() == std::ffi::OsStr::new("skipped") { Filter::IgnoreFolder } else { Filter::Continue }
    };
    let paths = relative_paths(root.path(), Walker::new(root.path()).min_depth(1).filter(predicate));

    assert_eq!(paths, vec![PathBuf::from("kept.txt")]);
}

#[test]
fn walker_ignores_an_entry_but_still_traverses_it() {
    let root = create_folder().unwrap();
    fs_tools::create_folder_all(root.path().join("visible")).unwrap();
    write(root.path().join("visible").join("child.txt"), b"child").unwrap();

    let predicate = |entry: &Entry| {
        if entry.file_name() == std::ffi::OsStr::new("visible") { Filter::Ignore } else { Filter::Continue }
    };
    let paths = relative_paths(root.path(), Walker::new(root.path()).min_depth(1).filter(predicate));

    assert_eq!(paths, vec![Path::new("visible").join("child.txt")]);
}

#[test]
fn walker_visits_sorted_children_depth_first() {
    let root = create_folder().unwrap();
    fs_tools::create_folder_all(root.path().join("b")).unwrap();
    fs_tools::create_folder_all(root.path().join("a")).unwrap();
    write(root.path().join("b").join("d.txt"), b"d").unwrap();
    write(root.path().join("a").join("c.txt"), b"c").unwrap();

    let paths = relative_paths(root.path(), Walker::new(root.path()).min_depth(1).sort_by_file_name());

    assert_eq!(paths, vec![PathBuf::from("a"), Path::new("a").join("c.txt"), PathBuf::from("b"), Path::new("b").join("d.txt")]);
}

#[test]
fn walker_observes_min_and_max_depth() {
    let root = create_folder().unwrap();
    fs_tools::create_folder_all(root.path().join("outer")).unwrap();
    write(root.path().join("outer").join("inner.txt"), b"inner").unwrap();
    write(root.path().join("top.txt"), b"top").unwrap();

    let deep = relative_paths(root.path(), Walker::new(root.path()).sort_by_file_name().min_depth(2));
    assert_eq!(deep, vec![Path::new("outer").join("inner.txt")]);

    let shallow = relative_paths(root.path(), Walker::new(root.path()).sort_by_file_name().max_depth(1));
    assert_eq!(shallow, vec![PathBuf::new(), PathBuf::from("outer"), PathBuf::from("top.txt")]);
}

#[test]
fn strict_policy_stops_the_iteration_at_the_first_error() {
    let root = create_folder().unwrap();
    let missing = root.path().join("missing");

    let mut entries = Walker::new(&missing).error_policy(ErrorPolicy::Strict).into_iter();
    assert!(entries.next().expect("the missing root is reported").is_err());
    assert!(entries.next().is_none(), "strict stopped after the error");

    let lenient: Vec<_> = Walker::new(&missing).error_policy(ErrorPolicy::Lenient).into_iter().collect();
    assert_eq!(lenient.len(), 1);
    assert!(lenient[0].is_err());
}

/// A folder that cannot be read is an error, and the entries that are still pending are visited
/// anyway, so a lenient walk does not hide the rest of the tree behind the first failure.
#[cfg(unix)]
#[test]
fn lenient_policy_keeps_visiting_after_an_error() {
    use std::os::unix::fs::PermissionsExt;

    let root = create_folder().unwrap();
    let blocked = root.path().join("blocked");
    fs_tools::create_folder_all(&blocked).unwrap();
    write(root.path().join("visible.txt"), b"visible").unwrap();

    let mut permissions = std::fs::metadata(&blocked).unwrap().permissions();
    permissions.set_mode(0o000);
    fs_tools::set_permissions(&blocked, permissions).unwrap();

    let items: Vec<_> = Walker::new(root.path()).error_policy(ErrorPolicy::Lenient).into_iter().collect();

    let mut permissions = std::fs::metadata(&blocked).unwrap().permissions();
    permissions.set_mode(0o700);
    fs_tools::set_permissions(&blocked, permissions).unwrap();

    assert!(items.iter().any(|item| item.is_err()), "the unreadable folder is reported");
    assert!(
        items.iter().any(|item| item.as_ref().is_ok_and(|entry| entry.file_name() == std::ffi::OsStr::new("visible.txt"))),
        "the entries after the error are still visited"
    );
}

/// A link that resolves to a folder on its own branch is a loop, and reporting it is what keeps the
/// walk finite.
#[cfg(unix)]
#[test]
fn walker_reports_a_symlink_loop() {
    let root = create_folder().unwrap();
    let folder = root.path().join("folder");
    fs_tools::create_folder_all(&folder).unwrap();
    std::os::unix::fs::symlink(&folder, folder.join("back")).unwrap();

    let items: Vec<_> = Walker::new(root.path()).follow_links(true).into_iter().collect();

    let loop_ancestor = items
        .iter()
        .filter_map(|item| item.as_ref().err())
        .find_map(|error| error.loop_ancestor().map(Path::to_path_buf))
        .expect("the link back into its own branch is reported as a loop");
    assert_eq!(loop_ancestor, std::fs::canonicalize(&folder).unwrap());
}

/// Two sibling links to one folder are not a loop, because neither is a folder on the other's branch.
#[cfg(unix)]
#[test]
fn walker_accepts_sibling_symlinks_to_one_folder() {
    let root = create_folder().unwrap();
    let target = root.path().join("target");
    fs_tools::create_folder_all(&target).unwrap();
    write(target.join("leaf.txt"), b"leaf").unwrap();
    std::os::unix::fs::symlink(&target, root.path().join("left")).unwrap();
    std::os::unix::fs::symlink(&target, root.path().join("right")).unwrap();

    let paths = relative_paths(root.path(), Walker::new(root.path()).follow_links(true).min_depth(1).sort_by_file_name());

    assert_eq!(
        paths,
        vec![
            PathBuf::from("left"),
            Path::new("left").join("leaf.txt"),
            PathBuf::from("right"),
            Path::new("right").join("leaf.txt"),
            PathBuf::from("target"),
            Path::new("target").join("leaf.txt")
        ]
    );
}

#[cfg(unix)]
#[test]
fn walker_visits_paths_that_are_not_utf8() {
    use std::{ffi::OsStr, os::unix::ffi::OsStrExt};

    let root = create_folder().unwrap();
    let name = OsStr::from_bytes(b"not-utf8-\xff.txt");
    write(root.path().join(name), b"bytes").unwrap();

    let found = relative_paths(root.path(), Walker::new(root.path().min_depth(1))).into_iter().any(|path| path.file_name() == Some(name));

    assert!(found, "a path that is not valid UTF-8 is still visited");
}

#[cfg(feature = "async")]
#[test]
fn async_walker_visits_nested_files() {
    use futures_lite::stream::StreamExt;

    let root = create_folder().unwrap();
    fs_tools::create_folder_all(root.path().join("nested")).unwrap();
    write(root.path().join("nested").join("child.txt"), b"ok").unwrap();

    let paths = futures_lite::future::block_on(async {
        let mut walker = core::pin::pin!(AsyncWalker::new(root.path()));
        let mut paths = Vec::new();
        while let Some(entry) = walker.next().await {
            paths.push(entry.unwrap().path().strip_prefix(root.path()).unwrap().to_path_buf());
        }
        paths
    });
    assert!(paths.contains(&PathBuf::from("nested")));
    assert!(paths.contains(&Path::new("nested").join("child.txt")));
}

#[cfg(feature = "async")]
#[test]
fn async_walker_prunes_folders_that_the_filter_rejects() {
    use futures_lite::stream::StreamExt;

    let root = create_folder().unwrap();
    fs_tools::create_folder_all(root.path().join("skipped")).unwrap();
    write(root.path().join("skipped").join("hidden.txt"), b"hidden").unwrap();
    write(root.path().join("kept.txt"), b"kept").unwrap();

    let paths = futures_lite::future::block_on(async {
        let walker = AsyncWalker::new(root.path()).filter(|entry| {
            let rejected = entry.file_name() == std::ffi::OsStr::new("skipped");
            async move { if rejected { Filter::IgnoreFolder } else { Filter::Continue } }
        });
        let mut walker = core::pin::pin!(walker);
        let mut paths = Vec::new();
        while let Some(entry) = walker.next().await {
            paths.push(entry.expect("walk error").path().strip_prefix(root.path()).unwrap().to_path_buf());
        }
        paths
    });

    assert_eq!(paths, vec![PathBuf::from("kept.txt")]);
}
