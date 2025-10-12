use oxc_ast::ast::*;
use oxc_span::GetSpan;
use oxc_syntax::precedence::Precedence;

use crate::{Codegen, Context, fast_gen};

pub(crate) const PURE_COMMENT: &str = "/* @__PURE__ */ ";
pub(crate) const NO_SIDE_EFFECTS_NEW_LINE_COMMENT: &str = "/* @__NO_SIDE_EFFECTS__ */\n";
pub(crate) const NO_SIDE_EFFECTS_COMMENT: &str = "/* @__NO_SIDE_EFFECTS__ */ ";

/// Generate source code for an AST node.
pub trait Gen: GetSpan {
    /// Generate code for an AST node.
    fn r#gen(&self, p: &mut Codegen, ctx: Context);

    /// Generate code for an AST node. Alias for `gen`.
    #[inline(always)]
    fn print(&self, p: &mut Codegen, ctx: Context) {
        self.r#gen(p, ctx);
    }
}

/// Generate source code for an expression.
pub trait GenExpr: GetSpan {
    /// Generate code for an expression, respecting operator precedence.
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context);

    /// Generate code for an expression, respecting operator precedence. Alias for `gen_expr`.
    #[inline(always)]
    fn print_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        self.gen_expr(p, precedence, ctx);
    }
}

impl Gen for Program<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_program(self, p, ctx);
        }
    }
}

impl Gen for Hashbang<'_> {
    fn r#gen(&self, p: &mut Codegen, _ctx: Context) {
        unsafe {
            fast_gen::emit_hashbang(self, p, Context::empty());
        }
    }
}

impl Gen for Directive<'_> {
    fn r#gen(&self, p: &mut Codegen, _ctx: Context) {
        unsafe {
            fast_gen::emit_directive(self, p, Context::empty());
        }
    }
}

impl Gen for Statement<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_statement(self, p, ctx);
        }
    }
}

impl Gen for ExpressionStatement<'_> {
    fn r#gen(&self, p: &mut Codegen, _ctx: Context) {
        unsafe {
            fast_gen::emit_expression_statement(self, p);
        }
    }
}

impl Gen for IfStatement<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_if_statement(self, p, ctx);
        }
    }
}

impl Gen for BlockStatement<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_block_statement(self, p, ctx);
        }
    }
}

impl Gen for ForStatement<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_for_statement(self, p, ctx);
        }
    }
}

impl Gen for ForInStatement<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_for_in_statement(self, p, ctx);
        }
    }
}

impl Gen for ForOfStatement<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_for_of_statement(self, p, ctx);
        }
    }
}

impl Gen for ForStatementInit<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_for_statement_init(self, p, ctx);
        }
    }
}

impl Gen for ForStatementLeft<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_for_statement_left(self, p, ctx);
        }
    }
}

impl Gen for WhileStatement<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_while_statement(self, p, ctx);
        }
    }
}

impl Gen for DoWhileStatement<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_do_while_statement(self, p, ctx);
        }
    }
}

impl Gen for EmptyStatement {
    fn r#gen(&self, p: &mut Codegen, _ctx: Context) {
        unsafe {
            fast_gen::emit_empty_statement(self, p);
        }
    }
}

impl Gen for ContinueStatement<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_continue_statement(self, p, ctx);
        }
    }
}

impl Gen for BreakStatement<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_break_statement(self, p, ctx);
        }
    }
}

impl Gen for SwitchStatement<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_switch_statement(self, p, ctx);
        }
    }
}

impl Gen for SwitchCase<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_switch_case(self, p, ctx);
        }
    }
}

impl Gen for ReturnStatement<'_> {
    fn r#gen(&self, p: &mut Codegen, _ctx: Context) {
        unsafe {
            fast_gen::emit_return_statement(self, p);
        }
    }
}

impl Gen for LabeledStatement<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_labeled_statement(self, p, ctx);
        }
    }
}

impl Gen for TryStatement<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_try_statement(self, p, ctx);
        }
    }
}

