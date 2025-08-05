use crate::codegen::{CodegenError, LLVMCodegen, LLVMValue};
use crate::ast::{Expr, ExprKind, BinaryOp, UnaryOp, Literal};
use crate::resolver::InferType;
use crate::codegen::types::infer_to_llvm_type;

impl LLVMCodegen {
    pub fn compile_expression(&mut self, expr: &Expr) -> Result<LLVMValue, CodegenError> {
        match &expr.kind {
            ExprKind::Literal(lit) => self.compile_literal(lit),
            ExprKind::Identifier(name) => {
                if let Some(llvm_val) = self.symbols.lookup_variable(name) {
                    return Ok(llvm_val.clone());
                }
                if let Some(obj_type) = self.lookup_object_type(name) {
                    return Ok(LLVMValue {
                        type_info: obj_type.clone(),
                        llvm_type: infer_to_llvm_type(&obj_type),
                        value_id: format!("@{}", name),
                    });
                }
                Err(CodegenError::UndefinedVariable(name.clone()))
            }
            ExprKind::Binary { op, left, right } => {
                self.compile_binary_operation(op, left, right)
            }
            ExprKind::Unary { op, operand } => {
                self.compile_unary_operation(op, operand)
            }
            ExprKind::Call { callee, args } => {
                let arg_exprs: Vec<_> = args.iter().map(|arg| arg.value.clone()).collect();
                match &callee.kind {
                    ExprKind::Identifier(type_or_func) => {
                        if let Some(obj_type) = self.lookup_object_type(type_or_func) {
                            self.compile_object_instantiation(type_or_func, obj_type, &arg_exprs)
                        } else {
                            self.compile_function_call(callee, &arg_exprs)
                        }
                    }
                    ExprKind::FieldAccess { object, field } => {
                        self.compile_method_call(object, field, &arg_exprs)
                    }
                    _ => self.compile_function_call(callee, &arg_exprs),
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
            ExprKind::StringInterpolation { parts } => self.compile_string_interpolation(parts),
            ExprKind::ObjectInstantiation { name, fields } => {
                let obj_type = self.lookup_object_type(name).unwrap();
                let mut args = Vec::new();
                for (_, field_expr) in fields {
                    args.push(field_expr.clone());
                }
                self.compile_object_instantiation(name, obj_type, &args)
            }
            _ => Err(CodegenError::UnsupportedFeature(
                format!("Expression type not implemented: {:?}", expr.kind)
            ))
        }
    }

    pub fn compile_literal(&mut self, lit: &Literal) -> Result<LLVMValue, CodegenError> {
        match lit {
            Literal::Integer(i) => Ok(LLVMValue {
                type_info: InferType::Int,
                llvm_type: crate::codegen::types::LLVMType::Int(64),
                value_id: i.to_string(),
            }),
            Literal::Float(f) => Ok(LLVMValue {
                type_info: InferType::Float,
                llvm_type: crate::codegen::types::LLVMType::Double,
                value_id: f.to_string(),
            }),
            Literal::String(s) => {
                let string_const_name = format!("@.str.{}", self.next_temp());
                let string_len = s.len() + 1;
                let global_string_def = format!("{} = private unnamed_addr constant [{} x i8] c\"{}\\00\", align 1", 
                    string_const_name,
                    string_len,
                    s.escape_default().to_string().replace("\"", "\\22")
                );
                self.global_strings.push(global_string_def);
                let global_ptr_temp = self.next_temp();
                self.emit(&format!("  %{} = getelementptr inbounds [{} x i8], [{} x i8]* {}, i64 0, i64 0", global_ptr_temp, string_len, string_len, string_const_name));
                Ok(LLVMValue {
                    type_info: InferType::String,
                    llvm_type: crate::codegen::types::LLVMType::Pointer(Box::new(crate::codegen::types::LLVMType::Int(8))),
                    value_id: format!("%{}", global_ptr_temp),
                })
            },
            Literal::Bool(b) => Ok(LLVMValue {
                type_info: InferType::Bool,
                llvm_type: crate::codegen::types::LLVMType::Int(1),
                value_id: if *b { "true".to_string() } else { "false".to_string() },
            }),
            Literal::Unit => Ok(LLVMValue {
                type_info: InferType::Unit,
                llvm_type: crate::codegen::types::LLVMType::Void,
                value_id: "".to_string(),
            }),
            _ => Err(CodegenError::UnsupportedFeature(format!("Literal type not implemented: {:?}", lit))),
        }
    }

    pub fn compile_binary_operation(&mut self, op: &BinaryOp, left: &Expr, right: &Expr) -> Result<LLVMValue, CodegenError> {
        let left_val = self.compile_expression(left)?;
        let right_val = self.compile_expression(right)?;
        
        let result_temp = self.next_temp();
        
        let (instruction, result_type) = match op {
            BinaryOp::Add => (if left_val.type_info == InferType::Float { "fadd" } else { "add" }, left_val.type_info.clone()),
            BinaryOp::Sub => (if left_val.type_info == InferType::Float { "fsub" } else { "sub" }, left_val.type_info.clone()),
            BinaryOp::Mul => (if left_val.type_info == InferType::Float { "fmul" } else { "mul" }, left_val.type_info.clone()),
            BinaryOp::Div => (if left_val.type_info == InferType::Float { "fdiv" } else { "sdiv" }, left_val.type_info.clone()),
            BinaryOp::Mod => ("srem", InferType::Int),
            BinaryOp::Eq => (if left_val.type_info == InferType::Float { "fcmp oeq" } else { "icmp eq" }, InferType::Bool),
            BinaryOp::Ne => (if left_val.type_info == InferType::Float { "fcmp one" } else { "icmp ne" }, InferType::Bool),
            BinaryOp::Lt => (if left_val.type_info == InferType::Float { "fcmp olt" } else { "icmp slt" }, InferType::Bool),
            BinaryOp::Le => (if left_val.type_info == InferType::Float { "fcmp ole" } else { "icmp sle" }, InferType::Bool),
            BinaryOp::Gt => (if left_val.type_info == InferType::Float { "fcmp ogt" } else { "icmp sgt" }, InferType::Bool),
            BinaryOp::Ge => (if left_val.type_info == InferType::Float { "fcmp oge" } else { "icmp sge" }, InferType::Bool),
            BinaryOp::And => ("and", InferType::Bool),
            BinaryOp::Or => ("or", InferType::Bool),
            BinaryOp::Xor => ("xor", InferType::Bool),
            BinaryOp::BitAnd => ("and", InferType::Int),
            BinaryOp::BitOr => ("or", InferType::Int),
            BinaryOp::BitXor => ("xor", InferType::Int),
            BinaryOp::Shl => ("shl", InferType::Int),
            BinaryOp::Shr => ("lshr", InferType::Int),
            BinaryOp::Is => ("icmp eq", InferType::Bool),
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
            type_info: result_type.clone(),
            llvm_type: infer_to_llvm_type(&result_type),
            value_id: format!("%{}", result_temp),
        })
    }

    pub fn compile_unary_operation(&mut self, op: &UnaryOp, operand: &Expr) -> Result<LLVMValue, CodegenError> {
        let operand_val = self.compile_expression(operand)?;
        let result_temp = self.next_temp();

        let (instruction, result_type) = match op {
            UnaryOp::Neg => {
                if operand_val.type_info == InferType::Float {
                    ("fsub", InferType::Float)
                } else {
                    ("sub", InferType::Int)
                }
            }
            UnaryOp::Not => ("xor", InferType::Bool),
            UnaryOp::BitNot => ("xor", InferType::Int),
        };

        let llvm_result_type = infer_to_llvm_type(&result_type);

        match op {
            UnaryOp::Neg => {
                if result_type == InferType::Float {
                    self.emit(&format!(
                        "  %{} = {} {} 0.0, {}",
                        result_temp, instruction, llvm_result_type, operand_val.value_id
                    ));
                } else {
                    self.emit(&format!(
                        "  %{} = {} {} 0, {}",
                        result_temp, instruction, llvm_result_type, operand_val.value_id
                    ));
                }
            }
            UnaryOp::Not => {
                self.emit(&format!(
                    "  %{} = {} {} {}, true",
                    result_temp, instruction, llvm_result_type, operand_val.value_id
                ));
            }
            UnaryOp::BitNot => {
                self.emit(&format!(
                    "  %{} = {} {} {}, -1",
                    result_temp, instruction, llvm_result_type, operand_val.value_id
                ));
            }
        }

        Ok(LLVMValue {
            type_info: result_type,
            llvm_type: llvm_result_type,
            value_id: format!("%{}", result_temp),
        })
    }
}