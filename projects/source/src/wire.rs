use crate::identity::SourceRef;

/// Encode a source reference into a reversible wire identifier.
pub fn encode_source_ref(source: &SourceRef) -> String {
    let mut out = String::from("v1|");
    out.push_str(&escape_component(source.namespace()));
    out.push('|');
    out.push_str(&escape_component(source.id()));
    if let Some(revision) = source.revision() {
        out.push('|');
        out.push_str(&escape_component(revision.as_str()));
    }
    out
}

/// Decode a wire identifier into a source reference.
pub fn decode_source_ref(wire: &str) -> Result<SourceRef, WireError> {
    let components = split_wire_components(wire)?;
    if components.is_empty() || components[0] != "v1" {
        return Err(WireError::UnsupportedVersion);
    }
    if components.len() < 3 {
        return Err(WireError::InvalidFormat);
    }
    let namespace = unescape_component(components[1])?;
    let id = unescape_component(components[2])?;
    let mut source = SourceRef::new(namespace, id)?;
    if let Some(revision) = components.get(3) {
        source = source.with_revision(unescape_component(revision)?)?;
    }
    if components.len() > 4 {
        return Err(WireError::InvalidFormat);
    }
    Ok(source)
}

fn split_wire_components(wire: &str) -> Result<Vec<&str>, WireError> {
    let mut parts = Vec::new();
    let mut start = 0usize;
    let bytes = wire.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] == b'|' {
            parts.push(&wire[start..index]);
            start = index + 1;
        }
        else if bytes[index] == b'\\' {
            index += 1;
            if index >= bytes.len() {
                return Err(WireError::InvalidEscape);
            }
        }
        index += 1;
    }
    parts.push(&wire[start..]);
    Ok(parts)
}

fn escape_component(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' | '|' => {
                out.push('\\');
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }
    out
}

fn unescape_component(value: &str) -> Result<String, WireError> {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            let next = chars.next().ok_or(WireError::InvalidEscape)?;
            out.push(next);
        }
        else {
            out.push(ch);
        }
    }
    Ok(out)
}

/// Wire encoding failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WireError {
    /// Wire string does not match the expected layout.
    InvalidFormat,
    /// Trailing escape without a following character.
    InvalidEscape,
    /// Unsupported wire version prefix.
    UnsupportedVersion,
    /// Identity validation failed while decoding.
    Identity(crate::identity::IdentityError),
}

impl From<crate::identity::IdentityError> for WireError {
    fn from(error: crate::identity::IdentityError) -> Self {
        Self::Identity(error)
    }
}
