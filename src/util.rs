#![allow(dead_code)]

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

    return line_no;
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

    return col_no;
}
