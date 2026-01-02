use std::any::TypeId;

use thiserror::Error;

use crate::{DependencyInfo, InjectionContext, InjectionStack, TypeInfo};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Error, Debug, Clone)]
pub enum InjectionError {
    #[error(transparent)]
    Unregistered(UnregisteredTypeError),
    #[error(transparent)]
    Ambiguous(AmbiguousTypeError),
    #[error(transparent)]
    ScopeInversion(Box<ScopeInversionError>),
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

impl InjectionError {
    pub fn unregistered<Iface: 'static + ?Sized>(ctx: &InjectionContext) -> Self {
        Self::Unregistered(UnregisteredTypeError {
            dep_type: TypeInfo::of::<Iface>(),
            injection_stack: ctx.to_stack(),
        })
    }

    // TODO: Should contain information about which implementations were found
    pub fn ambiguous<Iface: 'static + ?Sized>(ctx: &InjectionContext) -> Self {
        Self::Ambiguous(AmbiguousTypeError {
            dep_type: TypeInfo::of::<Iface>(),
            injection_stack: ctx.to_stack(),
        })
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Error, Debug, Clone)]
pub struct UnregisteredTypeError {
    pub dep_type: TypeInfo,
    pub injection_stack: InjectionStack,
}

impl std::fmt::Display for UnregisteredTypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Unregistered type: {}", self.dep_type.name)?;
        write!(f, "Injection stack:\n{}", self.injection_stack)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Error, Debug, Clone)]
pub struct AmbiguousTypeError {
    pub dep_type: TypeInfo,
    pub injection_stack: InjectionStack,
}

impl std::fmt::Display for AmbiguousTypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Ambiguous type: {}", self.dep_type.name)?;
        write!(f, "Injection stack:\n{}", self.injection_stack)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Error, Debug, Clone)]
pub struct ScopeInversionError {
    pub inst_type: TypeInfo,
    pub inst_scope: TypeInfo,
    pub inst_dep: DependencyInfo,
    pub dep_type: TypeInfo,
    pub dep_scope: TypeInfo,
    pub injection_stack: InjectionStack,
}

impl std::fmt::Display for ScopeInversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Scope inversion: {} in {} scope injects {} in {} scope",
            self.inst_type.name, self.inst_scope.name, self.dep_type.name, self.dep_scope.name,
        )?;

        write!(f, "Injection stack:\n{}", self.injection_stack)
    }
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
            InjectionError::Unregistered(e) => e.dep_type.id != type_id,
            InjectionError::Ambiguous(e) => e.dep_type.id != type_id,
            InjectionError::ScopeInversion(e) => e.dep_type.id != type_id,
        });

        if err.errors.is_empty() {
            Ok(())
        } else {
            Err(err)
        }
    }
}
