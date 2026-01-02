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

    pub fn push(&'a self, frame: InjectionStackFrame) -> InjectionContext<'a> {
        InjectionContext {
            frame: Some(frame),
            prev: Some(self),
        }
    }

    pub fn push_resolve<Spec: DependencySpec + 'static>(&'a self) -> InjectionContext<'a> {
        self.push(InjectionStackFrame::Resolve {
            iface: TypeInfo::of::<Spec::IfaceType>(),
            spec: TypeInfo::of::<Spec>(),
        })
    }

    pub fn push_build(&'a self, b: &dyn Builder) -> InjectionContext<'a> {
        self.push(InjectionStackFrame::Build {
            instance: b.instance_type(),
            scope: b.scope_type(),
        })
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
    Resolve { iface: TypeInfo, spec: TypeInfo },
    Build { instance: TypeInfo, scope: TypeInfo },
}

impl std::fmt::Display for InjectionStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (line, frame) in self.frames.iter().rev().enumerate() {
            match frame {
                InjectionStackFrame::Resolve { iface: _, spec } => {
                    writeln!(f, "  {line}: Resolve: {}", spec.name)?;
                }
                InjectionStackFrame::Build { instance, scope } => {
                    writeln!(f, "  {line}: Build:   {} <{}>", instance.name, scope.name)?;
                }
            }
        }
        Ok(())
    }
}
