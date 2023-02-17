use diagnostic::terminal::{structured_to_terminal, Config, SourceRegistry, StructuredRenderError};
use source_cache::SourceCache;

use crate::convert::{qerror_to_structured, source_ref_for_id};
use crate::{QError, QResult};

/// Print quick errors through the structured terminal renderer.
pub fn print_errors(cache: &SourceCache, errors: &[QError]) -> QResult {
    let mut registry = SourceRegistry::new();
    for error in errors {
        if let Some(file) = error.source_id() {
            registry.register(&source_ref_for_id(file), file.clone());
        }
        if let QErrorKind::Syntax(syntax) = &*error.error {
            if syntax.file != SourceID::default() {
                registry.register(&source_ref_for_id(&syntax.file), syntax.file.clone());
            }
        }
    }

    let config = Config::default();
    for error in errors {
        let diagnostic = qerror_to_structured(error);
        structured_to_terminal(&diagnostic, &registry, config)
            .map_err(structured_render_error)?
            .eprint(cache)?;
    }
    Ok(())
}

use crate::QErrorKind;
use source_cache::SourceID;

fn structured_render_error(error: StructuredRenderError) -> QError {
    QError::runtime_error(error.to_string())
}
