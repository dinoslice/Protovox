use std::hash::{DefaultHasher, Hash, Hasher};
use rand::random;
use uuid::{NonNilUuid, Uuid};

fn main() {
    let id = Identity::create("joshua", "password123");

    dbg!(id.verify("joshua", "password123"));
}

#[derive(Eq, PartialEq)]
pub struct Identity(pub NonNilUuid);

impl Identity {
    // TODO: make generic for any hasher
    // TODO: hash into u128?
    pub fn create(username: &str, encrypted_pwd: &str) -> Self {
        let uuid = Self::hash_inner(random(), username, encrypted_pwd);

        Self(NonNilUuid::new(Uuid::from_u128(uuid)).expect("cannot be null"))
    }

    pub fn verify(&self, username: &str, encrypted_pwd: &str) -> bool {
        let self_u128 = self.0.get().as_u128();

        let hash = Self::hash_inner(self_u128 as u64, username, encrypted_pwd);

        self_u128 == hash
    }

    fn hash_inner(rand: u64, username: &str, encrypted_pwd: &str) -> u128 {
        let mut hasher = DefaultHasher::new();

        username.hash(&mut hasher);
        encrypted_pwd.hash(&mut hasher);

        (rand as u32).hash(&mut hasher);

        let hash = hasher.finish();

        let full = ((hash as u128) << 64) | (rand as u128);

        full & 0xFFFFFFFFFFFFcFFFBFFFFFFFFFFFFFFF | 0x000000000000c0008000000000000000
    }
}