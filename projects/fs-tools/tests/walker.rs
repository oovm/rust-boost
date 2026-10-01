use fs_tools::walk::Walker;

#[cfg(feature = "async")]
use fs_tools::async_walk::Walker as AsyncWalker;

#[test]
fn sync_walker_visits_nested_files() {
    let root = tempfile_dir("sync");
    fs_tools::create_dir_all(root.join("nested")).unwrap();
    fs_tools::write(root.join("nested").join("child.txt"), b"ok").unwrap();

    let paths: Vec<_> = Walker::new(&root)
        .into_iter()
        .map(|entry| entry.unwrap().path().strip_prefix(&root).unwrap().to_path_buf())
        .collect();

    assert!(paths.contains(&std::path::Path::new("nested").to_path_buf()));
    assert!(paths.contains(&std::path::Path::new("nested").join("child.txt")));
}

#[cfg(feature = "async")]
#[test]
fn async_walker_visits_nested_files() {
    use futures_lite::stream::StreamExt;

    let root = tempfile_dir("async");
    fs_tools::create_dir_all(root.join("nested")).unwrap();
    fs_tools::write(root.join("nested").join("child.txt"), b"ok").unwrap();

    let paths = futures_lite::future::block_on(async {
        let mut walker = core::pin::pin!(AsyncWalker::new(&root));
        let mut paths = Vec::new();
        while let Some(entry) = walker.next().await {
            paths.push(entry.unwrap().path().strip_prefix(&root).unwrap().to_path_buf());
        }
        paths
    });

    assert!(paths.contains(&std::path::Path::new("nested").to_path_buf()));
    assert!(paths.contains(&std::path::Path::new("nested").join("child.txt")));
}

fn tempfile_dir(label: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("fs-tools-{label}-{}", std::process::id()));
    let _ = fs_tools::remove_dir_all(&dir);
    fs_tools::create_dir_all(&dir).unwrap();
    dir
}
