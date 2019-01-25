use std::io::{self, Read, Write};
use std::str;

use python_json_read_adapter::translate_slice;

fn main() {
    let mut buffer = vec![];
    io::stdin().read_to_end(&mut buffer).unwrap();
    let old_buffer = buffer.clone();

    translate_slice(&mut buffer[..]);
    assert_eq!(str::from_utf8(&buffer[..]), str::from_utf8(&old_buffer[..]));
}
