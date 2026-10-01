use crate::Validation;
use alloc::{vec, vec::Vec};
use Validation::{Failure, Success};

/// Convert standard result-like types into [`Validation`].
pub trait Validate<T, E> {
    /// Convert `self` into a [`Validation`], treating absence or error as fatal.
    fn valid(self) -> Validation<T, E>;
    /// Convert `self` into a [`Validation`] and append collected diagnostics to `errors`.
    fn validate(self, errors: &mut Vec<E>) -> Validation<T, E>
    where
        E: Clone;
    /// Convert `self` into a success [`Validation`], recording recoverable errors in `errors`.
    fn recover(self, errors: &mut Vec<E>) -> Validation<T, E>
    where
        T: Default;
}

impl<T, E> Validate<T, E> for Option<T>
where
    E: From<()>,
{
    fn valid(self) -> Validation<T, E> {
        match self {
            Some(s) => Success { value: s, diagnostics: vec![] },
            None => Failure { fatal: E::from(()), diagnostics: vec![] },
        }
    }

    fn validate(self, errors: &mut Vec<E>) -> Validation<T, E>
    where
        E: Clone,
    {
        match self {
            Some(s) => Success { value: s, diagnostics: vec![] },
            None => Failure { fatal: E::from(()), diagnostics: errors.clone() },
        }
    }

    fn recover(self, errors: &mut Vec<E>) -> Validation<T, E>
    where
        T: Default,
    {
        match self {
            Some(s) => Success { value: s, diagnostics: vec![] },
            None => {
                errors.push(E::from(()));
                Success { value: T::default(), diagnostics: vec![] }
            }
        }
    }
}
impl<T, E, F> Validate<T, E> for Result<T, F>
where
    E: From<F>,
{
    fn valid(self) -> Validation<T, E> {
        match self {
            Ok(o) => Success { value: o, diagnostics: vec![] },
            Err(e) => Failure { fatal: E::from(e), diagnostics: vec![] },
        }
    }

    fn validate(self, errors: &mut Vec<E>) -> Validation<T, E>
    where
        E: Clone,
    {
        match self {
            Ok(o) => Success { value: o, diagnostics: vec![] },
            Err(e) => Failure { fatal: E::from(e), diagnostics: errors.clone() },
        }
    }

    fn recover(self, errors: &mut Vec<E>) -> Validation<T, E>
    where
        T: Default,
    {
        match self {
            Ok(s) => Success { value: s, diagnostics: vec![] },
            Err(e) => {
                errors.push(E::from(e));
                Success { value: T::default(), diagnostics: vec![] }
            }
        }
    }
}

impl<T, E, F> Validate<T, E> for Validation<T, F>
where
    E: From<F>,
{
    fn valid(self) -> Validation<T, E> {
        match self {
            Success { value, diagnostics } => Success { value, diagnostics: diagnostics.into_iter().map(E::from).collect() },
            Failure { fatal, diagnostics } => {
                Failure { fatal: E::from(fatal), diagnostics: diagnostics.into_iter().map(E::from).collect() }
            }
        }
    }

    fn validate(self, errors: &mut Vec<E>) -> Validation<T, E>
    where
        E: Clone,
    {
        match self {
            Success { value, diagnostics } => {
                errors.extend(diagnostics.into_iter().map(E::from));
                Success { value, diagnostics: vec![] }
            }
            Failure { fatal, diagnostics } => {
                errors.extend(diagnostics.into_iter().map(E::from));
                Failure { fatal: E::from(fatal), diagnostics: errors.clone() }
            }
        }
    }

    fn recover(self, errors: &mut Vec<E>) -> Validation<T, E>
    where
        T: Default,
    {
        match self {
            Success { value, diagnostics } => {
                errors.extend(diagnostics.into_iter().map(E::from));
                Success { value, diagnostics: vec![] }
            }
            Failure { fatal, diagnostics } => {
                errors.extend(diagnostics.into_iter().map(E::from));
                errors.push(E::from(fatal));
                Success { value: T::default(), diagnostics: vec![] }
            }
        }
    }
}
