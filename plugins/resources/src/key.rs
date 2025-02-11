use crate::resource_type::ResourceType;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ResourceKeyParseFail {
    #[error("Failed to split the identifier from the key and domain")]
    IdentifierSplitFail,
    #[error("Failed to split the domain from the key")]
    DomainSplitFail,
    #[error("The domain in the string does not match the type given")]
    InvalidDomain
}

pub type ResourceKey<T: ResourceType + Sized> = Arc<ResourceIdent<T>>;

#[derive(Clone, Hash, Serialize, Deserialize)]
pub struct ResourceIdent<T: ResourceType + Sized> {
    data: Box<str>,
    _phantom: PhantomData<T>
}

impl<T: ResourceType> PartialEq for ResourceIdent<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id() && self.key() == other.key()
    }

    fn ne(&self, other: &Self) -> bool {
        self.id() != other.id() || self.key() != other.key()
    }
}

impl<T: ResourceType> Eq for ResourceIdent<T> {}

impl<T: ResourceType> Default for ResourceIdent<T> {
    fn default() -> Self {
        T::default_resource()
    }
}

impl<T: ResourceType> ResourceIdent<T> {
    pub fn new(id: impl Into<String>, key: impl Into<String>) -> ResourceIdent<T> {
        Self {
            data: format!("{}:{}", id.into(), key.into()).into_boxed_str(),
            _phantom: PhantomData::<T>
        }
    }
    
    pub fn id(&self) -> String {
        self.data.split(":").nth(0).unwrap().to_string()
    }
    
    pub fn key(&self) -> String {
        self.data.split(":").nth(1).unwrap().to_string()
    }
}

impl<T: ResourceType> Display for ResourceIdent<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}/{}", self.id(), T::resource_name(), self.key())
    }
}
impl<T: ResourceType> Debug for ResourceIdent<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ResourceKey<{}> {{ {}, {} }}", T::resource_name(), self.id(), self.key())
    }
}
impl<T: ResourceType> TryFrom<&String> for ResourceIdent<T> {
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
                data: format!("{}:{}", id_split[0], domain_split[0]).into_boxed_str(),
                _phantom: PhantomData::<T>
            }),
            2 => {
                if domain_split[0] != T::resource_name() {
                    Err(ResourceKeyParseFail::InvalidDomain)
                } else {
                    Ok(Self {
                        data: format!("{}:{}", id_split[0], domain_split[1]).into_boxed_str(),
                        _phantom: PhantomData::<T>
                    })
                }
            },
            _ => Err(ResourceKeyParseFail::DomainSplitFail)
        }
    }
}