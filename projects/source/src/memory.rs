use crate::access::{AccessError, ByteAccess};
use crate::identity::IdentityError;
use crate::range::ByteRange;
use crate::snapshot::{SnapshotHandle, SnapshotRef};
use crate::text::{LineIndex, TextView};

/// Immutable in-memory snapshot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemorySnapshot {
    reference: SnapshotRef,
    bytes: Vec<u8>,
    lines: LineIndex,
}

impl MemorySnapshot {
    /// Create a memory snapshot from raw bytes.
    pub fn from_bytes(reference: SnapshotRef, bytes: Vec<u8>) -> Self {
        let lines = LineIndex::from_bytes(&bytes);
        Self { reference, bytes, lines }
    }

    /// Create a memory snapshot with an explicit revision.
    pub fn with_revision(
        namespace: impl Into<String>,
        id: impl Into<String>,
        revision: impl Into<String>,
        bytes: Vec<u8>,
    ) -> Result<Self, IdentityError> {
        Ok(Self::from_bytes(SnapshotRef::with_revision(namespace, id, revision)?, bytes))
    }

    /// Returns the snapshot reference.
    pub fn reference(&self) -> &SnapshotRef {
        &self.reference
    }

    /// Returns a text view over this snapshot.
    pub fn text_view(&self) -> TextView<'_, Self> {
        TextView::new(self, &self.lines)
    }
}

impl ByteAccess for MemorySnapshot {
    fn len(&self) -> u64 {
        self.bytes.len() as u64
    }

    fn read(&self, range: ByteRange) -> Result<&[u8], AccessError> {
        self.bytes.as_slice().read(range)
    }
}

/// Process-local store for immutable snapshots.
#[derive(Clone, Debug)]
pub struct MemoryStore {
    snapshots: Vec<Option<MemorySnapshot>>,
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryStore {
    /// Create an empty store.
    ///
    /// Index `0` is reserved for [`SnapshotHandle::INVALID`]; the first inserted snapshot
    /// receives handle `1`.
    pub fn new() -> Self {
        Self { snapshots: vec![None] }
    }

    /// Insert a snapshot and return a process-local handle.
    pub fn insert(&mut self, snapshot: MemorySnapshot) -> SnapshotHandle {
        let index = self.snapshots.len() as u32;
        self.snapshots.push(Some(snapshot));
        SnapshotHandle::from_index(index)
    }

    /// Resolve a handle to a snapshot.
    pub fn get(&self, handle: SnapshotHandle) -> Result<&MemorySnapshot, AccessError> {
        self.snapshots
            .get(handle.index() as usize)
            .and_then(|entry| entry.as_ref())
            .ok_or(AccessError::SnapshotReleased)
    }

    /// Release a snapshot handle.
    pub fn release(&mut self, handle: SnapshotHandle) {
        if let Some(entry) = self.snapshots.get_mut(handle.index() as usize) {
            *entry = None;
        }
    }
}
