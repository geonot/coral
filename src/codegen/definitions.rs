use crate::ast::Program;
use crate::resolver::types::InferType;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LLVMValue {
    pub type_info: InferType,
    pub llvm_type: String,
    pub value_id: String,
}

#[derive(Debug, Clone)]
pub struct LLVMFunction {
    pub name: String,
    pub params: Vec<LLVMValue>,
    pub return_type: String,
}

#[derive(Debug)]
pub enum CodegenError {
    UnsupportedFeature(String),
    UndefinedVariable(String),
    InvalidOperation(String),
}

#[derive(Debug, Clone)]
pub struct SymbolTable {
    variables: HashMap<String, LLVMValue>,
    parent: Option<Box<SymbolTable>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            parent: None,
        }
    }

    pub fn define_variable(&mut self, name: String, value: LLVMValue) {
        self.variables.insert(name, value);
    }

    pub fn lookup_variable(&self, name: &str) -> Option<LLVMValue> {
        self.variables.get(name).cloned().or_else(|| {
            self.parent
                .as_ref()
                .and_then(|p| p.lookup_variable(name))
        })
    }
}

pub struct LLVMCodegen {
    pub output: String,
    pub symbols: SymbolTable,
    pub global_strings: Vec<String>,
    temp_counter: usize,
    label_counter: usize,
    object_types: HashMap<String, InferType>,
    current_bb: usize,
}

impl LLVMCodegen {
    pub fn new(module_name: String) -> Self {
        Self {
            output: format!("; ModuleID = '{}'\n", module_name),
            symbols: SymbolTable::new(),
            global_strings: Vec::new(),
            temp_counter: 0,
            label_counter: 0,
            object_types: HashMap::new(),
            current_bb: 0,
        }
    }

    pub fn compile_program(&mut self, program: &Program) -> Result<String, CodegenError> {
        self.emit_object_structs(program)?;
        for stmt in &program.statements {
            self.compile_statement(stmt)?;
        }
        let mut final_ir = self.output.clone();
        for g_str in &self.global_strings {
            final_ir.push_str(g_str);
            final_ir.push('\n');
        }
        Ok(final_ir)
    }

    pub fn emit_object_structs(&mut self, program: &Program) -> Result<(), CodegenError> {
        for stmt in &program.statements {
            if let crate::ast::StmtKind::Object { name, fields, .. } = &stmt.kind {
                let mut field_types = Vec::new();
                for field in fields {
                    let field_type = self.ast_type_to_infer_type(&field.type_);
                    field_types.push(self.infer_type_to_llvm_type(&field_type));
                }
                self.emit(&format!("%{} = type {{ {} }}", name, field_types.join(", ")));
            }
        }
        Ok(())
    }

    pub(crate) fn emit(&mut self, instruction: &str) {
        self.output.push_str(instruction);
        self.output.push('\n');
    }

    pub(crate) fn next_temp(&mut self) -> usize {
        self.temp_counter += 1;
        self.temp_counter
    }

    pub(crate) fn next_label(&mut self) -> usize {
        self.label_counter += 1;
        self.label_counter
    }
    
pub(crate) fn ast_type_to_infer_type(&self, ast_type: &crate::ast::Type) -> InferType {
        // This is a simplified conversion
        match ast_type {
            crate::ast::Type::I32 => InferType::Int,
            crate::ast::Type::F64 => InferType::Float,
            crate::ast::Type::String => InferType::String,
            crate::ast::Type::Bool => InferType::Bool,
            _ => InferType::Unknown,
        }
    }

