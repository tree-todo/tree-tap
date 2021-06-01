use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::Hasher;

use argon2;

pub type ID = u64;
pub type Email = String;

pub struct TreeStore {
    pub emails: HashMap<Email, ID>,
    pub users: HashMap<ID, User>,
    pub tasks: HashMap<ID, serde_json::Value>,
}

impl TreeStore {
    pub fn new() -> TreeStore {
        TreeStore {
            emails: HashMap::<Email, ID>::new(),
            users: HashMap::<ID, User>::new(),
            tasks: HashMap::<ID, serde_json::Value>::new(),
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

const SALT: &[u8] = b"tree-tap";

pub struct User {
    pub pwhash: String,
}

impl User {
    pub fn new(password: &str) -> User {
        let config = argon2::Config::default();
        let pwhash = argon2::hash_encoded(password.as_bytes(), SALT, &config).unwrap();
        User { pwhash }
    }

    pub fn verify_pw(&self, password: &str) -> bool {
        argon2::verify_encoded(&self.pwhash, password.as_bytes()).unwrap()
    }
}
