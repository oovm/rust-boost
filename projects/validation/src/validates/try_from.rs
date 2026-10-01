use super::*;
use alloc::vec;
use core::convert::Infallible;

use core::ops::{ControlFlow, FromResidual, Residual, Try};

impl<T, F, E> FromResidual<Result<Infallible, E>> for Validation<T, F>
where
    F: From<E>,
{
    fn from_residual(residual: Result<Infallible, E>) -> Self {
        match residual {
            Ok(_) => unreachable!(),
            Err(e) => Failure { fatal: e.into(), diagnostics: vec![] },
        }
    }
}

impl<T, E, A> FromResidual<Validation<A, E>> for Validation<T, E> {
    fn from_residual(residual: Validation<A, E>) -> Self {
        match residual {
            Success { .. } => unreachable!(),
            Failure { fatal, diagnostics } => Failure { fatal, diagnostics },
        }
    }
}

impl<T, F> Residual<T> for Validation<Infallible, F> {
    type TryType = Validation<T, F>;
}

impl<T, F> Try for Validation<T, F> {
    type Output = T;
    type Residual = Validation<Infallible, F>;

    fn from_output(output: Self::Output) -> Self {
        Success { value: output, diagnostics: vec![] }
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Success { value, diagnostics: _ } => ControlFlow::Continue(value),
            Failure { fatal, diagnostics } => ControlFlow::Break(Failure { fatal, diagnostics }),
        }
    }
}
