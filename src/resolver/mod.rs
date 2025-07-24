pub mod error;
pub mod types;
pub mod env;
pub mod inference;
pub mod unification;
pub mod visitor;

use crate::ast::*;
use std::collections::HashMap;

use self::error::TypeError;
pub use self::types::{Constraint, InferType, TypeVar, TypeVarGen};
use self::env::TypeEnv;

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
}