impl Gen for CatchClause<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_catch_clause(self, p, ctx);
        }
    }
}

impl Gen for ThrowStatement<'_> {
    fn r#gen(&self, p: &mut Codegen, _ctx: Context) {
        unsafe {
            fast_gen::emit_throw_statement(self, p);
        }
    }
}

impl Gen for WithStatement<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_with_statement(self, p, ctx);
        }
    }
}

impl Gen for DebuggerStatement {
    fn r#gen(&self, p: &mut Codegen, _ctx: Context) {
        unsafe {
            fast_gen::emit_debugger_statement(self, p);
        }
    }
}

impl Gen for VariableDeclaration<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        p.add_source_mapping(self.span);
        unsafe {
            fast_gen::emit_variable_declaration_for_head(self, p, ctx);
        }
    }
}

impl Gen for VariableDeclarator<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_variable_declarator(self, p, ctx);
        }
    }
}

impl Gen for Function<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_function(self, p, ctx);
        }
    }
}

impl Gen for FunctionBody<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_function_body(self, p, ctx);
        }
    }
}

impl Gen for FormalParameter<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_formal_parameter(self, p, ctx);
        }
    }
}

impl Gen for FormalParameters<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_formal_parameters(self, p, ctx);
        }
    }
}

impl Gen for ImportDeclaration<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_import_declaration(self, p, ctx);
        }
    }
}

impl Gen for WithClause<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_with_clause(self, p, ctx);
        }
    }
}

impl Gen for ImportAttribute<'_> {
    fn r#gen(&self, p: &mut Codegen, _ctx: Context) {
        unsafe {
            fast_gen::emit_import_attribute(self, p, Context::empty());
        }
    }
}

impl Gen for ExportNamedDeclaration<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_export_named_declaration(self, p, ctx);
        }
    }
}

impl Gen for TSExportAssignment<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_export_assignment(self, p, ctx);
        }
    }
}

impl Gen for TSNamespaceExportDeclaration<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_namespace_export_declaration(self, p, ctx);
        }
    }
}

impl Gen for ExportSpecifier<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_export_specifier(self, p, ctx);
        }
    }
}

impl Gen for ModuleExportName<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_module_export_name(self, p, ctx);
        }
    }
}

impl Gen for ExportAllDeclaration<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_export_all_declaration(self, p, ctx);
        }
    }
}

impl Gen for ExportDefaultDeclaration<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_export_default_declaration(self, p, ctx);
        }
    }
}
impl Gen for ExportDefaultDeclarationKind<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_export_default_declaration_kind(self, p, ctx);
        }
    }
}

impl GenExpr for Expression<'_> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        unsafe {
            fast_gen::emit_expression(self, p, precedence, ctx);
        }
    }
}

impl GenExpr for ParenthesizedExpression<'_> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        unsafe {
            fast_gen::emit_parenthesized_expression(self, p, precedence, ctx);
        }
    }
}

impl Gen for IdentifierReference<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_identifier_reference(self, p, ctx);
        }
    }
}

impl Gen for IdentifierName<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_identifier_name(self, p, ctx);
        }
    }
}

impl Gen for BindingIdentifier<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_binding_identifier(self, p, ctx);
        }
    }
}

impl Gen for LabelIdentifier<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_label_identifier(self, p, ctx);
        }
    }
}

impl Gen for BooleanLiteral {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_boolean_literal(self, p, ctx);
        }
    }
}

impl Gen for NullLiteral {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_null_literal(self, p, ctx);
        }
    }
}

impl GenExpr for NumericLiteral<'_> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        unsafe {
            fast_gen::emit_numeric_literal(self, p, precedence, ctx);
        }
    }
}

impl GenExpr for BigIntLiteral<'_> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        unsafe {
            fast_gen::emit_bigint_literal(self, p, precedence, ctx);
        }
    }
}

impl Gen for RegExpLiteral<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_reg_exp_literal(self, p, ctx);
        }
    }
}

impl Gen for StringLiteral<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_string_literal(self, p, ctx);
        }
    }
}