    pub(crate) fn infer_type_to_llvm_type(&self, infer_type: &InferType) -> String {
        match infer_type {
            InferType::Unit => "void".to_string(),
            InferType::Bool => "i1".to_string(),
            InferType::Int => "i64".to_string(),
            InferType::Float => "double".to_string(),
            InferType::String => "%string*".to_string(), // Pointer to a custom string struct
            InferType::List(_) => "%list*".to_string(),
            InferType::Map(_, _) => "%map*".to_string(),
            InferType::Function { params, return_type, .. } => {
                let ret_type = self.infer_type_to_llvm_type(return_type);
                let param_types: Vec<String> = params.iter().map(|p| self.infer_type_to_llvm_type(p)).collect();
                format!("{} ({})*", ret_type, param_types.join(", "))
            }
            InferType::Object { name, .. } => format!("%{}*", name), // Pointer to object struct
            InferType::Store { name, .. } => format!("%{}*", name),
            InferType::Actor { name, .. } => format!("%{}*", name),
            InferType::Var(_) | InferType::Unknown => "i8*".to_string(), // Should be resolved before codegen
            InferType::Result(ok, _) => self.infer_type_to_llvm_type(ok), // Simplified
            InferType::Pipe(_) => "%pipe*".to_string(),
            InferType::Iterator(_) => "%iterator*".to_string(),
            _ => "i8*".to_string(), // Default for other complex types
        }
    }

    pub(crate) fn lookup_object_type(&self, name: &str) -> Option<InferType> {
        self.object_types.get(name).cloned()
    }

    pub fn compile_object_instantiation(&mut self, type_name: &str, obj_type: InferType, args: &[crate::ast::Expr]) -> Result<LLVMValue, CodegenError> {
        // For now, we'll assume a simple malloc. A real implementation would need to know the size.
        let size = 1024; // Placeholder size
        let malloc_temp = self.next_temp();
        self.emit(&format!("  %{} = call i8* @malloc(i64 {})", malloc_temp, size));
        
        let ptr_temp = self.next_temp();
        let llvm_type = self.infer_type_to_llvm_type(&obj_type);
        self.emit(&format!("  %{} = bitcast i8* %{} to {}*", ptr_temp, malloc_temp, llvm_type));

        // TODO: Call constructor/initializer if one exists (`make`)

        Ok(LLVMValue {
            type_info: obj_type,
            llvm_type: format!("{}*", llvm_type),
            value_id: format!("%{}", ptr_temp),
        })
    }

    pub fn compile_function_call(&mut self, _callee: &crate::ast::Expr, _args: &[crate::ast::Expr]) -> Result<LLVMValue, CodegenError> {
        Err(CodegenError::UnsupportedFeature("compile_function_call not implemented".to_string()))
    }

    pub fn compile_method_call(&mut self, object: &crate::ast::Expr, field: &str, args: &[crate::ast::Expr]) -> Result<LLVMValue, CodegenError> {
        let object_val = self.compile_expression(object)?;
        
        let mut arg_vals = Vec::new();
        // First argument is always the object itself
        arg_vals.push(object_val.clone());

        for arg in args {
            arg_vals.push(self.compile_expression(arg)?);
        }

        let mangled_name = format!("_ZN{}{}{}", object_val.type_info.to_string().len(), object_val.type_info.to_string(), field.len(), field);

        let result_temp = self.next_temp();
        let result_type = InferType::Unknown; // Placeholder
        let result_llvm_type = self.infer_type_to_llvm_type(&result_type);

        let call_args = arg_vals.iter()
            .map(|v| format!("{} {}", v.llvm_type, v.value_id))
            .collect::<Vec<String>>()
            .join(", ");

        self.emit(&format!("  %{} = call {} @{}({})", result_temp, result_llvm_type, mangled_name, call_args));

        Ok(LLVMValue {
            type_info: result_type,
            llvm_type: result_llvm_type,
            value_id: format!("%{}", result_temp),
        })
    }

