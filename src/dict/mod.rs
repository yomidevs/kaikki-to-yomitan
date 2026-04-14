//! Dictionary processing pipeline.

mod core;
mod index;
mod main;
mod other;
mod release;
mod scan;
mod writer;

pub use core::{Dictionary, Intermediate, Langs, make_dict_from_jsonl};
use core::{LabelledYomitanEntries, LangCodeProbe, iter_datasets};

// Dictionary types
pub use main::DMain;
pub use other::{DGlossary, DGlossaryExtended, DIpa, DIpaMerged};

pub use release::{make_dict_from_db, release};
pub use scan::scan;
pub use writer::WriterFormat;
