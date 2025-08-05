use crate::codegen::{CodegenError, LLVMCodegen, LLVMValue};
use crate::ast::{Stmt, StmtKind, Expr, Type, Parameter, ExprKind, ObjectMethod};
use crate::resolver::InferType;
use crate::codegen::types::{infer_to_llvm_type, LLVMType};

impl LLVMCodegen {
    pub fn compile_statement(&mut self, stmt: &Stmt) -> Result<Option<LLVMValue>, CodegenError> {
        match &stmt.kind {
            StmtKind::Function { name, params, return_type, body } => {
                self.compile_function_definition(name, params, return_type.as_ref(), body)?;
                Ok(None)
            }
            StmtKind::Store { name, fields, methods } => {
                self.compile_store_definition(name, fields, methods)?;
                Ok(None)
            }
            StmtKind::Assignment { target, value } => {
                self.compile_assignment(target, value)?;
                Ok(None)
            }
            StmtKind::Expression(expr) => {
                let value = self.compile_expression(expr)?;
                Ok(Some(value))
            }
            StmtKind::Return(expr_opt) => {
                if let Some(expr) = expr_opt {
                    let return_value = self.compile_expression(expr)?;
                    self.emit(&format!(
                        "  ret {} {}",
                        return_value.llvm_type,
                        return_value.value_id
                    ));
                    Ok(Some(return_value))
                } else {
                    self.emit("  ret void");
                    Ok(None)
                }
            }
            StmtKind::If { condition, then_branch, else_branch } => {
                self.compile_if_statement(condition, then_branch, else_branch.as_deref())?;
                Ok(None)
            }
            StmtKind::While { condition, body } => {
                self.compile_while_statement(condition, body)?;
                Ok(None)
            }
            StmtKind::Until { condition, body } => {
                self.compile_until_statement(condition, body)?;
                Ok(None)
            }
            StmtKind::Iterate { iterable, body } => {
                self.compile_iterate_statement(iterable, body)?;
                Ok(None)
            }
            _ => Err(CodegenError::UnsupportedFeature(
                format!("Statement type not implemented: {:?}", stmt.kind)
            ))
        }
    }

    pub fn compile_store_definition(&mut self, name: &str, _fields: &[crate::ast::Field], methods: &[ObjectMethod]) -> Result<(), CodegenError> {
        let struct_name = format!("%{}", name);
        self.emit(&format!("@{} = common global {} zeroinitializer, align 8", name, struct_name));

        for method in methods {
            self.compile_function_definition(&format!("{}_{}", name, method.name), &method.params, method.return_type.as_ref(), &method.body)?;
        }

        // Declare runtime functions for store operations
        self.emit("declare void @store_save(i8*, i8*)");
        self.emit("declare i8* @store_load(i8*)");

        Ok(())
    }

    pub fn compile_until_statement(&mut self, condition: &Expr, body: &[Stmt]) -> Result<(), CodegenError> {
        let body_label = self.next_label();
        let cond_label = self.next_label();
        let merge_label = self.next_label();

        self.emit(&format!("  br label %L{}", body_label));
        self.emit(&format!("L{}:", body_label));

        for stmt in body {
            self.compile_statement(stmt)?;
        }
        self.emit(&format!("  br label %L{}", cond_label));

        self.emit(&format!("L{}:", cond_label));
        let cond_val = self.compile_expression(condition)?;
        self.emit(&format!("  br i1 {}, label %L{}, label %L{}", cond_val.value_id, merge_label, body_label));

        self.emit(&format!("L{}:", merge_label));
        Ok(())
    }

    pub fn compile_iterate_statement(&mut self, iterable: &Expr, body: &[Stmt]) -> Result<(), CodegenError> {
        let iterable_val = self.compile_expression(iterable)?;

        // Declare runtime functions for iteration
        self.emit("declare i8* @iterator_new(i8*)");
        self.emit("declare i1 @iterator_next(i8*)");
        self.emit("declare i8* @iterator_get_value(i8*)");

        let iterator_ptr = self.next_temp();
        self.emit(&format!("  %{} = call i8* @iterator_new({} {})", iterator_ptr, iterable_val.llvm_type, iterable_val.value_id));

        let loop_cond_label = self.next_label();
        let loop_body_label = self.next_label();
        let loop_end_label = self.next_label();

        self.emit(&format!("  br label %L{}", loop_cond_label));
        self.emit(&format!("L{}:", loop_cond_label));

        let has_next_ptr = self.next_temp();
        self.emit(&format!("  %{} = call i1 @iterator_next(i8* %{})", has_next_ptr, iterator_ptr));
        self.emit(&format!("  br i1 %{}, label %L{}, label %L{}", has_next_ptr, loop_body_label, loop_end_label));

        self.emit(&format!("L{}:", loop_body_label));

        let value_ptr = self.next_temp();
        self.emit(&format!("  %{} = call i8* @iterator_get_value(i8* %{})", value_ptr, iterator_ptr));
        
        // The '$' variable holds the current iteration value.
        // This is a simplified approach; a real implementation would need to know the type of the value.
        self.symbols.define_variable("$".to_string(), LLVMValue {
            type_info: InferType::Unknown, // This should be the element type of the iterable
            llvm_type: LLVMType::Pointer(Box::new(LLVMType::Int(8))),
            value_id: format!("%{}", value_ptr),
        });

        for stmt in body {
            self.compile_statement(stmt)?;
        }

        self.emit(&format!("  br label %L{}", loop_cond_label));
        self.emit(&format!("L{}:", loop_end_label));

        Ok(())
    }

