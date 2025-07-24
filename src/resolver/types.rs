use crate::ast::Type;
use std::collections::HashMap;

/// Type variable generator for Hindley-Milner style inference
#[derive(Debug, Clone)]
pub struct TypeVarGen {
    counter: usize,
}

impl TypeVarGen {
    pub fn new() -> Self {
        Self { counter: 0 }
    }
    
    pub fn fresh(&mut self) -> TypeVar {
        let var = TypeVar(self.counter);
        self.counter += 1;
        var
    }
}

/// Type variables for inference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeVar(pub usize);

impl std::fmt::Display for TypeVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "t{}", self.0)
    }
}

/// Extended type system for complete inference
#[derive(Debug, Clone)]
pub enum InferType {
    // Concrete types
    Unit,
    Bool,
    Int,
    Float,
    String,
    List(Box<InferType>),
    Map(Box<InferType>, Box<InferType>),
    
    // Function types with effect tracking
    Function {
        params: Vec<InferType>,
        return_type: Box<InferType>,
        effects: EffectSet,
    },
    
    // Object types with structural typing
    Object {
        name: String,
        fields: HashMap<String, InferType>,
        methods: HashMap<String, InferType>,
        is_actor: bool,
        is_store: bool,
    },
    
    // Store type for data persistence
    Store {
        name: String,
        value_type: Box<InferType>,
        methods: HashMap<String, InferType>,
    },
    
    // Actor type for message handling
    Actor {
        name: String,
        fields: HashMap<String, InferType>,
        handlers: HashMap<Type, InferType>,
    },
    
    // Type variables for inference
    Var(TypeVar),
    
    // Quantified types for polymorphism
    Forall(Vec<TypeVar>, Box<InferType>),
    
    // Union types for error handling
    Union(Vec<InferType>),
    
    // Result type for error propagation
    Result(Box<InferType>, Box<InferType>),
    
    // Pipe type for data streaming
    Pipe(Box<InferType>),
    
    // Iterator types for Coral's iteration model
    Iterator(Box<InferType>),
    
    // Unknown type that needs resolution
    Unknown,
}

impl PartialEq for InferType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (InferType::Unit, InferType::Unit) => true,
            (InferType::Bool, InferType::Bool) => true,
            (InferType::Int, InferType::Int) => true,
            (InferType::Float, InferType::Float) => true,
            (InferType::String, InferType::String) => true,
            (InferType::List(a), InferType::List(b)) => a == b,
            (InferType::Map(k1, v1), InferType::Map(k2, v2)) => k1 == k2 && v1 == v2,
            (InferType::Function { params: p1, return_type: r1, .. }, InferType::Function { params: p2, return_type: r2, .. }) => {
                p1 == p2 && r1 == r2
            }
            (InferType::Object { name: n1, .. }, InferType::Object { name: n2, .. }) => n1 == n2,
            (InferType::Store { name: n1, .. }, InferType::Store { name: n2, .. }) => n1 == n2,
            (InferType::Actor { name: n1, .. }, InferType::Actor { name: n2, .. }) => n1 == n2,
            (InferType::Var(v1), InferType::Var(v2)) => v1 == v2,
            (InferType::Result(o1, e1), InferType::Result(o2, e2)) => o1 == o2 && e1 == e2,
            (InferType::Pipe(t1), InferType::Pipe(t2)) => t1 == t2,
            (InferType::Iterator(t1), InferType::Iterator(t2)) => t1 == t2,
            (InferType::Unknown, InferType::Unknown) => true,
            _ => false,
        }
    }
}

/// Effect system for tracking side effects
#[derive(Debug, Clone, PartialEq, Default)]
pub struct EffectSet {
    pub io: bool,
    pub store: bool,
    pub actor_send: bool,
    pub mutation: bool,
}

impl EffectSet {
    pub fn pure() -> Self {
        Self::default()
    }
    
    pub fn io() -> Self {
        Self { io: true, ..Default::default() }
    }
    