    pub fn compile_property_access(&mut self, object: &crate::ast::Expr, field: &str) -> Result<LLVMValue, CodegenError> {
        let object_val = self.compile_expression(object)?;
        
        // This requires knowing the field index. We'll have to look it up from the object definition.
        // For now, let's assume we can get this index.
        let field_index = 0; // Placeholder
        
        let result_temp = self.next_temp();
        let field_ptr_temp = self.next_temp();

        let object_llvm_type = self.infer_type_to_llvm_type(&object_val.type_info);

        self.emit(&format!("  %{} = getelementptr inbounds {}, {} {}, i32 0, i32 {}", 
            field_ptr_temp, 
            object_llvm_type.strip_suffix('*').unwrap_or(&object_llvm_type), 
            object_llvm_type, 
            object_val.value_id, 
            field_index
        ));

        // For now, we'll just return the pointer. A real implementation would load from it.
        let field_type = InferType::Unknown; // Placeholder
        let field_llvm_type = self.infer_type_to_llvm_type(&field_type);

        Ok(LLVMValue {
            type_info: field_type,
            llvm_type: format!("{}*", field_llvm_type),
            value_id: format!("%{}", field_ptr_temp),
        })
    }

    pub fn compile_if_expression(&mut self, condition: &crate::ast::Expr, then_branch: &crate::ast::Expr, else_branch: Option<&crate::ast::Expr>) -> Result<LLVMValue, CodegenError> {
        let cond_val = self.compile_expression(condition)?;

        let then_label = self.next_label();
        let else_label = self.next_label();
        let merge_label = self.next_label();

        self.emit(&format!("  br i1 {}, label %{}, label %{}", cond_val.value_id, then_label, else_label));

        self.emit(&format!("{}:", then_label));
        let then_val = self.compile_expression(then_branch)?;
        self.emit(&format!("  br label %{}", merge_label));
        let then_bb = self.current_bb; // Basic block where `then_val` is valid

        self.emit(&format!("{}:", else_label));
        let else_val = if let Some(else_expr) = else_branch {
            self.compile_expression(else_expr)?
        } else {
            // Default value for if without else (e.g., unit type)
            LLVMValue {
                type_info: InferType::Unit,
                llvm_type: "void".to_string(),
                value_id: "".to_string(),
            }
        };
        self.emit(&format!("  br label %{}", merge_label));
        let else_bb = self.current_bb;

        self.emit(&format!("{}:", merge_label));
        
        let result_type = then_val.type_info.clone();
        let result_llvm_type = self.infer_type_to_llvm_type(&result_type);

        if result_type != InferType::Unit {
            let result_temp = self.next_temp();
            self.emit(&format!("  %{} = phi {} [ {}, %{} ], [ {}, %{} ]",
                result_temp,
                result_llvm_type,
                then_val.value_id, then_bb,
                else_val.value_id, else_bb
            ));
            Ok(LLVMValue {
                type_info: result_type,
                llvm_type: result_llvm_type,
                value_id: format!("%{}", result_temp),
            })
        } else {
            Ok(LLVMValue {
                type_info: InferType::Unit,
                llvm_type: "void".to_string(),
                value_id: "".to_string(),
            })
        }
    }

    pub fn compile_list_literal(&mut self, _elements: &[crate::ast::Expr]) -> Result<LLVMValue, CodegenError> {
        Err(CodegenError::UnsupportedFeature("compile_list_literal not implemented".to_string()))
    }

    pub fn compile_list_append(&mut self, _list: &crate::ast::Expr, _element: &crate::ast::Expr) -> Result<LLVMValue, CodegenError> {
        Err(CodegenError::UnsupportedFeature("compile_list_append not implemented".to_string()))
    }

    pub fn compile_map_literal(&mut self, _elements: &[(crate::ast::Expr, crate::ast::Expr)]) -> Result<LLVMValue, CodegenError> {
        Err(CodegenError::UnsupportedFeature("compile_map_literal not implemented".to_string()))
    }

    pub fn compile_map_insert(&mut self, _map: &crate::ast::Expr, _key: &crate::ast::Expr, _value: &crate::ast::Expr) -> Result<LLVMValue, CodegenError> {
        Err(CodegenError::UnsupportedFeature("compile_map_insert not implemented".to_string()))
    }

    pub fn compile_string_interpolation(&mut self, _parts: &[crate::ast::StringPart]) -> Result<LLVMValue, CodegenError> {
        Err(CodegenError::UnsupportedFeature("compile_string_interpolation not implemented".to_string()))
    }
}
