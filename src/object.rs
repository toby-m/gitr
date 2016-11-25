use std::error::Error;
use std::fmt;
use std::fmt::{Display,Formatter};

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ObjectType {
    Blob,
    Commit,
    Tree
}
impl Display for ObjectType {
    fn fmt(&self, f : &mut Formatter) -> fmt::Result {
        match *self {
            ObjectType::Blob => write!(f, "blob"),
            ObjectType::Tree => write!(f, "tree"),
            ObjectType::Commit => write!(f, "commit"),
        }
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ParseError;
impl Display for ParseError {
    fn fmt(&self, f : &mut Formatter) -> fmt::Result {
        write!(f, "Failed to read header")
    }
}
impl Error for ParseError {
    fn description(&self) -> &str { return "Failed to read header" }
    fn cause(&self) -> Option<&Error> { return None; }
}

pub struct Object {
    pub object_type : ObjectType,
    pub length : usize,
    offset : usize,
    raw : Box<[u8]>
}

impl Object {
    fn new(object_type : ObjectType, length : usize, raw : Box<[u8]>, offset : usize) -> Object {
        return Object {
            object_type: object_type,
            length : length,
            raw : raw,
            offset : offset
        };
    }

    pub fn data(&self) -> &[u8] {
        return &self.raw[self.offset..];
    }

    pub fn from_raw(data : Box<[u8]>) -> Result<Object, Box<Error>> {
        use std::str;

        let (object_type, length, offset) = {
            let header_bytes = data.iter().take_while(|&c| *c != 0).map(|&c|c).collect::<Vec<u8>>();
            let header = str::from_utf8(&header_bytes)?;
            let parts = header.split_whitespace().collect::<Vec<&str>>();
            assert!(parts.len() == 2, "Unable to read header");

            let length = parts[1].parse::<usize>()?;
            let object_type = match parts[0] {
                "blob"   => ObjectType::Blob,
                "commit" => ObjectType::Commit,
                "tree"   => ObjectType::Tree,
                _        => return Err(Box::new(ParseError))
            };
            let offset = &parts[0].len() + &parts[1].len() + 2;

            (object_type, length, offset)
        };

        return Ok(Object::new(object_type, length, data, offset))
    }
}
