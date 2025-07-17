use crate::ast::*;
use crate::resolver::InferType;
use std::collections::HashMap;
use std::fmt;

/// Errors that can occur during LLVM IR generation
#[derive(Debug, Clone)]
pub enum CodegenError {
    TypeResolutionFailed(String),
    UnsupportedFeature(String),
    UndefinedFunction(String),
    UndefinedVariable(String),
    InvalidOperation(String),
    MemoryAllocationFailed,
    LLVMError(String),
}

impl fmt::Display for CodegenError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CodegenError::TypeResolutionFailed(msg) => write!(f, "Type resolution failed: {}", msg),
            CodegenError::UnsupportedFeature(feature) => write!(f, "Unsupported feature: {}", feature),
            CodegenError::UndefinedFunction(name) => write!(f, "Undefined function: {}", name),
            CodegenError::UndefinedVariable(name) => write!(f, "Undefined variable: {}", name),
            CodegenError::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
            CodegenError::MemoryAllocationFailed => write!(f, "Memory allocation failed"),
            CodegenError::LLVMError(msg) => write!(f, "LLVM error: {}", msg),
        }
    }
}

impl std::error::Error for CodegenError {}

/// Represents a compiled LLVM value
#[derive(Debug, Clone)]
pub struct LLVMValue {
    pub type_info: InferType,
    pub llvm_type: String,
    pub value_id: String,
}

/// Represents a compiled LLVM function
#[derive(Debug, Clone)]
pub struct LLVMFunction {
    pub name: String,
    pub parameters: Vec<(String, InferType)>,
    pub return_type: InferType,
    pub body: String, // LLVM IR body
}

/// Symbol table for tracking variables and functions during compilation
#[derive(Debug, Clone, Default)]
struct SymbolTable {
    variables: HashMap<String, LLVMValue>,
    functions: HashMap<String, LLVMFunction>,
    parent: Option<Box<SymbolTable>>,
}

impl SymbolTable {
    fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            parent: None,
        }
    }

    fn with_parent(parent: SymbolTable) -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            parent: Some(Box::new(parent)),
        }
    }

    fn define_variable(&mut self, name: String, value: LLVMValue) {
        self.variables.insert(name, value);
    }

    fn define_function(&mut self, name: String, function: LLVMFunction) {
        self.functions.insert(name, function);
    }

    fn lookup_variable(&self, name: &str) -> Option<&LLVMValue> {
        self.variables.get(name).or_else(|| {
            self.parent.as_ref().and_then(|p| p.lookup_variable(name))
        })
    }

    fn lookup_function(&self, name: &str) -> Option<&LLVMFunction> {
        self.functions.get(name).or_else(|| {
            self.parent.as_ref().and_then(|p| p.lookup_function(name))
        })
    }
}

/// LLVM IR code generator for Coral
pub struct LLVMCodegen {
    module_name: String,
    symbols: SymbolTable,
    ir_output: Vec<String>,
    next_label_id: usize,
    next_temp_id: usize,
}

impl LLVMCodegen {
    pub fn new(module_name: String) -> Self {
        let mut codegen = Self {
            module_name,
            symbols: SymbolTable::new(),
            ir_output: Vec::new(),
            next_label_id: 0,
            next_temp_id: 0,
        };

        // Add built-in functions
        codegen.add_builtin_functions();
        codegen
    }

    /// Generate LLVM IR for a complete program
    pub fn compile_program(&mut self, program: &Program) -> Result<String, CodegenError> {
        self.emit_module_header();
        self.emit_builtin_declarations();

        for stmt in &program.statements {
            self.compile_statement(stmt)?;
        }

        self.emit_main_function()?;
        Ok(self.ir_output.join("\n"))
    }

    /// Compile a single statement
    fn compile_statement(&mut self, stmt: &Stmt) -> Result<(), CodegenError> {
        match &stmt.kind {
            StmtKind::Function { name, params, return_type, body } => {
                self.compile_function_definition(name, params, return_type.as_ref(), body)
            }
            StmtKind::Assignment { target, value } => {
                self.compile_assignment(target, value)
            }
            StmtKind::Expression(expr) => {
                self.compile_expression(expr)?;
                Ok(())
            }
            StmtKind::Return(expr_opt) => {
                if let Some(expr) = expr_opt {
                    let return_value = self.compile_expression(expr)?;
                    self.emit(&format!(
                        "  ret {} {}",
                        return_value.llvm_type,
                        return_value.value_id
                    ));
                } else {
                    self.emit("  ret void");
                }
                Ok(())
            }
            StmtKind::Object { name, fields, methods: _ } => {
                // Emit LLVM struct type for object
                self.emit_object_struct_type(name, fields);
                // Track object type in symbol table
                let mut field_types = HashMap::new();
                for field in fields {
                    field_types.insert(field.name.clone(), self.ast_type_to_infer_type(&field.type_));
                }
                let obj_type_info = InferType::Object {
                    name: name.clone(),
                    fields: field_types.clone(),
                    methods: HashMap::new(),
                    is_actor: false,
                    is_store: false,
                };
                self.symbols.define_variable(name.clone(), LLVMValue {
                    type_info: obj_type_info.clone(),
                    llvm_type: self.infer_type_to_llvm_type(&obj_type_info),
                    value_id: format!("%struct.{}", name),
                });
                Ok(())
            }
            StmtKind::Store { .. } => {
                Err(CodegenError::UnsupportedFeature("Store definitions not yet implemented".to_string()))
            }
            StmtKind::Actor { .. } => {
                Err(CodegenError::UnsupportedFeature("Actor definitions not yet implemented".to_string()))
            }
            StmtKind::ErrorHandler { handler, inner } => {
                // Compile the guarded statement
                self.compile_statement(inner)?;
                // Emit error handling actions as comments for now (stub)
                for action in &handler.actions {
                    match action {
                        ErrorAction::Log(opt_expr) => {
                            if let Some(expr) = opt_expr {
                                self.emit(&format!("  ; err log: {:?}", expr));
                            } else {
                                self.emit("  ; err log");
                            }
                        }
                        ErrorAction::Return(opt_expr) => {
                            if let Some(expr) = opt_expr {
                                self.emit(&format!("  ; err return: {:?}", expr));
                            } else {
                                self.emit("  ; err return");
                            }
                        }
                        ErrorAction::Custom(expr) => {
                            self.emit(&format!("  ; err custom: {:?}", expr));
                        }
                    }
                }
                Ok(())
            }
            StmtKind::If { condition, then_branch, else_branch } => {
                self.compile_if_statement(condition, then_branch, else_branch.as_deref())
            }
            StmtKind::While { condition, body } => {
                self.compile_while_statement(condition, body)
            }
            StmtKind::Unless { condition, body } => {
                self.compile_unless_statement(condition, body)
            }
            StmtKind::Until { condition, body } => {
                self.compile_until_statement(condition, body)
            }
            StmtKind::For { variable, iterable, body } => {
                self.compile_for_statement(variable, iterable, body)
            }
            StmtKind::Iterate { iterable, body } => {
                self.compile_iterate_statement(iterable, body)
            }
            StmtKind::Break => {
                self.emit("  br label %L_break"); // Placeholder for now
                Ok(())
            }
            StmtKind::Continue => {
                self.emit("  br label %L_continue"); // Placeholder for now
                Ok(())
            }
            // StmtKind::Pipe { name, source, destination, nocopy } => {
                //     // Compile a pipe statement (with nocopy IO)
                //     self.compile_pipe_statement(name, source, destination, *nocopy)
                // }
                // StmtKind::Io { op, args, nocopy } => {
                //     // Compile an IO statement (with nocopy)
                //     self.compile_io_statement(op, args, *nocopy)
                // }
            _ => {
                Err(CodegenError::UnsupportedFeature(format!("Statement type not implemented: {:?}", stmt.kind)))
            }
        }
    }

    /// Compile an expression and return its LLVM value
    fn compile_expression(&mut self, expr: &Expr) -> Result<LLVMValue, CodegenError> {
        match &expr.kind {
            ExprKind::Literal(lit) => self.compile_literal(lit),
            ExprKind::Identifier(name) => {
                self.symbols.lookup_variable(name)
                    .cloned()
                    .ok_or_else(|| CodegenError::UndefinedVariable(name.clone()))
            }
            ExprKind::Binary { op, left, right } => {
                self.compile_binary_operation(op, left, right)
            }
            ExprKind::Call { callee, args } => {
                match &callee.kind {
                    ExprKind::Identifier(type_or_func) => {
                        if let Some(obj_type) = self.lookup_object_type(type_or_func) {
                            self.compile_object_instantiation(type_or_func, obj_type, args)
                        } else {
                            self.compile_function_call(callee, args)
                        }
                    }
                    ExprKind::FieldAccess { object, field } => {
                        self.compile_method_call(object, field, args)
                    }
                    _ => self.compile_function_call(callee, args),
                }
            }
            ExprKind::FieldAccess { object, field } => {
                // Property access: object.field
                self.compile_property_access(object, field)
            }
            ExprKind::If { condition, then_branch, else_branch } => {
                self.compile_if_expression(condition, then_branch, else_branch.as_deref())
            }
            ExprKind::ListLiteral(elements) => self.compile_list_literal(elements),
            ExprKind::ListAppend { list, element } => self.compile_list_append(list, element),
            ExprKind::MapLiteral(elements) => self.compile_map_literal(elements),
            ExprKind::MapInsert { map, key, value } => self.compile_map_insert(map, key, value),
            _ => Err(CodegenError::UnsupportedFeature(
                format!("Expression type not implemented: {:?}", expr.kind)
            ))
        }
    }

    /// Lookup object type by name (stub)
    fn lookup_object_type(&self, name: &str) -> Option<InferType> {
        if let Some(val) = self.symbols.lookup_variable(name) {
            if let InferType::Object { name: obj_name, fields, .. } = &val.type_info {
                if obj_name == name {
                    return Some(InferType::Object {
                        name: obj_name.clone(),
                        fields: fields.clone(),
                        methods: HashMap::new(),
                        is_actor: false,
                        is_store: false,
                    });
                }
            }
        }
        None
    }

    /// Emit LLVM struct type for object (stub)
    fn emit_object_struct_type(&mut self, name: &str, fields: &[Field]) {
        let mut llvm_fields = Vec::new();
        for field in fields {
            let llvm_type = self.infer_type_to_llvm_type(&self.ast_type_to_infer_type(&field.type_));
            llvm_fields.push(llvm_type.to_string());
        }
        self.emit(&format!("%struct.{} = type {{ {} }}", name, llvm_fields.join(", ")));
    }

