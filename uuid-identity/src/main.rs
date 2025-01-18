use std::hash::{DefaultHasher, Hash, Hasher};
use argon2::Argon2;
use uuid::{NonNilUuid, Uuid};

const UUID_MASK: u128 = 0xFFFFFFFFFFFFcFFFBFFFFFFFFFFFFFFF;
const UUID_SET: u128 = 0x000000000000c0008000000000000000;

fn main() {
    let id = Identity::create("joshua", "password123").expect("no argon err");
    dbg!(id.0.get());
    dbg!(id.verify("joshua", "password123").expect("no argon err"));
}

#[derive(Eq, PartialEq)]
pub struct Identity(pub NonNilUuid);

#[derive(Debug, Clone, Hash)]
struct IdentityPassword {
    pub pwd_hash: [u8; 56],
    pub rand_salt: IdRand,
}

impl IdentityPassword {
    pub fn new(password: &str) -> Result<Self, argon2::Error> {
        Self::with_rand_salt(password, IdRand::rand())
    }

    pub(crate) fn with_rand_salt(password: &str, rand_salt: IdRand) -> Result<Self, argon2::Error> {
        let argon2 = Argon2::default();

        let mut pwd_hash = [0; 56];

        argon2.hash_password_into(password.as_bytes(), &rand_salt.bytes(), &mut pwd_hash)?;

        Ok(Self { pwd_hash, rand_salt })
    }
}

#[derive(Debug, Clone, Hash, Copy, Eq, PartialEq)]
struct IdRand(u64);

impl IdRand {
    pub fn rand() -> Self {
        Self::new(rand::random())
    }

    pub fn new(n: u64) -> Self {
        Self(n & (UUID_MASK as u64) | (UUID_SET as u64))
    }

    pub fn get(&self) -> u64 {
        self.0
    }

    pub fn bytes(&self) -> [u8; size_of::<u64>()] {
        self.get().to_le_bytes()
    }
}

impl Identity {
    // TODO: make generic for any hasher
    // TODO: hash into u128?
    pub fn create(username: &str, password: &str) -> Result<Self, argon2::Error> {
        let uuid = Self::hash_inner(IdRand::rand(), username, password)?;

        Ok(Self(NonNilUuid::new(Uuid::from_u128(uuid)).expect("cannot be null")))
    }

    pub fn verify(&self, username: &str, password: &str) -> Result<bool, argon2::Error> {
        let self_u128 = self.0.get().as_u128();

        let hash = Self::hash_inner(IdRand::new(self_u128 as u64), username, password)?;

        Ok(self_u128 == hash)
    }

    fn hash_inner(rand: IdRand, username: &str, password: &str) -> Result<u128, argon2::Error> {
        let password = IdentityPassword::with_rand_salt(password, rand)?;

        let rand = rand.get();

        let mut hasher = DefaultHasher::new();

        username.hash(&mut hasher);
        password.hash(&mut hasher);

        rand.hash(&mut hasher);

        let hash = hasher.finish();

        let full = ((hash as u128) << 64) | (rand as u128);

        Ok(full & 0xFFFFFFFFFFFFcFFFBFFFFFFFFFFFFFFF | 0x000000000000c0008000000000000000)
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
        for _ in 0..20 {
            let username: String = random_string(5..=32);

            let password: String = random_string(8..=64);
            let id = Identity::create(&username, &password).expect("no argon err");

            assert!(id.verify(&username, &password).expect("no argon err"));

            assert!(!id.verify(&random_string(5..=32), &password).expect("no argon err"));
            assert!(!id.verify(&username, &random_string(8..=64)).expect("no argon err"));
        }
    }
}