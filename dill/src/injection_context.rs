////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

use crate::{Builder, DependencySpec, TypeInfo};

pub struct InjectionContext<'a> {
    pub frame: Option<InjectionStackFrame>,
    pub prev: Option<&'a InjectionContext<'a>>,
}

impl<'a> InjectionContext<'a> {
    pub fn new_root() -> InjectionContext<'static> {
        InjectionContext {
            frame: None,
            prev: None,
        }
    }

    pub fn push_resolve<Spec: DependencySpec + 'static>(&'a self) -> InjectionContext<'a> {
        InjectionContext {
            frame: Some(InjectionStackFrame::Resolve {
                spec_type: TypeInfo::of::<Spec>(),
                iface_type: TypeInfo::of::<Spec::IfaceType>(),
            }),
            prev: Some(self),
        }
    }

    pub fn push_build(&'a self, b: &dyn Builder) -> InjectionContext<'a> {
        InjectionContext {
            frame: Some(InjectionStackFrame::Build {
                instance_type: b.instance_type(),
                scope_type: b.scope_type(),
            }),
            prev: Some(self),
        }
    }

    pub fn to_stack(&self) -> InjectionStack {
        let mut stack = InjectionStack { frames: Vec::new() };
        let mut current = Some(self);
        while let Some(c) = current {
            if let Some(f) = &c.frame {
                stack.frames.push(f.clone());
            }
            current = c.prev;
        }
        stack
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct InjectionStack {
    pub frames: Vec<InjectionStackFrame>,
}

#[derive(Clone, Debug)]
pub enum InjectionStackFrame {
    Resolve {
        spec_type: TypeInfo,
        iface_type: TypeInfo,
    },
    Build {
        instance_type: TypeInfo,
        scope_type: TypeInfo,
    },
}

impl std::fmt::Display for InjectionStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (line, frame) in self.frames.iter().rev().enumerate() {
            match frame {
                InjectionStackFrame::Resolve {
                    spec_type,
                    iface_type: _,
                } => {
                    writeln!(f, "  {line}: Resolve: {}", spec_type.type_name)?;
                }
                InjectionStackFrame::Build {
                    instance_type,
                    scope_type,
                } => {
                    writeln!(
                        f,
                        "  {line}: Build:   {} <{}>",
                        instance_type.type_name, scope_type.type_name
                    )?;
                }
            }
        }
        Ok(())
    }
}
