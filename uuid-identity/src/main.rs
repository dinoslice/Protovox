use std::hash::{DefaultHasher, Hash, Hasher};
use argon2::Argon2;
use uuid::{NonNilUuid, Uuid};

const UUID_MASK: u128 = 0xFFFFFFFFFFFFcFFFBFFFFFFFFFFFFFFF;
const UUID_SET: u128 = 0x000000000000c0008000000000000000;

fn main() {
    let id = Identity::create("j", "p");
    dbg!(id.0.get());
    dbg!(id.verify("j", "p"));
}

#[derive(Eq, PartialEq)]
pub struct Identity(pub NonNilUuid);

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
    pub fn create(username: &str, password: &str) -> Self {
        let uuid = Self::hash_inner(IdRand::rand(), username, password);

        Self(NonNilUuid::new(Uuid::from_u128(uuid)).expect("cannot be null"))
    }

    pub fn verify(&self, username: &str, password: &str) -> bool {
        let self_u128 = self.0.get().as_u128();

        let hash = Self::hash_inner(IdRand::new(self_u128 as u64), username, password);

        self_u128 == hash
    }

    fn hash_inner(rand: IdRand, username: &str, password: &str) -> u128 {
        let password = Self::hash_pwd(password, rand);

        let rand = rand.get();

        let mut hasher = DefaultHasher::new();

        username.hash(&mut hasher);
        password.hash(&mut hasher);

        let hash = hasher.finish();

        let full = ((hash as u128) << 64) | (rand as u128);

        full & UUID_MASK | UUID_SET
    }

    fn hash_pwd(password: &str, rand: IdRand) -> [u8; 32] {
        let argon2 = Argon2::default();

        let mut pwd_hash = [0; 32];

        use argon2::Error as E;

        match argon2.hash_password_into(password.as_bytes(), &rand.bytes(), &mut pwd_hash) {
            Ok(_) => pwd_hash,
            Err(err) => match err {
                E::AdTooLong | E::AlgorithmInvalid | E::KeyIdTooLong | E::VersionInvalid |
                E::SecretTooLong | E::MemoryTooLittle | E::MemoryTooMuch |
                E::ThreadsTooFew | E::ThreadsTooMany | E::TimeTooSmall => unreachable!("cannot happen with default params"),
                E::B64Encoding(_) => unreachable!("PHC strings are never used"),
                E::OutputTooShort | E::OutputTooLong => unreachable!("32 byte buffer is valid"),
                E::SaltTooShort | E::SaltTooLong => unreachable!("salt should be a u64, which is 8 bytes"),
                E::PwdTooLong => unreachable!("very unlikely + length should be constrained"),
            }
        }
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
            let id = Identity::create(&username, &password);

            assert!(id.verify(&username, &password));

            assert!(!id.verify(&random_string(5..=32), &password));
            assert!(!id.verify(&username, &random_string(8..=64)));
        }
    }
}