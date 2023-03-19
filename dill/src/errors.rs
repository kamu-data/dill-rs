use std::any::{type_name, TypeId};

use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum InjectionError {
    #[error("Unregistered type")]
    Unregistered(UnregisteredTypeError),
    #[error("Ambiguous type")]
    Ambiguous(AmbiguousTypeError),
}

impl InjectionError {
    pub fn unregistered<Iface: 'static + ?Sized>() -> Self {
        Self::Unregistered(UnregisteredTypeError {
            type_id: TypeId::of::<Iface>(),
            type_name: type_name::<Iface>(),
        })
    }

    pub fn ambiguous<Iface: 'static + ?Sized>() -> Self {
        Self::Ambiguous(AmbiguousTypeError {
            type_id: TypeId::of::<Iface>(),
            type_name: type_name::<Iface>(),
        })
    }
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
#[error("Unregistered type: ${type_name}")]
pub struct UnregisteredTypeError {
    type_id: TypeId,
    type_name: &'static str,
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
#[error("Ambiguous type: ${type_name}")]
pub struct AmbiguousTypeError {
    type_id: TypeId,
    type_name: &'static str,
}
