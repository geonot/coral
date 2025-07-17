use crate::ast::*;
use std::collections::{HashMap, VecDeque};

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
pub struct TypeVar(usize);

impl std::fmt::Display for TypeVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "t{}", self.0)
    }
}

/// Extended type system for complete inference
#[derive(Debug, Clone, PartialEq)]
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
        handlers: HashMap<String, InferType>, // Use String instead of ast::Type
    },
    
    // Type variables for inference
    Var(TypeVar),
    
    // Quantified types for polymorphism
    Forall(Vec<TypeVar>, Box<InferType>),
    
    // Union types for error handling
    Union(Vec<InferType>),
    
    // Result type for error propagation
    Result(Box<InferType>, Box<InferType>),
    
    // Iterator types for Coral's iteration model
    Iterator(Box<InferType>),
    
    // Unknown type that needs resolution
    Unknown,
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
type Substitution = HashMap<TypeVar, InferType>;

/// Constraint between types for inference
#[derive(Debug, Clone)]
pub enum Constraint {
    Equal(InferType, InferType),
    HasField(InferType, String, InferType),
    HasMethod(InferType, String, InferType),
    IsCallable(InferType, Vec<InferType>, InferType),
    IsIterable(InferType, InferType),
}

/// Type environment for scoped type checking
#[derive(Debug, Clone)]
pub struct TypeEnv {
    bindings: HashMap<String, InferType>,
    parent: Option<Box<TypeEnv>>,
}

impl TypeEnv {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            parent: None,
        }
    }
    
    pub fn extend(&self) -> Self {
        Self {
            bindings: HashMap::new(),
            parent: Some(Box::new(self.clone())),
        }
    }
    
    pub fn bind(&mut self, name: String, ty: InferType) {
        self.bindings.insert(name, ty);
    }
    
    pub fn lookup(&self, name: &str) -> Option<InferType> {
        self.bindings.get(name).cloned()
            .or_else(|| self.parent.as_ref()?.lookup(name))
    }
}

/// The main type resolver - this is where the magic happens
pub struct TypeResolver {
    var_gen: TypeVarGen,
    constraints: Vec<Constraint>,
    env: TypeEnv,
    builtin_types: HashMap<String, InferType>,
    object_definitions: HashMap<String, InferType>,
    store_types: HashMap<String, InferType>,
    actor_types: HashMap<String, InferType>,
}

impl TypeResolver {
    pub fn new() -> Self {
        let mut resolver = Self {
            var_gen: TypeVarGen::new(),
            constraints: Vec::new(),
            env: TypeEnv::new(),
            builtin_types: HashMap::new(),
            object_definitions: HashMap::new(),
            store_types: HashMap::new(),
            actor_types: HashMap::new(),
        };
        
        resolver.initialize_builtins();
        resolver
    }
    
    /// Initialize built-in types and functions
    fn initialize_builtins(&mut self) {
        // Built-in types
        self.builtin_types.insert("int".to_string(), InferType::Int);
        self.builtin_types.insert("float".to_string(), InferType::Float);
        self.builtin_types.insert("string".to_string(), InferType::String);
        self.builtin_types.insert("bool".to_string(), InferType::Bool);
        
        // Built-in functions with polymorphic types
        let log_type = InferType::Function {
            params: vec![InferType::Var(self.var_gen.fresh())],
            return_type: Box::new(InferType::Unit),
            effects: EffectSet::io(),
        };
        self.env.bind("log".to_string(), log_type);
        
        // Hash functions
        let hash_type = InferType::Function {
            params: vec![InferType::String],
            return_type: Box::new(InferType::String),
            effects: EffectSet::pure(),
        };
        self.env.bind("hash.blake3".to_string(), hash_type);
        
        // Now function
        self.env.bind("now".to_string(), InferType::Int); // timestamp
        
        // Empty value
        self.env.bind("empty".to_string(), InferType::Var(self.var_gen.fresh()));
    }
    
    /// Main entry point for type resolution
    pub fn resolve_program(&mut self, program: &mut Program) -> Result<(), TypeError> {
        // Phase 1: Collect all type definitions (objects, stores, actors) and function signatures
        self.collect_type_definitions(program)?;
        self.collect_function_signatures(program)?;
        
        // Phase 2: Generate constraints for all statements
        for stmt in &program.statements {
            self.infer_statement(stmt)?;
        }
        
        // Phase 3: Solve constraints using unification
        let subst = self.solve_constraints()?;
        
        // Phase 4: Apply substitutions to resolve all types
        self.apply_substitutions_to_program(program, &subst)?;
        
        Ok(())
    }
    
    /// Collect function signatures for forward references
    fn collect_function_signatures(&mut self, program: &Program) -> Result<(), TypeError> {
        for stmt in &program.statements {
            if let StmtKind::Function { name, params, return_type, .. } = &stmt.kind {
                let mut param_types = Vec::new();
                for param in params {
                    let param_type = match &param.type_ {
                        Type::Unknown => InferType::Var(self.var_gen.fresh()),
                        _ => self.ast_type_to_infer_type(&param.type_)?,
                    };
                    param_types.push(param_type);
                }
                
                let return_infer_type = if let Some(ty) = return_type {
                    self.ast_type_to_infer_type(ty)?
                } else {
                    InferType::Var(self.var_gen.fresh())
                };
                
                let func_type = InferType::Function {
                    params: param_types,
                    return_type: Box::new(return_infer_type),
                    effects: EffectSet::pure(), // Effects will be inferred later
                };
                self.env.bind(name.clone(), func_type);
            }
        }
        Ok(())
    }
    
    /// Collect object, store, and actor definitions for forward references
    fn collect_type_definitions(&mut self, program: &Program) -> Result<(), TypeError> {
        for stmt in &program.statements {
            match &stmt.kind {
                StmtKind::Object { name, fields, methods } => {
                    let obj_type = self.create_object_type(name, fields, methods, false, false)?;
                    self.object_definitions.insert(name.clone(), obj_type.clone());
                    self.env.bind(name.clone(), obj_type);
                }
                
                StmtKind::Store { name, value_type, initial_value: _ } => {
                    // Convert value_type to InferType
                    let store_type = self.ast_type_to_infer_type(value_type)?;
                    self.store_types.insert(name.clone(), store_type.clone());
                    self.env.bind(name.clone(), store_type);
                }
                
                StmtKind::Actor { name, fields: _fields, handlers } => {
                    let actor_type = self.create_actor_type(name, handlers)?;
                    self.actor_types.insert(name.clone(), actor_type.clone());
                    self.env.bind(name.clone(), actor_type);
                }
                
                _ => {}
            }
        }
        Ok(())
    }
    
