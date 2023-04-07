//! Distribution parameter models and their sampling strategies.
//!
//! A distribution is a concrete, immutable parameter object. It holds the sample space, the
//! support set, and the parameter constraints, but no random state: it never owns a stream, never
//! reads a source during construction, and never caches the previous sample. Sampling is an
//! inherent method that consumes an arbitrary [`RandomSource`](crate::RandomSource), so the
//! deterministic stream and the host entropy handle share one algorithm.
//!
//! Every sampling entry returns [`Result`](core::result::Result). Invalid parameters fail during
//! construction, before any byte is consumed, and a source failure is reported instead of being
//! replaced by a zero sample.

mod bernoulli;
mod categorical;
mod uniform;
mod uniform_float;

pub use bernoulli::Bernoulli;
pub use categorical::Categorical;
pub use uniform::{UniformI32, UniformI64, UniformU32, UniformU64};
pub use uniform_float::{UniformF32, UniformF64};
