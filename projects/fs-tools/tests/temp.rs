use fs_tools::{
    read_to_string, remove_file, remove_folder_all,
    temp::{NamedFile, TempFolder, create_file, create_file_in, create_folder, create_folder_in, create_named, create_named_in},
    write,
};

/// A base folder taken from the host temporary folder.
///
/// A test that does not cover the process temporary folder names its base explicitly, so it never
/// reads the override while `configure_folder_creates_missing_root` replaces it.
fn base_folder() -> TempFolder {
    create_folder_in(std::env::temp_dir()).unwrap()
}

/// A named temporary file in the host temporary folder, for a test that does not cover the override.
fn named_file() -> NamedFile {
    create_named_in(std::env::temp_dir()).unwrap()
}

#[test]
fn temp_folder_is_writable() {
    let folder = base_folder();
    let path = folder.path().join("note.txt");
    write(&path, b"ok").unwrap();
    assert_eq!(read_to_string(path).unwrap(), "ok");
}

#[test]
fn temp_folder_in_uses_explicit_base() {
    let base = base_folder();
    let nested = create_folder_in(base.path()).unwrap();
    write(nested.path().join("child.txt"), b"nested").unwrap();
}

#[test]
fn anonymous_temp_file_roundtrip() {
    use std::io::{Read, Seek, SeekFrom, Write};

    let mut file = create_file_in(std::env::temp_dir()).unwrap();
    file.write_all(b"anonymous").unwrap();
    file.sync_all().unwrap();
    file.seek(SeekFrom::Start(0)).unwrap();
    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap();
    assert_eq!(buf, "anonymous");
}

#[test]
fn named_temp_file_exposes_path() {
    let file = named_file();
    assert!(file.path().exists());
    write(file.path(), b"named").unwrap();
    assert_eq!(read_to_string(file.path()).unwrap(), "named");
}

#[test]
fn create_named_in_uses_explicit_folder() {
    let base = base_folder();
    let nested = base.path().join("names");
    fs_tools::create_folder_all(&nested).unwrap();
    let file: NamedFile = create_named_in(&nested).unwrap();
    assert!(file.path().starts_with(&nested));
}

/// The process temporary folder is replaced here and nowhere else, so this test owns the override.
///
/// Every constructor that takes no base reads the configured folder, and each one is covered here
/// while the replacement is in place.
#[test]
fn configure_folder_creates_missing_root() {
    let base = base_folder();
    let custom = base.path().join("custom-temp-root");
    assert!(!custom.exists());
    fs_tools::temp::configure_folder(&custom).unwrap();
    assert!(custom.is_dir());
    assert_eq!(fs_tools::temp::folder_path().unwrap(), custom);
    assert!(create_folder().unwrap().path().starts_with(&custom), "the folder constructor follows the configured base");
    assert!(create_named().unwrap().path().starts_with(&custom), "the named file constructor follows the configured base");
    let mut anonymous = create_file().unwrap();
    std::io::Write::write_all(&mut anonymous, b"configured").unwrap();
    fs_tools::temp::set_folder(std::env::temp_dir());
    assert_eq!(fs_tools::temp::folder_path().unwrap(), std::env::temp_dir(), "the configured base is replaced again");
}

/// The candidate name carries host entropy, so two candidates for the same label differ in the suffix
/// instead of only in a counter or a timestamp.
#[test]
fn candidate_names_are_not_predictable_from_a_counter() {
    let base = base_folder();
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
    let base = base_folder();
    let file = create_named_in(base.path()).unwrap();
    let path = file.path().to_path_buf();

    // The candidate name is already taken by this very file, so a second atomic creation of the same
    // path must fail instead of overwriting the file that is there.
    let again = std::fs::OpenOptions::new().read(true).write(true).create_new(true).open(&path);
    assert!(matches!(again, Err(ref err) if err.kind() == std::io::ErrorKind::AlreadyExists));
}

/// Removing a temporary path is the job of the value that created it, so dropping that value removes
/// the path without any separate call.
#[test]
fn dropping_a_temporary_path_removes_it() {
    let folder = base_folder();
    let folder_path = folder.path().to_path_buf();
    drop(folder);
    assert!(!folder_path.exists(), "the dropped folder is removed");

    let file = named_file();
    let file_path = file.path().to_path_buf();
    drop(file);
    assert!(!file_path.exists(), "the dropped file is removed");
}

/// Keeping hands the path to the caller, and the value no longer removes it on drop.
#[test]
fn keeping_a_temporary_path_hands_it_over() {
    let folder = base_folder();
    let kept_folder = folder.keep();
    assert!(kept_folder.is_dir(), "the kept folder stays on disk");

    let file = named_file();
    let (mut handle, kept_file) = file.keep();
    std::io::Write::write_all(&mut handle, b"kept").unwrap();
    drop(handle);
    assert_eq!(read_to_string(&kept_file).unwrap(), "kept");

    remove_folder_all(kept_folder).unwrap();
    remove_file(kept_file).unwrap();
}

