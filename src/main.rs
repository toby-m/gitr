#![allow(dead_code)]

extern crate crypto;
extern crate flate2;

mod file;
mod object;

use crypto::sha1::Sha1;
use crypto::digest::Digest;

use std::str;
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

fn read_object(mut base_dir : PathBuf, hash : &str) -> Result<Object, Box<Error>> {
    let dir = &hash[0..2];
    let file = &hash[2..];
    base_dir.push("objects");
    base_dir.push(dir);
    base_dir.push(file);
    println!("Trying to read {:?} ", base_dir);

    let (file_content, _) = file::read_file(&base_dir)?;
    let data = file::decompress(&file_content)?;
    return object::Object::from_raw(data.into_boxed_slice());
}

fn find_git_dir(start : &str) -> Result<Box<PathBuf>, &str> {
    let mut path = PathBuf::from(start);
    loop {
        path.push(".git");
        match std::fs::metadata(&path) {
            Ok(_) => return Ok(Box::new(path)),
            Err(_) => ()
        };

        path.pop(); // <- .git
        if !path.pop() {
            return Err("Not in git dir")
        }
    }
}

enum Ref {
    Ref(String),
    Hash(String)
}

#[derive(Copy, Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ShitError;
impl std::fmt::Display for ShitError {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Failed to specify error")
    }
}
impl Error for ShitError {
    fn description(&self) -> &str { return "Failed to specify error" }
    fn cause(&self) -> Option<&Error> { return None; }
}

fn find_head(mut path : PathBuf) -> Result<Ref, Box<Error>> {
    path.push("HEAD");
    let (data, _) = file::read_file(&path)?;
    let contents = str::from_utf8(&data)?;

    if contents.starts_with("ref: ") {
        let r = contents[5..].lines().next().expect("Couldn't take a line out of the HEAD");
        return Ok(Ref::Ref(r.to_owned()));
    }

    if contents.len() == 40 {
        let r = contents.lines().next().expect("Couldn't take a line out of the HEAD");
        return Ok(Ref::Hash(r.to_owned()));
    }

    return Err(Box::new(ShitError))
}

fn lookup_ref(mut path : PathBuf, r : Ref) -> Result<String, Box<Error>> {
    let r = match r {
        Ref::Hash(hash) => return Ok(hash),
        Ref::Ref(r)     => r
    };

    path.push(r);
    let (data, _) = file::read_file(&path)?;
    let contents = str::from_utf8(&data).expect("Ref contents didn't read as UTF8");
    let trimmed = contents.lines().next().expect("Couldn't take a line out of the ref file");
    return Ok(trimmed.to_owned());
}

fn main() {
    let cwd = env::current_dir().unwrap();
    let path_result = cwd.to_str().unwrap();
    let git_dir = *find_git_dir(path_result).expect("Not it git dir?");

    for argument in env::args().skip(1) {
        let obj = match read_object(git_dir.clone(), &argument) {
            Err(e)  => panic!("Failed to read object with hash {}, error: {}", argument, e),
            Ok(obj) => obj
        };

        println!("Found {} {} ", obj.object_type, argument);
        println!("");
    }

    let reference = find_head(git_dir.clone()).expect("Didn't get a reference from HEAD");
    let commit = lookup_ref(git_dir.clone(), reference).expect("Didn't find the ref");

    println!("HEAD is currently at commit '{}'", commit);
    let obj = read_object(git_dir.clone(), &commit).expect("Unable to read object file");
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
