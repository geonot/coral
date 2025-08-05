use crate::ast::*;
use crate::resolver::{
    error::TypeError,
    types::{Constraint, EffectSet, InferType},
    TypeResolver,
};
use std::collections::HashMap;

impl TypeResolver {
    /// Initialize built-in types and functions
    pub(super) fn initialize_builtins(&mut self) {
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

        // Print function
        let print_type = InferType::Function {
            params: vec![InferType::String],
            return_type: Box::new(InferType::Unit),
            effects: EffectSet::io(),
        };
        self.env.bind("print".to_string(), print_type);
    }
    
    /// Collect function signatures for forward references
    pub(super) fn collect_function_signatures(&mut self, program: &Program) -> Result<(), TypeError> {
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
    pub(super) fn collect_type_definitions(&mut self, program: &Program) -> Result<(), TypeError> {
        for stmt in &program.statements {
            match &stmt.kind {
                StmtKind::Object { name, fields, methods } => {
                    let obj_type = self.create_object_type(name, fields, methods, false, false)?;
                    self.object_definitions.insert(name.clone(), obj_type.clone());
                    self.env.bind(name.clone(), obj_type);
                }
                
                StmtKind::Store { name, fields, methods } => {
                    let store_type = self.create_object_type(name, fields, methods, false, true)?;
                    self.store_types.insert(name.clone(), store_type.clone());
                    self.env.bind(name.clone(), store_type);
                }
                
                StmtKind::Actor { name, fields, handlers } => {
                    let actor_type = self.create_actor_type(name, fields, handlers)?;
                    self.actor_types.insert(name.clone(), actor_type.clone());
                    self.env.bind(name.clone(), actor_type);
                }
                
                _ => {}
            }
        }
        Ok(())
    }
    
    /// Create object type from AST definition
    pub(super) fn create_object_type(
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

        // Add the 'make' constructor as a static method
        let mut object_type_for_make = InferType::Object {
            name: name.to_string(),
            fields: field_types.clone(),
            methods: method_types.clone(),
            is_actor,
            is_store,
        };
        if let InferType::Object { methods, .. } = &mut object_type_for_make {
            let make_params: Vec<InferType> = field_types.values().cloned().collect();
            let make_return = InferType::Object {
                name: name.to_string(),
                fields: field_types.clone(),
                methods: HashMap::new(), // An instance doesn't have the 'make' method
                is_actor,
                is_store,
            };
            methods.insert("make".to_string(), InferType::Function {
                params: make_params,
                return_type: Box::new(make_return),
                effects: EffectSet::pure(),
            });
        }
        
        Ok(object_type_for_make)
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
        let return_type = if let Some(ret_type) = &method.return_type {
            self.ast_type_to_infer_type(ret_type)?
        } else if method.body.is_empty() {
            InferType::Unit
        } else {
            self.infer_block(&method.body)?
        };
        
        Ok(InferType::Function {
            params: param_types,
            return_type: Box::new(return_type),
            effects: EffectSet::pure(), // Will be inferred from body
        })
    }
    
    /// Convert AST type to inference type
    pub(super) fn ast_type_to_infer_type(&mut self, ast_type: &Type) -> Result<InferType, TypeError> {
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
            Type::TypeVar(id) => Ok(InferType::Var(crate::resolver::types::TypeVar(*id as usize))),
            Type::Result(ok, err) => {
                let ok_type = self.ast_type_to_infer_type(ok)?;
                let err_type = self.ast_type_to_infer_type(err)?;
                Ok(InferType::Result(Box::new(ok_type), Box::new(err_type)))
            }
            Type::Unknown => Ok(InferType::Var(self.var_gen.fresh())),
        }
    }

    /// Convert InferType back to AST Type for updating the AST
    pub(super) fn infer_type_to_ast_type(&self, infer_type: &InferType) -> Type {
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
    #[allow(dead_code)]
    pub(super) fn type_to_string(&self, ty: &InferType) -> String {
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
    pub(super) fn infer_expression(&mut self, expr: &Expr) -> Result<InferType, TypeError> {
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
                let arg_exprs: Vec<_> = args.iter().map(|arg| arg.value.clone()).collect();
                self.infer_call_expression(callee, &arg_exprs)
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
            },

            ExprKind::ObjectInstantiation { name, fields } => {
                if let Some(obj_type) = self.object_definitions.get(name) {
                    let mut obj_type = obj_type.clone();
                    if let InferType::Object { fields: obj_fields, .. } = &mut obj_type {
                        for (field_name, field_expr) in fields {
                            let field_type = self.infer_expression(field_expr)?;
                            if let Some(obj_field_type) = obj_fields.get(field_name) {
                                self.constraints.push(Constraint::Equal(field_type, obj_field_type.clone()));
                            } else {
                                return Err(TypeError::FieldNotFound(field_name.clone()));
                            }
                        }
                    }
                    Ok(obj_type)
                } else {
                    Err(TypeError::UnknownVariable(name.clone()))
                }
            },
            
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
            _ => Ok(InferType::Unknown),
        }
    }

