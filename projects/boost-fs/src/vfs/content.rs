use super::version::FileVersion;

/// Bytes read from the VFS together with a consistency version.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileContent {
    /// File bytes.
    pub bytes: Vec<u8>,
    /// Version token paired with this read.
    pub version: FileVersion,
}

impl FileContent {
    /// Interprets the bytes as UTF-8 text.
    pub fn into_string(self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.bytes)
    }
}