    /// Compile object instantiation (stub)
    fn compile_object_instantiation(&mut self, _name: &str, obj_type: InferType, args: &[Expr]) -> Result<LLVMValue, CodegenError> {
        // Allocate struct and initialize fields
        let temp_id = self.next_temp();
        let llvm_type = self.infer_type_to_llvm_type(&obj_type);
        self.emit(&format!("  %{} = alloca {}", temp_id, llvm_type));
        // Initialize fields
        if let InferType::Object { ref fields, .. } = obj_type {
            for (i, (_field_name, field_infer_type)) in fields.iter().enumerate() {
                let value = if i < args.len() {
                    self.compile_expression(&args[i])?
                } else {
                    // Default to zero/null based on type
                    LLVMValue {
                        type_info: field_infer_type.clone(),
                        llvm_type: self.infer_type_to_llvm_type(field_infer_type),
                        value_id: match field_infer_type {
                            InferType::Int => "0".to_string(),
                            InferType::Float => "0.0".to_string(),
                            InferType::Bool => "false".to_string(),
                            InferType::String => "null".to_string(),
                            _ => "null".to_string(), // Default for complex types
                        },
                    }
                };
                let field_llvm_type = self.infer_type_to_llvm_type(field_infer_type);
                self.emit(&format!(
                    "  %fieldptr{} = getelementptr inbounds {}, {}* %{}, i32 0, i32 {}",
                    temp_id, llvm_type, llvm_type, temp_id, i
                ));
                self.emit(&format!(
                    "  store {} {}, {}* %fieldptr{}",
                    value.llvm_type, value.value_id, field_llvm_type, temp_id
                ));
            }
        }
        Ok(LLVMValue {
            type_info: obj_type,
            llvm_type: llvm_type.clone(),
            value_id: format!("%{}", temp_id),
        })
    }

    /// Compile method call on object (stub)
    fn compile_method_call(&mut self, object: &Expr, method: &str, args: &[Expr]) -> Result<LLVMValue, CodegenError> {
        let obj_val = self.compile_expression(object)?;
        // Method lookup: for now, assume method is a function named "<object_type>_<method>"
        let method_func_name = format!("{}_{}", obj_val.llvm_type.trim_start_matches("%struct.").trim_end_matches('*'), method);
        let mut arg_values = vec![obj_val];
        for arg in args {
            arg_values.push(self.compile_expression(arg)?);
        }
        let result_temp = self.next_temp();
        let arg_list = arg_values.iter()
            .map(|v| format!("{} {}", v.llvm_type, v.value_id))
            .collect::<Vec<_>>()
            .join(", ");
        // TODO: Infer return type of the method call
        let return_llvm_type = "i64"; // Placeholder for now
        self.emit(&format!(
            "  %{} = call {} @{}({})",
            result_temp,
            return_llvm_type,
            method_func_name,
            arg_list
        ));
        Ok(LLVMValue {
            type_info: InferType::Unknown, // TODO: Infer actual type
            llvm_type: return_llvm_type.to_string(),
            value_id: format!("%{}", result_temp),
        })
    }

    /// Compile property access (stub)
    fn compile_property_access(&mut self, object: &Expr, field: &str) -> Result<LLVMValue, CodegenError> {
        let obj_val = self.compile_expression(object)?;
        // Lookup field index
        let field_index = if let InferType::Object { fields, .. } = &obj_val.type_info {
            fields.keys().position(|k| k == field)
        } else {
            None
        };
        let idx = field_index.ok_or_else(|| CodegenError::InvalidOperation(format!("Field '{}' not found", field)))?;
        let temp_id = self.next_temp();
        let field_infer_type = if let InferType::Object { fields, .. } = &obj_val.type_info {
            fields.get(field).cloned().unwrap_or(InferType::Unknown)
        } else {
            InferType::Unknown
        };
        let field_llvm_type = self.infer_type_to_llvm_type(&field_infer_type);
        self.emit(&format!(
            "  %fieldptr{} = getelementptr inbounds {}, {}* %{}, i32 0, i32 {}",
            temp_id,
            obj_val.llvm_type.trim_end_matches('*'),
            obj_val.llvm_type.trim_end_matches('*'),
            obj_val.value_id.trim_start_matches('%'),
            idx
        ));
        let result_temp = self.next_temp();
        self.emit(&format!(
            "  %{} = load {}, {}* %fieldptr{}",
            result_temp, field_llvm_type, field_llvm_type, temp_id
        ));
        Ok(LLVMValue {
            type_info: field_infer_type,
            llvm_type: field_llvm_type,
            value_id: format!("%{}", result_temp),
        })
    }

    /// Compile an if/ternary expression
    fn compile_if_expression(&mut self, condition: &Expr, then_branch: &Expr, else_branch: Option<&Expr>) -> Result<LLVMValue, CodegenError> {
        let cond_val = self.compile_expression(condition)?;

        let then_label = self.next_label();
        let else_label = self.next_label();
        let merge_label = self.next_label();

        self.emit(&format!(
            "  br i1 {}, label %L{}, label %L{}",
            cond_val.value_id,
            then_label,
            else_label
        ));

        // Then branch
        self.emit(&format!("L{}:", then_label));
        let then_val = self.compile_expression(then_branch)?;
        self.emit(&format!("  br label %L{}", merge_label));

        // Else branch
        self.emit(&format!("L{}:", else_label));
        let else_val = if let Some(else_expr) = else_branch {
            self.compile_expression(else_expr)?
        } else {
            // Handle case with no else branch (e.g., if statement)
            // For now, assume a default value for the type
            LLVMValue {
                type_info: then_val.type_info.clone(),
                llvm_type: self.infer_type_to_llvm_type(&then_val.type_info),
                value_id: "null".to_string(), // Placeholder
            }
        };
        self.emit(&format!("  br label %L{}", merge_label));

        // Merge block
        self.emit(&format!("L{}:", merge_label));
        let result_temp = self.next_temp();

        self.emit(&format!(
            "  %{} = phi {} [ {}, %L{} ], [ {}, %L{} ]",
            result_temp,
            then_val.llvm_type,
            then_val.value_id,
            then_label,
            else_val.value_id,
            else_label
        ));

        Ok(LLVMValue {
            type_info: then_val.type_info.clone(),
            llvm_type: self.infer_type_to_llvm_type(&then_val.type_info),
            value_id: format!("%{}", result_temp),
        })
    }

    /// Compile an if statement
    fn compile_if_statement(&mut self, condition: &Expr, then_branch: &[Stmt], else_branch: Option<&[Stmt]>) -> Result<(), CodegenError> {
        let cond_val = self.compile_expression(condition)?;

        let then_label = self.next_label();
        let else_label = self.next_label();
        let merge_label = self.next_label();

        self.emit(&format!(
            "  br i1 {}, label %L{}, label %L{}",
            cond_val.value_id,
            then_label,
            else_label
        ));

        // Then branch
        self.emit(&format!("L{}:", then_label));
        for stmt in then_branch {
            self.compile_statement(stmt)?;
        }
        self.emit(&format!("  br label %L{}", merge_label));

        // Else branch
        self.emit(&format!("L{}:", else_label));
        if let Some(else_stmts) = else_branch {
            for stmt in else_stmts {
                self.compile_statement(stmt)?;
            }
        }
        self.emit(&format!("  br label %L{}", merge_label));

        // Merge block
        self.emit(&format!("L{}:", merge_label));
        Ok(())
    }

    /// Compile a while statement
    fn compile_while_statement(&mut self, condition: &Expr, body: &[Stmt]) -> Result<(), CodegenError> {
        let loop_header_label = self.next_label();
        let loop_body_label = self.next_label();
        let loop_end_label = self.next_label();

        self.emit(&format!("  br label %L{}", loop_header_label));

        self.emit(&format!("L{}:", loop_header_label));
        let cond_val = self.compile_expression(condition)?;
        self.emit(&format!(
            "  br i1 {}, label %L{}, label %L{}",
            cond_val.value_id,
            loop_body_label,
            loop_end_label
        ));

        self.emit(&format!("L{}:", loop_body_label));
        for stmt in body {
            self.compile_statement(stmt)?;
        }
        self.emit(&format!("  br label %L{}", loop_header_label));

        self.emit(&format!("L{}:", loop_end_label));
        Ok(())
    }

    /// Compile an unless statement
    fn compile_unless_statement(&mut self, condition: &Expr, body: &[Stmt]) -> Result<(), CodegenError> {
        let cond_val = self.compile_expression(condition)?;

        let then_label = self.next_label();
        let merge_label = self.next_label();

        // Unless is equivalent to if not condition
        self.emit(&format!(
            "  br i1 {}, label %L{}, label %L{}",
            cond_val.value_id,
            merge_label, // If condition is true, skip body
            then_label   // If condition is false, execute body
        ));

        self.emit(&format!("L{}:", then_label));
        for stmt in body {
            self.compile_statement(stmt)?;
        }
        self.emit(&format!("  br label %L{}", merge_label));

        self.emit(&format!("L{}:", merge_label));
        Ok(())
    }

    /// Compile an until statement
    fn compile_until_statement(&mut self, condition: &Expr, body: &[Stmt]) -> Result<(), CodegenError> {
        let loop_header_label = self.next_label();
        let loop_body_label = self.next_label();
        let loop_end_label = self.next_label();

        self.emit(&format!("  br label %L{}", loop_header_label));

        self.emit(&format!("L{}:", loop_header_label));
        let cond_val = self.compile_expression(condition)?;
        // Until is equivalent to while not condition
        self.emit(&format!(
            "  br i1 {}, label %L{}, label %L{}",
            cond_val.value_id,
            loop_end_label, // If condition is true, end loop
            loop_body_label // If condition is false, continue loop
        ));

        self.emit(&format!("L{}:", loop_body_label));
        for stmt in body {
            self.compile_statement(stmt)?;
        }
        self.emit(&format!("  br label %L{}", loop_header_label));

        self.emit(&format!("L{}:", loop_end_label));
        Ok(())
    }