/// Closing removes the path now instead of at the end of scope, and the destructor does not remove it
/// a second time.
#[test]
fn closing_a_temporary_path_deletes_it_now() {
    let folder = base_folder();
    let folder_path = folder.path().to_path_buf();
    folder.close().unwrap();
    assert!(!folder_path.exists(), "the closed folder is removed");

    let file = named_file();
    let file_path = file.path().to_path_buf();
    file.close().unwrap();
    assert!(!file_path.exists(), "the closed file is removed");
}

/// Giving up the name of a temporary file leaves the bytes reachable through the handle only, so the
/// caller cannot open the path again.
#[test]
fn into_file_gives_up_the_temporary_name() {
    use std::io::{Read, Seek, SeekFrom, Write};

    let file = named_file();
    let path = file.path().to_path_buf();
    let mut handle = file.into_file();

    handle.write_all(b"kept through the handle").unwrap();
    handle.seek(SeekFrom::Start(0)).unwrap();
    let mut text = String::new();
    handle.read_to_string(&mut text).unwrap();
    assert_eq!(text, "kept through the handle");

    assert!(std::fs::read(&path).is_err(), "the temporary name is gone");
}

/// An anonymous temporary file has no name to keep, on the host that unlinks at creation and on the
/// host that removes the name when the last handle is closed.
#[test]
fn an_anonymous_file_leaves_no_name_behind() {
    let base = base_folder();
    let base_path = base.path().to_path_buf();

    {
        let mut handle = create_file_in(&base_path).unwrap();
        std::io::Write::write_all(&mut handle, b"anonymous").unwrap();
    }

    assert_eq!(std::fs::read_dir(&base_path).unwrap().count(), 0, "closing the handle left no name behind");
}

/// Only a collision is retried, so a base folder that is missing is reported straight away instead of
/// being retried until the candidate limit is reached.
#[test]
fn a_missing_base_is_reported_instead_of_retried() {
    let base = base_folder();
    let missing = base.path().join("missing");

    let reported =
        |error: std::io::Error| assert_eq!(error.kind(), std::io::ErrorKind::NotFound, "the missing base is reported as missing: {error}");

    reported(create_folder_in(&missing).err().expect("a missing base is reported"));
    reported(create_named_in(&missing).err().expect("a missing base is reported"));
    reported(create_file_in(&missing).expect_err("a missing base is reported"));
}

/// A base folder that is named relatively is resolved before the candidate is created, so the
/// temporary path is absolute and does not move when the current folder changes later.
#[test]
fn a_relative_base_yields_an_absolute_path() {
    let current = std::env::current_dir().unwrap();
    let base = create_folder_in(&current).unwrap();
    let relative = base.path().strip_prefix(&current).unwrap().to_path_buf();

    let folder = create_folder_in(&relative).unwrap();
    assert!(folder.path().is_absolute(), "the folder path is absolute: {}", folder.path().display());
    assert_eq!(folder.path().parent(), Some(base.path()), "the folder sits under the base that was named relatively");

    let file = create_named_in(&relative).unwrap();
    assert!(file.path().is_absolute(), "the file path is absolute: {}", file.path().display());
    assert_eq!(file.path().parent(), Some(base.path()), "the file sits under the base that was named relatively");
}

/// A temporary path is reachable by its owner only, so a file that has not been published yet is not
/// readable by another account.
#[cfg(unix)]
#[test]
fn temporary_paths_are_restricted_to_the_owner() {
    use std::os::unix::fs::PermissionsExt;

    let folder = base_folder();
    let folder_mode = std::fs::metadata(folder.path()).unwrap().permissions().mode() & 0o777;
    assert_eq!(folder_mode & 0o077, 0, "the temporary folder is not exposed to other accounts");
    assert_eq!(folder_mode & 0o700, 0o700, "the owner can still use the temporary folder");

    let file = create_named_in(folder.path()).unwrap();
    let file_mode = std::fs::metadata(file.path()).unwrap().permissions().mode() & 0o777;
    assert_eq!(file_mode & 0o077, 0, "the temporary file is not exposed to other accounts");
    assert_eq!(file_mode & 0o600, 0o600, "the owner can still read and write the temporary file");
}

/// Cleanup cannot report a failure through a destructor, so a path that is already gone must not turn
/// the drop into a panic.
#[test]
fn dropping_a_path_that_is_already_gone_is_not_an_error() {
    let folder = base_folder();
    let folder_path = folder.path().to_path_buf();
    remove_folder_all(&folder_path).unwrap();
    drop(folder);

    let file = named_file();
    let file_path = file.path().to_path_buf();
    remove_file(&file_path).unwrap();
    drop(file);
}
