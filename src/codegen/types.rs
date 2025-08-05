//! This module defines the mapping from Coral's `InferType` to LLVM IR types.

use crate::resolver::types::InferType;

/// Represents LLVM IR types.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LLVMType {
    /// Void type.
    Void,
    /// Integer types.
    Int(u32),
    /// Floating-point types.
    Float,
    Double,
    /// Pointer to another type.
    Pointer(Box<LLVMType>),
    /// Structure types.
    Struct(Vec<LLVMType>),
    /// Named structure types.
    NamedStruct(String),
    /// Function types.
    Function {
        ret: Box<LLVMType>,
        params: Vec<LLVMType>,
    },
}

impl std::fmt::Display for LLVMType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LLVMType::Void => write!(f, "void"),
            LLVMType::Int(bits) => write!(f, "i{}", bits),
            LLVMType::Float => write!(f, "float"),
            LLVMType::Double => write!(f, "double"),
            LLVMType::Pointer(ty) => write!(f, "{}*", ty),
            LLVMType::Struct(tys) => {
                let members = tys
                    .iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "{{ {} }}", members)
            }
            LLVMType::NamedStruct(name) => write!(f, "%{}", name),
            LLVMType::Function { ret, params } => {
                let param_types = params
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "{} ({})", ret, param_types)
            }
        }
    }
}

/// Converts an `InferType` to its corresponding `LLVMType`.
pub fn infer_to_llvm_type(ty: &InferType) -> LLVMType {
    match ty {
        InferType::Unit => LLVMType::Void,
        InferType::Bool => LLVMType::Int(1),
        InferType::Int => LLVMType::Int(64),
        InferType::Float => LLVMType::Double,
        InferType::String => LLVMType::NamedStruct("string".to_string()),
        InferType::List(it) => {
            let inner_type = infer_to_llvm_type(it);
            LLVMType::Pointer(Box::new(LLVMType::NamedStruct(format!(
                "list.{}",
                inner_type
            ))))
        }
        InferType::Map(k, v) => {
            let key_type = infer_to_llvm_type(k);
            let val_type = infer_to_llvm_type(v);
            LLVMType::Pointer(Box::new(LLVMType::NamedStruct(format!(
                "map.{}.{}",
                key_type, val_type
            ))))
        }
        InferType::Function {
            params,
            return_type,
            ..
        } => {
            let ret = Box::new(infer_to_llvm_type(return_type));
            let params = params.iter().map(infer_to_llvm_type).collect();
            LLVMType::Pointer(Box::new(LLVMType::Function { ret, params }))
        }
        InferType::Object { name, .. } => LLVMType::NamedStruct(name.clone()),
        InferType::Store { name, .. } => LLVMType::Pointer(Box::new(LLVMType::NamedStruct(name.clone()))),
        InferType::Actor { name, .. } => LLVMType::Pointer(Box::new(LLVMType::NamedStruct(name.clone()))),
        InferType::Result(ok, err) => {
            let ok_type = infer_to_llvm_type(ok);
            let err_type = infer_to_llvm_type(err);
            LLVMType::Struct(vec![LLVMType::Int(1), ok_type, err_type])
        }
        InferType::Iterator(it) => {
            let inner_type = infer_to_llvm_type(it);
            LLVMType::Pointer(Box::new(LLVMType::NamedStruct(format!(
                "iterator.{}",
                inner_type
            ))))
        }
        // These should be resolved before codegen
        InferType::Var(_) | InferType::Unknown => LLVMType::Pointer(Box::new(LLVMType::Int(8))),
        // Placeholder for complex types
        _ => LLVMType::Pointer(Box::new(LLVMType::Int(8))),
    }
}