    /// Compile a for statement
    fn compile_for_statement(&mut self, variable: &str, iterable: &Expr, body: &[Stmt]) -> Result<(), CodegenError> {
        let iterable_val = self.compile_expression(iterable)?;
        let loop_header_label = self.next_label();
        let loop_body_label = self.next_label();
        let loop_end_label = self.next_label();

        // For now, only support iterating over lists (simplified)
        if let InferType::List(element_type) = iterable_val.type_info {
            let element_llvm_type = self.infer_type_to_llvm_type(&element_type);
            let element_size = self.get_type_size(&element_type);

            // Get list properties
            let list_ptr_temp = iterable_val.value_id.trim_start_matches('%');
            let data_ptr_gep = self.next_temp();
            self.emit(&format!(
                "  %{} = getelementptr inbounds %coral.list, %coral.list* %{}, i32 0, i32 0",
                data_ptr_gep,
                list_ptr_temp
            ));
            let current_data_ptr = self.next_temp();
            self.emit(&format!(
                "  %{} = load i8*, i8** %{}",
                current_data_ptr,
                data_ptr_gep
            ));

            let len_gep = self.next_temp();
            self.emit(&format!(
                "  %{} = getelementptr inbounds %coral.list, %coral.list* %{}, i32 0, i32 1",
                len_gep,
                list_ptr_temp
            ));
            let list_len = self.next_temp();
            self.emit(&format!(
                "  %{} = load i64, i64* %{}",
                list_len,
                len_gep
            ));

            // Loop counter
            let counter_alloca = self.next_temp();
            self.emit(&format!("  %{} = alloca i64", counter_alloca));
            self.emit(&format!("  store i64 0, i64* %{}", counter_alloca));

            self.emit(&format!("  br label %L{}", loop_header_label));

            self.emit(&format!("L{}:", loop_header_label));
            let current_counter = self.next_temp();
            self.emit(&format!("  %{} = load i64, i64* %{}", current_counter, counter_alloca));

            let loop_cond = self.next_temp();
            self.emit(&format!("  %{} = icmp slt i64 %{}, %{}", loop_cond, current_counter, list_len));
            self.emit(&format!("  br i1 %{}, label %L{}, label %L{}", loop_cond, loop_body_label, loop_end_label));

            self.emit(&format!("L{}:", loop_body_label));
            // Get current element
            let offset = self.next_temp();
            self.emit(&format!("  %{} = mul i64 %{}, {}", offset, current_counter, element_size));
            let element_ptr = self.next_temp();
            self.emit(&format!("  %{} = getelementptr inbounds i8, i8* %{}, i64 %{}", element_ptr, current_data_ptr, offset));
            let cast_element_ptr = self.next_temp();
            self.emit(&format!("  %{} = bitcast i8* %{} to {}*", cast_element_ptr, element_ptr, element_llvm_type));
            let loaded_element = self.next_temp();
            self.emit(&format!("  %{} = load {}, {}* %{}", loaded_element, element_llvm_type, element_llvm_type, cast_element_ptr));

            // Define loop
            let parent_symbols = std::mem::take(&mut self.symbols);
            self.symbols = SymbolTable::with_parent(parent_symbols.clone());
            self.symbols.define_variable(variable.to_string(), LLVMValue {
                type_info: *element_type.clone(),
                llvm_type: element_llvm_type.clone(),
                value_id: format!("%{}", loaded_element),
            });

            for stmt in body {
                self.compile_statement(stmt)?;
            }

            // Restore parent scope
            self.symbols = parent_symbols;

            // Increment counter
            let next_counter = self.next_temp();
            self.emit(&format!("  %{} = add i64 %{}, 1", next_counter, current_counter));
            self.emit(&format!("  store i64 %{}, i64* %{}", next_counter, counter_alloca));
            self.emit(&format!("  br label %L{}", loop_header_label));

            self.emit(&format!("L{}:", loop_end_label));
            Ok(())
        } else {
            Err(CodegenError::UnsupportedFeature("For loop over non-list iterable not yet implemented".to_string()))
        }
    }

    /// Compile an iterate statement
    fn compile_iterate_statement(&mut self, iterable: &Expr, body: &[Stmt]) -> Result<(), CodegenError> {
        let iterable_val = self.compile_expression(iterable)?;
        let loop_header_label = self.next_label();
        let loop_body_label = self.next_label();
        let loop_end_label = self.next_label();

        // For now, only support iterating over lists (simplified)
        if let InferType::List(element_type) = iterable_val.type_info {
            let element_llvm_type = self.infer_type_to_llvm_type(&element_type);
            let element_size = self.get_type_size(&element_type);

            // Get list properties
            let list_ptr_temp = iterable_val.value_id.trim_start_matches('%');
            let data_ptr_gep = self.next_temp();
            self.emit(&format!(
                "  %{} = getelementptr inbounds %coral.list, %coral.list* %{}, i32 0, i32 0",
                data_ptr_gep,
                list_ptr_temp
            ));
            let current_data_ptr = self.next_temp();
            self.emit(&format!(
                "  %{} = load i8*, i8** %{}",
                current_data_ptr,
                data_ptr_gep
            ));

            let len_gep = self.next_temp();
            self.emit(&format!(
                "  %{} = getelementptr inbounds %coral.list, %coral.list* %{}, i32 0, i32 1",
                len_gep,
                list_ptr_temp
            ));
            let list_len = self.next_temp();
            self.emit(&format!(
                "  %{} = load i64, i64* %{}",
                list_len,
                len_gep
            ));

            // Loop counter
            let counter_alloca = self.next_temp();
            self.emit(&format!("  %{} = alloca i64", counter_alloca));
            self.emit(&format!("  store i64 0, i64* %{}", counter_alloca));

            self.emit(&format!("  br label %L{}", loop_header_label));

            self.emit(&format!("L{}:", loop_header_label));
            let current_counter = self.next_temp();
            self.emit(&format!("  %{} = load i64, i64* %{}", current_counter, counter_alloca));

            let loop_cond = self.next_temp();
            self.emit(&format!("  %{} = icmp slt i64 %{}, %{}", loop_cond, current_counter, list_len));
            self.emit(&format!("  br i1 %{}, label %L{}, label %L{}", loop_cond, loop_body_label, loop_end_label));

            self.emit(&format!("L{}:", loop_body_label));
            // Get current element
            let offset = self.next_temp();
            self.emit(&format!("  %{} = mul i64 %{}, {}", offset, current_counter, element_size));
            let element_ptr = self.next_temp();
            self.emit(&format!("  %{} = getelementptr inbounds i8, i8* %{}, i64 %{}", element_ptr, current_data_ptr, offset));
            let cast_element_ptr = self.next_temp();
            self.emit(&format!("  %{} = bitcast i8* %{} to {}*", cast_element_ptr, element_ptr, element_llvm_type));
            let loaded_element = self.next_temp();
            self.emit(&format!("  %{} = load {}, {}* %{}", loaded_element, element_llvm_type, element_llvm_type, cast_element_ptr));

        }
            // Define '

        // Merge block
        self.emit(&format!("L{}:", loop_end_label));
        Ok(())
    }

    /// Compile a list literal
    fn compile_list_literal(&mut self, elements: &[Expr]) -> Result<LLVMValue, CodegenError> {
        let element_type = if let Some(first) = elements.first() {
            self.ast_type_to_infer_type(&first.type_)
        } else {
            InferType::Unknown // Empty list
        };

        let llvm_element_type = self.infer_type_to_llvm_type(&element_type);
        let element_size = self.get_type_size(&element_type);

        let capacity = elements.len();
        let total_size = capacity * element_size;

        // Allocate memory for the list elements
        let data_ptr_temp = self.next_temp();
        self.emit(&format!(
            "  %{} = call i8* @malloc(i64 {})",
            data_ptr_temp,
            total_size
        ));

        // Allocate memory for the list struct
        let list_ptr_temp = self.next_temp();
        self.emit(&format!(
            "  %{} = alloca %coral.list",
            list_ptr_temp
        ));

        // Store data pointer, length, and capacity in the struct
        let data_gep_temp = self.next_temp();
        self.emit(&format!(
            "  %{} = getelementptr inbounds %coral.list, %coral.list* %{}, i32 0, i32 0",
            data_gep_temp,
            list_ptr_temp
        ));
        self.emit(&format!(
            "  store i8* %{}, i8** %{}",
            data_ptr_temp,
            data_gep_temp
        ));

        let len_gep_temp = self.next_temp();
        self.emit(&format!(
            "  %{} = getelementptr inbounds %coral.list, %coral.list* %{}, i32 0, i32 1",
            len_gep_temp,
            list_ptr_temp
        ));
        self.emit(&format!(
            "  store i64 {}, i64* %{}",
            elements.len(),
            len_gep_temp
        ));

        let cap_gep_temp = self.next_temp();
        self.emit(&format!(
            "  %{} = getelementptr inbounds %coral.list, %coral.list* %{}, i32 0, i32 2",
            cap_gep_temp,
            list_ptr_temp
        ));
        self.emit(&format!(
            "  store i64 {}, i64* %{}",
            capacity,
            cap_gep_temp
        ));

        // Store elements
        for (i, element_expr) in elements.iter().enumerate() {
            let element_val = self.compile_expression(element_expr)?;
            let offset = i * element_size;
            let element_ptr_temp = self.next_temp();
            self.emit(&format!(
                "  %{} = getelementptr inbounds i8, i8* %{}, i64 {}",
                element_ptr_temp,
                data_ptr_temp,
                offset
            ));
            let cast_ptr_temp = self.next_temp();
            self.emit(&format!(
                "  %{} = bitcast i8* %{} to {}*",
                cast_ptr_temp,
                element_ptr_temp,
                llvm_element_type
            ));
            self.emit(&format!(
                "  store {} {}, {}* %{}",
                element_val.llvm_type,
                element_val.value_id,
                llvm_element_type,
                cast_ptr_temp
            ));
        }

        Ok(LLVMValue {
            type_info: InferType::List(Box::new(element_type)),
            llvm_type: "%coral.list*".to_string(),
            value_id: format!("%{}", list_ptr_temp),
        })
    }

    /// Compile a list append operation
    fn compile_list_append(&mut self, list: &Expr, element: &Expr) -> Result<LLVMValue, CodegenError> {
        let list_val = self.compile_expression(list)?;
        let element_val = self.compile_expression(element)?;

        // Get list properties
        let list_ptr_temp = list_val.value_id.trim_start_matches('%');

        let data_ptr_gep = self.next_temp();
        self.emit(&format!(
            "  %{} = getelementptr inbounds %coral.list, %coral.list* %{}, i32 0, i32 0",
            data_ptr_gep,
            list_ptr_temp
        ));
        let current_data_ptr = self.next_temp();
        self.emit(&format!(
            "  %{} = load i8*, i8** %{}",
            current_data_ptr,
            data_ptr_gep
        ));

        let len_gep = self.next_temp();
        self.emit(&format!(
            "  %{} = getelementptr inbounds %coral.list, %coral.list* %{}, i32 0, i32 1",
            len_gep,
            list_ptr_temp
        ));
        let current_len = self.next_temp();
        self.emit(&format!(
            "  %{} = load i64, i64* %{}",
            current_len,
            len_gep
        ));

        let cap_gep = self.next_temp();
        self.emit(&format!(
            "  %{} = getelementptr inbounds %coral.list, %coral.list* %{}, i32 0, i32 2",
            cap_gep,
            list_ptr_temp
        ));
        let current_cap = self.next_temp();
        self.emit(&format!(
            "  %{} = load i64, i64* %{}",
            current_cap,
            cap_gep
        ));

        // Check if reallocation is needed
        let need_realloc_label = self.next_label();
        let no_realloc_label = self.next_label();
        let merge_realloc_label = self.next_label();

        let compare_temp = self.next_temp();
        self.emit(&format!(
            "  %{} = icmp eq i64 %{}, %{}",
            compare_temp,
            current_len,
            current_cap
        ));
        self.emit(&format!(
            "  br i1 %{}, label %L{}, label %L{}",
            compare_temp,
            need_realloc_label,
            no_realloc_label
        ));

        // Reallocation block
        self.emit(&format!("L{}:", need_realloc_label));
        let new_cap_temp = self.next_temp();
        self.emit(&format!(
            "  %{} = mul i64 %{}, 2",
            new_cap_temp,
            current_cap
        ));
        let new_size_temp = self.next_temp();
        let element_size = self.get_type_size(&element_val.type_info);
        self.emit(&format!(
            "  %{} = mul i64 %{}, {}",
            new_size_temp,
            new_cap_temp,
            element_size
        ));
        let reallocated_ptr = self.next_temp();
        self.emit(&format!(
            "  %{} = call i8* @realloc(i8* %{}, i64 %{})",
            reallocated_ptr,
            current_data_ptr,
            new_size_temp
        ));
        self.emit(&format!("  br label %L{}", merge_realloc_label));

        // No reallocation block
        self.emit(&format!("L{}:", no_realloc_label));
        self.emit(&format!("  br label %L{}", merge_realloc_label));

        // Merge reallocation block
        self.emit(&format!("L{}:", merge_realloc_label));
        let final_data_ptr = self.next_temp();
        self.emit(&format!(
            "  %{} = phi i8* [ %{}, %L{} ], [ %{}, %L{} ]",
            final_data_ptr,
            reallocated_ptr,
            need_realloc_label,
            current_data_ptr,
            no_realloc_label
        ));
        let final_cap = self.next_temp();
        self.emit(&format!(
            "  %{} = phi i64 [ %{}, %L{} ], [ %{}, %L{} ]",
            final_cap,
            new_cap_temp,
            need_realloc_label,
            current_cap,
            no_realloc_label
        ));

        // Store the new element
        let offset_temp = self.next_temp();
        self.emit(&format!(
            "  %{} = mul i64 %{}, {}",
            offset_temp,
            current_len,
            element_size
        ));
        let element_ptr_temp = self.next_temp();
        self.emit(&format!(
            "  %{} = getelementptr inbounds i8, i8* %{}, i64 %{}",
            element_ptr_temp,
            final_data_ptr,
            offset_temp
        ));
        let cast_ptr_temp = self.next_temp();
        self.emit(&format!(
            "  %{} = bitcast i8* %{} to {}*",
            cast_ptr_temp,
            element_ptr_temp,
            element_val.llvm_type
        ));
        self.emit(&format!(
            "  store {} {}, {}* %{}",
            element_val.llvm_type,
            element_val.value_id,
            element_val.llvm_type,
            cast_ptr_temp
        ));

        // Update list length and capacity
        let new_len_temp = self.next_temp();
        self.emit(&format!(
            "  %{} = add i64 %{}, 1",
            new_len_temp,
            current_len
        ));
        self.emit(&format!(
            "  store i8* %{}, i8** %{}",
            final_data_ptr,
            data_ptr_gep
        ));
        self.emit(&format!(
            "  store i64 %{}, i64* %{}",
            new_len_temp,
            len_gep
        ));
        self.emit(&format!(
            "  store i64 %{}, i64* %{}",
            final_cap,
            cap_gep
        ));

        Ok(LLVMValue {
            type_info: InferType::Unit,
            llvm_type: "void".to_string(),
            value_id: "".to_string(),
        })
    }

