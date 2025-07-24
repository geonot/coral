use crate::resolver::types::InferType;
use std::collections::HashMap;

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
