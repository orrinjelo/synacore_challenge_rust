// pub mod event;

use std::fs::File;
use std::io::Read;

pub fn get_file_as_byte_vec(filename: &String) -> Vec<u8> {
    let mut f = File::open(&filename).expect("no file found");
    let mut buffer: Vec<u8> = vec![0; 32000];
    f.read(&mut buffer).expect("buffer overflow");

    buffer
}