    pub fn compile_while_statement(&mut self, condition: &Expr, body: &[Stmt]) -> Result<(), CodegenError> {
        let cond_label = self.next_label();
        let body_label = self.next_label();
        let merge_label = self.next_label();

        self.emit(&format!("  br label %L{}", cond_label));
        self.emit(&format!("L{}:", cond_label));

        let cond_val = self.compile_expression(condition)?;
        self.emit(&format!("  br i1 {}, label %L{}, label %L{}", cond_val.value_id, body_label, merge_label));

        self.emit(&format!("L{}:", body_label));
        for stmt in body {
            self.compile_statement(stmt)?;
        }
        self.emit(&format!("  br label %L{}", cond_label));

        self.emit(&format!("L{}:", merge_label));
        Ok(())
    }

    pub fn compile_function_definition(&mut self, name: &str, params: &[Parameter], return_type: Option<&Type>, body: &[Stmt]) -> Result<(), CodegenError> {
        let mut param_llvm_types = Vec::new();
        let mut param_names = Vec::new();

        let inferred_return_type = if let Some(ty) = return_type {
            self.ast_type_to_infer_type(ty)
        } else {
            InferType::Unit // Default to Unit if no return type specified
        };

        for param in params {
            let infer_type = self.ast_type_to_infer_type(&param.type_);
            let llvm_type = infer_to_llvm_type(&infer_type);
            param_llvm_types.push(llvm_type.clone());
            param_names.push(param.name.clone());
        }

        let return_llvm_type = infer_to_llvm_type(&inferred_return_type);

        self.emit(&format!(
            "define {} @{}({}) {{",
            return_llvm_type,
            name,
            param_llvm_types.iter()
                .zip(param_names.iter())
                .map(|(ty, name)| format!("{} %{}", ty, name))
                .collect::<Vec<String>>()
                .join(", ")
        ));
        self.emit("entry:");

        for stmt in body {
            self.compile_statement(stmt)?;
        }

        if return_llvm_type == LLVMType::Void {
            self.emit("  ret void");
        } else {
            self.emit(&format!("  ret {} undef", return_llvm_type));
        }
        self.emit("}");

        Ok(())
    }

    pub fn compile_assignment(&mut self, target: &Expr, value: &Expr) -> Result<(), CodegenError> {
        let value_result = self.compile_expression(value)?;

        if let ExprKind::Identifier(var_name) = &target.kind {
            if let Some(existing_var) = self.symbols.lookup_variable(var_name) {
                if existing_var.llvm_type.to_string().ends_with("*") {
                    self.emit(&format!(
                        "  store {} {}, {} {}",
                        value_result.llvm_type,
                        value_result.value_id,
                        existing_var.llvm_type,
                        existing_var.value_id
                    ));
                } else {
                    return Err(CodegenError::InvalidOperation(format!("Cannot reassign to non-pointer type: {}", existing_var.llvm_type)));
                }
            } else {
                let alloca_temp = self.next_temp();
                self.emit(&format!("  %{} = alloca {}", alloca_temp, value_result.llvm_type));
                self.emit(&format!("  store {} {}, {}* %{}",
                    value_result.llvm_type,
                    value_result.value_id,
                    value_result.llvm_type,
                    alloca_temp
                ));

                self.symbols.define_variable(var_name.clone(), LLVMValue {
                    type_info: value_result.type_info,
                    llvm_type: LLVMType::Pointer(Box::new(value_result.llvm_type)),
                    value_id: format!("%{}", alloca_temp),
                });
            }
            Ok(())
        } else {
            Err(CodegenError::UnsupportedFeature("Complex assignment targets not yet supported".to_string()))
        }
    }

    pub fn compile_if_statement(&mut self, condition: &Expr, then_branch: &[Stmt], else_branch: Option<&[Stmt]>) -> Result<(), CodegenError> {
        let cond_val = self.compile_expression(condition)?;

        let then_label = self.next_label();
        let else_label = self.next_label();
        let merge_label = self.next_label();

        self.emit(&format!(
            "  br i1 {}, label %L{}, label %L{}",
            cond_val.value_id,
            then_label,
            if else_branch.is_some() { else_label } else { merge_label }
        ));

        self.emit(&format!("L{}:", then_label));
        for stmt in then_branch {
            self.compile_statement(stmt)?;
        }
        self.emit(&format!("  br label %L{}", merge_label));

        if let Some(else_stmts) = else_branch {
            self.emit(&format!("L{}:", else_label));
            for stmt in else_stmts {
                self.compile_statement(stmt)?;
            }
            self.emit(&format!("  br label %L{}", merge_label));
        }

        self.emit(&format!("L{}:", merge_label));
        Ok(())
    }
}