    /// Compile a map literal
    fn compile_map_literal(&mut self, elements: &[(Expr, Expr)]) -> Result<LLVMValue, CodegenError> {
        let key_type = if let Some((first_key, _)) = elements.first() {
            self.ast_type_to_infer_type(&first_key.type_)
        } else {
            InferType::Unknown // Empty map
        };
        let value_type = if let Some((_, first_value)) = elements.first() {
            self.ast_type_to_infer_type(&first_value.type_)
        } else {
            InferType::Unknown // Empty map
        };

        let _llvm_key_type = self.infer_type_to_llvm_type(&key_type);
        let _llvm_value_type = self.infer_type_to_llvm_type(&value_type);

        // Allocate memory for the map struct
        let map_ptr_temp = self.next_temp();
        self.emit(&format!(
            "  %{} = alloca %coral.map",
            map_ptr_temp
        ));

        // Call a runtime function to create the map
        let created_map_ptr = self.next_temp();
        self.emit(&format!(
            "  %{} = call i8* @map_create(i64 {}, i64 {}) ; key_size, value_size",
            created_map_ptr,
            self.get_type_size(&key_type),
            self.get_type_size(&value_type)
        ));

        // Store the created map pointer in the allocated struct
        let data_gep_temp = self.next_temp();
        self.emit(&format!(
            "  %{} = getelementptr inbounds %coral.map, %coral.map* %{}, i32 0, i32 0",
            data_gep_temp,
            map_ptr_temp
        ));
        self.emit(&format!(
            "  store i8* %{}, i8** %{}",
            created_map_ptr,
            data_gep_temp
        ));

        // Initialize length and capacity (size and capacity are handled by runtime map_create)
        let len_gep_temp = self.next_temp();
        self.emit(&format!(
            "  %{} = getelementptr inbounds %coral.map, %coral.map* %{}, i32 0, i32 1",
            len_gep_temp,
            map_ptr_temp
        ));
        self.emit(&format!(
            "  store i64 {}, i64* %{}",
            elements.len(),
            len_gep_temp
        ));

        let cap_gep_temp = self.next_temp();
        self.emit(&format!(
            "  %{} = getelementptr inbounds %coral.map, %coral.map* %{}, i32 0, i32 2",
            cap_gep_temp,
            map_ptr_temp
        ));
        self.emit(&format!(
            "  store i64 {}, i64* %{}",
            elements.len(), // Initial capacity is number of elements
            cap_gep_temp
        ));

        // Insert initial elements
        for (key_expr, value_expr) in elements {
            let compiled_key = self.compile_expression(key_expr)?;
            let compiled_value = self.compile_expression(value_expr)?;

            self.emit(&format!(
                "  call void @map_insert(i8* %{}, {} {}, {} {})",
                created_map_ptr,
                compiled_key.llvm_type,
                compiled_key.value_id,
                compiled_value.llvm_type,
                compiled_value.value_id
            ));
        }

        Ok(LLVMValue {
            type_info: InferType::Map(Box::new(key_type), Box::new(value_type)),
            llvm_type: "%coral.map*".to_string(),
            value_id: format!("%{}", map_ptr_temp),
        })
    }

    /// Compile a map insert operation
    fn compile_map_insert(&mut self, map: &Expr, key: &Expr, value: &Expr) -> Result<LLVMValue, CodegenError> {
        let compiled_map = self.compile_expression(map)?;
        let compiled_key = self.compile_expression(key)?;
        let compiled_value = self.compile_expression(value)?;

        // Extract the actual pointer to the map data
        let map_ptr_temp = self.next_temp();
        self.emit(&format!(
            "  %{} = load i8*, i8** getelementptr inbounds (%coral.map, %coral.map* {}, i32 0, i32 0)",
            map_ptr_temp,
            compiled_map.value_id.trim_start_matches('%')
        ));

        self.emit(&format!(
            "  call void @map_insert(i8* %{}, {} {}, {} {})",
            map_ptr_temp,
            compiled_key.llvm_type,
            compiled_key.value_id,
            compiled_value.llvm_type,
            compiled_value.value_id
        ));

        Ok(LLVMValue {
            type_info: InferType::Unit,
            llvm_type: "void".to_string(),
            value_id: "".to_string(),
        })
    }

    /// Get the size of a type in bytes
    fn get_type_size(&self, infer_type: &InferType) -> usize {
        match infer_type {
            InferType::Int => 8, // i64
            InferType::Float => 8, // double
            InferType::Bool => 1, // i1
            InferType::String => 8, // i8* (pointer size)
            InferType::List(_) => 24, // %coral.list (pointer + 2 i64s)
            InferType::Map(_, _) => 24, // %coral.map (pointer + 2 i64s)
            InferType::Object { .. } => 8, // Pointer to struct
            _ => 0, // Unknown or void types
        }
    }

    /// Compile a literal value
    fn compile_literal(&mut self, lit: &Literal) -> Result<LLVMValue, CodegenError> {
        match lit {
            Literal::Integer(value) => Ok(LLVMValue {
                type_info: InferType::Int,
                llvm_type: "i64".to_string(),
                value_id: value.to_string(),
            }),
            
            Literal::Float(value) => Ok(LLVMValue {
                type_info: InferType::Float,
                llvm_type: "double".to_string(),
                value_id: value.to_string(),
            }),
            
            Literal::String(value) => {
                let temp_id = self.next_temp();
                let str_len = value.len();
                
                // Create global string constant
                self.emit(&format!(
                    "@.str{} = private constant [{} x i8] c\"{}\\00\"",
                    temp_id, str_len + 1, value
                ));
                
                Ok(LLVMValue {
                    type_info: InferType::String,
                    llvm_type: "i8*".to_string(),
                    value_id: format!("getelementptr ([{} x i8], [{}*] @.str{}, i64 0, i64 0)", 
                                     str_len + 1, str_len + 1, temp_id),
                })
            }
            
            Literal::Bool(value) => Ok(LLVMValue {
                type_info: InferType::Bool,
                llvm_type: "i1".to_string(),
                value_id: if *value { "true" } else { "false" }.to_string(),
            }),
            
            Literal::Unit => Ok(LLVMValue {
                type_info: InferType::Unit,
                llvm_type: "void".to_string(),
                value_id: "".to_string(),
            }),
            Literal::No | Literal::None | Literal::Empty => Ok(LLVMValue {
                type_info: InferType::Unknown,
                llvm_type: "i8*".to_string(),
                value_id: "null".to_string(),
            }),
            Literal::Yes => Ok(LLVMValue {
                type_info: InferType::Bool,
                llvm_type: "i1".to_string(),
                value_id: "true".to_string(),
            }),
            Literal::Now => Ok(LLVMValue {
                type_info: InferType::Int,
                llvm_type: "i64".to_string(),
                value_id: "0".to_string(), // Placeholder for timestamp
            }),
            Literal::Err => Ok(LLVMValue {
                type_info: InferType::Unknown,
                llvm_type: "i8*".to_string(),
                value_id: "null".to_string(),
            }),
            
            
        }
    }