    /// Create object type from AST definition
    fn create_object_type(
        &mut self,
        name: &str,
        fields: &[Field],
        methods: &[ObjectMethod],
        is_actor: bool,
        is_store: bool,
    ) -> Result<InferType, TypeError> {
        let mut field_types = HashMap::new();
        let mut method_types = HashMap::new();
        
        // Infer field types
        for field in fields {
            let field_type = match &field.default_value {
                Some(expr) => self.infer_expression(expr)?,
                None => {
                    // Convert AST Type to InferType
                    self.ast_type_to_infer_type(&field.type_)?
                }
            };
            field_types.insert(field.name.clone(), field_type);
        }
        
        // Infer method types
        for method in methods {
            let method_type = self.infer_method_type(method)?;
            method_types.insert(method.name.clone(), method_type);
        }
        
        // Add built-in methods for all objects
        self.add_builtin_object_methods(&mut method_types, &field_types)?;
        
        // Add store-specific methods
        if is_store {
            self.add_store_methods(&mut method_types, name)?;
        }
        
        // Add actor-specific methods
        if is_actor {
            self.add_actor_methods(&mut method_types)?;
        }
        
        Ok(InferType::Object {
            name: name.to_string(),
            fields: field_types,
            methods: method_types,
            is_actor,
            is_store,
        })
    }
    
    /// Add built-in methods that all objects have
    fn add_builtin_object_methods(
        &mut self,
        method_types: &mut HashMap<String, InferType>,
        field_types: &HashMap<String, InferType>,
    ) -> Result<(), TypeError> {
        // make method - constructor
        let make_params: Vec<InferType> = field_types.values().cloned().collect();
        let make_return = InferType::Var(self.var_gen.fresh()); // Self type
        
        method_types.insert("make".to_string(), InferType::Function {
            params: make_params,
            return_type: Box::new(make_return),
            effects: EffectSet::pure(),
        });
        
        // as methods for type conversion
        method_types.insert("as".to_string(), InferType::Function {
            params: vec![InferType::String], // conversion target
            return_type: Box::new(InferType::Var(self.var_gen.fresh())),
            effects: EffectSet::pure(),
        });
        
        Ok(())
    }
    
    /// Add store-specific methods
    fn add_store_methods(
        &mut self,
        method_types: &mut HashMap<String, InferType>,
        _store_name: &str,
    ) -> Result<(), TypeError> {
        // with_id method
        method_types.insert("with_id".to_string(), InferType::Function {
            params: vec![InferType::Int],
            return_type: Box::new(InferType::Var(self.var_gen.fresh())),
            effects: EffectSet::store(),
        });
        
        // find methods
        method_types.insert("find".to_string(), InferType::Function {
            params: vec![InferType::Var(self.var_gen.fresh())], // predicate
            return_type: Box::new(InferType::List(Box::new(InferType::Var(self.var_gen.fresh())))),
            effects: EffectSet::store(),
        });
        
        Ok(())
    }
    
    /// Add actor-specific methods
    fn add_actor_methods(
        &mut self,
        method_types: &mut HashMap<String, InferType>,
    ) -> Result<(), TypeError> {
        // send method (!)
        method_types.insert("send".to_string(), InferType::Function {
            params: vec![InferType::Var(self.var_gen.fresh())], // message
            return_type: Box::new(InferType::Unit),
            effects: EffectSet { actor_send: true, ..Default::default() },
        });
        
        Ok(())
    }
    
    /// Infer method type from method definition
    fn infer_method_type(&mut self, method: &ObjectMethod) -> Result<InferType, TypeError> {
        let mut param_types = Vec::new();
        
        // Add implicit self parameter
        param_types.push(InferType::Var(self.var_gen.fresh()));
        
        // Add explicit parameters
        for param in &method.params {
            let param_type = match &param.type_ {
                Type::Unknown => InferType::Var(self.var_gen.fresh()),
                _ => self.ast_type_to_infer_type(&param.type_)?,
            };
            param_types.push(param_type);
        }
        
        // Infer return type from body
        let return_type = if method.body.is_empty() {
            InferType::Unit
        } else {
            // For now, use a type variable - we'll infer from body later
            InferType::Var(self.var_gen.fresh())
        };
        
        Ok(InferType::Function {
            params: param_types,
            return_type: Box::new(return_type),
            effects: EffectSet::pure(), // Will be inferred from body
        })
    }
    
    /// Convert AST type to inference type
    fn ast_type_to_infer_type(&mut self, ast_type: &Type) -> Result<InferType, TypeError> {
        match ast_type {
            Type::I8 | Type::I16 | Type::I32 | Type::I64 => Ok(InferType::Int),
            Type::F32 | Type::F64 => Ok(InferType::Float),
            Type::String => Ok(InferType::String),
            Type::Bool => Ok(InferType::Bool),
            Type::Unit => Ok(InferType::Unit),
            Type::List(inner) => {
                let inner_type = self.ast_type_to_infer_type(inner)?;
                Ok(InferType::List(Box::new(inner_type)))
            }
            Type::Map(key, value) => {
                let key_type = self.ast_type_to_infer_type(key)?;
                let value_type = self.ast_type_to_infer_type(value)?;
                Ok(InferType::Map(Box::new(key_type), Box::new(value_type)))
            }
            Type::Function { params, return_type } => {
                let param_types: Result<Vec<_>, _> = params.iter()
                    .map(|p| self.ast_type_to_infer_type(p))
                    .collect();
                let return_infer_type = self.ast_type_to_infer_type(return_type)?;
                
                Ok(InferType::Function {
                    params: param_types?,
                    return_type: Box::new(return_infer_type),
                    effects: EffectSet::pure(),
                })
            }
            Type::Object { name, .. } | Type::Store { name, .. } | Type::Actor { name, .. } => {
                // Look up in our type definitions
                if let Some(obj_type) = self.object_definitions.get(name) {
                    Ok(obj_type.clone())
                } else if let Some(store_type) = self.store_types.get(name) {
                    Ok(store_type.clone())
                } else if let Some(actor_type) = self.actor_types.get(name) {
                    Ok(actor_type.clone())
                } else {
                    // Create a type variable for unknown types
                    Ok(InferType::Var(self.var_gen.fresh()))
                }
            }
            Type::TypeVar(id) => Ok(InferType::Var(TypeVar(*id as usize))),
            Type::Result(ok, err) => {
                let ok_type = self.ast_type_to_infer_type(ok)?;
                let err_type = self.ast_type_to_infer_type(err)?;
                Ok(InferType::Result(Box::new(ok_type), Box::new(err_type)))
            }
            Type::Unknown => Ok(InferType::Var(self.var_gen.fresh())),
        }
    }

    /// Convert InferType back to AST Type for updating the AST
    fn infer_type_to_ast_type(&self, infer_type: &InferType) -> Type {
        match infer_type {
            InferType::Unit => Type::Unit,
            InferType::Bool => Type::Bool,
            InferType::Int => Type::I32, // Default to I32 for integers
            InferType::Float => Type::F64, // Default to F64 for floats
            InferType::String => Type::String,
            InferType::List(inner) => Type::List(Box::new(self.infer_type_to_ast_type(inner))),
            InferType::Map(key, value) => Type::Map(
                Box::new(self.infer_type_to_ast_type(key)),
                Box::new(self.infer_type_to_ast_type(value)),
            ),
            InferType::Function { params, return_type, .. } => Type::Function {
                params: params.iter().map(|p| self.infer_type_to_ast_type(p)).collect(),
                return_type: Box::new(self.infer_type_to_ast_type(return_type)),
            },
            InferType::Object { name, fields, .. } => Type::Object { 
                name: name.clone(),
                fields: fields.iter()
                    .map(|(k, v)| (k.clone(), self.infer_type_to_ast_type(v)))
                    .collect(),
            },
            InferType::Store { name, value_type, .. } => Type::Store { 
                name: name.clone(),
                value_type: Box::new(self.infer_type_to_ast_type(value_type)),
            },
            InferType::Actor { name, .. } => Type::Actor { 
                name: name.clone(),
                message_types: vec![], // Simplified for now
            },
            InferType::Result(ok, err) => Type::Result(
                Box::new(self.infer_type_to_ast_type(ok)),
                Box::new(self.infer_type_to_ast_type(err)),
            ),
            InferType::Var(_) | InferType::Unknown => Type::Unknown,
            _ => Type::Unknown,
        }
    }

