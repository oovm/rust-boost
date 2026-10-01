use core::fmt::{self, Display, Formatter};

/// Producer identity for a diagnostic record.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DiagnosticOrigin {
    namespace: String,
    component: String,
    stage: Option<String>,
}

impl DiagnosticOrigin {
    /// Create a producer origin.
    pub fn new(namespace: impl Into<String>, component: impl Into<String>) -> Self {
        Self { namespace: namespace.into(), component: component.into(), stage: None }
    }

    /// Attach an optional stage name.
    pub fn with_stage(mut self, stage: impl Into<String>) -> Self {
        self.stage = Some(stage.into());
        self
    }

    /// Returns the namespace string.
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Returns the component string.
    pub fn component(&self) -> &str {
        &self.component
    }

    /// Returns the optional stage string.
    pub fn stage(&self) -> Option<&str> {
        self.stage.as_deref()
    }
}

impl Display for DiagnosticOrigin {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.namespace, self.component)?;
        if let Some(stage) = &self.stage {
            write!(f, "/{stage}")?;
        }
        Ok(())
    }
}
