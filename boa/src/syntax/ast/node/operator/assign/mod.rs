use crate::{
    environment::lexical_environment::VariableScope,
    exec::Executable,
    gc::{Finalize, Trace},
    syntax::ast::node::Node,
    BoaProfiler, Context, Result, Value,
};
use std::fmt;

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

/// An assignment operator assigns a value to its left operand based on the value of its right
/// operand.
///
/// Assignment operator (`=`), assigns the value of its right operand to its left operand.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-AssignmentExpression
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Assignment_Operators
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct Assign {
    lhs: Box<Node>,
    rhs: Box<Node>,
}

impl Assign {
    /// Creates an `Assign` AST node.
    pub(in crate::syntax) fn new<L, R>(lhs: L, rhs: R) -> Self
    where
        L: Into<Node>,
        R: Into<Node>,
    {
        Self {
            lhs: Box::new(lhs.into()),
            rhs: Box::new(rhs.into()),
        }
    }

    /// Gets the left hand side of the assignment operation.
    pub fn lhs(&self) -> &Node {
        &self.lhs
    }

    /// Gets the right hand side of the assignment operation.
    pub fn rhs(&self) -> &Node {
        &self.rhs
    }
}

impl Executable for Assign {
    fn run(&self, context: &mut Context) -> Result<Value> {
        let _timer = BoaProfiler::global().start_event("Assign", "exec");
        let val = self.rhs().run(context)?;
        match self.lhs() {
            Node::Identifier(ref name) => {
                let environment = &mut context.realm_mut().environment;

                if environment.has_binding(name.as_ref()) {
                    // Binding already exists
                    environment
                        .set_mutable_binding(name.as_ref(), val.clone(), true)
                        .map_err(|e| e.to_error(context))?;
                } else {
                    environment
                        .create_mutable_binding(
                            name.as_ref().to_owned(),
                            true,
                            VariableScope::Function,
                        )
                        .map_err(|e| e.to_error(context))?;
                    let environment = &mut context.realm_mut().environment;
                    environment
                        .initialize_binding(name.as_ref(), val.clone())
                        .map_err(|e| e.to_error(context))?;
                }
            }
            Node::GetConstField(ref get_const_field) => {
                let val_obj = get_const_field.obj().run(context)?;
                val_obj.set_field(get_const_field.field(), val.clone());
            }
            Node::GetField(ref get_field) => {
                let object = get_field.obj().run(context)?;
                let field = get_field.field().run(context)?;
                let key = field.to_property_key(context)?;
                object.set_field(key, val.clone());
            }
            _ => (),
        }
        Ok(val)
    }
}

impl fmt::Display for Assign {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} = {}", self.lhs, self.rhs)
    }
}

impl From<Assign> for Node {
    fn from(op: Assign) -> Self {
        Self::Assign(op)
    }
}
