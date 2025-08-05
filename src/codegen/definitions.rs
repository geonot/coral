use crate::ast::Program;
use crate::resolver::types::InferType;
use crate::codegen::types::{LLVMType, infer_to_llvm_type};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LLVMValue {
    pub type_info: InferType,
    pub llvm_type: LLVMType,
    pub value_id: String,
}

#[derive(Debug, Clone)]
pub struct LLVMFunction {
    pub name: String,
    pub params: Vec<LLVMValue>,
    pub return_type: LLVMType,
}

#[derive(Debug)]
pub enum CodegenError {
    UnsupportedFeature(String),
    UndefinedVariable(String),
    InvalidOperation(String),
    NotCallable(InferType),
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
        println!("Compiling program...");
        self.emit_object_structs(program)?;
        for stmt in &program.statements {
            println!("Compiling statement: {:?}", stmt);
            self.compile_statement(stmt)?;
        }
        let mut final_ir = self.output.clone();
        for g_str in &self.global_strings {
            final_ir.push_str(g_str);
            final_ir.push('\n');
        }
        println!("Finished compiling program.");
        Ok(final_ir)
    }

    pub fn emit_object_structs(&mut self, program: &Program) -> Result<(), CodegenError> {
        for stmt in &program.statements {
            if let crate::ast::StmtKind::Object { name, fields, .. } = &stmt.kind {
                let mut field_types = Vec::new();
                for field in fields {
                    let field_type = self.ast_type_to_infer_type(&field.type_);
                    field_types.push(infer_to_llvm_type(&field_type));
                }
                self.emit(&format!("%{} = type {{ {} }}", name, field_types.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(", ")));
            } else if let crate::ast::StmtKind::Store { name, fields, .. } = &stmt.kind {
                let mut field_types = Vec::new();
                for field in fields {
                    let field_type = self.ast_type_to_infer_type(&field.type_);
                    field_types.push(infer_to_llvm_type(&field_type));
                }
                self.emit(&format!("%{} = type {{ {} }}", name, field_types.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(", ")));
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

    

    pub(crate) fn get_type_size(&self, ty: &InferType) -> usize {
        match ty {
            InferType::Int => 8,
            InferType::Float => 8,
            InferType::Bool => 1,
            InferType::String | InferType::List(_) | InferType::Map(_, _) => 8, // Pointer size
            InferType::Object { fields, .. } => {
                fields.values().map(|f| self.get_type_size(f)).sum()
            }
            _ => 8, // Default pointer size
        }
    }

    pub(crate) fn type_to_string(&self, ty: &InferType) -> String {
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

    pub(crate) fn lookup_object_type(&self, name: &str) -> Option<InferType> {
        self.object_types.get(name).cloned()
    }

    pub fn compile_object_instantiation(&mut self, type_name: &str, obj_type: InferType, args: &[crate::ast::Expr]) -> Result<LLVMValue, CodegenError> {
        let struct_type = infer_to_llvm_type(&obj_type);
        let size = self.get_type_size(&obj_type);

        let malloc_temp = self.next_temp();
        self.emit(&format!("  %{} = call i8* @malloc(i64 {})", malloc_temp, size));
        
        let ptr_temp = self.next_temp();
        self.emit(&format!("  %{} = bitcast i8* %{} to {}*", ptr_temp, malloc_temp, struct_type));

        // Call the 'make' constructor
        let make_method_name = format!("_ZN{}{}{}{}", type_name.len(), type_name, 4, "make");
        let mut arg_vals = Vec::new();
        for arg in args {
            arg_vals.push(self.compile_expression(arg)?);
        }
        let call_args = arg_vals.iter()
            .map(|v| format!("{} {}", v.llvm_type, v.value_id))
            .collect::<Vec<String>>()
            .join(", ");
        self.emit(&format!("  call void @{}({}* %{}, {})", make_method_name, struct_type, ptr_temp, call_args));

        Ok(LLVMValue {
            type_info: obj_type,
            llvm_type: LLVMType::Pointer(Box::new(struct_type)),
            value_id: format!("%{}", ptr_temp),
        })
    }

    pub fn compile_function_call(&mut self, callee: &crate::ast::Expr, args: &[crate::ast::Expr]) -> Result<LLVMValue, CodegenError> {
        let callee_val = self.compile_expression(callee)?;
        
        let mut arg_vals = Vec::new();
        for arg in args {
            arg_vals.push(self.compile_expression(arg)?);
        }

        let (return_type, result_llvm_type) = if let InferType::Function { return_type, .. } = &callee_val.type_info {
            (return_type.as_ref().clone(), infer_to_llvm_type(return_type))
        } else {
            return Err(CodegenError::NotCallable(callee_val.type_info));
        };

        let result_temp = self.next_temp();
        let call_args = arg_vals.iter()
            .map(|v| format!("{} {}", v.llvm_type, v.value_id))
            .collect::<Vec<String>>()
            .join(", ");

        self.emit(&format!("  %{} = call {} {}({})", result_temp, result_llvm_type, callee_val.value_id, call_args));

        Ok(LLVMValue {
            type_info: return_type,
            llvm_type: result_llvm_type,
            value_id: format!("%{}", result_temp),
        })
    }

    pub fn compile_method_call(&mut self, object: &crate::ast::Expr, field: &str, args: &[crate::ast::Expr]) -> Result<LLVMValue, CodegenError> {
        let object_val = self.compile_expression(object)?;
        
        let mut arg_vals = Vec::new();
        // First argument is always the object itself
        arg_vals.push(object_val.clone());

        for arg in args {
            arg_vals.push(self.compile_expression(arg)?);
        }

        let type_info_str = self.type_to_string(&object_val.type_info);
        let mangled_name = format!("_ZN{}{}{}{}", type_info_str.len(), type_info_str, field.len(), field);

        let (return_type, result_llvm_type) = if let InferType::Object { methods, .. } = &object_val.type_info {
            if let Some(InferType::Function { return_type, .. }) = methods.get(field) {
                (return_type.as_ref().clone(), infer_to_llvm_type(return_type))
            } else {
                (InferType::Unknown, LLVMType::Pointer(Box::new(LLVMType::Int(8))))
            }
        } else {
            (InferType::Unknown, LLVMType::Pointer(Box::new(LLVMType::Int(8))))
        };

        let result_temp = self.next_temp();
        let call_args = arg_vals.iter()
            .map(|v| format!("{} {}", v.llvm_type, v.value_id))
            .collect::<Vec<String>>()
            .join(", ");

        self.emit(&format!("  %{} = call {} @{}({})", result_temp, result_llvm_type, mangled_name, call_args));

        Ok(LLVMValue {
            type_info: return_type,
            llvm_type: result_llvm_type,
            value_id: format!("%{}", result_temp),
        })
    }

    pub fn compile_property_access(&mut self, object: &crate::ast::Expr, field: &str) -> Result<LLVMValue, CodegenError> {
        let object_val = self.compile_expression(object)?;
        
        let (field_index, field_type) = if let InferType::Object { fields, .. } = &object_val.type_info {
            let mut index = 0;
            let mut result = None;
            for (name, ty) in fields.iter() {
                if name == field {
                    result = Some((index, ty.clone()));
                    break;
                }
                index += 1;
            }
            if let Some(r) = result {
                r
            } else {
                return Err(CodegenError::InvalidOperation(format!("Field {} not found in object", field)));
            }
        } else {
            return Err(CodegenError::InvalidOperation("Cannot access field on non-object type".to_string()));
        };
        
        let field_ptr_temp = self.next_temp();
        let object_llvm_type = infer_to_llvm_type(&object_val.type_info);

        self.emit(&format!("  %{} = getelementptr inbounds {}, {} {}, i32 0, i32 {}", 
            field_ptr_temp, 
            object_llvm_type.to_string().strip_suffix('*').unwrap_or(&object_llvm_type.to_string()), 
            object_llvm_type, 
            object_val.value_id, 
            field_index
        ));

        let result_temp = self.next_temp();
        let field_llvm_type = infer_to_llvm_type(&field_type);
        self.emit(&format!("  %{} = load {}, {}* %{}", result_temp, field_llvm_type, field_llvm_type, field_ptr_temp));

        Ok(LLVMValue {
            type_info: field_type,
            llvm_type: field_llvm_type,
            value_id: format!("%{}", result_temp),
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
                llvm_type: LLVMType::Void,
                value_id: "".to_string(),
            }
        };
        self.emit(&format!("  br label %{}", merge_label));
        let else_bb = self.current_bb;

        self.emit(&format!("{}:", merge_label));
        
        let result_type = then_val.type_info.clone();
        let result_llvm_type = infer_to_llvm_type(&result_type);

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
                llvm_type: LLVMType::Void,
                value_id: "".to_string(),
            })
        }
    }

    pub fn compile_list_literal(&mut self, elements: &[crate::ast::Expr]) -> Result<LLVMValue, CodegenError> {
        self.emit("declare i8* @list_new()");
        self.emit("declare void @list_append(i8*, i8*)");

        let list_ptr = self.next_temp();
        self.emit(&format!("  %{} = call i8* @list_new()", list_ptr));

        for element in elements {
            let element_val = self.compile_expression(element)?;
            let element_ptr = self.next_temp();
            self.emit(&format!("  %{} = bitcast {} {} to i8*", element_ptr, element_val.llvm_type, element_val.value_id));
            self.emit(&format!("  call void @list_append(i8* %{}, i8* %{})", list_ptr, element_ptr));
        }

        Ok(LLVMValue {
            type_info: InferType::List(Box::new(InferType::Unknown)), // Placeholder
            llvm_type: LLVMType::Pointer(Box::new(LLVMType::NamedStruct("list".to_string()))),
            value_id: format!("%{}", list_ptr),
        })
    }

    pub fn compile_list_append(&mut self, list: &crate::ast::Expr, element: &crate::ast::Expr) -> Result<LLVMValue, CodegenError> {
        let list_val = self.compile_expression(list)?;
        let element_val = self.compile_expression(element)?;

        let element_ptr = self.next_temp();
        self.emit(&format!("  %{} = bitcast {} {} to i8*", element_ptr, element_val.llvm_type, element_val.value_id));
        self.emit(&format!("  call void @list_append({} {}, i8* %{})", list_val.llvm_type, list_val.value_id, element_ptr));

        Ok(LLVMValue {
            type_info: InferType::Unit,
            llvm_type: LLVMType::Void,
            value_id: "".to_string(),
        })
    }

    pub fn compile_map_literal(&mut self, elements: &[(crate::ast::Expr, crate::ast::Expr)]) -> Result<LLVMValue, CodegenError> {
        self.emit("declare i8* @map_new()");
        self.emit("declare void @map_insert(i8*, i8*, i8*)");

        let map_ptr = self.next_temp();
        self.emit(&format!("  %{} = call i8* @map_new()", map_ptr));

        for (key, value) in elements {
            let key_val = self.compile_expression(key)?;
            let value_val = self.compile_expression(value)?;

            let key_ptr = self.next_temp();
            self.emit(&format!("  %{} = bitcast {} {} to i8*", key_ptr, key_val.llvm_type, key_val.value_id));
            let value_ptr = self.next_temp();
            self.emit(&format!("  %{} = bitcast {} {} to i8*", value_ptr, value_val.llvm_type, value_val.value_id));

            self.emit(&format!("  call void @map_insert(i8* %{}, i8* %{}, i8* %{})", map_ptr, key_ptr, value_ptr));
        }

        Ok(LLVMValue {
            type_info: InferType::Map(Box::new(InferType::Unknown), Box::new(InferType::Unknown)), // Placeholder
            llvm_type: LLVMType::Pointer(Box::new(LLVMType::NamedStruct("map".to_string()))),
            value_id: format!("%{}", map_ptr),
        })
    }

    pub fn compile_map_insert(&mut self, map: &crate::ast::Expr, key: &crate::ast::Expr, value: &crate::ast::Expr) -> Result<LLVMValue, CodegenError> {
        let map_val = self.compile_expression(map)?;
        let key_val = self.compile_expression(key)?;
        let value_val = self.compile_expression(value)?;

        let key_ptr = self.next_temp();
        self.emit(&format!("  %{} = bitcast {} {} to i8*", key_ptr, key_val.llvm_type, key_val.value_id));
        let value_ptr = self.next_temp();
        self.emit(&format!("  %{} = bitcast {} {} to i8*", value_ptr, value_val.llvm_type, value_val.value_id));

        self.emit(&format!("  call void @map_insert({} {}, i8* %{}, i8* %{})", map_val.llvm_type, map_val.value_id, key_ptr, value_ptr));

        Ok(LLVMValue {
            type_info: InferType::Unit,
            llvm_type: LLVMType::Void,
            value_id: "".to_string(),
        })
    }

    pub fn compile_string_interpolation(&mut self, parts: &[crate::ast::StringPart]) -> Result<LLVMValue, CodegenError> {
        self.emit("declare i8* @string_concat(i8*, i8*)");
        self.emit("declare i8* @string_from_int(i64)");
        self.emit("declare i8* @string_from_float(double)");
        self.emit("declare i8* @string_new()");

        let mut last_string_ptr = self.next_temp();
        self.emit(&format!("  %{} = call i8* @string_new()", last_string_ptr));

        for part in parts {
            match part {
                crate::ast::StringPart::Literal(s) => {
                    let literal_val = self.compile_literal(&crate::ast::Literal::String(s.clone()))?;
                    let next_temp = self.next_temp();
                    self.emit(&format!("  %{} = call i8* @string_concat(i8* %{}, {} {})", next_temp, last_string_ptr, literal_val.llvm_type, literal_val.value_id));
                    last_string_ptr = next_temp;
                }
                crate::ast::StringPart::Expression(expr) => {
                    let expr_val = self.compile_expression(expr)?;
                    let converted_str_ptr = self.next_temp();
                    match expr_val.type_info {
                        InferType::Int => self.emit(&format!("  %{} = call i8* @string_from_int({} {})", converted_str_ptr, expr_val.llvm_type, expr_val.value_id)),
                        InferType::Float => self.emit(&format!("  %{} = call i8* @string_from_float({} {})", converted_str_ptr, expr_val.llvm_type, expr_val.value_id)),
                        InferType::String => self.emit(&format!("  %{} = bitcast {} {} to i8*", converted_str_ptr, expr_val.llvm_type, expr_val.value_id)),
                        _ => return Err(CodegenError::InvalidOperation("Cannot interpolate non-stringable type".to_string())),
                    }
                    let next_temp = self.next_temp();
                    self.emit(&format!("  %{} = call i8* @string_concat(i8* %{}, i8* %{})", next_temp, last_string_ptr, converted_str_ptr));
                    last_string_ptr = next_temp;
                }
            }
        }

        Ok(LLVMValue {
            type_info: InferType::String,
            llvm_type: LLVMType::Pointer(Box::new(LLVMType::NamedStruct("string".to_string()))),
            value_id: format!("%{}", last_string_ptr),
        })
    }
}
