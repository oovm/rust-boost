use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Build a collision-resistant name prefix for temporary paths.
pub(crate) fn prefix(label: &str) -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).map(|duration| duration.as_nanos()).unwrap_or(0);
    format!(".fs-tools-{label}-{stamp}-{count}")
}

#[cfg(test)]
mod tests {
    use super::prefix;

    #[test]
    fn prefixes_are_unique() {
        let a = prefix("test");
        let b = prefix("test");
        assert_ne!(a, b);
    }
}
