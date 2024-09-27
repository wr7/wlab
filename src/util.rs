mod intersperse;
pub use intersperse::Intersperse;

pub(crate) use binary_search_map::BinarySearchMap;
pub(crate) use hashmap_ext::HashMapExt;
pub(crate) use max_vec::MaxVec;
pub(crate) use maybe_vec::MaybeVec;
pub(crate) use memory_store::MemoryStore;
pub(crate) use push_vec::PushVec;
pub(crate) use slice_ext::SliceExt;

mod binary_search_map;
mod hashmap_ext;
mod max_vec;
mod maybe_vec;
mod memory_store;
mod push_vec;
mod slice_ext;

/// Gets the line and column number of a byte in some text
pub fn line_and_col(src: &str, byte_position: usize) -> (usize, usize) {
    (
        line_number(src, byte_position),
        column_number(src, byte_position),
    )
}

/// Gets the line number of a byte in some text
fn line_number(src: &str, byte_position: usize) -> usize {
    let mut line_no = 1;

    for (i, c) in src.char_indices() {
        if i >= byte_position {
            break;
        } else if c == '\n' {
            line_no += 1;
        }
    }

    line_no
}

/// Gets the column number of a byte in some text
fn column_number(src: &str, byte_position: usize) -> usize {
    let mut col_no = 1;

    for (i, c) in src.char_indices() {
        if i >= byte_position {
            break;
        }

        if c == '\n' {
            col_no = 1;
        } else {
            col_no += 1;
        }
    }

    col_no
}
