use std::fs;
use std::fs::File;
use std::error::Error;
use std::io::{Read,Write};
use std::path::{Path,PathBuf};

pub fn compress(input : &[u8]) -> Result<Vec<u8>, Box<Error>> {
    use flate2::Compression;
    use flate2::write::ZlibEncoder;

    let mut e = ZlibEncoder::new(Vec::new(), Compression::Default);
    let _ = e.write(input);
    let res = e.finish()?;
    return Ok(res)
}

pub fn decompress(input : &[u8]) -> Result<Vec<u8>, Box<Error>> {
    use flate2::read::ZlibDecoder;

    let mut decoder = ZlibDecoder::new(&input[..]);
    let mut ret = Vec::new();
    decoder.read_to_end(&mut ret)?;
    return Ok(ret);
}

pub fn write_object(base_dir : &str, content : &[u8], hash : &str) -> Result<(), Box<Error>> {
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

pub fn read_file(path : &Path) -> Result<([u8;4096], usize), Box<Error>> {
    let mut file = File::open(path)?;
    let mut buf = [0; 4096];
    let size = file.read(&mut buf)?;
    return Ok((buf, size));
}

#[test]
fn compress_decompress_round_trip() {
    use std::str;

    let input = "the quick brown fox jumps over the lazy dog";
    let compressed = compress(input.as_bytes()).unwrap();
    let decompressed = decompress(&compressed).unwrap();
    let output = str::from_utf8(&decompressed).unwrap();
    assert_eq!(output, input);
}