    /// Compile a binary operation
    fn compile_binary_operation(&mut self, op: &BinaryOp, left: &Expr, right: &Expr) -> Result<LLVMValue, CodegenError> {
        let left_val = self.compile_expression(left)?;
        let right_val = self.compile_expression(right)?;
        
        let result_temp = self.next_temp();
        
        let (instruction, result_type) = match op {
            BinaryOp::Add => {
                match (&left_val.type_info, &right_val.type_info) {
                    (InferType::Int, InferType::Int) => ("add", InferType::Int),
                    (InferType::Float, InferType::Float) => ("fadd", InferType::Float),
                    _ => return Err(CodegenError::InvalidOperation("Type mismatch in addition".to_string())),
                }
            }
            BinaryOp::Sub => {
                match (&left_val.type_info, &right_val.type_info) {
                    (InferType::Int, InferType::Int) => ("sub", InferType::Int),
                    (InferType::Float, InferType::Float) => ("fsub", InferType::Float),
                    _ => return Err(CodegenError::InvalidOperation("Type mismatch in subtraction".to_string())),
                }
            }
            BinaryOp::Mul => {
                match (&left_val.type_info, &right_val.type_info) {
                    (InferType::Int, InferType::Int) => ("mul", InferType::Int),
                    (InferType::Float, InferType::Float) => ("fmul", InferType::Float),
                    _ => return Err(CodegenError::InvalidOperation("Type mismatch in multiplication".to_string())),
                }
            }
            // Comparison Operators
            BinaryOp::Eq => ("icmp eq", InferType::Bool),
            BinaryOp::Ne => ("icmp ne", InferType::Bool),
            BinaryOp::Lt => {
                match (&left_val.type_info, &right_val.type_info) {
                    (InferType::Int, InferType::Int) => ("icmp slt", InferType::Bool),
                    (InferType::Float, InferType::Float) => ("fcmp olt", InferType::Bool),
                    _ => return Err(CodegenError::InvalidOperation("Type mismatch in less than comparison".to_string())),
                }
            }
            BinaryOp::Le => {
                match (&left_val.type_info, &right_val.type_info) {
                    (InferType::Int, InferType::Int) => ("icmp sle", InferType::Bool),
                    (InferType::Float, InferType::Float) => ("fcmp ole", InferType::Bool),
                    _ => return Err(CodegenError::InvalidOperation("Type mismatch in less than or equal comparison".to_string())),
                }
            }
            BinaryOp::Gt => {
                match (&left_val.type_info, &right_val.type_info) {
                    (InferType::Int, InferType::Int) => ("icmp sgt", InferType::Bool),
                    (InferType::Float, InferType::Float) => ("fcmp ogt", InferType::Bool),
                    _ => return Err(CodegenError::InvalidOperation("Type mismatch in greater than comparison".to_string())),
                }
            }
            BinaryOp::Ge => {
                match (&left_val.type_info, &right_val.type_info) {
                    (InferType::Int, InferType::Int) => ("icmp sge", InferType::Bool),
                    (InferType::Float, InferType::Float) => ("fcmp oge", InferType::Bool),
                    _ => return Err(CodegenError::InvalidOperation("Type mismatch in greater than or equal comparison".to_string())),
                }
            }
            // Logical Operators
            BinaryOp::And => ("and", InferType::Bool),
            BinaryOp::Or => ("or", InferType::Bool),
            BinaryOp::Xor => ("xor", InferType::Bool),
            // Bitwise Operators
            BinaryOp::BitAnd => ("and", InferType::Int),
            BinaryOp::BitOr => ("or", InferType::Int),
            BinaryOp::BitXor => ("xor", InferType::Int),
            BinaryOp::Shl => ("shl", InferType::Int),
            BinaryOp::Shr => ("lshr", InferType::Int), // Logical shift right for unsigned, arithmetic for signed
            _ => return Err(CodegenError::UnsupportedFeature(format!("Binary operation not implemented: {:?}", op))),
        };
        
        self.emit(&format!(
            "  %{} = {} {} {}, {}",
            result_temp,
            instruction,
            left_val.llvm_type,
            left_val.value_id,
            right_val.value_id
        ));
        
        Ok(LLVMValue {
            type_info: result_type,
            llvm_type: left_val.llvm_type, // Assume same type for now
            value_id: format!("%{}", result_temp),
        })
    }

    /// Compile a unary operation
    fn compile_unary_operation(&mut self, op: &UnaryOp, operand: &Expr) -> Result<LLVMValue, CodegenError> {
        let operand_val = self.compile_expression(operand)?;
        let result_temp = self.next_temp();

        let (instruction, result_type) = match op {
            UnaryOp::Neg => {
                match operand_val.type_info {
                    InferType::Int => ("sub", InferType::Int),
                    InferType::Float => ("fsub", InferType::Float),
                    _ => return Err(CodegenError::InvalidOperation("Numeric negation requires numeric type".to_string())),
                }
            }
            UnaryOp::Not => {
                if operand_val.type_info == InferType::Bool {
                    ("xor", InferType::Bool)
                } else {
                    return Err(CodegenError::InvalidOperation("Logical NOT requires boolean type".to_string()))
                }
            }
            UnaryOp::BitNot => {
                if operand_val.type_info == InferType::Int {
                    ("xor", InferType::Int)
                } else {
                    return Err(CodegenError::InvalidOperation("Bitwise NOT requires integer type".to_string()))
                }
            }
        };

        let operand_llvm_type = self.infer_type_to_llvm_type(&operand_val.type_info);
        let operand_value_id = operand_val.value_id;

        // For negation, we need a zero constant to subtract from
        let const_zero = match operand_val.type_info {
            InferType::Int => "0".to_string(),
            InferType::Float => "0.0".to_string(),
            InferType::Bool => "true".to_string(), // For boolean XOR with true
            _ => return Err(CodegenError::InvalidOperation("Unsupported type for unary operation".to_string())),
        };

        self.emit(&format!(
            "  %{} = {} {} {}, {}",
            result_temp,
            instruction,
            operand_llvm_type,
            const_zero,
            operand_value_id
        ));

        Ok(LLVMValue {
            type_info: result_type,
            llvm_type: operand_llvm_type,
            value_id: format!("%{}", result_temp),
        })
    }

    /// Compile a function call
    fn compile_function_call(&mut self, callee: &Expr, args: &[Expr]) -> Result<LLVMValue, CodegenError> {
        if let ExprKind::Identifier(func_name) = &callee.kind {
            let function = self.symbols.lookup_function(func_name)
                .ok_or_else(|| CodegenError::UndefinedFunction(func_name.clone()))?
                .clone();
            
            let mut arg_values = Vec::new();
            for arg in args {
                arg_values.push(self.compile_expression(arg)?);
            }
            
            let result_temp = self.next_temp();
            
            let arg_list = arg_values.iter()
                .map(|v| format!("{} {}", v.llvm_type, v.value_id))
                .collect::<Vec<_>>()
                .join(", ");
            
            let return_llvm_type = self.infer_type_to_llvm_type(&function.return_type);
            
            self.emit(&format!(
                "  %{} = call {} @{}({})",
                result_temp,
                return_llvm_type,
                func_name,
                arg_list
            ));
            
            Ok(LLVMValue {
                type_info: function.return_type.clone(),
                llvm_type: self.infer_type_to_llvm_type(&function.return_type),
                value_id: format!("%{}", result_temp),
            })
        } else {
            Err(CodegenError::UnsupportedFeature("Indirect function calls not yet supported".to_string()))
        }
    }

    /// Compile a function definition
    fn compile_function_definition(
        &mut self,
        name: &str,
        params: &[Parameter],
        return_type: Option<&Type>,
        body: &[Stmt],
    ) -> Result<(), CodegenError> {
        // Convert parameters to LLVM types
        let mut param_list = Vec::new();
        let mut param_info = Vec::new();
        
        for param in params {
            let param_type = self.ast_type_to_infer_type(&param.type_);
            let llvm_type = self.infer_type_to_llvm_type(&param_type);
            param_list.push(format!("{} %{}", llvm_type, param.name));
            param_info.push((param.name.clone(), param_type));
        }
        
        let return_infer_type = return_type
            .map(|t| self.ast_type_to_infer_type(t))
            .unwrap_or(InferType::Unit);
        let return_llvm_type = self.infer_type_to_llvm_type(&return_infer_type);
        
        // Emit function declaration
        self.emit(&format!(
            "define {} @{}({}) {{",
            return_llvm_type,
            name,
            param_list.join(", ")
        ));
        
        // Create new scope for function
        let parent_symbols = std::mem::take(&mut self.symbols);
        self.symbols = SymbolTable::with_parent(parent_symbols.clone());
        
        // Bind parameters in new scope
        for (param_name, param_type) in &param_info {
            self.symbols.define_variable(param_name.clone(), LLVMValue {
                type_info: param_type.clone(),
                llvm_type: self.infer_type_to_llvm_type(param_type),
                value_id: format!("%{}", param_name),
            });
        }
        
        // Compile function body
        for stmt in body {
            self.compile_statement(stmt)?;
        }
        
        // Add default return if needed
        if return_infer_type == InferType::Unit {
            self.emit("  ret void");
        }
        
        self.emit("}");
        
        // Register function in symbol table
        let function = LLVMFunction {
            name: name.to_string(),
            parameters: param_info.clone(),
            return_type: return_infer_type,
            body: String::new(), // We emit directly, don't store body
        };
        
        // Restore parent scope and register function
        self.symbols = parent_symbols;
        self.symbols.define_function(name.to_string(), function);
        
        Ok(())
    }

    /// Compile an assignment statement
    fn compile_assignment(&mut self, target: &Expr, value: &Expr) -> Result<(), CodegenError> {
        let value_result = self.compile_expression(value)?;
        
        if let ExprKind::Identifier(var_name) = &target.kind {
            // For now, we'll use allocas for local variables
            let alloca_temp = self.next_temp();
            
            self.emit(&format!(
                "  %{} = alloca {}",
                alloca_temp,
                value_result.llvm_type
            ));
            
            self.emit(&format!(
                "  store {} {}, {}* %{}",
                value_result.llvm_type,
                value_result.value_id,
                value_result.llvm_type,
                alloca_temp
            ));
            
            // Register variable in symbol table
            self.symbols.define_variable(var_name.clone(), LLVMValue {
                type_info: value_result.type_info,
                llvm_type: format!("{}*", value_result.llvm_type),
                value_id: format!("%{}", alloca_temp),
            });
            
            Ok(())
        } else {
            Err(CodegenError::UnsupportedFeature("Complex assignment targets not yet supported".to_string()))
        }
    }

    /// Convert AST type to InferType (simplified version)
    fn ast_type_to_infer_type(&self, ast_type: &Type) -> InferType {
        match ast_type {
            Type::I32 | Type::I64 => InferType::Int,
            Type::F32 | Type::F64 => InferType::Float,
            Type::String => InferType::String,
            Type::Bool => InferType::Bool,
            Type::Unit => InferType::Unit,
            _ => InferType::Unknown,
        }
    }

    /// Convert InferType to its LLVM string representation
    fn infer_type_to_llvm_type(&self, infer_type: &InferType) -> String {
        match infer_type {
            InferType::Int => "i64".to_string(),
            InferType::Float => "double".to_string(),
            InferType::Bool => "i1".to_string(),
            InferType::String => "i8*".to_string(),
            InferType::Unit => "void".to_string(),
            InferType::List(_) => "%coral.list*".to_string(),
            InferType::Map(_, _) => "%coral.map*".to_string(),
            InferType::Object { name, .. } => format!("%struct.{}*", name),
            InferType::Function { params, return_type, .. } => {
                let param_types: Vec<String> = params.iter()
                    .map(|p| self.infer_type_to_llvm_type(p))
                    .collect();
                format!("{}({})", self.infer_type_to_llvm_type(return_type), param_types.join(", "))
            },
            _ => "i8*".to_string(), // Default to a generic pointer for unknown/complex types
        }
    }

    /// Add built-in function declarations
    fn add_builtin_functions(&mut self) {
        // log function
        self.symbols.define_function("log".to_string(), LLVMFunction {
            name: "log".to_string(),
            parameters: vec![("message".to_string(), InferType::String)],
            return_type: InferType::Unit,
            body: String::new(),
        });
    }

    /// Emit module header
    fn emit_module_header(&mut self) {
        self.emit(&format!("target triple = \"x86_64-unknown-linux-gnu\""));
        self.emit("%coral.list = type { i8*, i64, i64 }");
        self.emit("%coral.map = type { i8*, i64, i64 }"); // data, size, capacity
        self.emit("");
    }

