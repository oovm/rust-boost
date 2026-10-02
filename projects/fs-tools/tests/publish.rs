use std::io::ErrorKind;

use fs_tools::{
    publish_bytes, publish_bytes_with_options, read_to_string, AtomicPublisher, OverwritePolicy,
    PublishOptions,
};

#[test]
fn publish_bytes_creates_destination_atomically() {
    let base = fs_tools::temp::create_folder().expect("temp folder");
    let target = base.path().join("report.json");

    publish_bytes(&target, br#"{"ok":true}"#).expect("publish");

    assert_eq!(read_to_string(&target).expect("read"), r#"{"ok":true}"#);
    let entries: Vec<_> = base
        .path()
        .read_dir()
        .expect("read dir")
        .map(|entry| entry.expect("dir entry").file_name())
        .collect();
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

    publish_bytes_with_options(
        &target,
        b"v2",
        PublishOptions {
            overwrite: OverwritePolicy::Replace,
            fsync_file: false,
            fsync_parent: false,
        },
    )
    .expect("replace publish");

    assert_eq!(read_to_string(&target).expect("read"), "v2");
}

#[test]
fn atomic_publisher_drops_staging_file_on_abort() {
    let base = fs_tools::temp::create_folder().expect("temp folder");
    let target = base.path().join("report.json");

    let publisher = AtomicPublisher::create(
        &target,
        PublishOptions {
            overwrite: OverwritePolicy::RefuseExisting,
            fsync_file: false,
            fsync_parent: false,
        },
    )
    .expect("create publisher");
    let staging_path = publisher.staging_path().to_path_buf();
    assert!(staging_path.exists());
    drop(publisher);
    assert!(!staging_path.exists());
    assert!(!target.exists());
}
