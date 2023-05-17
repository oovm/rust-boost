use std::{
    env,
    error::Error,
    fmt::{self, Display, Formatter},
    string::String,
    vec::Vec,
};

use crate::{event::LogEvent, level::Level};

/// Event filter with a global level and optional per-target directives.
///
/// A target directive matches an event when the event target is the directive target or starts
/// with it followed by `::`, so a directive for `oak` never matches `oak_vfs`. When several
/// directives match, the longest target wins. A target with no matching directive uses the global
/// level, or is dropped when no global level is configured.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Filter {
    default: Option<Level>,
    directives: Vec<Directive>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Directive {
    target: String,
    level: Level,
}

impl Filter {
    /// Create a filter that accepts events at or above `min_level` regardless of target.
    pub fn new(min_level: Level) -> Self {
        Self { default: Some(min_level), directives: Vec::new() }
    }

    /// Add or replace the level for one target.
    ///
    /// The target must be an exact target or a `::`-separated prefix of one.
    pub fn with_target(mut self, target: impl Into<String>, level: Level) -> Self {
        let target = target.into();
        match self.directives.iter_mut().find(|directive| directive.target == target) {
            Some(directive) => directive.level = level,
            None => self.directives.push(Directive { target, level }),
        }
        self
    }

    /// Returns whether the event passes this filter.
    pub fn accepts(&self, event: &LogEvent) -> bool {
        match self.level_for(event.target()) {
            Some(level) => event.level().is_at_least(level),
            None => false,
        }
    }

    /// Returns the minimum level that applies to `target`.
    ///
    /// `None` means the target is dropped because no directive matched and no global level is
    /// configured.
    pub fn level_for(&self, target: &str) -> Option<Level> {
        let mut matched: Option<&Directive> = None;
        for directive in &self.directives {
            if !target_matches(&directive.target, target) {
                continue;
            }
            if matched.is_none_or(|current| directive.target.len() > current.target.len()) {
                matched = Some(directive);
            }
        }
        matched.map(|directive| directive.level).or(self.default)
    }

    /// Returns the global level, or `None` when only target directives are configured.
    pub fn min_level(&self) -> Option<Level> {
        self.default
    }

    /// Parse a comma-separated filter description.
    ///
    /// `level` sets the global level, `target=level` sets one target level, and a bare `target`
    /// means `trace` for that target. Empty directives, empty targets or levels, unknown level
    /// names, and repeated targets or global levels are rejected rather than silently ignored.
    pub fn parse(text: &str) -> Result<Self, FilterParseError> {
        if text.trim().is_empty() {
            return Err(FilterParseError::Empty);
        }

        let mut default = None;
        let mut directives: Vec<Directive> = Vec::new();

        for (index, raw) in text.split(',').enumerate() {
            let directive = raw.trim();
            if directive.is_empty() {
                return Err(FilterParseError::EmptyDirective { position: index + 1 });
            }

            let Some((target, level)) = directive.split_once('=') else {
                if let Some(level) = parse_level(directive) {
                    if default.replace(level).is_some() {
                        return Err(FilterParseError::DuplicateGlobalLevel { directive: directive.to_string() });
                    }
                }
                else {
                    push_target(&mut directives, directive, Level::Trace)?;
                }
                continue;
            };

            let target = target.trim();
            let level = level.trim();
            if target.is_empty() {
                return Err(FilterParseError::EmptyTarget { directive: directive.to_string() });
            }
            if level.is_empty() {
                return Err(FilterParseError::EmptyLevel { directive: directive.to_string() });
            }
            let level = parse_level(level)
                .ok_or_else(|| FilterParseError::UnknownLevel { directive: directive.to_string(), level: level.to_string() })?;
            push_target(&mut directives, target, level)?;
        }

        Ok(Self { default, directives })
    }

    /// Read a filter description from the environment variable named `variable`.
    ///
    /// A missing variable returns `None`. An invalid directive or a value that is not valid
    /// Unicode returns an error instead of falling back to a default level.
    pub fn from_env(variable: &str) -> Result<Option<Self>, FilterParseError> {
        let Some(value) = env::var_os(variable) else {
            return Ok(None);
        };
        let text = value.into_string().map_err(|_| FilterParseError::NotUnicodeEnvironment { variable: variable.to_string() })?;
        Self::parse(&text).map(Some)
    }
}

fn push_target(directives: &mut Vec<Directive>, target: &str, level: Level) -> Result<(), FilterParseError> {
    if directives.iter().any(|directive| directive.target == target) {
        return Err(FilterParseError::DuplicateTarget { target: target.to_string() });
    }
    directives.push(Directive { target: target.to_string(), level });
    Ok(())
}

fn target_matches(rule: &str, target: &str) -> bool {
    target == rule || target.strip_prefix(rule).is_some_and(|rest| rest.starts_with("::"))
}

fn parse_level(name: &str) -> Option<Level> {
    if name.eq_ignore_ascii_case("trace") {
        Some(Level::Trace)
    }
    else if name.eq_ignore_ascii_case("debug") {
        Some(Level::Debug)
    }
    else if name.eq_ignore_ascii_case("info") {
        Some(Level::Info)
    }
    else if name.eq_ignore_ascii_case("warn") {
        Some(Level::Warn)
    }
    else if name.eq_ignore_ascii_case("error") {
        Some(Level::Error)
    }
    else {
        None
    }
}

/// Error returned when a filter description cannot be parsed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FilterParseError {
    /// The description contains no directive.
    Empty,
    /// A comma-separated directive is empty.
    EmptyDirective {
        /// One-based position of the empty directive.
        position: usize,
    },
    /// A `target=level` directive has an empty target.
    EmptyTarget {
        /// The offending directive.
        directive: String,
    },
    /// A `target=level` directive has an empty level.
    EmptyLevel {
        /// The offending directive.
        directive: String,
    },
    /// A directive names a level that does not exist.
    UnknownLevel {
        /// The offending directive.
        directive: String,
        /// The level name that was not recognized.
        level: String,
    },
    /// The same target is configured more than once.
    DuplicateTarget {
        /// The repeated target.
        target: String,
    },
    /// The global level is configured more than once.
    DuplicateGlobalLevel {
        /// The offending directive.
        directive: String,
    },
    /// The environment value is not valid Unicode.
    NotUnicodeEnvironment {
        /// Name of the environment variable that was read.
        variable: String,
    },
}

impl Display for FilterParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("the filter description contains no directive"),
            Self::EmptyDirective { position } => write!(f, "directive {position} is empty"),
            Self::EmptyTarget { directive } => write!(f, "directive `{directive}` has an empty target"),
            Self::EmptyLevel { directive } => write!(f, "directive `{directive}` has an empty level"),
            Self::UnknownLevel { directive, level } => write!(f, "directive `{directive}` names an unknown level `{level}`"),
            Self::DuplicateTarget { target } => write!(f, "target `{target}` is configured more than once"),
            Self::DuplicateGlobalLevel { directive } => write!(f, "the global level is configured more than once, at `{directive}`"),
            Self::NotUnicodeEnvironment { variable } => write!(f, "the value of `{variable}` is not valid Unicode"),
        }
    }
}

impl Error for FilterParseError {}