    /// Emit built-in function declarations
    fn emit_builtin_declarations(&mut self) {
        self.emit("declare i32 @printf(i8*, ...)");
        self.emit("declare i32 @puts(i8*)");
        self.emit("declare i8* @malloc(i64)");
        self.emit("declare i8* @realloc(i8*, i64)");
        self.emit("declare i8* @map_create(i64, i64)");
        self.emit("declare void @map_insert(i8*, i8*, i8*)");
        self.emit("");
    }

    /// Emit a main function wrapper
    fn emit_main_function(&mut self) -> Result<(), CodegenError> {
        self.emit("define i32 @main() {");
        self.emit("  ret i32 0");
        self.emit("}");
        Ok(())
    }

    /// Emit a line of LLVM IR
    fn emit(&mut self, line: &str) {
        self.ir_output.push(line.to_string());
    }

    /// Generate next temporary variable ID
    fn next_temp(&mut self) -> usize {
        let id = self.next_temp_id;
        self.next_temp_id += 1;
        id
    }

    /// Generate next label ID
    fn next_label(&mut self) -> usize {
        let id = self.next_label_id;
        self.next_label_id += 1;
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_function_compilation() {
        let mut codegen = LLVMCodegen::new("test".to_string());
        
        // Create a simple function: fn add(a: i64, b: i64) -> i64 { return a + b; }
        let params = vec![
            Parameter { name: "a".to_string(), type_: Type::I64, default_value: None, span: SourceSpan::default() },
            Parameter { name: "b".to_string(), type_: Type::I64, default_value: None, span: SourceSpan::default() },
        ];
        
        let body = vec![
            Stmt::new(SourceSpan::default(), StmtKind::Expression(
                Expr::new(SourceSpan::default(), ExprKind::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(Expr::new(SourceSpan::default(), ExprKind::Identifier("a".to_string()))),
                    right: Box::new(Expr::new(SourceSpan::default(), ExprKind::Identifier("b".to_string()))),
                })
            ))
        ];
        
        let result = codegen.compile_function_definition("add", &params, Some(&Type::I64), &body);
        assert!(result.is_ok());
        
        let ir = codegen.ir_output.join("\n");
        assert!(ir.contains("define i64 @add(i64 %a, i64 %b)"));
    }

    #[test]
    fn test_complete_program_compilation() {
        let mut codegen = LLVMCodegen::new("test_program".to_string());
        
        // Create a simple program with function and assignment
        let add_func = Stmt::new(SourceSpan::default(), StmtKind::Function {
            name: "add".to_string(),
            params: vec![
                Parameter { name: "x".to_string(), type_: Type::I64, default_value: None, span: SourceSpan::default() },
                Parameter { name: "y".to_string(), type_: Type::I64, default_value: None, span: SourceSpan::default() },
            ],
            return_type: Some(Type::I64),
            body: vec![
                Stmt::new(SourceSpan::default(), StmtKind::Return(Some(
                    Expr::new(SourceSpan::default(), ExprKind::Binary {
                        op: BinaryOp::Add,
                        left: Box::new(Expr::new(SourceSpan::default(), ExprKind::Identifier("x".to_string()))),
                        right: Box::new(Expr::new(SourceSpan::default(), ExprKind::Identifier("y".to_string()))),
                    })
                )))
            ],
        });

        let assignment = Stmt::new(SourceSpan::default(), StmtKind::Assignment {
            target: Expr::new(SourceSpan::default(), ExprKind::Identifier("result".to_string())),
            value: Expr::new(SourceSpan::default(), ExprKind::Call {
                callee: Box::new(Expr::new(SourceSpan::default(), ExprKind::Identifier("add".to_string()))),
                args: vec![
                    Expr::new(SourceSpan::default(), ExprKind::Literal(Literal::Integer(10))),
                    Expr::new(SourceSpan::default(), ExprKind::Literal(Literal::Integer(20))),
                ],
            }),
        });

        let program = Program {
            statements: vec![add_func, assignment],
            span: SourceSpan::default(),
        };

        let result = codegen.compile_program(&program);
        match result {
            Ok(ir) => {
                println!("\n=== Generated LLVM IR ===\n{}\n========================", ir);
                
                // Verify key components are present
                assert!(ir.contains("target triple"));
                assert!(ir.contains("define i64 @add(i64 %x, i64 %y)"));
                assert!(ir.contains("define i32 @main()"));
                assert!(ir.contains("add i64"));
                assert!(ir.contains("alloca"));
                assert!(ir.contains("store"));
            }
            Err(e) => {
                panic!("Compilation failed with error: {:?}", e);
            }
        }
    }

    #[test]
    fn test_literal_compilation() {
        let mut codegen = LLVMCodegen::new("test".to_string());
        
        // Test integer literal
        let int_lit = Literal::Integer(42);
        let result = codegen.compile_literal(&int_lit);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value.llvm_type, "i64");
        assert_eq!(value.value_id, "42");
        
