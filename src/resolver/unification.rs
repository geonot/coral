use crate::resolver::{
    error::TypeError,
    types::{Constraint, InferType, Substitution},
    TypeResolver,
};
use std::collections::{HashMap, VecDeque};

impl TypeResolver {
    /// Solve all constraints using unification algorithm
    pub(super) fn solve_constraints(&mut self) -> Result<Substitution, TypeError> {
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
            
            (InferType::Result(o1, e1), InferType::Result(o2, e2)) => {
                let ok_subst = self.unify(o1, o2)?;
                let err_subst = self.unify(e1, e2)?;
                Ok(self.compose_substitutions(&ok_subst, &err_subst))
            }

            (InferType::Union(t1), InferType::Union(t2)) => {
                // This is a simplification. A more robust implementation would
                // handle unions with different numbers of types and find a
                // common supertype.
                if t1.len() != t2.len() {
                    return Err(TypeError::TypeMismatch(InferType::Union(t1.clone()), InferType::Union(t2.clone())));
                }
                let mut subst = Substitution::new();
                for (sub_t1, sub_t2) in t1.iter().zip(t2.iter()) {
                    let s = self.unify(sub_t1, sub_t2)?;
                    subst = self.compose_substitutions(&subst, &s);
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
    fn occurs_check(&self, var: crate::resolver::types::TypeVar, ty: &InferType) -> bool {
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
            InferType::Pipe(inner) => self.occurs_check(var, inner),
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
    pub(super) fn apply_substitution(&self, ty: &InferType, subst: &Substitution) -> InferType {
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
            InferType::Store { methods, .. } => {
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
                    effects: crate::resolver::types::EffectSet::pure(),
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
}
