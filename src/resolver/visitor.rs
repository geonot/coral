use crate::{
    ast::{Program, Stmt, StmtKind, Expr, Type, VisitorMut},
    resolver::{types::Substitution, TypeResolver},
};

impl TypeResolver {
    /// Apply final substitutions to the program AST
    pub(super) fn apply_substitutions_to_program(
        &mut self,
        program: &mut Program,
        subst: &Substitution,
    ) -> Result<(), super::error::TypeError> {
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
    resolver: &'a mut TypeResolver,
    substitution: &'a Substitution,
}

impl<'a> TypeUpdater<'a> {
    fn update_type(&mut self, ast_type: &mut Type) {
        if let Ok(infer_type) = self.resolver.ast_type_to_infer_type(ast_type) {
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
                for stmt in body {
                    self.visit_stmt_mut(stmt);
                }
            }
            StmtKind::If { condition, then_branch, else_branch } => {
                self.visit_expr_mut(condition);
                for stmt in then_branch {
                    self.visit_stmt_mut(stmt);
                }
                if let Some(else_block) = else_branch {
                    for stmt in else_block {
                        self.visit_stmt_mut(stmt);
                    }
                }
            }
            StmtKind::While { condition, body } => {
                self.visit_expr_mut(condition);
                for stmt in body {
                    self.visit_stmt_mut(stmt);
                }
            }
            StmtKind::Return(value) => {
                if let Some(val) = value {
                    self.visit_expr_mut(val);
                }
            }
            StmtKind::Expression(expression) => {
                self.visit_expr_mut(expression);
            }
            // Other statements...
            _ => {}
        }
    }
    
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        self.update_type(&mut expr.type_);
        // If you need to traverse child expressions, do so here manually or rely on the visitor pattern
        // Removed call to walk_expr_mut as it does not exist
    }

    fn visit_type_mut(&mut self, type_: &mut Type) {
        self.update_type(type_);
    }
}
