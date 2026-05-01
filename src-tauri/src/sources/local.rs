//! Local filesystem source: thin wrapper around [`crate::scan::list_local`].
//!
//! The local source has no sync step; it relies on `notify`-based filesystem
//! watching wired in [`crate::scan::start`].
