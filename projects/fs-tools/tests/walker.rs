use fs_tools::temp::create_dir;
use fs_tools::walk::Walker;

#[cfg(feature = "async")]
use fs_tools::async_walk::Walker as AsyncWalker;

#[test]
fn sync_walker_visits_nested_files() {
    let root = create_dir().unwrap();
    fs_tools::create_dir_all(root.path().join("nested")).unwrap();
    fs_tools::write(root.path().join("nested").join("child.txt"), b"ok").unwrap();

    let paths: Vec<_> = Walker::new(root.path())
        .into_iter()
        .map(|entry| entry.unwrap().path().strip_prefix(root.path()).unwrap().to_path_buf())
        .collect();

    assert!(paths.contains(&std::path::Path::new("nested").to_path_buf()));
    assert!(paths.contains(&std::path::Path::new("nested").join("child.txt")));
}
#[cfg(feature = "async")]
#[test]
fn async_walker_visits_nested_files() {
    use futures_lite::stream::StreamExt;

    let root = create_dir().unwrap();
    fs_tools::create_dir_all(root.path().join("nested")).unwrap();
    fs_tools::write(root.path().join("nested").join("child.txt"), b"ok").unwrap();

    let paths = futures_lite::future::block_on(async {
        let mut walker = core::pin::pin!(AsyncWalker::new(root.path()));
        let mut paths = Vec::new();
        while let Some(entry) = walker.next().await {
            paths.push(entry.unwrap().path().strip_prefix(root.path()).unwrap().to_path_buf());
        }
        paths
    });
    assert!(paths.contains(&std::path::Path::new("nested").to_path_buf()));
    assert!(paths.contains(&std::path::Path::new("nested").join("child.txt")));
}