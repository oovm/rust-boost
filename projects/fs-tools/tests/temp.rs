use fs_tools::{
    read_to_string,
    temp::{NamedFile, create_file, create_folder, create_folder_in, create_named, create_named_in},
    write,
};

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

/// The candidate name carries host entropy, so two candidates for the same label differ in the suffix
/// instead of only in a counter or a timestamp.
#[test]
fn candidate_names_are_not_predictable_from_a_counter() {
    let base = create_folder().unwrap();
    let names: Vec<String> = (0..32)
        .map(|_| {
            let file = create_named_in(base.path()).unwrap();
            file.path().file_name().unwrap().to_string_lossy().into_owned()
        })
        .collect();

    let unique: std::collections::HashSet<&String> = names.iter().collect();
    assert_eq!(unique.len(), names.len(), "candidate names repeated: {names:?}");

    for name in &names {
        let suffix = name.rsplit('-').next().unwrap();
        assert_eq!(suffix.len(), 32, "the suffix is not 16 entropy bytes in hex: {name}");
        assert!(suffix.chars().all(|digit| digit.is_ascii_hexdigit()), "the suffix is not hex: {name}");
    }

    // A counter-style suffix would differ only in its last digits, so the leading byte of the suffix
    // must differ too.
    let leading: std::collections::HashSet<&str> = names.iter().map(|name| &name[name.len() - 32..][..2]).collect();
    assert!(leading.len() > 1, "every candidate started with the same entropy byte: {names:?}");
}

/// Randomness names a candidate, but it does not reserve the path: creation still has to be atomic.
#[test]
fn candidate_names_still_rely_on_atomic_creation() {
    let base = create_folder().unwrap();
    let file = create_named_in(base.path()).unwrap();
    let path = file.path().to_path_buf();

    // The candidate name is already taken by this very file, so a second atomic creation of the same
    // path must fail instead of overwriting the file that is there.
    let again = std::fs::OpenOptions::new().read(true).write(true).create_new(true).open(&path);
    assert!(matches!(again, Err(ref err) if err.kind() == std::io::ErrorKind::AlreadyExists));
}