    /// Convert InferType to a readable string for debugging and error reporting
    fn type_to_string(&self, ty: &InferType) -> String {
        match ty {
            InferType::Unit => "unit".to_string(),
            InferType::Bool => "bool".to_string(),
            InferType::Int => "int".to_string(),
            InferType::Float => "float".to_string(),
            InferType::String => "string".to_string(),
            InferType::List(inner) => format!("List[{}]", self.type_to_string(inner)),
            InferType::Map(key, value) => format!("Map[{}, {}]", 
                self.type_to_string(key), self.type_to_string(value)),
            InferType::Function { params, return_type, .. } => {
                let param_strs: Vec<String> = params.iter().map(|p| self.type_to_string(p)).collect();
                format!("({}) -> {}", param_strs.join(", "), self.type_to_string(return_type))
            },
            InferType::Object { name, .. } => format!("object {}", name),
            InferType::Store { name, .. } => format!("store {}", name),
            InferType::Actor { name, .. } => format!("actor {}", name),
            InferType::Result(ok, err) => format!("Result[{}, {}]", 
                self.type_to_string(ok), self.type_to_string(err)),
            InferType::Var(v) => format!("?{}", v),
            InferType::Unknown => "?".to_string(),
            _ => "?".to_string(),
        }
    }

    /// Main expression inference method - dispatches to specific expression types
    fn infer_expression(&mut self, expr: &Expr) -> Result<InferType, TypeError> {
        match &expr.kind {
            ExprKind::Literal(lit) => self.infer_literal(lit),
            
            ExprKind::Identifier(name) => {
                if let Some(ty) = self.env.lookup(name) {
                    Ok(ty)
                } else {
                    Err(TypeError::UnknownVariable(name.clone()))
                }
            }
            
            ExprKind::Binary { left, op, right } => {
                self.infer_binary_expression(left, op, right)
            }
            
            ExprKind::Unary { op, operand } => {
                self.infer_unary_expression(op, operand)
            }
            
            ExprKind::Call { callee, args } => {
                self.infer_call_expression(callee, args)
            }
            
            ExprKind::Index { object, index } => {
                let object_type = self.infer_expression(object)?;
                let index_type = self.infer_expression(index)?;
                
                // For now, assume it's a list or map access
                let element_type = InferType::Var(self.var_gen.fresh());
                
                // Add constraints based on container type
                match &object_type {
                    InferType::List(_) => {
                        self.constraints.push(Constraint::Equal(index_type, InferType::Int));
                    }
                    InferType::Map(key_type, _) => {
                        self.constraints.push(Constraint::Equal(index_type, *key_type.clone()));
                    }
                    _ => {}
                }
                
                Ok(element_type)
            }
            
            ExprKind::FieldAccess { object, field } => {
                self.infer_field_access(object, field)
            }
            
            ExprKind::ListLiteral(elements) => {
                let element_type_var = InferType::Var(self.var_gen.fresh());
                if !elements.is_empty() {
                    for elem in elements {
                        let elem_type = self.infer_expression(elem)?;
                        self.constraints.push(Constraint::Equal(element_type_var.clone(), elem_type));
                    }
                }
                Ok(InferType::List(Box::new(element_type_var)))
            }
            
            ExprKind::MapLiteral(pairs) => {
                let key_type_var = InferType::Var(self.var_gen.fresh());
                let value_type_var = InferType::Var(self.var_gen.fresh());
                if !pairs.is_empty() {
                    for (key, value) in pairs {
                        let k_type = self.infer_expression(key)?;
                        let v_type = self.infer_expression(value)?;
                        self.constraints.push(Constraint::Equal(key_type_var.clone(), k_type));
                        self.constraints.push(Constraint::Equal(value_type_var.clone(), v_type));
                    }
                }
                Ok(InferType::Map(Box::new(key_type_var), Box::new(value_type_var)))
            }
            
            ExprKind::StringInterpolation { parts } => {
                // Each expression part must be inferrable 
                for part in parts {
                    if let crate::ast::StringPart::Expression(expr) = part {
                        self.infer_expression(expr)?;
                    }
                }
                Ok(InferType::String)
            }
            
            ExprKind::If { condition, then_branch, else_branch } => {
                let cond_type = self.infer_expression(condition)?;
                self.constraints.push(Constraint::Equal(cond_type, InferType::Bool));
                
                let then_type = self.infer_expression(then_branch)?;
                
                let else_type = if let Some(else_expr) = else_branch {
                    self.infer_expression(else_expr)?
                } else {
                    InferType::Unit
                };
                
                // Both branches must have same type
                self.constraints.push(Constraint::Equal(then_type.clone(), else_type));
                
                Ok(then_type)
            }
            
            ExprKind::Block(stmts) => {
                self.infer_block(stmts)
            }
            
            ExprKind::Lambda { params, body } => {
                // Create new scope for lambda parameters
                let mut lambda_env = self.env.extend();
                let mut param_types = Vec::new();
                
                for param in params {
                    let param_type = match &param.type_ {
                        Type::Unknown => InferType::Var(self.var_gen.fresh()),
                        _ => self.ast_type_to_infer_type(&param.type_)?,
                    };
                    lambda_env.bind(param.name.clone(), param_type.clone());
                    param_types.push(param_type);
                }
                
                // Infer body type in new scope
                let old_env = std::mem::replace(&mut self.env, lambda_env);
                let return_type = self.infer_expression(body)?;
                self.env = old_env;
                
                Ok(InferType::Function {
                    params: param_types,
                    return_type: Box::new(return_type),
                    effects: EffectSet::pure(),
                })
            }
            
            ExprKind::Pipe { .. } => Ok(InferType::Unit),
            ExprKind::Io { .. } => Ok(InferType::Unit),
            
            ExprKind::ListAppend { list, element } => {
                let list_type = self.infer_expression(list)?;
                let element_type = self.infer_expression(element)?;

                // Ensure list_type is a List and its inner type matches element_type
                let list_element_type = InferType::Var(self.var_gen.fresh());
                self.constraints.push(Constraint::Equal(list_type, InferType::List(Box::new(list_element_type.clone()))));
                self.constraints.push(Constraint::Equal(list_element_type, element_type));

                Ok(InferType::Unit)
            }

            ExprKind::MapInsert { map, key, value } => {
                let map_type = self.infer_expression(map)?;
                let key_type = self.infer_expression(key)?;
                let value_type = self.infer_expression(value)?;

                // Ensure map_type is a Map and its key/value types match
                let map_key_type = InferType::Var(self.var_gen.fresh());
                let map_value_type = InferType::Var(self.var_gen.fresh());
                self.constraints.push(Constraint::Equal(
                    map_type,
                    InferType::Map(Box::new(map_key_type.clone()), Box::new(map_value_type.clone())),
                ));
                self.constraints.push(Constraint::Equal(map_key_type, key_type));
                self.constraints.push(Constraint::Equal(map_value_type, value_type));

                Ok(InferType::Unit)
            }
        }
    }

