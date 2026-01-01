use std::any::{TypeId, type_name};

use thiserror::Error;

use crate::{InjectionContext, InjectionStack};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Error, Debug, Clone)]
pub enum InjectionError {
    #[error(transparent)]
    Unregistered(UnregisteredTypeError),
    #[error(transparent)]
    Ambiguous(AmbiguousTypeError),
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

impl InjectionError {
    pub fn unregistered<Iface: 'static + ?Sized>(ctx: &InjectionContext) -> Self {
        Self::Unregistered(UnregisteredTypeError {
            type_id: TypeId::of::<Iface>(),
            type_name: type_name::<Iface>(),
            injection_stack: ctx.to_stack(),
        })
    }

    // TODO: Should contain information about which implementations were found
    pub fn ambiguous<Iface: 'static + ?Sized>(ctx: &InjectionContext) -> Self {
        Self::Ambiguous(AmbiguousTypeError {
            type_id: TypeId::of::<Iface>(),
            type_name: type_name::<Iface>(),
            injection_stack: ctx.to_stack(),
        })
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Error, Debug, Clone)]
#[error("Unregistered type: {type_name}\nInjection stack:\n{injection_stack}")]
pub struct UnregisteredTypeError {
    pub type_id: TypeId,
    pub type_name: &'static str,
    pub injection_stack: InjectionStack,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Error, Debug, Clone)]
#[error("Ambiguous type: {type_name}\nInjection stack:\n{injection_stack}")]
pub struct AmbiguousTypeError {
    pub type_id: TypeId,
    pub type_name: &'static str,
    pub injection_stack: InjectionStack,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Error, Debug, Clone)]
pub struct ValidationError {
    pub errors: Vec<InjectionError>,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "DI graph validation failed:")?;
        for (i, err) in self.errors.iter().enumerate() {
            writeln!(f, "{i}: {err}")?;
        }
        Ok(())
    }
}

pub trait ValidationErrorExt {
    fn ignore<T: 'static + ?Sized>(self) -> Self;
}

impl ValidationErrorExt for Result<(), ValidationError> {
    fn ignore<T: 'static + ?Sized>(self) -> Self {
        let type_id = TypeId::of::<T>();
        let Err(mut err) = self else { return Ok(()) };

        err.errors.retain(|e| match e {
            InjectionError::Unregistered(e) => e.type_id != type_id,
            InjectionError::Ambiguous(e) => e.type_id != type_id,
        });

        if err.errors.is_empty() {
            Ok(())
        } else {
            Err(err)
        }
    }
}