impl Gen for ThisExpression {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_this_expression(self, p, ctx);
        }
    }
}

impl GenExpr for MemberExpression<'_> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        match self {
            Self::ComputedMemberExpression(expr) => expr.print_expr(p, precedence, ctx),
            Self::StaticMemberExpression(expr) => expr.print_expr(p, precedence, ctx),
            Self::PrivateFieldExpression(expr) => expr.print_expr(p, precedence, ctx),
        }
    }
}

impl GenExpr for ComputedMemberExpression<'_> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        unsafe {
            fast_gen::emit_computed_member_expression(self, p, precedence, ctx);
        }
    }
}

impl GenExpr for StaticMemberExpression<'_> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        unsafe {
            fast_gen::emit_static_member_expression(self, p, precedence, ctx);
        }
    }
}

impl GenExpr for PrivateFieldExpression<'_> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        unsafe {
            fast_gen::emit_private_field_expression(self, p, precedence, ctx);
        }
    }
}

impl GenExpr for CallExpression<'_> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        unsafe {
            fast_gen::emit_call_expression(self, p, precedence, ctx);
        }
    }
}

impl Gen for Argument<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_argument(self, p, ctx);
        }
    }
}

impl Gen for ArrayExpressionElement<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_array_expression_element(self, p, ctx);
        }
    }
}

impl Gen for SpreadElement<'_> {
    fn r#gen(&self, p: &mut Codegen, _ctx: Context) {
        unsafe {
            fast_gen::emit_spread_element(self, p, Context::empty());
        }
    }
}

impl Gen for ArrayExpression<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_array_expression(self, p, ctx);
        }
    }
}

impl GenExpr for ObjectExpression<'_> {
    fn gen_expr(&self, p: &mut Codegen, _precedence: Precedence, ctx: Context) {
        unsafe {
            fast_gen::emit_object_expression(self, p, ctx);
        }
    }
}

impl Gen for ObjectPropertyKind<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_object_property_kind(self, p, ctx);
        }
    }
}

impl Gen for ObjectProperty<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_object_property(self, p, ctx);
        }
    }
}

impl Gen for PropertyKey<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_property_key(self, p, ctx);
        }
    }
}

impl GenExpr for ArrowFunctionExpression<'_> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        unsafe {
            fast_gen::emit_arrow_function_expression(self, p, precedence, ctx);
        }
    }
}

impl GenExpr for YieldExpression<'_> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        unsafe {
            fast_gen::emit_yield_expression(self, p, precedence, ctx);
        }
    }
}

impl GenExpr for UpdateExpression<'_> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        unsafe {
            fast_gen::emit_update_expression(self, p, precedence, ctx);
        }
    }
}

impl GenExpr for UnaryExpression<'_> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        unsafe {
            fast_gen::emit_unary_expression(self, p, precedence, ctx);
        }
    }
}

impl GenExpr for BinaryExpression<'_> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        unsafe {
            fast_gen::emit_binary_expression(self, p, precedence, ctx);
        }
    }
}

impl GenExpr for PrivateInExpression<'_> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        unsafe {
            fast_gen::emit_private_in_expression(self, p, precedence, ctx);
        }
    }
}

impl GenExpr for LogicalExpression<'_> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        unsafe {
            fast_gen::emit_logical_expression(self, p, precedence, ctx);
        }
    }
}

impl GenExpr for ConditionalExpression<'_> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        unsafe {
            fast_gen::emit_conditional_expression(self, p, precedence, ctx);
        }
    }
}

impl GenExpr for AssignmentExpression<'_> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        unsafe {
            fast_gen::emit_assignment_expression(self, p, precedence, ctx);
        }
    }
}

impl Gen for AssignmentTarget<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_assignment_target(self, p, ctx);
        }
    }
}

impl GenExpr for SimpleAssignmentTarget<'_> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        unsafe {
            fast_gen::emit_simple_assignment_target(self, p, precedence, ctx);
        }
    }
}

impl Gen for AssignmentTargetPattern<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_assignment_target_pattern(self, p, ctx);
        }
    }
}

