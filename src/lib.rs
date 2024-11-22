//! `stomp_parser` implements a model for STOMP Frames, as specified in the [STOMP Protocol Specification,Version 1.2](https://stomp.github.io/stomp-specification-1.2.html).
//! These frames can be parsed from and serialized to byte arrays.
//!
//! The types primarily of interest to users of the library are the enums `client::ClientFrame` and `server::ServerFrame`, which model the frames that can be sent
//! by STOMP clients and STOMP servers respectively. Obtaining a frame from a message is achieved via `try_from` on those types.   
//!
//! # Example
//! ```
//! use std::convert::TryFrom;
//!
//! use stomp_parser::client::ClientFrame;
//! use stomp_parser::headers::HeaderValue;
//!
//! let message = b"SEND\n\
//!                 destination:stairway/to/heaven\n\
//!                 \n\
//!                 Lorem ipsum dolor sit amet,...\x00"
//!                 .to_vec();
//!
//! if let Ok(ClientFrame::Send(frame)) = ClientFrame::try_from(message) {
//!     assert_eq!("stairway/to/heaven", frame.destination().value());
//!     assert_eq!(b"Lorem ipsum dolor sit amet,...", frame.body().unwrap());
//! } else {
//!     panic!("Send Frame not parsed correctly");
//! }
//! ```
#![warn(clippy::all)]
#[macro_use]
mod common;
pub mod error;
mod model;
mod parser;

pub use model::client;
pub use model::headers;
pub use model::server;