        // Test string literal  
        let str_lit = Literal::String("hello".to_string());
        let result = codegen.compile_literal(&str_lit);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value.llvm_type, "i8*");
        assert!(value.value_id.contains("getelementptr"));
    }

    #[test]
    fn test_binary_operation_compilation() {
        let mut codegen = LLVMCodegen::new("test".to_string());
        
        let left = Expr::new(SourceSpan::default(), ExprKind::Literal(Literal::Integer(10)));
        let right = Expr::new(SourceSpan::default(), ExprKind::Literal(Literal::Integer(20)));
        
        let result = codegen.compile_binary_operation(&BinaryOp::Add, &left, &right);
        assert!(result.is_ok());
        
        let value = result.unwrap();
        assert_eq!(value.llvm_type, "i64");
        assert!(value.value_id.starts_with("%"));
        
        let ir = codegen.ir_output.join("\n");
        assert!(ir.contains("add i64"));
    }

    #[test]
    fn test_object_codegen() {
        let mut codegen = LLVMCodegen::new("objdemo".to_string());
        // Define an object type
        let fields = vec![
            Field { name: "x".to_string(), type_: Type::I64, default_value: None, span: SourceSpan::default() },
            Field { name: "y".to_string(), type_: Type::I64, default_value: None, span: SourceSpan::default() },
        ];
        let methods = vec![];
        let obj_stmt = Stmt::new(SourceSpan::default(), StmtKind::Object {
            name: "Point".to_string(),
            fields: fields.clone(),
            methods,
        });
        codegen.compile_statement(&obj_stmt).unwrap();
        // Create an object (constructor)
        let create_expr = Expr::new(SourceSpan::default(), ExprKind::Call {
            callee: Box::new(Expr::new(SourceSpan::default(), ExprKind::Identifier("Point".to_string()))),
            args: vec![
                Expr::new(SourceSpan::default(), ExprKind::Literal(Literal::Integer(10))),
                Expr::new(SourceSpan::default(), ExprKind::Literal(Literal::Integer(20))),
            ],
        });
        let obj_val = codegen.compile_expression(&create_expr).unwrap();
        assert!(obj_val.llvm_type.contains("%struct.Point*"));
        // Access property
        let prop_expr = Expr::new(SourceSpan::default(), ExprKind::FieldAccess {
            object: Box::new(create_expr.clone()),
            field: "x".to_string(),
        });
        let prop_val = codegen.compile_expression(&prop_expr).unwrap();
        assert_eq!(prop_val.llvm_type, "i64");
        // Update property (simulate assignment to field)
        let temp_id = codegen.next_temp();
        codegen.emit(&format!(
            "  %fieldptr{} = getelementptr inbounds %struct.Point, %struct.Point* {}, i32 0, i32 0",
            temp_id, obj_val.value_id
        ));
        codegen.emit(&format!(
            "  store i64 {}, i64* %fieldptr{}",
            99, temp_id
        ));
        // Call method (simulate Point_move)
        let method_expr = Expr::new(SourceSpan::default(), ExprKind::Call {
            callee: Box::new(Expr::new(SourceSpan::default(), ExprKind::FieldAccess {
                object: Box::new(create_expr.clone()),
                field: "move".to_string(),
            })),
            args: vec![Expr::new(SourceSpan::default(), ExprKind::Literal(Literal::Integer(5)))],
        });
        let _ = codegen.compile_expression(&method_expr); // Just check IR is emitted
        let ir = codegen.ir_output.join("\n");
        assert!(ir.contains("%struct.Point = type { i64, i64 }"));
        assert!(ir.contains("alloca %struct.Point"));
        assert!(ir.contains("getelementptr inbounds %struct.Point"));
        assert!(ir.contains("store i64"));
        assert!(ir.contains("call i64 @Point_move"));
    }

    #[test]
    fn test_list_append_compilation() {
        let mut codegen = LLVMCodegen::new("test_list_append".to_string());

        // Create an initial list literal
        let initial_list = Expr::new(SourceSpan::default(), ExprKind::ListLiteral(vec![
            Expr::new(SourceSpan::default(), ExprKind::Literal(Literal::Integer(1))),
            Expr::new(SourceSpan::default(), ExprKind::Literal(Literal::Integer(2))),
        ]));

        // Element to append
        let element_to_append = Expr::new(SourceSpan::default(), ExprKind::Literal(Literal::Integer(3)));

        // Create the ListAppend expression
        let append_expr = Expr::new(SourceSpan::default(), ExprKind::ListAppend {
            list: Box::new(initial_list),
            element: Box::new(element_to_append),
        });

        let result = codegen.compile_expression(&append_expr);
        assert!(result.is_ok());

        let ir = codegen.ir_output.join("\n");
        println!("\n=== Generated LLVM IR for List Append ===\n{}\n========================", ir);

        // Assertions to check for key LLVM IR instructions for list append
        assert!(ir.contains("getelementptr inbounds %coral.list"));
        assert!(ir.contains("load i8*"));
        assert!(ir.contains("load i64"));
        assert!(ir.contains("icmp eq i64")); // Check for reallocation condition
        assert!(ir.contains("br i1"));      // Branch for reallocation
        assert!(ir.contains("mul i64"));      // Calculate new capacity/size
        assert!(ir.contains("call i8* @realloc")); // Reallocation call
        assert!(ir.contains("phi i8*"));    // Phi node for data pointer
        assert!(ir.contains("phi i64"));    // Phi node for capacity
        assert!(ir.contains("store i64"));    // Store new length and capacity
        assert!(ir.contains("store i8*"));    // Store new data pointer
    }

    #[test]
    fn test_map_literal_compilation() {
        let mut codegen = LLVMCodegen::new("test_map_literal".to_string());

        // Create a map literal: {"a": 1, "b": 2}
        let map_literal = Expr::new(SourceSpan::default(), ExprKind::MapLiteral(vec![
            (Expr::new(SourceSpan::default(), ExprKind::Literal(Literal::String("a".to_string()))),
             Expr::new(SourceSpan::default(), ExprKind::Literal(Literal::Integer(1)))),
            (Expr::new(SourceSpan::default(), ExprKind::Literal(Literal::String("b".to_string()))),
             Expr::new(SourceSpan::default(), ExprKind::Literal(Literal::Integer(2)))),
        ]));

        let result = codegen.compile_expression(&map_literal);
        assert!(result.is_ok());

        let ir = codegen.ir_output.join("\n");
        println!("\n=== Generated LLVM IR for Map Literal ===\n{}\n========================", ir);

        // Assertions to check for key LLVM IR instructions for map literal
        assert!(ir.contains("alloca %coral.map"));
        assert!(ir.contains("call i8* @map_create"));
        assert!(ir.contains("call void @map_insert"));
        assert!(ir.contains("store i64 2, i64* ")); // Check initial length and capacity
    }

    #[test]
    fn test_map_insert_compilation() {
        let mut codegen = LLVMCodegen::new("test_map_insert".to_string());

        // Create a dummy map (for testing purposes, assume it's already created)
        let dummy_map = LLVMValue {
            type_info: InferType::Map(Box::new(InferType::String), Box::new(InferType::Int)),
            llvm_type: "%coral.map*".to_string(),
            value_id: "%map_ptr".to_string(),
        };
        codegen.symbols.define_variable("my_map".to_string(), dummy_map.clone());

        // Create a map insert expression: my_map["c"] = 3
        let map_insert = Expr::new(SourceSpan::default(), ExprKind::MapInsert {
            map: Box::new(Expr::new(SourceSpan::default(), ExprKind::Identifier("my_map".to_string()))),
            key: Box::new(Expr::new(SourceSpan::default(), ExprKind::Literal(Literal::String("c".to_string())))),
            value: Box::new(Expr::new(SourceSpan::default(), ExprKind::Literal(Literal::Integer(3)))),
        });

        let result = codegen.compile_expression(&map_insert);
        assert!(result.is_ok());

        let ir = codegen.ir_output.join("\n");
        println!("\n=== Generated LLVM IR for Map Insert ===\n{}\n========================", ir);

        // Assertions to check for key LLVM IR instructions for map insert
        assert!(ir.contains("call void @map_insert"));
        assert!(ir.contains("load i8*"));
    }

    #[test]
    fn test_error_handler_codegen_log_return() {
        let mut codegen = LLVMCodegen::new("errdemo".to_string());
        // Simulate guarded statement: x is 1
        let guarded_stmt = Stmt::new(SourceSpan::default(), StmtKind::Assignment {
            target: Expr::new(SourceSpan::default(), ExprKind::Identifier("x".to_string())),
            value: Expr::new(SourceSpan::default(), ExprKind::Literal(Literal::Integer(1))),
        });
        // Error handler: log 'fail', return 42
        let err_handler = ErrorHandler {
            actions: vec![
                ErrorAction::Log(Some(Expr::new(SourceSpan::default(), ExprKind::Literal(Literal::String("fail".to_string()))))),
                ErrorAction::Return(Some(Expr::new(SourceSpan::default(), ExprKind::Literal(Literal::Integer(42))))),
            ],
            span: SourceSpan::default(),
        };
        let stmt = Stmt::new(SourceSpan::default(), StmtKind::ErrorHandler {
            handler: err_handler,
            inner: Box::new(guarded_stmt),
        });
        let result = codegen.compile_statement(&stmt);
        assert!(result.is_ok());
        let ir = codegen.ir_output.join("\n");
        assert!(ir.contains("call @log"));
        assert!(ir.contains("ret"));
    }

    #[test]
    fn test_module_import_and_builtin_function() {
        let mut codegen = LLVMCodegen::new("moddemo".to_string());
        // Simulate import statement
        let import_stmt = Stmt::new(SourceSpan::default(), StmtKind::Import {
            module: "coral.net.web".to_string(),
            items: None,
        });
        let result = codegen.compile_statement(&import_stmt);
        assert!(result.is_ok());
        // Simulate builtin function call: get('https://somesite.com')
        let get_call = Expr::new(SourceSpan::default(), ExprKind::Call {
            callee: Box::new(Expr::new(SourceSpan::default(), ExprKind::Identifier("get".to_string()))),
            args: vec![Expr::new(SourceSpan::default(), ExprKind::Literal(Literal::String("https://somesite.com".to_string())))]
        });
        let result = codegen.compile_expression(&get_call);
        assert!(result.is_ok());
        let ir = codegen.ir_output.join("\n");
        assert!(ir.contains("@get"));
    }

    #[test]
    fn test_local_module_link_without_definition() {
        let mut codegen = LLVMCodegen::new("localmoddemo".to_string());
        // Simulate use xyz
        let use_stmt = Stmt::new(SourceSpan::default(), StmtKind::Import {
            module: "xyz".to_string(),
            items: None,
        });
        let result = codegen.compile_statement(&use_stmt);
        assert!(result.is_ok());
        // Simulate x is abc(123) where abc is not defined locally
        let assign_stmt = Stmt::new(SourceSpan::default(), StmtKind::Assignment {
            target: Expr::new(SourceSpan::default(), ExprKind::Identifier("x".to_string())),
            value: Expr::new(SourceSpan::default(), ExprKind::Call {
                callee: Box::new(Expr::new(SourceSpan::default(), ExprKind::Identifier("abc".to_string()))),
                args: vec![Expr::new(SourceSpan::default(), ExprKind::Literal(Literal::Integer(123)))]
            })
        });
        let result = codegen.compile_statement(&assign_stmt);
        assert!(result.is_ok());
        let ir = codegen.ir_output.join("\n");
        assert!(ir.contains("@abc"));
    }

    #[test]
    fn test_builtin_class_and_function_usage() {
        let mut codegen = LLVMCodegen::new("builtindemo".to_string());
        // Simulate usage of builtin class: glob.glob('*.cor')
        let glob_call = Expr::new(SourceSpan::default(), ExprKind::Call {
            callee: Box::new(Expr::new(SourceSpan::default(), ExprKind::FieldAccess {
                object: Box::new(Expr::new(SourceSpan::default(), ExprKind::Identifier("glob".to_string()))),
                field: "glob".to_string(),
            })),
            args: vec![Expr::new(SourceSpan::default(), ExprKind::Literal(Literal::String("*.cor".to_string())))]
        });
        let result = codegen.compile_expression(&glob_call);
        assert!(result.is_ok());
        let ir = codegen.ir_output.join("\n");
        assert!(ir.contains("@glob_glob"));
    }


    #[test]
    fn test_simple_function_compilation() {
        let mut codegen = LLVMCodegen::new("test".to_string());
        
        // Create a simple function: fn add(a: i64, b: i64) -> i64 { return a + b; }
        let params = vec![
            Parameter { name: "a".to_string(), type_: Type::I64, default_value: None, span: SourceSpan::default() },
            Parameter { name: "b".to_string(), type_: Type::I64, default_value: None, span: SourceSpan::default() },
        ];
        
        let body = vec![
            Stmt::new(SourceSpan::default(), StmtKind::Expression(
                Expr::new(SourceSpan::default(), ExprKind::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(Expr::new(SourceSpan::default(), ExprKind::Identifier("a".to_string()))),
                    right: Box::new(Expr::new(SourceSpan::default(), ExprKind::Identifier("b".to_string()))),
                })
            ))
        ];
        
        let result = codegen.compile_function_definition("add", &params, Some(&Type::I64), &body);
        assert!(result.is_ok());
        
        let ir = codegen.ir_output.join("\n");
        assert!(ir.contains("define i64 @add(i64 %a, i64 %b)"));
    }

    #[test]
    fn test_complete_program_compilation() {
        let mut codegen = LLVMCodegen::new("test_program".to_string());
        
        // Create a simple program with function and assignment
        let add_func = Stmt::new(SourceSpan::default(), StmtKind::Function {
            name: "add".to_string(),
            params: vec![
                Parameter { name: "x".to_string(), type_: Type::I64, default_value: None, span: SourceSpan::default() },
                Parameter { name: "y".to_string(), type_: Type::I64, default_value: None, span: SourceSpan::default() },
            ],
            return_type: Some(Type::I64),
            body: vec![
                Stmt::new(SourceSpan::default(), StmtKind::Return(Some(
                    Expr::new(SourceSpan::default(), ExprKind::Binary {
                        op: BinaryOp::Add,
                        left: Box::new(Expr::new(SourceSpan::default(), ExprKind::Identifier("x".to_string()))),
                        right: Box::new(Expr::new(SourceSpan::default(), ExprKind::Identifier("y".to_string()))),
                    })
                )))
            ],
        });

        let assignment = Stmt::new(SourceSpan::default(), StmtKind::Assignment {
            target: Expr::new(SourceSpan::default(), ExprKind::Identifier("result".to_string())),
            value: Expr::new(SourceSpan::default(), ExprKind::Call {
                callee: Box::new(Expr::new(SourceSpan::default(), ExprKind::Identifier("add".to_string()))),
                args: vec![
                    Expr::new(SourceSpan::default(), ExprKind::Literal(Literal::Integer(10))),
                    Expr::new(SourceSpan::default(), ExprKind::Literal(Literal::Integer(20))),
                ],
            }),
        });

        let program = Program {
            statements: vec![add_func, assignment],
            span: SourceSpan::default(),
        };

        let result = codegen.compile_program(&program);
        match result {
            Ok(ir) => {
                println!("\n=== Generated LLVM IR ===\n{}\n========================", ir);
                
                // Verify key components are present
                assert!(ir.contains("target triple"));
                assert!(ir.contains("define i64 @add(i64 %x, i64 %y)"));
                assert!(ir.contains("define i32 @main()"));
                assert!(ir.contains("add i64"));
                assert!(ir.contains("alloca"));
                assert!(ir.contains("store"));
            }
            Err(e) => {
                panic!("Compilation failed with error: {:?}", e);
            }
        }
    }

    #[test]
    fn test_literal_compilation() {
        let mut codegen = LLVMCodegen::new("test".to_string());
        
        // Test integer literal
        let int_lit = Literal::Integer(42);
        let result = codegen.compile_literal(&int_lit);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value.llvm_type, "i64");
        assert_eq!(value.value_id, "42");
        
        // Test string literal  
        let str_lit = Literal::String("hello".to_string());
        let result = codegen.compile_literal(&str_lit);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value.llvm_type, "i8*");
        assert!(value.value_id.contains("getelementptr"));
    }

    #[test]
    fn test_binary_operation_compilation() {
        let mut codegen = LLVMCodegen::new("test".to_string());
        
        let left = Expr::new(SourceSpan::default(), ExprKind::Literal(Literal::Integer(10)));
        let right = Expr::new(SourceSpan::default(), ExprKind::Literal(Literal::Integer(20)));
        
        let result = codegen.compile_binary_operation(&BinaryOp::Add, &left, &right);
        assert!(result.is_ok());
        
        let value = result.unwrap();
        assert_eq!(value.llvm_type, "i64");
        assert!(value.value_id.starts_with("%"));
        
        let ir = codegen.ir_output.join("\n");
        assert!(ir.contains("add i64"));
    }

    #[test]
    fn test_object_codegen() {
        let mut codegen = LLVMCodegen::new("objdemo".to_string());
        // Define an object type
        let fields = vec![
            Field { name: "x".to_string(), type_: Type::I64, default_value: None, span: SourceSpan::default() },
            Field { name: "y".to_string(), type_: Type::I64, default_value: None, span: SourceSpan::default() },
        ];
        let methods = vec![];
        let obj_stmt = Stmt::new(SourceSpan::default(), StmtKind::Object {
            name: "Point".to_string(),
            fields: fields.clone(),
            methods,
        });
        codegen.compile_statement(&obj_stmt).unwrap();
        // Create an object (constructor)
        let create_expr = Expr::new(SourceSpan::default(), ExprKind::Call {
            callee: Box::new(Expr::new(SourceSpan::default(), ExprKind::Identifier("Point".to_string()))),
            args: vec![
                Expr::new(SourceSpan::default(), ExprKind::Literal(Literal::Integer(10))),
                Expr::new(SourceSpan::default(), ExprKind::Literal(Literal::Integer(20))),
            ],
        });
        let obj_val = codegen.compile_expression(&create_expr).unwrap();
        assert!(obj_val.llvm_type.contains("%struct.Point*"));
        // Access property
        let prop_expr = Expr::new(SourceSpan::default(), ExprKind::FieldAccess {
            object: Box::new(create_expr.clone()),
            field: "x".to_string(),
        });
        let prop_val = codegen.compile_expression(&prop_expr).unwrap();
        assert_eq!(prop_val.llvm_type, "i64");
        // Update property (simulate assignment to field)
        let temp_id = codegen.next_temp();
        codegen.emit(&format!(
            "  %fieldptr{} = getelementptr inbounds %struct.Point, %struct.Point* {}, i32 0, i32 0",
            temp_id, obj_val.value_id
        ));
        codegen.emit(&format!(
            "  store i64 {}, i64* %fieldptr{}",
            99, temp_id
        ));
        // Call method (simulate Point_move)
        let method_expr = Expr::new(SourceSpan::default(), ExprKind::Call {
            callee: Box::new(Expr::new(SourceSpan::default(), ExprKind::FieldAccess {
                object: Box::new(create_expr.clone()),
                field: "move".to_string(),
            })),
            args: vec![Expr::new(SourceSpan::default(), ExprKind::Literal(Literal::Integer(5)))],
        });
        let _ = codegen.compile_expression(&method_expr); // Just check IR is emitted
        let ir = codegen.ir_output.join("\n");
        assert!(ir.contains("%struct.Point = type { i64, i64 }"));
        assert!(ir.contains("alloca %struct.Point"));
        assert!(ir.contains("getelementptr inbounds %struct.Point"));
        assert!(ir.contains("store i64"));
        assert!(ir.contains("call i64 @Point_move"));
    }

    #[test]
    fn test_list_append_compilation() {
        let mut codegen = LLVMCodegen::new("test_list_append".to_string());

        // Create an initial list literal
        let initial_list = Expr::new(SourceSpan::default(), ExprKind::ListLiteral(vec![
            Expr::new(SourceSpan::default(), ExprKind::Literal(Literal::Integer(1))),
            Expr::new(SourceSpan::default(), ExprKind::Literal(Literal::Integer(2))),
        ]));

        // Element to append
        let element_to_append = Expr::new(SourceSpan::default(), ExprKind::Literal(Literal::Integer(3)));

        // Create the ListAppend expression
        let append_expr = Expr::new(SourceSpan::default(), ExprKind::ListAppend {
            list: Box::new(initial_list),
            element: Box::new(element_to_append),
        });

        let result = codegen.compile_expression(&append_expr);
        assert!(result.is_ok());

        let ir = codegen.ir_output.join("\n");
        println!("\n=== Generated LLVM IR for List Append ===\n{}\n========================", ir);

        // Assertions to check for key LLVM IR instructions for list append
        assert!(ir.contains("getelementptr inbounds %coral.list"));
        assert!(ir.contains("load i8*"));
        assert!(ir.contains("load i64"));
        assert!(ir.contains("icmp eq i64")); // Check for reallocation condition
        assert!(ir.contains("br i1"));      // Branch for reallocation
        assert!(ir.contains("mul i64"));      // Calculate new capacity/size
        assert!(ir.contains("call i8* @realloc")); // Reallocation call
        assert!(ir.contains("phi i8*"));    // Phi node for data pointer
        assert!(ir.contains("phi i64"));    // Phi node for capacity
        assert!(ir.contains("store i64"));    // Store new length and capacity
        assert!(ir.contains("store i8*"));    // Store new data pointer
    }

    #[test]
    fn test_map_literal_compilation() {
        let mut codegen = LLVMCodegen::new("test_map_literal".to_string());

        // Create a map literal: {"a": 1, "b": 2}
        let map_literal = Expr::new(SourceSpan::default(), ExprKind::MapLiteral(vec![
            (Expr::new(SourceSpan::default(), ExprKind::Literal(Literal::String("a".to_string()))),
             Expr::new(SourceSpan::default(), ExprKind::Literal(Literal::Integer(1)))),
            (Expr::new(SourceSpan::default(), ExprKind::Literal(Literal::String("b".to_string()))),
             Expr::new(SourceSpan::default(), ExprKind::Literal(Literal::Integer(2)))),
        ]));

        let result = codegen.compile_expression(&map_literal);
        assert!(result.is_ok());

        let ir = codegen.ir_output.join("\n");
        println!("\n=== Generated LLVM IR for Map Literal ===\n{}\n========================", ir);

        // Assertions to check for key LLVM IR instructions for map literal
        assert!(ir.contains("alloca %coral.map"));
        assert!(ir.contains("call i8* @map_create"));
        assert!(ir.contains("call void @map_insert"));
        assert!(ir.contains("store i64 2, i64* ")); // Check initial length and capacity
    }

    #[test]
    fn test_map_insert_compilation() {
        let mut codegen = LLVMCodegen::new("test_map_insert".to_string());

        // Create a dummy map (for testing purposes, assume it's already created)
        let dummy_map = LLVMValue {
            type_info: InferType::Map(Box::new(InferType::String), Box::new(InferType::Int)),
            llvm_type: "%coral.map*".to_string(),
            value_id: "%map_ptr".to_string(),
        };
        codegen.symbols.define_variable("my_map".to_string(), dummy_map.clone());

        // Create a map insert expression: my_map["c"] = 3
        let map_insert = Expr::new(SourceSpan::default(), ExprKind::MapInsert {
            map: Box::new(Expr::new(SourceSpan::default(), ExprKind::Identifier("my_map".to_string()))),
            key: Box::new(Expr::new(SourceSpan::default(), ExprKind::Literal(Literal::String("c".to_string())))),
            value: Box::new(Expr::new(SourceSpan::default(), ExprKind::Literal(Literal::Integer(3)))),
        });

        let result = codegen.compile_expression(&map_insert);
        assert!(result.is_ok());

        let ir = codegen.ir_output.join("\n");
        println!("\n=== Generated LLVM IR for Map Insert ===\n{}\n========================", ir);

        // Assertions to check for key LLVM IR instructions for map insert
        assert!(ir.contains("call void @map_insert"));
        assert!(ir.contains("load i8*"));
    }

    #[test]
    fn test_error_handler_codegen_log_return() {
        let mut codegen = LLVMCodegen::new("errdemo".to_string());
        // Simulate guarded statement: x is 1
        let guarded_stmt = Stmt::new(SourceSpan::default(), StmtKind::Assignment {
            target: Expr::new(SourceSpan::default(), ExprKind::Identifier("x".to_string())),
            value: Expr::new(SourceSpan::default(), ExprKind::Literal(Literal::Integer(1))),
        });
        // Error handler: log 'fail', return 42
        let err_handler = ErrorHandler {
            actions: vec![
                ErrorAction::Log(Some(Expr::new(SourceSpan::default(), ExprKind::Literal(Literal::String("fail".to_string()))))),
                ErrorAction::Return(Some(Expr::new(SourceSpan::default(), ExprKind::Literal(Literal::Integer(42))))),
            ],
            span: SourceSpan::default(),
        };
        let stmt = Stmt::new(SourceSpan::default(), StmtKind::ErrorHandler {
            handler: err_handler,
            inner: Box::new(guarded_stmt),
        });
        let result = codegen.compile_statement(&stmt);
        assert!(result.is_ok());
        let ir = codegen.ir_output.join("\n");
        assert!(ir.contains("call @log"));
        assert!(ir.contains("ret"));
    }

    #[test]
    fn test_module_import_and_builtin_function() {
        let mut codegen = LLVMCodegen::new("moddemo".to_string());
        // Simulate import statement
        let import_stmt = Stmt::new(SourceSpan::default(), StmtKind::Import {
            module: "coral.net.web".to_string(),
            items: None,
        });
        let result = codegen.compile_statement(&import_stmt);
        assert!(result.is_ok());
        // Simulate builtin function call: get('https://somesite.com')
        let get_call = Expr::new(SourceSpan::default(), ExprKind::Call {
            callee: Box::new(Expr::new(SourceSpan::default(), ExprKind::Identifier("get".to_string()))),
            args: vec![Expr::new(SourceSpan::default(), ExprKind::Literal(Literal::String("https://somesite.com".to_string())))]
        });
        let result = codegen.compile_expression(&get_call);
        assert!(result.is_ok());
        let ir = codegen.ir_output.join("\n");
        assert!(ir.contains("@get"));
    }

    #[test]
    fn test_local_module_link_without_definition() {
        let mut codegen = LLVMCodegen::new("localmoddemo".to_string());
        // Simulate use xyz
        let use_stmt = Stmt::new(SourceSpan::default(), StmtKind::Import {
            module: "xyz".to_string(),
            items: None,
        });
        let result = codegen.compile_statement(&use_stmt);
        assert!(result.is_ok());
        // Simulate x is abc(123) where abc is not defined locally
        let assign_stmt = Stmt::new(SourceSpan::default(), StmtKind::Assignment {
            target: Expr::new(SourceSpan::default(), ExprKind::Identifier("x".to_string())),
            value: Expr::new(SourceSpan::default(), ExprKind::Call {
                callee: Box::new(Expr::new(SourceSpan::default(), ExprKind::Identifier("abc".to_string()))),
                args: vec![Expr::new(SourceSpan::default(), ExprKind::Literal(Literal::Integer(123)))]
            })
        });
        let result = codegen.compile_statement(&assign_stmt);
        assert!(result.is_ok());
        let ir = codegen.ir_output.join("\n");
        assert!(ir.contains("@abc"));
    }

    #[test]
    fn test_builtin_class_and_function_usage() {
        let mut codegen = LLVMCodegen::new("builtindemo".to_string());
        // Simulate usage of builtin class: glob.glob('*.cor')
        let glob_call = Expr::new(SourceSpan::default(), ExprKind::Call {
            callee: Box::new(Expr::new(SourceSpan::default(), ExprKind::FieldAccess {
                object: Box::new(Expr::new(SourceSpan::default(), ExprKind::Identifier("glob".to_string()))),
                field: "glob".to_string(),
            })),
            args: vec![Expr::new(SourceSpan::default(), ExprKind::Literal(Literal::String("*.cor".to_string())))]
        });
        let result = codegen.compile_expression(&glob_call);
        assert!(result.is_ok());
        let ir = codegen.ir_output.join("\n");
        assert!(ir.contains("@glob_glob"));
    }
}