    pub fn store() -> Self {
        Self { store: true, ..Default::default() }
    }
    
    pub fn actor() -> Self {
        Self { actor_send: true, ..Default::default() }
    }
    
    pub fn union(&self, other: &Self) -> Self {
        Self {
            io: self.io || other.io,
            store: self.store || other.store,
            actor_send: self.actor_send || other.actor_send,
            mutation: self.mutation || other.mutation,
        }
    }
}

/// Type substitution map
pub type Substitution = HashMap<TypeVar, InferType>;

/// Constraint between types for inference
#[derive(Debug, Clone)]
pub enum Constraint {
    Equal(InferType, InferType),
    HasField(InferType, String, InferType),
    HasMethod(InferType, String, InferType),
    IsCallable(InferType, Vec<InferType>, InferType),
    IsIterable(InferType, InferType),
}

impl InferType {
    /// Convert InferType to a simplified AST Type for error messages and final AST representation
    pub fn to_ast_type(&self) -> Type {
        match self {
            InferType::Unit => Type::Unit,
            InferType::Bool => Type::Bool,
            InferType::Int => Type::I64, // Default to i64 for now
            InferType::Float => Type::F64, // Default to f64
            InferType::String => Type::String,
            InferType::List(t) => Type::List(Box::new(t.to_ast_type())),
            InferType::Map(k, v) => Type::Map(Box::new(k.to_ast_type()), Box::new(v.to_ast_type())),
            InferType::Function { params, return_type, .. } => Type::Function {
                params: params.iter().map(|p| p.to_ast_type()).collect(),
                return_type: Box::new(return_type.to_ast_type()),
            },
            InferType::Object { name, fields, .. } => Type::Object {
                name: name.clone(),
                fields: fields.iter().map(|(k, v)| (k.clone(), v.to_ast_type())).collect(),
            },
            InferType::Store { name, value_type, .. } => Type::Store {
                name: name.clone(),
                value_type: Box::new(value_type.to_ast_type()),
            },
            InferType::Actor { name, handlers, .. } => Type::Actor {
                name: name.clone(),
                message_types: handlers.keys().cloned().collect(),
            },
            InferType::Var(v) => Type::TypeVar(v.0 as u32),
            InferType::Result(ok, err) => Type::Result(Box::new(ok.to_ast_type()), Box::new(err.to_ast_type())),
            InferType::Pipe(t) => Type::Pipe(Box::new(t.to_ast_type())),
            _ => Type::Unknown,
        }
    }

    /// Convert InferType to its LLVM IR type representation
    pub fn to_llvm_type(&self) -> String {
        match self {
            InferType::Unit => "void".to_string(),
            InferType::Bool => "i1".to_string(),
            InferType::Int => "i64".to_string(), // Default to i64
            InferType::Float => "double".to_string(), // Default to double
            InferType::String => "%string".to_string(), // Assuming a custom string type
            InferType::List(_) => format!("%list*"), // Pointer to a list struct
            InferType::Map(_, _) => format!("%map*"), // Pointer to a map struct
            InferType::Function { params, return_type, .. } => {
                let ret_type = return_type.to_llvm_type();
                let param_types: Vec<String> = params.iter().map(|p| p.to_llvm_type()).collect();
                format!("{} ({})*", ret_type, param_types.join(", "))
            }
            InferType::Object { name, .. } => format!("%{}", name),
            InferType::Store { name, .. } => format!("%{}*", name), // Pointer to the store object
            InferType::Actor { name, .. } => format!("%{}*", name), // Pointer to the actor object
            InferType::Var(_) => "i8*".to_string(), // Should be resolved before codegen
            InferType::Result(ok, _err) => ok.to_llvm_type(), // Simplified: just use the ok type
            InferType::Pipe(_) => "%pipe*".to_string(),
            InferType::Iterator(_) => format!("%iterator*"),
            InferType::Unknown => "i8*".to_string(), // Should be resolved
            _ => "i8*".to_string(), // Default for other complex types
        }
    }
}
