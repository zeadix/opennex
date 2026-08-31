//! Remote phone control: embedded HTTP server, snapshot sharing and
//! command queue plumbing.

pub mod ansi;
pub mod protocol;
pub mod server;
pub mod ws;

/// The phone web page, embedded at compile time.
pub(crate) const REMOTE_PAGE: &str = include_str!("../../assets/remote.html");