impl Gen for ArrayAssignmentTarget<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_array_assignment_target(self, p, ctx);
        }
    }
}

impl Gen for ObjectAssignmentTarget<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_object_assignment_target(self, p, ctx);
        }
    }
}

impl Gen for AssignmentTargetMaybeDefault<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_assignment_target_maybe_default(self, p, ctx);
        }
    }
}

impl Gen for AssignmentTargetWithDefault<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_assignment_target_with_default(self, p, ctx);
        }
    }
}

impl Gen for AssignmentTargetProperty<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_assignment_target_property(self, p, ctx);
        }
    }
}

impl Gen for AssignmentTargetPropertyIdentifier<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_assignment_target_property_identifier(self, p, ctx);
        }
    }
}

impl Gen for AssignmentTargetPropertyProperty<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_assignment_target_property_property(self, p, ctx);
        }
    }
}

impl Gen for AssignmentTargetRest<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_assignment_target_rest(self, p, ctx);
        }
    }
}

impl GenExpr for SequenceExpression<'_> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        unsafe {
            fast_gen::emit_sequence_expression(self, p, precedence, ctx);
        }
    }
}

impl GenExpr for ImportExpression<'_> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        unsafe {
            fast_gen::emit_import_expression(self, p, precedence, ctx);
        }
    }
}

impl Gen for TemplateLiteral<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_template_literal(self, p, ctx);
        }
    }
}

impl Gen for TaggedTemplateExpression<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_tagged_template_expression(self, p, ctx);
        }
    }
}

impl Gen for Super {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_super(self, p, ctx);
        }
    }
}

impl GenExpr for AwaitExpression<'_> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        unsafe {
            fast_gen::emit_await_expression(self, p, precedence, ctx);
        }
    }
}

impl GenExpr for ChainExpression<'_> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        unsafe {
            fast_gen::emit_chain_expression(self, p, precedence, ctx);
        }
    }
}

impl GenExpr for NewExpression<'_> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        unsafe {
            fast_gen::emit_new_expression(self, p, precedence, ctx);
        }
    }
}

impl GenExpr for TSAsExpression<'_> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_as_expression(self, p, precedence, ctx);
        }
    }
}

impl GenExpr for TSSatisfiesExpression<'_> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_satisfies_expression(self, p, precedence, ctx);
        }
    }
}

impl GenExpr for TSNonNullExpression<'_> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_non_null_expression(self, p, precedence, ctx);
        }
    }
}

impl GenExpr for TSInstantiationExpression<'_> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_instantiation_expression(self, p, precedence, ctx);
        }
    }
}

impl GenExpr for TSTypeAssertion<'_> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_type_assertion(self, p, precedence, ctx);
        }
    }
}

impl Gen for MetaProperty<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_meta_property(self, p, ctx);
        }
    }
}

impl Gen for Class<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_class(self, p, ctx);
        }
    }
}

impl Gen for ClassBody<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_class_body(self, p, ctx);
        }
    }
}

impl Gen for ClassElement<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_class_element(self, p, ctx);
        }
    }
}

impl Gen for JSXIdentifier<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_jsx_identifier(self, p, ctx);
        }
    }
}

impl Gen for JSXMemberExpressionObject<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_jsx_member_expression_object(self, p, ctx);
        }
    }
}

impl Gen for JSXMemberExpression<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_jsx_member_expression(self, p, ctx);
        }
    }
}

impl Gen for JSXElementName<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_jsx_element_name(self, p, ctx);
        }
    }
}

impl Gen for JSXNamespacedName<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_jsx_namespaced_name(self, p, ctx);
        }
    }
}

impl Gen for JSXAttributeName<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_jsx_attribute_name(self, p, ctx);
        }
    }
}

impl Gen for JSXAttribute<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_jsx_attribute(self, p, ctx);
        }
    }
}

impl Gen for JSXEmptyExpression {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_jsx_empty_expression(self, p, ctx);
        }
    }
}