    /// Infer function type from definition
    pub(super) fn infer_function(
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
    pub(super) fn infer_statement(&mut self, stmt: &Stmt) -> Result<InferType, TypeError> {
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
                self.object_definitions.insert(name.clone(), obj_type.clone());
                self.env.bind(name.clone(), obj_type);
                Ok(InferType::Unit)
            }
            
            StmtKind::Store { name, fields, methods } => {
                let store_type = self.create_object_type(name, fields, methods, false, true)?;
                self.store_types.insert(name.clone(), store_type.clone());
                self.env.bind(name.clone(), store_type);
                Ok(InferType::Unit)
            }
            
            StmtKind::Actor { name, fields, handlers } => {
                let actor_type = self.create_actor_type(name, fields, handlers)?;
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
            
            StmtKind::While { condition, body } => {
                let cond_type = self.infer_expression(condition)?;
                self.constraints.push(Constraint::Equal(cond_type, InferType::Bool));
                
                self.infer_block(body)?;
                Ok(InferType::Unit)
            }
            
            StmtKind::Return(value) => {
                if let Some(val) = value {
                    self.infer_expression(val)
                } else {
                    Ok(InferType::Unit)
                }
            }
            _ => Ok(InferType::Unit),
        }
    }

    /// Create store type with built-in methods
    pub(super) fn create_store_type(
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

        // Add 'with_id' as a static method
        methods.insert("with_id".to_string(), InferType::Function {
            params: vec![InferType::Int],
            return_type: Box::new(stored_type.clone()),
            effects: EffectSet::store(),
        });
        
        Ok(InferType::Store {
            name: name.to_string(),
            value_type: Box::new(stored_type),
            methods,
        })
    }

    /// Create actor type with message handlers
    pub(super) fn create_actor_type(
        &mut self,
        name: &str,
        fields: &[Field],
        handlers: &[MessageHandler],
    ) -> Result<InferType, TypeError> {
        let mut handler_types = HashMap::new();
        let mut actor_fields = HashMap::new();

        // Infer field types for the actor's internal state
        for field in fields {
            let field_type = self.ast_type_to_infer_type(&field.type_)?;
            actor_fields.insert(field.name.clone(), field_type);
        }
        
        // Infer handler types
        for handler in handlers {
            // Handler returns unit (async processing)
            let handler_type = InferType::Function {
                params: vec![
                    // Implicit actor instance parameter (self)
                    InferType::Object { 
                        name: name.to_string(), 
                        fields: actor_fields.clone(), 
                        methods: HashMap::new(), // Methods will be added later if needed
                        is_actor: true, 
                        is_store: false 
                    },
                    // Message parameter
                    self.ast_type_to_infer_type(&handler.message_type)?,
                ],
                return_type: Box::new(InferType::Unit),
                effects: EffectSet::actor(),
            };
            
            handler_types.insert(handler.message_type.clone(), handler_type);
        }
        
        Ok(InferType::Actor {
            name: name.to_string(),
            fields: actor_fields,
            handlers: handler_types,
        })
    }

    /// Infer type for a block of statements
    pub(super) fn infer_block(&mut self, stmts: &[Stmt]) -> Result<InferType, TypeError> {
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
        if let ExprKind::FieldAccess { object, field } = &callee.kind {
            if field == "make" {
                let object_type = self.infer_expression(object)?;
                if let InferType::Object { fields, .. } = &object_type {
                    let make_params: Vec<InferType> = fields.values().cloned().collect();
                    let make_return = object_type.clone();
                    let func_type = InferType::Function {
                        params: make_params,
                        return_type: Box::new(make_return),
                        effects: EffectSet::pure(),
                    };
                    let mut arg_types = Vec::new();
                    for arg in args {
                        arg_types.push(self.infer_expression(arg)?);
                    }
                    self.constraints.push(Constraint::IsCallable(func_type, arg_types, object_type.clone()));
                    return Ok(object_type);
                }
            }
            if field == "with_id" {
                let object_type = self.infer_expression(object)?;
                if let InferType::Store { value_type, .. } = &object_type {
                    let func_type = InferType::Function {
                        params: vec![InferType::Int],
                        return_type: value_type.clone(),
                        effects: EffectSet::store(),
                    };
                    let mut arg_types = Vec::new();
                    for arg in args {
                        arg_types.push(self.infer_expression(arg)?);
                    }
                    self.constraints.push(Constraint::IsCallable(func_type, arg_types, *value_type.clone()));
                    return Ok(*value_type.clone());
                }
            }
            return self.infer_method_call(object, field, args);
        }

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
        let mut arg_types = Vec::new();

        // Static methods like 'make' and 'with_id' don't have an implicit 'self'
        if method != "make" && method != "with_id" {
            arg_types.push(object_type.clone()); // self parameter
        }
        
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
}
