#![allow(dead_code)]

extern crate crypto;
extern crate flate2;

use crypto::sha1::Sha1;
use crypto::digest::Digest;

use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path,PathBuf};


fn compress(input : &[u8]) -> std::io::Result<Vec<u8>> {
    use flate2::Compression;
    use flate2::write::ZlibEncoder;

    let mut e = ZlibEncoder::new(Vec::new(), Compression::Default);
    let _ = e.write(input);
    return e.finish();
}

fn decompress(input : &[u8]) -> Vec<u8> {
    use flate2::read::ZlibDecoder;

    let mut decoder = ZlibDecoder::new(&input[..]);
    let mut ret = Vec::new();
    decoder.read_to_end(&mut ret).unwrap();
    return ret;
}

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

fn write_file(base_dir : &str, content : &[u8], hash : &str) -> std::io::Result<()> {
    let dir = &hash[0..2];
    let file = &hash[2..];
    let mut path = PathBuf::from(base_dir);

    path.push(dir);
    fs::create_dir_all(&path)?;

    path.push(file);
    let mut file = File::create(&path)?;
    file.write_all(content)?;
    return Ok(());
}

fn read_file(path : &Path) -> std::io::Result<([u8;4096], usize)> {
    let mut file = File::open(path)?;
    let mut buf = [0; 4096];
    let size = file.read(&mut buf)?;
    return Ok((buf, size));
}

fn main() {
    for argument in env::args().skip(1) {
        println!("Attempting to read {}", argument);
        let path = Path::new(&argument);
        let (buf, size) = read_file(path).unwrap();
        println!("Got {} bytes", size);

        let(content, hash) = pack_blob(&buf, size);

        println!("Got hash {}", hash);
        let compressed = compress(&content).unwrap();

        write_file(".", &compressed, &hash).unwrap();
    }
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

#[test]
fn compress_decompress_round_trip() {
    use std::str;

    let input = "the quick brown fox jumps over the lazy dog";
    let compressed = compress(input.as_bytes()).unwrap();
    let decompressed = decompress(&compressed);
    let output = str::from_utf8(&decompressed).unwrap();
    assert_eq!(output, input);
}
