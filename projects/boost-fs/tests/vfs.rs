use std::path::Path;

use boost_fs::{
    temp::create_folder,
    vfs::{LayeredFs, VfsError},
};

#[test]
fn layered_read_prefers_overlay() {
    let folder = create_folder().unwrap();
    let root = folder.path();
    let file = root.join("sample.txt");
    boost_fs::write(&file, b"disk").unwrap();

    let fs = LayeredFs::new(root);
    let disk = fs.read(Path::new("sample.txt")).unwrap();
    assert_eq!(disk.bytes, b"disk");

    fs.write_overlay(Path::new("sample.txt").to_path_buf(), b"buffer".to_vec());
    let overlay = fs.read(Path::new("sample.txt")).unwrap();
    assert_eq!(overlay.bytes, b"buffer");
    assert_ne!(disk.version, overlay.version);
}

#[test]
fn overlay_remove_hides_disk_file() {
    let folder = create_folder().unwrap();
    let root = folder.path();
    let file = root.join("gone.txt");
    boost_fs::write(&file, b"disk").unwrap();

    let fs = LayeredFs::new(root);
    fs.remove_overlay(Path::new("gone.txt"));
    assert!(matches!(fs.read(Path::new("gone.txt")), Err(VfsError::NotFound(_))));

    fs.clear_overlay(Path::new("gone.txt"));
    let restored = fs.read(Path::new("gone.txt")).unwrap();
    assert_eq!(restored.bytes, b"disk");
}
