#![allow(dead_code)]

use std::io::Read;
use std::error::Error;
use std::fs::File;
use std::path::PathBuf;
use std::fmt;
use byteorder::{ReadBytesExt, BigEndian};

use error::ShitError;

struct Hash([u8;20]);

impl fmt::LowerHex for Hash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let hash = self.0.iter()
            .map(|x| { return format!("{:x}", x) })
            .fold("".to_owned(), |mut acc, x| { acc.push_str(&x); acc });

        write!(f, "{}", hash)?;
        Ok(())
    }
}

pub fn read_pack(mut path : PathBuf, pack_name : &str) -> Result<Vec<String>, Box<Error>> {
    path.push("objects");
    path.push("pack");
    path.push(pack_name);
    path.set_extension("idx");

    let mut file = File::open(path)?;
    let mut signature = [0; 4];
    file.read(&mut signature)?;

    if !signature.eq(&[255, 116, 79, 99]) {
        return ShitError::as_result("Pack idx header incorrect");
    }

    let version = file.read_i32::<BigEndian>().expect("Failed to read in pack idx version number");
    if version != 2 {
        return ShitError::as_result("Version not 2!?!");
    }

    // layer 1, fanout table
    let mut table = [0; 256];
    for n in 0..256 {
        table[n] = file.read_i32::<BigEndian>().unwrap();
    }
    let count = table[255];

    // layer 2, 20 byte object names
    let mut names : Vec<String> = Vec::new();
    let mut single = [0; 20];
    for _ in 0..count {
        file.read(&mut single)?;
        let hash = format!("{:x}", Hash(single));
        names.push(hash);
    }

    return Ok(names)
}
