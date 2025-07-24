use crate::resolver::types::{Constraint, InferType, TypeVar};

/// Error type for type inference failures
#[derive(Debug, Clone)]
pub enum TypeError {
    TypeMismatch(InferType, InferType),
    InfiniteType(TypeVar, InferType),
    ArityMismatch(usize, usize),
    FieldNotFound(String),
    MethodNotFound(String),
    NotAnObject(InferType),
    NotCallable(InferType),
    NotIterable(InferType),
    UnknownVariable(String),
    ConstraintUnsatisfied(Constraint),
}

impl std::fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeError::TypeMismatch(expected, actual) => {
                write!(f, "Type mismatch: expected {:?}, found {:?}", expected, actual)
            }
            TypeError::InfiniteType(var, ty) => {
                write!(f, "Infinite type: {:?} occurs in {:?}", var, ty)
            }
            TypeError::ArityMismatch(expected, actual) => {
                write!(f, "Arity mismatch: expected {} arguments, found {}", expected, actual)
            }
            TypeError::FieldNotFound(field) => {
                write!(f, "Field '{}' not found", field)
            }
            TypeError::MethodNotFound(method) => {
                write!(f, "Method '{}' not found", method)
            }
            TypeError::NotAnObject(ty) => {
                write!(f, "Type {:?} is not an object", ty)
            }
            TypeError::NotCallable(ty) => {
                write!(f, "Type {:?} is not callable", ty)
            }
            TypeError::NotIterable(ty) => {
                write!(f, "Type {:?} is not iterable", ty)
            }
            TypeError::UnknownVariable(name) => {
                write!(f, "Unknown variable '{}'", name)
            }
            TypeError::ConstraintUnsatisfied(constraint) => {
                write!(f, "Constraint unsatisfied: {:?}", constraint)
            }
        }
    }
}