    /// Infer function type from definition
    fn infer_function(
        &mut self,
        _name: &str,
        params: &[Parameter],
        return_type: Option<&Type>,
        body: &[Stmt],
    ) -> Result<InferType, TypeError> {
        // Create new scope for function parameters
        let mut func_env = self.env.extend();
        let mut param_types = Vec::new();
        
        for param in params {
            let param_type = match &param.type_ {
                Type::Unknown => InferType::Var(self.var_gen.fresh()),
                _ => self.ast_type_to_infer_type(&param.type_)?,
            };
            func_env.bind(param.name.clone(), param_type.clone());
            param_types.push(param_type);
        }
        
        // Infer return type from body
        let old_env = std::mem::replace(&mut self.env, func_env);
        let inferred_return = if body.is_empty() {
            InferType::Unit
        } else {
            self.infer_block(body)?
        };
        self.env = old_env;
        
        // Check against declared return type
        let final_return = if let Some(declared) = return_type {
            let declared_type = self.ast_type_to_infer_type(declared)?;
            self.constraints.push(Constraint::Equal(inferred_return, declared_type.clone()));
            declared_type
        } else {
            inferred_return
        };
        
        Ok(InferType::Function {
            params: param_types,
            return_type: Box::new(final_return),
            effects: EffectSet::pure(), // Will be inferred from body
        })
    }

    /// Infer type for a statement
    fn infer_statement(&mut self, stmt: &Stmt) -> Result<InferType, TypeError> {
        match &stmt.kind {
            StmtKind::Expression(expr) => self.infer_expression(expr),
            
            StmtKind::Assignment { target, value } => {
                let value_type = self.infer_expression(value)?;
                
                // For now, treat target as a variable binding
                if let ExprKind::Identifier(name) = &target.kind {
                    self.env.bind(name.clone(), value_type);
                }
                Ok(InferType::Unit)
            }
            
            StmtKind::Function { name, params, return_type, body } => {
                // Bind function name in current scope before inferring body
                let func_type_var = InferType::Var(self.var_gen.fresh());
                self.env.bind(name.clone(), func_type_var.clone());

                let func_type = self.infer_function(name, params, return_type.as_ref(), body)?;
                self.constraints.push(Constraint::Equal(func_type_var, func_type));
                Ok(InferType::Unit)
            },
            
            StmtKind::Object { name, fields, methods } => {
                let obj_type = self.create_object_type(name, fields, methods, false, false)?;
                self.object_definitions.insert(name.clone(), obj_type);
                Ok(InferType::Unit)
            }
            
            StmtKind::Store { name, value_type, initial_value } => {
                let store_type = self.create_store_type(name, value_type, initial_value)?;
                self.store_types.insert(name.clone(), store_type);
                Ok(InferType::Unit)
            }
            
            StmtKind::Actor { name, fields: _fields, handlers } => {
                let actor_type = self.create_actor_type(name, handlers)?;
                self.actor_types.insert(name.clone(), actor_type);
                Ok(InferType::Unit)
            }
            
            StmtKind::If { condition, then_branch, else_branch } => {
                let cond_type = self.infer_expression(condition)?;
                self.constraints.push(Constraint::Equal(cond_type, InferType::Bool));
                
                let then_type = self.infer_block(then_branch)?;
                
                if let Some(else_stmts) = else_branch {
                    let else_type = self.infer_block(else_stmts)?;
                    self.constraints.push(Constraint::Equal(then_type.clone(), else_type));
                }
                
                Ok(then_type)
            }
            
            StmtKind::For { variable, iterable, body } => {
                let iterable_type = self.infer_expression(iterable)?;
                let element_type = InferType::Var(self.var_gen.fresh());
                
                self.constraints.push(Constraint::IsIterable(iterable_type, element_type.clone()));
                
                // Add loop variable to scope
                let mut loop_env = self.env.extend();
                loop_env.bind(variable.clone(), element_type);
                
                let old_env = std::mem::replace(&mut self.env, loop_env);
                let _body_type = self.infer_block(body)?;
                self.env = old_env;
                
                Ok(InferType::Unit)
            }
            
            StmtKind::While { condition, body } => {
                let cond_type = self.infer_expression(condition)?;
                self.constraints.push(Constraint::Equal(cond_type, InferType::Bool));
                
                self.infer_block(body)?;
                Ok(InferType::Unit)
            }
            
            StmtKind::Return(expr) => {
                if let Some(return_expr) = expr {
                    self.infer_expression(return_expr)
                } else {
                    Ok(InferType::Unit)
                }
            }
            
            _ => Ok(InferType::Unit),
        }
    }

    /// Create store type with built-in methods
    fn create_store_type(
        &mut self,
        name: &str,
        value_type: &Type,
        initial_value: &Option<Expr>,
    ) -> Result<InferType, TypeError> {
        let stored_type = self.ast_type_to_infer_type(value_type)?;
        
        // Validate initial value if provided
        if let Some(init_expr) = initial_value {
            let init_type = self.infer_expression(init_expr)?;
            self.constraints.push(Constraint::Equal(stored_type.clone(), init_type));
        }
        
        let mut methods = HashMap::new();
        
        // get() -> T
        methods.insert("get".to_string(), InferType::Function {
            params: vec![],
            return_type: Box::new(stored_type.clone()),
            effects: EffectSet::store(),
        });
        
        // set(value: T) -> unit
        methods.insert("set".to_string(), InferType::Function {
            params: vec![stored_type.clone()],
            return_type: Box::new(InferType::Unit),
            effects: EffectSet::store(),
        });
        
        // update(f: T -> T) -> unit
        methods.insert("update".to_string(), InferType::Function {
            params: vec![InferType::Function {
                params: vec![stored_type.clone()],
                return_type: Box::new(stored_type.clone()),
                effects: EffectSet::pure(),
            }],
            return_type: Box::new(InferType::Unit),
            effects: EffectSet::store(),
        });
        
        Ok(InferType::Store {
            name: name.to_string(),
            value_type: Box::new(stored_type),
            methods,
        })
    }

