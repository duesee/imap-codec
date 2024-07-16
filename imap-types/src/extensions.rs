//! IMAP extensions.

pub mod binary;
pub mod compress;
pub mod enable;
pub mod idle;
#[cfg(feature = "ext_metadata")]
pub mod metadata;
pub mod r#move;
pub mod quota;
#[cfg(feature = "ext_sort_thread")]
pub mod sort;
#[cfg(feature = "ext_sort_thread")]
pub mod thread;
pub mod uidplus;
pub mod unselect;
