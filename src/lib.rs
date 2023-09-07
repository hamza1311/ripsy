#[cfg(feature = "server")]
mod axum_bincode;

#[path = "macro_helpers.rs"]
#[doc(hidden)]
pub mod __macro_helpers;
#[cfg(feature = "client")]
pub mod client;

pub use ripsy_macros::{endpoint, ripsy};
use serde::Serialize;

#[derive(Debug, Clone, Copy)]
pub enum EndpointType {
    Query,
    Mutation,
}

pub struct Bincode<T>(pub T);

impl<T: Serialize> Bincode<T> {
    pub fn serialize(&self) -> bincode::Result<Vec<u8>> {
        bincode::serialize(&self.0)
    }
}