    /// Create actor type with message handlers
    fn create_actor_type(
        &mut self,
        name: &str,
        handlers: &[MessageHandler],
    ) -> Result<InferType, TypeError> {
        let mut handler_types = HashMap::new();
        
        // Infer handler types
        for handler in handlers {
            // Handler returns unit (async processing)
            let handler_type = InferType::Function {
                params: vec![
                    // Implicit actor instance parameter
                    InferType::Object { 
                        name: name.to_string(), 
                        fields: HashMap::new(), 
                        methods: HashMap::new(), 
                        is_actor: true, 
                        is_store: false 
                    },
                    // Message parameter
                    self.ast_type_to_infer_type(&handler.message_type)?,
                ],
                return_type: Box::new(InferType::Unit),
                effects: EffectSet::actor(),
            };
            
            handler_types.insert(handler.message_type.to_string(), handler_type);
        }
        
        Ok(InferType::Actor {
            name: name.to_string(),
            fields: HashMap::new(),
            handlers: handler_types,
        })
    }

    /// Infer type for a block of statements
    fn infer_block(&mut self, stmts: &[Stmt]) -> Result<InferType, TypeError> {
        if stmts.is_empty() {
            return Ok(InferType::Unit);
        }
        
        let mut last_type = InferType::Unit;
        
        for stmt in stmts {
            last_type = self.infer_statement(stmt)?;
        }
        
        Ok(last_type)
    }

    /// Infer literal types
    fn infer_literal(&mut self, lit: &Literal) -> Result<InferType, TypeError> {
        match lit {
            Literal::Integer(_) => Ok(InferType::Int),
            Literal::Float(_) => Ok(InferType::Float),
            Literal::String(_) => Ok(InferType::String),
            Literal::Bool(_) => Ok(InferType::Bool),
            Literal::Unit => Ok(InferType::Unit), // null/nil
            Literal::No => Ok(InferType::Unit), // null/nil
            Literal::Yes => Ok(InferType::Bool),
            Literal::Empty => Ok(InferType::Var(self.var_gen.fresh())), // empty collection
            Literal::None => Ok(InferType::Unit),
            Literal::Now => Ok(InferType::Int), // timestamp
            Literal::Err => Ok(InferType::Unknown),
        }
    }

    /// Infer binary expression types with operator overloading support
    fn infer_binary_expression(
        &mut self,
        left: &Expr,
        op: &BinaryOp,
        right: &Expr,
    ) -> Result<InferType, TypeError> {
        let left_type = self.infer_expression(left)?;
        let right_type = self.infer_expression(right)?;
        
        match op {
            // Arithmetic operators
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                // Both operands must be numeric and same type
                self.constraints.push(Constraint::Equal(left_type.clone(), right_type.clone()));
                
                // Result type is same as operands
                match &left_type {
                    InferType::Int | InferType::Float => Ok(left_type),
                    InferType::Var(_) => {
                        // Could be int or float
                        let numeric_type = InferType::Var(self.var_gen.fresh());
                        self.constraints.push(Constraint::Equal(left_type, numeric_type.clone()));
                        Ok(numeric_type)
                    }
                    _ => Err(TypeError::TypeMismatch(InferType::Int, left_type)),
                }
            }
            
            // Comparison operators
            BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
                // Operands must be comparable (numeric)
                self.constraints.push(Constraint::Equal(left_type, right_type));
                Ok(InferType::Bool)
            }
            
            // Equality operators
            BinaryOp::Eq | BinaryOp::Ne => {
                // Any types can be compared for equality
                self.constraints.push(Constraint::Equal(left_type, right_type));
                Ok(InferType::Bool)
            }
            
            // Logical operators
            BinaryOp::And | BinaryOp::Or => {
                self.constraints.push(Constraint::Equal(left_type, InferType::Bool));
                self.constraints.push(Constraint::Equal(right_type, InferType::Bool));
                Ok(InferType::Bool)
            }
            
