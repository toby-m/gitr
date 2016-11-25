#![allow(dead_code)]

extern crate crypto;
extern crate flate2;

mod file;
mod object;

use crypto::sha1::Sha1;
use crypto::digest::Digest;

use std::env;
use std::io::prelude::*;
use std::path::{Path,PathBuf};
use std::error::Error;

use object::{Object};

fn pack_blob(content : &[u8], len : usize) -> (Vec<u8>, String) {
    let blah = format!("blob {}\0", len);
    let header = blah.as_bytes();

    let mut pack = Vec::new();
    pack.extend_from_slice(&header);
    pack.extend_from_slice(&content[0..len]);

    let mut hasher = Sha1::new();
    hasher.input(&pack);

    return (pack, hasher.result_str());
}

fn read_object(base_dir : &str, hash : &str) -> Result<Object, Box<Error>> {
    let dir = &hash[0..2];
    let file = &hash[2..];
    let mut path = PathBuf::from(base_dir);
    path.push(dir);
    path.push(file);

    let (file_content, _) = file::read_file(&path)?;
    let data = file::decompress(&file_content)?;
    return object::Object::from_raw(data.into_boxed_slice());
}

fn main() {
    for argument in env::args().skip(1) {
        let path = Path::new(&argument);
        let (buf, size) = file::read_file(path).unwrap();
        let(content, hash) = pack_blob(&buf, size);
        let compressed = file::compress(&content).unwrap();

        file::write_object(".", &compressed, &hash).unwrap();
    }

    let obj = read_object(".git/objects", "4bd19e65663b008a7bc37643b53a834fa747a5bd").unwrap();
    println!("Found {} with length {}", obj.object_type, obj.length);
    std::io::stdout().write(obj.data()).unwrap();
}

#[test]
fn hash_blob_works_for_simple_input() {
    let s = "test content\n";
    let (_, hex) = pack_blob(s.as_bytes(), s.len());
    assert_eq!(hex, "d670460b4b4aece5915caf5c68d12f560a9fe3e4");
}

#[test]
fn hash_blob_works_for_simple_input2() {
    let s = "one\n";
    let (_, hex) = pack_blob(s.as_bytes(), s.len());
    assert_eq!(hex, "5626abf0f72e58d7a153368ba57db4c673c0e171");
}
