#[cfg(test)]
use include_dir::Dir;

pub mod agi_version;
pub mod color_palettes;
pub mod compression;
pub mod data_encoding;
pub mod logic;
pub mod object_list;
pub mod project;
pub mod resources;
pub mod views;
pub mod word_list;
pub mod xor_encryption;

mod buffer;

#[cfg(test)]
pub static TEST_DATA_DIR: Dir<'_> = include_dir::include_dir!("$CARGO_MANIFEST_DIR/test_data");