            // Bitwise operators
            BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor | BinaryOp::Shl | BinaryOp::Shr => {
                // Both operands must be integers and same type
                self.constraints.push(Constraint::Equal(left_type.clone(), InferType::Int));
                self.constraints.push(Constraint::Equal(right_type.clone(), InferType::Int));
                Ok(InferType::Int)
            }
            BinaryOp::Is => {
                self.constraints.push(Constraint::Equal(left_type, right_type));
                Ok(InferType::Bool)
            }
            BinaryOp::Xor => {
                self.constraints.push(Constraint::Equal(left_type, InferType::Bool));
                self.constraints.push(Constraint::Equal(right_type, InferType::Bool));
                Ok(InferType::Bool)
            }
        }
    }

    /// Infer unary expression types
    fn infer_unary_expression(&mut self, op: &UnaryOp, operand: &Expr) -> Result<InferType, TypeError> {
        let operand_type = self.infer_expression(operand)?;
        
        match op {
            UnaryOp::Neg => {
                // Operand must be numeric
                match &operand_type {
                    InferType::Int | InferType::Float => Ok(operand_type),
                    InferType::Var(_) => {
                        // Constrain to be numeric
                        Ok(operand_type)
                    }
                    _ => Err(TypeError::TypeMismatch(InferType::Int, operand_type)),
                }
            }
            
            UnaryOp::Not => {
                self.constraints.push(Constraint::Equal(operand_type, InferType::Bool));
                Ok(InferType::Bool)
            }
            
            UnaryOp::BitNot => {
                self.constraints.push(Constraint::Equal(operand_type.clone(), InferType::Int));
                Ok(InferType::Int)
            }
        }
    }

    /// Infer function call types - handles Coral's flexible call syntax  
    fn infer_call_expression(&mut self, callee: &Expr, args: &[Expr]) -> Result<InferType, TypeError> {
        let callee_type = self.infer_expression(callee)?;
        let mut arg_types = Vec::new();
        
        for arg in args {
            arg_types.push(self.infer_expression(arg)?);
        }
        
        let return_type = InferType::Var(self.var_gen.fresh());
        
        // Callee must be callable with these arguments
        self.constraints.push(Constraint::IsCallable(callee_type, arg_types, return_type.clone()));
        
        Ok(return_type)
    }
    
    /// Infer field access types
    fn infer_field_access(&mut self, object: &Expr, field: &str) -> Result<InferType, TypeError> {
        let object_type = self.infer_expression(object)?;
        let field_type = InferType::Var(self.var_gen.fresh());
        
        // Object must have this field
        self.constraints.push(Constraint::HasField(object_type, field.to_string(), field_type.clone()));
        
        Ok(field_type)
    }
    
    /// Infer method call types
    fn infer_method_call(
        &mut self,
        object: &Expr,
        method: &str,
        args: &[Expr],
    ) -> Result<InferType, TypeError> {
        let object_type = self.infer_expression(object)?;
        let mut arg_types = vec![object_type.clone()]; // self parameter
        
        for arg in args {
            arg_types.push(self.infer_expression(arg)?);
        }
        
        let method_type = InferType::Var(self.var_gen.fresh());
        let return_type = InferType::Var(self.var_gen.fresh());
        
        // Object must have this method
        self.constraints.push(Constraint::HasMethod(object_type, method.to_string(), method_type.clone()));
        
        // Method must be callable with these arguments
        self.constraints.push(Constraint::IsCallable(method_type, arg_types, return_type.clone()));
        
        Ok(return_type)
    }
    
    /// Solve all constraints using unification algorithm
    fn solve_constraints(&mut self) -> Result<Substitution, TypeError> {
        let mut subst = Substitution::new();
        let mut work_queue: VecDeque<Constraint> = self.constraints.drain(..).collect();
        
        while let Some(constraint) = work_queue.pop_front() {
            match constraint {
                Constraint::Equal(t1, t2) => {
                    let unified_subst = self.unify(&t1, &t2)?;
                    subst = self.compose_substitutions(&subst, &unified_subst);
                    
                    // Apply new substitution to remaining constraints
                    for constraint in &mut work_queue {
                        *constraint = self.apply_subst_to_constraint(constraint, &unified_subst);
                    }
                }
                
                Constraint::HasField(obj_type, field_name, field_type) => {
                    self.solve_has_field_constraint(obj_type, field_name, field_type, &mut subst, &mut work_queue)?;
                }
                
                Constraint::HasMethod(obj_type, method_name, method_type) => {
                    self.solve_has_method_constraint(obj_type, method_name, method_type, &mut subst, &mut work_queue)?;
                }
                
                Constraint::IsCallable(func_type, arg_types, return_type) => {
                    self.solve_callable_constraint(func_type, arg_types, return_type, &mut subst, &mut work_queue)?;
                }
                
                Constraint::IsIterable(container_type, element_type) => {
                    self.solve_iterable_constraint(container_type, element_type, &mut subst, &mut work_queue)?;
                }
            }
        }
        
        Ok(subst)
    }
    
    /// Unification algorithm - the core of type inference
    fn unify(&mut self, t1: &InferType, t2: &InferType) -> Result<Substitution, TypeError> {
        match (t1, t2) {
            // Same types unify trivially
            (InferType::Unit, InferType::Unit) |
            (InferType::Bool, InferType::Bool) |
            (InferType::Int, InferType::Int) |
            (InferType::Float, InferType::Float) |
            (InferType::String, InferType::String) => Ok(Substitution::new()),
            
            // Variable unification
            (InferType::Var(v), t) | (t, InferType::Var(v)) => {
                if t == &InferType::Var(*v) {
                    Ok(Substitution::new())
                } else if self.occurs_check(*v, t) {
                    Err(TypeError::InfiniteType(*v, t.clone()))
                } else {
                    let mut subst = Substitution::new();
                    subst.insert(*v, t.clone());
                    Ok(subst)
                }
            }
            
            // Structural types
            (InferType::List(t1), InferType::List(t2)) => {
                self.unify(t1, t2)
            }
            
            (InferType::Map(k1, v1), InferType::Map(k2, v2)) => {
                let key_subst = self.unify(k1, k2)?;
                let val_subst = self.unify(v1, v2)?;
                Ok(self.compose_substitutions(&key_subst, &val_subst))
            }
            
            (InferType::Function { params: p1, return_type: r1, effects: e1 },
             InferType::Function { params: p2, return_type: r2, effects: e2 }) => {
                if p1.len() != p2.len() {
                    return Err(TypeError::ArityMismatch(p1.len(), p2.len()));
                }
                
                let mut subst = Substitution::new();
                
                // Unify parameters
                for (param1, param2) in p1.iter().zip(p2.iter()) {
                    let param_subst = self.unify(param1, param2)?;
                    subst = self.compose_substitutions(&subst, &param_subst);
                }
                
                // Unify return types
                let return_subst = self.unify(r1, r2)?;
                subst = self.compose_substitutions(&subst, &return_subst);
                
                // Effects must be compatible (simplified)
                if e1 != e2 {
                    // For now, just warn - in a real system we'd have effect subtyping
                }
                
                Ok(subst)
            }
            
            (InferType::Object { name: n1, fields: f1, .. },
             InferType::Object { name: n2, fields: f2, .. }) => {
                if n1 != n2 {
                    return Err(TypeError::TypeMismatch(t1.clone(), t2.clone()));
                }
                
                // Objects with same name should have same structure
                let mut subst = Substitution::new();
                for (field_name, field_type1) in f1 {
                    if let Some(field_type2) = f2.get(field_name) {
                        let field_subst = self.unify(field_type1, field_type2)?;
                        subst = self.compose_substitutions(&subst, &field_subst);
                    }
                }
                
                Ok(subst)
            }
            
            (InferType::Unknown, t) | (t, InferType::Unknown) => {
                // Unknown should unify with the known type, or a fresh type variable if both are unknown
                if t == &InferType::Unknown {
                    Ok(Substitution::new())
                } else {
                    let mut subst = Substitution::new();
                    // Create a fresh type variable and unify it with the known type
                    let fresh_var = self.var_gen.fresh();
                    subst.insert(fresh_var, t.clone());
                    Ok(subst)
                }
            }
            
            // Everything else fails to unify
            _ => Err(TypeError::TypeMismatch(t1.clone(), t2.clone())),
        }
    }
    
    /// Occurs check to prevent infinite types
    fn occurs_check(&self, var: TypeVar, ty: &InferType) -> bool {
        match ty {
            InferType::Var(v) => var == *v,
            InferType::List(inner) => self.occurs_check(var, inner),
            InferType::Map(k, v) => self.occurs_check(var, k) || self.occurs_check(var, v),
            InferType::Function { params, return_type, .. } => {
                params.iter().any(|p| self.occurs_check(var, p)) || self.occurs_check(var, return_type)
            }
            InferType::Object { fields, methods, .. } => {
                fields.values().any(|f| self.occurs_check(var, f)) ||
                methods.values().any(|m| self.occurs_check(var, m))
            }
            InferType::Forall(vars, ty) => !vars.contains(&var) && self.occurs_check(var, ty),
            InferType::Union(types) => types.iter().any(|t| self.occurs_check(var, t)),
            InferType::Result(ok, err) => self.occurs_check(var, ok) || self.occurs_check(var, err),
            InferType::Iterator(inner) => self.occurs_check(var, inner),
            _ => false,
        }
    }
    
    /// Compose two substitutions
    fn compose_substitutions(&self, s1: &Substitution, s2: &Substitution) -> Substitution {
        let mut result = s1.clone();
        
        // Apply s2 to the range of s1
        for (_var, ty) in result.iter_mut() {
            *ty = self.apply_substitution(ty, s2);
        }
        
        // Add bindings from s2 that aren't in s1
        for (var, ty) in s2 {
            if !result.contains_key(var) {
                result.insert(*var, ty.clone());
            }
        }
        
        result
    }
    
    /// Apply substitution to a type
    fn apply_substitution(&self, ty: &InferType, subst: &Substitution) -> InferType {
        match ty {
            InferType::Var(v) => {
                if let Some(replacement) = subst.get(v) {
                    replacement.clone()
                } else {
                    ty.clone()
                }
            }
            InferType::List(inner) => {
                InferType::List(Box::new(self.apply_substitution(inner, subst)))
            }
            InferType::Map(k, v) => {
                InferType::Map(
                    Box::new(self.apply_substitution(k, subst)),
                    Box::new(self.apply_substitution(v, subst)),
                )
            }
            InferType::Function { params, return_type, effects } => {
                InferType::Function {
                    params: params.iter().map(|p| self.apply_substitution(p, subst)).collect(),
                    return_type: Box::new(self.apply_substitution(return_type, subst)),
                    effects: effects.clone(),
                }
            }
            InferType::Object { name, fields, methods, is_actor, is_store } => {
                let new_fields: HashMap<String, InferType> = fields.iter()
                    .map(|(k, v)| (k.clone(), self.apply_substitution(v, subst)))
                    .collect();
                let new_methods: HashMap<String, InferType> = methods.iter()
                    .map(|(k, v)| (k.clone(), self.apply_substitution(v, subst)))
                    .collect();
                
                InferType::Object {
                    name: name.clone(),
                    fields: new_fields,
                    methods: new_methods,
                    is_actor: *is_actor,
                    is_store: *is_store,
                }
            }
            _ => ty.clone(),
        }
    }
    
    /// Apply substitution to constraint
    fn apply_subst_to_constraint(&self, constraint: &Constraint, subst: &Substitution) -> Constraint {
        match constraint {
            Constraint::Equal(t1, t2) => {
                Constraint::Equal(
                    self.apply_substitution(t1, subst),
                    self.apply_substitution(t2, subst),
                )
            }
            Constraint::HasField(obj, field, field_type) => {
                Constraint::HasField(
                    self.apply_substitution(obj, subst),
                    field.clone(),
                    self.apply_substitution(field_type, subst),
                )
            }
            Constraint::HasMethod(obj, method, method_type) => {
                Constraint::HasMethod(
                    self.apply_substitution(obj, subst),
                    method.clone(),
                    self.apply_substitution(method_type, subst),
                )
            }
            Constraint::IsCallable(func, args, ret) => {
                Constraint::IsCallable(
                    self.apply_substitution(func, subst),
                    args.iter().map(|a| self.apply_substitution(a, subst)).collect(),
                    self.apply_substitution(ret, subst),
                )
            }
            Constraint::IsIterable(container, element) => {
                Constraint::IsIterable(
                    self.apply_substitution(container, subst),
                    self.apply_substitution(element, subst),
                )
            }
        }
    }
    
    /// Solve HasField constraint
    fn solve_has_field_constraint(
        &mut self,
        obj_type: InferType,
        field_name: String,
        field_type: InferType,
        subst: &mut Substitution,
        work_queue: &mut VecDeque<Constraint>,
    ) -> Result<(), TypeError> {
        match obj_type {
            InferType::Object { fields, .. } => {
                if let Some(actual_field_type) = fields.get(&field_name) {
                    work_queue.push_back(Constraint::Equal(field_type, actual_field_type.clone()));
                } else {
                    return Err(TypeError::FieldNotFound(field_name));
                }
            }
            InferType::Var(v) => {
                // Create object type with this field
                let mut fields = HashMap::new();
                fields.insert(field_name, field_type);
                
                let obj_type = InferType::Object {
                    name: format!("Inferred_{}", v.0),
                    fields,
                    methods: HashMap::new(),
                    is_actor: false,
                    is_store: false,
                };
                
                subst.insert(v, obj_type);
            }
            _ => return Err(TypeError::NotAnObject(obj_type)),
        }
        
        Ok(())
    }
    
    /// Solve HasMethod constraint
    fn solve_has_method_constraint(
        &mut self,
        obj_type: InferType,
        method_name: String,
        method_type: InferType,
        subst: &mut Substitution,
        work_queue: &mut VecDeque<Constraint>,
    ) -> Result<(), TypeError> {
        match obj_type {
            InferType::Object { methods, .. } => {
                if let Some(actual_method_type) = methods.get(&method_name) {
                    work_queue.push_back(Constraint::Equal(method_type, actual_method_type.clone()));
                } else {
                    return Err(TypeError::MethodNotFound(method_name));
                }
            }
            InferType::Var(v) => {
                // Create object type with this method
                let mut methods = HashMap::new();
                methods.insert(method_name, method_type);
                
                let obj_type = InferType::Object {
                    name: format!("Inferred_{}", v.0),
                    fields: HashMap::new(),
                    methods,
                    is_actor: false,
                    is_store: false,
                };
                
                subst.insert(v, obj_type);
            }
            _ => return Err(TypeError::NotAnObject(obj_type)),
        }
        
        Ok(())
    }
    
    /// Solve IsCallable constraint
    fn solve_callable_constraint(
        &mut self,
        func_type: InferType,
        arg_types: Vec<InferType>,
        return_type: InferType,
        subst: &mut Substitution,
        work_queue: &mut VecDeque<Constraint>,
    ) -> Result<(), TypeError> {
        match func_type {
            InferType::Function { params, return_type: func_return, .. } => {
                if params.len() != arg_types.len() {
                    return Err(TypeError::ArityMismatch(params.len(), arg_types.len()));
                }
                
                // Unify parameters
                for (param, arg) in params.iter().zip(arg_types.iter()) {
                    work_queue.push_back(Constraint::Equal(param.clone(), arg.clone()));
                }
                
                // Unify return type
                work_queue.push_back(Constraint::Equal(*func_return, return_type));
            }
            InferType::Var(v) => {
                // Create function type
                let func_type = InferType::Function {
                    params: arg_types,
                    return_type: Box::new(return_type),
                    effects: EffectSet::pure(),
                };
                
                subst.insert(v, func_type);
            }
            _ => return Err(TypeError::NotCallable(func_type)),
        }
        
        Ok(())
    }
    
    /// Solve IsIterable constraint
    fn solve_iterable_constraint(
        &mut self,
        container_type: InferType,
        element_type: InferType,
        subst: &mut Substitution,
        work_queue: &mut VecDeque<Constraint>,
    ) -> Result<(), TypeError> {
        match container_type {
            InferType::List(inner) => {
                work_queue.push_back(Constraint::Equal(*inner, element_type));
            }
            InferType::Iterator(inner) => {
                work_queue.push_back(Constraint::Equal(*inner, element_type));
            }
            InferType::Var(v) => {
                // Assume it's a list
                let list_type = InferType::List(Box::new(element_type));
                subst.insert(v, list_type);
            }
            _ => return Err(TypeError::NotIterable(container_type)),
        }
        
        Ok(())
    }
    
    /// Apply final substitutions to the program AST
    fn apply_substitutions_to_program(
        &self,
        program: &mut Program,
        subst: &Substitution,
    ) -> Result<(), TypeError> {
        // Create an AST mutation visitor to update types
        let mut type_updater = TypeUpdater {
            resolver: self,
            substitution: subst,
        };
        
        type_updater.visit_program_mut(program);
        Ok(())
    }
}

