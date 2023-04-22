use std::io::{ErrorKind, Write};

use fs_tools::{AtomicPublisher, OverwritePolicy, PublishOptions, publish_bytes, publish_bytes_with_options, read_to_string};

#[test]
fn publish_bytes_creates_destination_atomically() {
    let base = fs_tools::temp::create_folder().expect("temp folder");
    let target = base.path().join("report.json");

    publish_bytes(&target, br#"{"ok":true}"#).expect("publish");

    assert_eq!(read_to_string(&target).expect("read"), r#"{"ok":true}"#);
    let entries: Vec<_> = base.path().read_dir().expect("read dir").map(|entry| entry.expect("dir entry").file_name()).collect();
    assert_eq!(entries, vec![std::ffi::OsString::from("report.json")]);
}

#[test]
fn publish_refuses_existing_destination() {
    let base = fs_tools::temp::create_folder().expect("temp folder");
    let target = base.path().join("report.json");
    publish_bytes(&target, b"v1").expect("first publish");

    let error = publish_bytes(&target, b"v2").expect_err("second publish");
    assert_eq!(error.kind(), ErrorKind::AlreadyExists);
    assert_eq!(read_to_string(&target).expect("read"), "v1");
}

#[test]
fn publish_replace_overwrites_destination() {
    let base = fs_tools::temp::create_folder().expect("temp folder");
    let target = base.path().join("report.json");
    publish_bytes(&target, b"v1").expect("first publish");

    publish_bytes_with_options(&target, b"v2", PublishOptions { overwrite: OverwritePolicy::Replace, fsync_file: false, fsync_parent: false })
        .expect("replace publish");

    assert_eq!(read_to_string(&target).expect("read"), "v2");
}

#[test]
fn atomic_publisher_drops_staging_file_on_abort() {
    let base = fs_tools::temp::create_folder().expect("temp folder");
    let target = base.path().join("report.json");

    let publisher =
        AtomicPublisher::create(&target, PublishOptions { overwrite: OverwritePolicy::RefuseExisting, fsync_file: false, fsync_parent: false })
            .expect("create publisher");
    let staging_path = publisher.staging_path().to_path_buf();
    assert!(staging_path.exists());
    drop(publisher);
    assert!(!staging_path.exists());
    assert!(!target.exists());
}

/// A destination that appears while the file is staged is refused at publish time, without replacing
/// the destination and without leaving the staging file behind.
#[test]
fn refused_publish_keeps_the_destination_and_removes_the_staging_file() {
    let base = fs_tools::temp::create_folder().expect("temp folder");
    let target = base.path().join("report.json");

    let mut publisher =
        AtomicPublisher::create(&target, PublishOptions { overwrite: OverwritePolicy::RefuseExisting, fsync_file: false, fsync_parent: false })
            .expect("create publisher");
    let staging_path = publisher.staging_path().to_path_buf();
    publisher.write_all(b"staged").expect("write staged bytes");

    std::fs::write(&target, b"external").expect("create the destination after staging");

    let error = publisher.commit().expect_err("the publish is refused");
    assert_eq!(error.kind(), ErrorKind::AlreadyExists);
    assert_eq!(read_to_string(&target).expect("read"), "external");
    assert!(!staging_path.exists(), "the refused publish left its staging file behind");

    let entries: Vec<_> = base.path().read_dir().expect("read dir").map(|entry| entry.expect("dir entry").file_name()).collect();
    assert_eq!(entries, vec![std::ffi::OsString::from("report.json")]);
}

/// Publishing by rename replaces the destination and closes the handle it staged, so a replaced file
/// is readable again immediately.
#[test]
fn replace_publish_leaves_no_staging_file() {
    let base = fs_tools::temp::create_folder().expect("temp folder");
    let target = base.path().join("report.json");
    publish_bytes(&target, b"v1").expect("first publish");

    let mut publisher =
        AtomicPublisher::create(&target, PublishOptions { overwrite: OverwritePolicy::Replace, fsync_file: true, fsync_parent: true })
            .expect("create publisher");
    publisher.write_all(b"v2").expect("write staged bytes");
    let staging_path = publisher.staging_path().to_path_buf();
    let published = publisher.commit().expect("replace publish");

    assert_eq!(published, target);
    assert_eq!(read_to_string(&target).expect("read"), "v2");
    assert!(!staging_path.exists());

    let entries: Vec<_> = base.path().read_dir().expect("read dir").map(|entry| entry.expect("dir entry").file_name()).collect();
    assert_eq!(entries, vec![std::ffi::OsString::from("report.json")]);
}