impl Gen for JSXExpression<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_jsx_expression(self, p, ctx);
        }
    }
}

impl Gen for JSXExpressionContainer<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_jsx_expression_container(self, p, ctx);
        }
    }
}

impl Gen for JSXAttributeValue<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_jsx_attribute_value(self, p, ctx);
        }
    }
}

impl Gen for JSXSpreadAttribute<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_jsx_spread_attribute(self, p, ctx);
        }
    }
}

impl Gen for JSXAttributeItem<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_jsx_attribute_item(self, p, ctx);
        }
    }
}

impl Gen for JSXElement<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_jsx_element(self, p, ctx);
        }
    }
}

impl Gen for JSXOpeningFragment {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_jsx_opening_fragment(self, p, ctx);
        }
    }
}

impl Gen for JSXClosingFragment {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_jsx_closing_fragment(self, p, ctx);
        }
    }
}

impl Gen for JSXText<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_jsx_text(self, p, ctx);
        }
    }
}

impl Gen for JSXSpreadChild<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_jsx_spread_child(self, p, ctx);
        }
    }
}

impl Gen for JSXChild<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_jsx_child(self, p, ctx);
        }
    }
}

impl Gen for JSXFragment<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_jsx_fragment(self, p, ctx);
        }
    }
}

impl Gen for StaticBlock<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_static_block(self, p, ctx);
        }
    }
}

impl Gen for MethodDefinition<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_method_definition(self, p, ctx);
        }
    }
}

impl Gen for PropertyDefinition<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_property_definition(self, p, ctx);
        }
    }
}

impl Gen for AccessorProperty<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_accessor_property(self, p, ctx);
        }
    }
}

impl Gen for PrivateIdentifier<'_> {
    fn r#gen(&self, p: &mut Codegen, _ctx: Context) {
        unsafe {
            fast_gen::emit_private_identifier(self, p);
        }
    }
}

impl Gen for BindingPattern<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_binding_pattern(self, p, ctx);
        }
    }
}

impl Gen for BindingPatternKind<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_binding_pattern_kind(self, p, ctx);
        }
    }
}

impl Gen for ObjectPattern<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_object_pattern(self, p, ctx);
        }
    }
}

impl Gen for BindingProperty<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_binding_property(self, p, ctx);
        }
    }
}

impl Gen for BindingRestElement<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_binding_rest_element(self, p, ctx);
        }
    }
}

impl Gen for ArrayPattern<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_array_pattern(self, p, ctx);
        }
    }
}

impl Gen for AssignmentPattern<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_assignment_pattern(self, p, ctx);
        }
    }
}

impl Gen for Decorator<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_decorator(self, p, ctx);
        }
    }
}

impl Gen for TSClassImplements<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_class_implements(self, p, ctx);
        }
    }
}

impl Gen for TSTypeParameterDeclaration<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_type_parameter_declaration(self, p, ctx);
        }
    }
}

impl Gen for TSTypeAnnotation<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_type_annotation(self, p, ctx);
        }
    }
}

impl Gen for TSType<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_type(self, p, ctx);
        }
    }
}

impl Gen for TSArrayType<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_array_type(self, p, ctx);
        }
    }
}

impl Gen for TSTupleType<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_tuple_type(self, p, ctx);
        }
    }
}

impl Gen for TSUnionType<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_union_type(self, p, ctx);
        }
    }
}

impl Gen for TSParenthesizedType<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_parenthesized_type(self, p, ctx);
        }
    }
}

impl Gen for TSIntersectionType<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_intersection_type(self, p, ctx);
        }
    }
}

impl Gen for TSConditionalType<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_conditional_type(self, p, ctx);
        }
    }
}

impl Gen for TSInferType<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_infer_type(self, p, ctx);
        }
    }
}

impl Gen for TSIndexedAccessType<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_indexed_access_type(self, p, ctx);
        }
    }
}

impl Gen for TSMappedType<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_mapped_type(self, p, ctx);
        }
    }
}

impl Gen for TSQualifiedName<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_qualified_name(self, p, ctx);
        }
    }
}