/// AST visitor for updating types after inference
struct TypeUpdater<'a> {
    resolver: &'a TypeResolver,
    substitution: &'a Substitution,
}

impl<'a> TypeUpdater<'a> {
    fn update_type(&self, ast_type: &mut Type) {
        // Convert AST type to InferType, apply substitution, convert back
        if let Ok(infer_type) = self.resolver.ast_type_to_infer_type_readonly(ast_type) {
            let substituted = self.resolver.apply_substitution(&infer_type, self.substitution);
            *ast_type = self.resolver.infer_type_to_ast_type(&substituted);
        }
    }
}

impl<'a> VisitorMut for TypeUpdater<'a> {
    fn visit_program_mut(&mut self, program: &mut Program) {
        for stmt in &mut program.statements {
            self.visit_stmt_mut(stmt);
        }
    }
    
    fn visit_stmt_mut(&mut self, stmt: &mut Stmt) {
        match &mut stmt.kind {
            StmtKind::Assignment { target, value } => {
                self.visit_expr_mut(target);
                self.visit_expr_mut(value);
            }
            
            StmtKind::Function { params, return_type, body, .. } => {
                for param in params {
                    self.update_type(&mut param.type_);
                    if let Some(default) = &mut param.default_value {
                        self.visit_expr_mut(default);
                    }
                }
                
                if let Some(ret_type) = return_type {
                    self.update_type(ret_type);
                }
                
                for body_stmt in body {
                    self.visit_stmt_mut(body_stmt);
                }
            }
            
            StmtKind::Object { fields, methods, .. } => {
                for field in fields {
                    self.update_type(&mut field.type_);
                    if let Some(default) = &mut field.default_value {
                        self.visit_expr_mut(default);
                    }
                }
                
                for method in methods {
                    for param in &mut method.params {
                        self.update_type(&mut param.type_);
                    }
                    for body_stmt in &mut method.body {
                        self.visit_stmt_mut(body_stmt);
                    }
                }
            }
            
            StmtKind::Store { value_type, initial_value, .. } => {
                self.update_type(value_type);
                if let Some(init) = initial_value {
                    self.visit_expr_mut(init);
                }
            }
            
            StmtKind::Actor { handlers, .. } => {
                for handler in handlers {
                    for body_stmt in &mut handler.body {
                        self.visit_stmt_mut(body_stmt);
                    }
                }
            }
            
            StmtKind::If { condition, then_branch, else_branch } => {
                self.visit_expr_mut(condition);
                for stmt in then_branch {
                    self.visit_stmt_mut(stmt);
                }
                if let Some(else_stmts) = else_branch {
                    for stmt in else_stmts {
                        self.visit_stmt_mut(stmt);
                    }
                }
            }
            
            StmtKind::For { iterable, body, .. } => {
                self.visit_expr_mut(iterable);
                for stmt in body {
                    self.visit_stmt_mut(stmt);
                }
            }
            
            StmtKind::While { condition, body } => {
                self.visit_expr_mut(condition);
                for stmt in body {
                    self.visit_stmt_mut(stmt);
                }
            }
            
            StmtKind::Return(expr) => {
                if let Some(return_expr) = expr {
                    self.visit_expr_mut(return_expr);
                }
            }
            
            StmtKind::Expression(expr) => {
                self.visit_expr_mut(expr);
            }
            
            _ => {}
        }
    }
    
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        match &mut expr.kind {
            ExprKind::Binary { left, right, .. } => {
                self.visit_expr_mut(left);
                self.visit_expr_mut(right);
            }
            
            ExprKind::Unary { operand, .. } => {
                self.visit_expr_mut(operand);
            }
            
            ExprKind::Call { callee, args } => {
                self.visit_expr_mut(callee);
                for arg in args {
                    self.visit_expr_mut(arg);
                }
            }
            
            ExprKind::Index { object, index } => {
                self.visit_expr_mut(object);
                self.visit_expr_mut(index);
            }
            
            ExprKind::FieldAccess { object, .. } => {
                self.visit_expr_mut(object);
            }
            
            ExprKind::ListLiteral(elements) => {
                for elem in elements {
                    self.visit_expr_mut(elem);
                }
            }
            
            ExprKind::MapLiteral(pairs) => {
                for (key, value) in pairs {
                    self.visit_expr_mut(key);
                    self.visit_expr_mut(value);
                }
            }
            
            ExprKind::StringInterpolation { parts } => {
                for part in parts {
                    if let crate::ast::StringPart::Expression(expr) = part {
                        self.visit_expr_mut(expr);
                    }
                }
            }
            
            ExprKind::If { condition, then_branch, else_branch } => {
                self.visit_expr_mut(condition);
                self.visit_expr_mut(then_branch);
                if let Some(else_expr) = else_branch {
                    self.visit_expr_mut(else_expr);
                }
            }
            
            ExprKind::Block(stmts) => {
                for stmt in stmts {
                    self.visit_stmt_mut(stmt);
                }
            }
            
            ExprKind::Lambda { params, body } => {
                for param in params {
                    self.update_type(&mut param.type_);
                }
                self.visit_expr_mut(body);
            }
            
            ExprKind::ListAppend { list, element } => {
                self.visit_expr_mut(list);
                self.visit_expr_mut(element);
            }
            
            _ => {}
        }
    }
    
    fn visit_type_mut(&mut self, type_: &mut Type) {
        self.update_type(type_);
    }
}

