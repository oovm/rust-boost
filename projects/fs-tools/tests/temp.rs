use fs_tools::temp::{create_file, create_folder, create_folder_in, create_named, create_named_in, NamedFile};
use fs_tools::{read_to_string, write};

#[test]
fn temp_folder_is_writable() {
    let folder = create_folder().unwrap();
    let path = folder.path().join("note.txt");
    write(&path, b"ok").unwrap();
    assert_eq!(read_to_string(path).unwrap(), "ok");
}

#[test]
fn temp_folder_in_uses_explicit_base() {
    let base = create_folder().unwrap();
    let nested = create_folder_in(base.path()).unwrap();
    write(nested.path().join("child.txt"), b"nested").unwrap();
}

#[test]
fn anonymous_temp_file_roundtrip() {
    use std::io::{Read, Seek, SeekFrom, Write};

    let mut file = create_file().unwrap();
    file.write_all(b"anonymous").unwrap();
    file.sync_all().unwrap();
    file.seek(SeekFrom::Start(0)).unwrap();
    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap();
    assert_eq!(buf, "anonymous");
}

#[test]
fn named_temp_file_exposes_path() {
    let file = create_named().unwrap();
    assert!(file.path().exists());
    write(file.path(), b"named").unwrap();
    assert_eq!(read_to_string(file.path()).unwrap(), "named");
}

#[test]
fn create_named_in_uses_explicit_folder() {
    let base = create_folder().unwrap();
    let nested = base.path().join("names");
    fs_tools::create_folder_all(&nested).unwrap();
    let file: NamedFile = create_named_in(&nested).unwrap();
    assert!(file.path().starts_with(&nested));
}

#[test]
fn configure_folder_creates_missing_root() {
    let base = create_folder().unwrap();
    let custom = base.path().join("custom-temp-root");
    assert!(!custom.exists());
    fs_tools::temp::configure_folder(&custom).unwrap();
    assert!(custom.is_dir());
    assert_eq!(fs_tools::temp::folder_path().unwrap(), custom);
    fs_tools::temp::set_folder(std::env::temp_dir());
}