impl Gen for TSTypeOperator<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_type_operator(self, p, ctx);
        }
    }
}

impl Gen for TSTypePredicate<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_type_predicate(self, p, ctx);
        }
    }
}

impl Gen for TSTypeReference<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_type_reference(self, p, ctx);
        }
    }
}

impl Gen for JSDocNullableType<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_jsdoc_nullable_type(self, p, ctx);
        }
    }
}

impl Gen for JSDocNonNullableType<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_jsdoc_non_nullable_type(self, p, ctx);
        }
    }
}

impl Gen for TSTemplateLiteralType<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_template_literal_type(self, p, ctx);
        }
    }
}

impl Gen for TSTypeLiteral<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_type_literal(self, p, ctx);
        }
    }
}

impl Gen for TSTypeName<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_type_name(self, p, ctx);
        }
    }
}

impl Gen for TSLiteral<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_literal(self, p, ctx);
        }
    }
}

impl Gen for TSTypeParameter<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_type_parameter(self, p, ctx);
        }
    }
}

impl Gen for TSFunctionType<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_function_type(self, p, ctx);
        }
    }
}

impl Gen for TSThisParameter<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_this_parameter(self, p, ctx);
        }
    }
}

impl Gen for TSSignature<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_signature(self, p, ctx);
        }
    }
}

impl Gen for TSPropertySignature<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_property_signature(self, p, ctx);
        }
    }
}

impl Gen for TSTypeQuery<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_type_query(self, p, ctx);
        }
    }
}

impl Gen for TSTypeQueryExprName<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_type_query_expr_name(self, p, ctx);
        }
    }
}

impl Gen for TSImportType<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_import_type(self, p, ctx);
        }
    }
}

impl Gen for TSImportTypeQualifier<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_import_type_qualifier(self, p, ctx);
        }
    }
}

impl Gen for TSImportTypeQualifiedName<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_import_type_qualified_name(self, p, ctx);
        }
    }
}

impl Gen for TSTypeParameterInstantiation<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_type_parameter_instantiation(self, p, ctx);
        }
    }
}

impl Gen for TSIndexSignature<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_index_signature(self, p, ctx);
        }
    }
}

impl Gen for TSTupleElement<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_tuple_element(self, p, ctx);
        }
    }
}

impl Gen for TSNamedTupleMember<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_named_tuple_member(self, p, ctx);
        }
    }
}

impl Gen for TSModuleDeclaration<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_module_declaration(self, p, ctx);
        }
    }
}

impl Gen for TSModuleDeclarationName<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_module_declaration_name(self, p, ctx);
        }
    }
}

impl Gen for TSModuleBlock<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_module_block(self, p, ctx);
        }
    }
}

impl Gen for TSTypeAliasDeclaration<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_type_alias_declaration(self, p, ctx);
        }
    }
}

impl Gen for TSInterfaceDeclaration<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_interface_declaration(self, p, ctx);
        }
    }
}

impl Gen for TSInterfaceHeritage<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_interface_heritage(self, p, ctx);
        }
    }
}

impl Gen for TSEnumDeclaration<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_enum_declaration(self, p, ctx);
        }
    }
}

impl Gen for TSEnumBody<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_enum_body(self, p, ctx);
        }
    }
}

impl Gen for TSEnumMember<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_enum_member(self, p, ctx);
        }
    }
}

impl Gen for TSConstructorType<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_constructor_type(self, p, ctx);
        }
    }
}

impl Gen for TSImportEqualsDeclaration<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_import_equals_declaration(self, p, ctx);
        }
    }
}

impl Gen for TSModuleReference<'_> {
    fn r#gen(&self, p: &mut Codegen, ctx: Context) {
        unsafe {
            fast_gen::emit_ts_import_equals_module_reference(self, p, ctx);
        }
    }
}

impl GenExpr for V8IntrinsicExpression<'_> {
    fn gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context) {
        unsafe {
            fast_gen::emit_v8_intrinsic_expression(self, p, precedence, ctx);
        }
    }
}
