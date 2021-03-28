use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::Hasher;
use std::vec::Vec;

pub type ID = u64;
pub type Email = String;

pub struct TreeStore {
    pub emails: HashMap<Email, ID>,
    pub users: HashMap<ID, User>,
    pub contents: HashMap<ID, Vec<u8>>,
}

impl TreeStore {
    pub fn new() -> TreeStore {
        TreeStore {
            emails: HashMap::<Email, ID>::new(),
            users: HashMap::<ID, User>::new(),
            contents: HashMap::<ID, Vec<u8>>::new(),
        }
    }

    // Not a stable algorithm. Can change results for the same email address
    // between Rust versions.
    pub fn make_id(email: &str) -> ID {
        let mut hasher = DefaultHasher::new();
        hasher.write(email.as_bytes());
        hasher.finish()
    }
}

pub struct User {
    id: ID,
    pwhash: Vec<u8>,
    pwsalt: Vec<u8>,
}
