use std::hash::{DefaultHasher, Hash, Hasher};
use uuid::{NonNilUuid, Uuid};

fn main() {
    let id = Identity::create("joshua", "password123");

    dbg!(id.verify("joshua", "password123"));
}

#[derive(Eq, PartialEq)]
pub struct Identity(pub NonNilUuid);

impl Identity {
    const UUID_MASK: u128 = 0xFFFFFFFFFFFFcFFFBFFFFFFFFFFFFFFF;
    const UUID_SET: u128 = 0x000000000000c0008000000000000000;

    // TODO: make generic for any hasher
    // TODO: hash into u128?
    pub fn create(username: &str, encrypted_pwd: &str) -> Self {
        let uuid = Self::hash_inner(Self::gen_rand(), username, encrypted_pwd);

        Self(NonNilUuid::new(Uuid::from_u128(uuid)).expect("cannot be null"))
    }

    pub fn verify(&self, username: &str, encrypted_pwd: &str) -> bool {
        let self_u128 = self.0.get().as_u128();

        let hash = Self::hash_inner(self_u128 as u64, username, encrypted_pwd);

        self_u128 == hash
    }

    fn gen_rand() -> u64 {
        rand::random::<u64>() & (Self::UUID_MASK as u64) | (Self::UUID_SET as u64)
    }

    fn hash_inner(rand: u64, username: &str, encrypted_pwd: &str) -> u128 {
        let mut hasher = DefaultHasher::new();

        username.hash(&mut hasher);
        encrypted_pwd.hash(&mut hasher);

        rand.hash(&mut hasher);

        let hash = hasher.finish();

        let full = ((hash as u128) << 64) | (rand as u128);

        full & 0xFFFFFFFFFFFFcFFFBFFFFFFFFFFFFFFF | 0x000000000000c0008000000000000000
    }
}

#[cfg(test)]
mod tests {
    use rand::{rng, Rng};
    use rand::distr::uniform::SampleRange;
    use super::*;

    fn random_string(len: impl SampleRange<usize>) -> String {
        (0..rng().random_range(len))
            .map(|_| rng().sample(rand::distr::Alphanumeric))
            .map(char::from)
            .collect()
    }

    #[test]
    fn verify() {
        for _ in 0..100000 {
            let username: String = random_string(5..=32);

            let password: String = random_string(8..=64);
            let id = Identity::create(&username, &password);

            assert!(id.verify(&username, &password));

            assert!(!id.verify(&random_string(5..=32), &password));
            assert!(!id.verify(&username, &random_string(8..=64)));
        }
    }
}