impl TypeResolver {
    /// Read-only version of ast_type_to_infer_type for the visitor
    pub fn ast_type_to_infer_type_readonly(&self, ast_type: &Type) -> Result<InferType, TypeError> {
        match ast_type {
            Type::I8 | Type::I16 | Type::I32 | Type::I64 => Ok(InferType::Int),
            Type::F32 | Type::F64 => Ok(InferType::Float),
            Type::String => Ok(InferType::String),
            Type::Bool => Ok(InferType::Bool),
            Type::Unit => Ok(InferType::Unit),
            Type::List(inner) => {
                let inner_type = self.ast_type_to_infer_type_readonly(inner)?;
                Ok(InferType::List(Box::new(inner_type)))
            }
            Type::Map(key, value) => {
                let key_type = self.ast_type_to_infer_type_readonly(key)?;
                let value_type = self.ast_type_to_infer_type_readonly(value)?;
                Ok(InferType::Map(Box::new(key_type), Box::new(value_type)))
            }
            Type::Function { params, return_type } => {
                let param_types: Result<Vec<_>, _> = params.iter()
                    .map(|p| self.ast_type_to_infer_type_readonly(p))
                    .collect();
                let return_infer_type = self.ast_type_to_infer_type_readonly(return_type)?;
                
                Ok(InferType::Function {
                    params: param_types?,
                    return_type: Box::new(return_infer_type),
                    effects: EffectSet::pure(),
                })
            }
            Type::Object { name, .. } | Type::Store { name, .. } | Type::Actor { name, .. } => {
                // Look up in our type definitions
                if let Some(obj_type) = self.object_definitions.get(name) {
                    Ok(obj_type.clone())
                } else if let Some(store_type) = self.store_types.get(name) {
                    Ok(store_type.clone())
                } else if let Some(actor_type) = self.actor_types.get(name) {
                    Ok(actor_type.clone())
                } else {
                    Ok(InferType::Unknown)
                }
            }
            Type::TypeVar(id) => Ok(InferType::Var(TypeVar(*id as usize))),
            Type::Result(ok, err) => {
                let ok_type = self.ast_type_to_infer_type_readonly(ok)?;
                let err_type = self.ast_type_to_infer_type_readonly(err)?;
                Ok(InferType::Result(Box::new(ok_type), Box::new(err_type)))
            }
            Type::Unknown => Ok(InferType::Unknown),
        }
    }
}