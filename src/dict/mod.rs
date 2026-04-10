//! Dictionary processing pipeline.

mod core;
mod index;
mod main;
mod other;
mod release;
mod scan;
mod writer;

pub use core::*;
pub use main::DMain;
pub use other::*;
pub use release::{make_dict_from_db, release};
pub use scan::scan;
