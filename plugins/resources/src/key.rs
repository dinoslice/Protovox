use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use thiserror::Error;
use crate::resource_type::ResourceType;

#[derive(Error, Debug)]
pub enum ResourceKeyParseFail {
    #[error("Failed to split the identifier from the key and domain")]
    IdentifierSplitFail,
    #[error("Failed to split the domain from the key")]
    DomainSplitFail,
    #[error("The domain in the string does not match the type given")]
    InvalidDomain
}

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct ResourceKey<T: ResourceType> {
    id: String,
    key: String,
    _phantom: PhantomData<T>
}

impl<T: ResourceType> ResourceKey<T> {
    pub fn new(id: impl Into<String>, key: impl Into<String>) -> ResourceKey<T> {
        Self {
            id: id.into(),
            key: key.into(),
            _phantom: PhantomData::<T>
        }
    }
    
    pub fn id(&self) -> &String {
        &self.id
    }
    
    pub fn key(&self) -> &String {
        &self.key
    }
}

impl<T: ResourceType> Display for ResourceKey<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}/{}", self.id, T::resource_name(), self.key)
    }
}
impl<T: ResourceType> Debug for ResourceKey<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ResourceKey<{}> {{ {}, {} }}", T::resource_name(), self.id, self.key)
    }
}
impl<T: ResourceType> TryFrom<&String> for ResourceKey<T> {
    type Error = ResourceKeyParseFail;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        let id_split = value.split(":").collect::<Vec<_>>();
        if id_split.len() != 2 {
            return Err(ResourceKeyParseFail::IdentifierSplitFail)
        }
        
        let domain_split = id_split[1].split("/").collect::<Vec<_>>();
        if domain_split.len() > 2 {
            return Err(ResourceKeyParseFail::DomainSplitFail)
        }
        
        match domain_split.len() { 
            1 => Ok(Self {
                id: id_split[0].into(),
                key: domain_split[0].into(),
                _phantom: PhantomData::<T>
            }),
            2 => {
                if domain_split[0] != T::resource_name() {
                    Err(ResourceKeyParseFail::InvalidDomain)
                } else {
                    Ok(Self {
                        id: id_split[0].into(),
                        key: domain_split[1].into(),
                        _phantom: PhantomData::<T>
                    })
                }
            },
            _ => Err(ResourceKeyParseFail::DomainSplitFail)
        }
    }
}