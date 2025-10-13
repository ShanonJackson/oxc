#![expect(clippy::redundant_pub_crate)]

use cow_utils::CowUtils;
use oxc_ast::ast::*;
use oxc_span::GetSpan;
use std::{ops::Not, ptr};

use oxc_syntax::{
    operator::UnaryOperator,
    precedence::{GetPrecedence, Precedence},
};

use crate::{
    Codegen, Context, Operator, Quote,
    binary_expr_visitor::{BinaryExpressionVisitor, Binaryish, BinaryishOperator},
    r#gen::{
        Gen, GenExpr, NO_SIDE_EFFECTS_COMMENT, NO_SIDE_EFFECTS_NEW_LINE_COMMENT, PURE_COMMENT,
    },
};

#[inline(always)]
unsafe fn cast_codegen<'a>(p: &mut Codegen) -> &'a mut Codegen<'a> {
    unsafe { &mut *ptr::from_mut(p).cast::<Codegen<'a>>() }
}

#[inline(always)]
pub(crate) unsafe fn emit_hashbang<'a>(hashbang: &Hashbang<'a>, p: &mut Codegen, ctx: Context) {
    let _ = ctx;
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_hashbang_impl(hashbang, p);
}

#[inline(always)]
pub(crate) unsafe fn emit_directive<'a>(directive: &Directive<'a>, p: &mut Codegen, ctx: Context) {
    let _ = ctx;
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_directive_impl(directive, p);
}

#[inline(always)]
pub(crate) unsafe fn emit_program<'a>(program: &Program<'a>, p: &mut Codegen, ctx: Context) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_program_impl(program, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_statement<'a>(stmt: &Statement<'a>, p: &mut Codegen, ctx: Context) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_statement_impl(stmt, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_block_statement<'a>(
    block: &BlockStatement<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_block_statement_impl(block, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_meta_property<'a>(
    meta: &MetaProperty<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_meta_property_impl(meta, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_class<'a>(class: &Class<'a>, p: &mut Codegen, ctx: Context) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_class_impl(class, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_class_body<'a>(body: &ClassBody<'a>, p: &mut Codegen, ctx: Context) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_class_body_impl(body, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_class_element<'a>(
    elem: &ClassElement<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_class_element_impl(elem, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_static_block<'a>(block: &StaticBlock<'a>, p: &mut Codegen, ctx: Context) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_static_block_impl(block, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_template_literal<'a>(
    template: &TemplateLiteral<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let _ = ctx;
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_template_literal_impl(template, p);
}

#[inline(always)]
pub(crate) unsafe fn emit_tagged_template_expression<'a>(
    expr: &TaggedTemplateExpression<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_tagged_template_expression_impl(expr, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_method_definition<'a>(
    method: &MethodDefinition<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_method_definition_impl(method, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_property_definition<'a>(
    property: &PropertyDefinition<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_property_definition_impl(property, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_accessor_property<'a>(
    property: &AccessorProperty<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_accessor_property_impl(property, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_private_identifier<'a>(ident: &PrivateIdentifier<'a>, p: &mut Codegen) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_private_identifier_impl(ident, p);
}

#[inline(always)]
pub(crate) unsafe fn emit_binding_pattern<'a>(
    pattern: &BindingPattern<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_binding_pattern_impl(pattern, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_binding_pattern_kind<'a>(
    pattern: &BindingPatternKind<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_binding_pattern_kind_impl(pattern, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_object_pattern<'a>(
    pattern: &ObjectPattern<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_object_pattern_impl(pattern, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_binding_property<'a>(
    property: &BindingProperty<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_binding_property_impl(property, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_binding_rest_element<'a>(
    element: &BindingRestElement<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_binding_rest_element_impl(element, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_array_pattern<'a>(
    pattern: &ArrayPattern<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_array_pattern_impl(pattern, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_jsx_identifier<'a>(
    ident: &JSXIdentifier<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_jsx_identifier_impl(ident, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_jsx_member_expression_object<'a>(
    object: &JSXMemberExpressionObject<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_jsx_member_expression_object_impl(object, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_jsx_member_expression<'a>(
    expr: &JSXMemberExpression<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_jsx_member_expression_impl(expr, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_jsx_element_name<'a>(
    name: &JSXElementName<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_jsx_element_name_impl(name, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_jsx_namespaced_name<'a>(
    name: &JSXNamespacedName<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_jsx_namespaced_name_impl(name, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_jsx_attribute_name<'a>(
    name: &JSXAttributeName<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_jsx_attribute_name_impl(name, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_jsx_attribute<'a>(
    attribute: &JSXAttribute<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_jsx_attribute_impl(attribute, p, ctx);
}

#[inline(always)]
#[expect(clippy::extra_unused_lifetimes)]
pub(crate) unsafe fn emit_jsx_empty_expression<'a>(
    expr: &JSXEmptyExpression,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_jsx_empty_expression_impl(expr, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_jsx_expression<'a>(
    expr: &JSXExpression<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_jsx_expression_impl(expr, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_jsx_expression_container<'a>(
    expr: &JSXExpressionContainer<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_jsx_expression_container_impl(expr, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_jsx_attribute_value<'a>(
    value: &JSXAttributeValue<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_jsx_attribute_value_impl(value, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_jsx_spread_attribute<'a>(
    attr: &JSXSpreadAttribute<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_jsx_spread_attribute_impl(attr, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_jsx_attribute_item<'a>(
    item: &JSXAttributeItem<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_jsx_attribute_item_impl(item, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_jsx_element<'a>(element: &JSXElement<'a>, p: &mut Codegen, ctx: Context) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_jsx_element_impl(element, p, ctx);
}

#[inline(always)]
#[expect(clippy::extra_unused_lifetimes)]
pub(crate) unsafe fn emit_jsx_opening_fragment<'a>(
    fragment: &JSXOpeningFragment,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_jsx_opening_fragment_impl(fragment, p, ctx);
}

#[inline(always)]
#[expect(clippy::extra_unused_lifetimes)]
pub(crate) unsafe fn emit_jsx_closing_fragment<'a>(
    fragment: &JSXClosingFragment,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_jsx_closing_fragment_impl(fragment, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_jsx_text<'a>(text: &JSXText<'a>, p: &mut Codegen, ctx: Context) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_jsx_text_impl(text, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_jsx_spread_child<'a>(
    child: &JSXSpreadChild<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_jsx_spread_child_impl(child, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_jsx_child<'a>(child: &JSXChild<'a>, p: &mut Codegen, ctx: Context) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_jsx_child_impl(child, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_jsx_fragment<'a>(
    fragment: &JSXFragment<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_jsx_fragment_impl(fragment, p, ctx);
}
#[inline(always)]
pub(crate) unsafe fn emit_expression<'a>(
    expr: &Expression<'a>,
    p: &mut Codegen,
    precedence: Precedence,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_expression_impl(expr, p, precedence, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_expression_statement<'a>(
    stmt: &ExpressionStatement<'a>,
    p: &mut Codegen,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_expression_statement_impl(stmt, p);
}

#[inline(always)]
pub(crate) unsafe fn emit_if_statement<'a>(stmt: &IfStatement<'a>, p: &mut Codegen, ctx: Context) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_if_statement_impl(stmt, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_for_statement<'a>(
    stmt: &ForStatement<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_for_statement_impl(stmt, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_for_statement_init<'a>(
    init: &ForStatementInit<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_for_statement_init_impl(init, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_for_statement_left<'a>(
    left: &ForStatementLeft<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_for_statement_left_impl(left, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_for_in_statement<'a>(
    stmt: &ForInStatement<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_for_in_statement_impl(stmt, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_for_of_statement<'a>(
    stmt: &ForOfStatement<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_for_of_statement_impl(stmt, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_while_statement<'a>(
    stmt: &WhileStatement<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_while_statement_impl(stmt, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_do_while_statement<'a>(
    stmt: &DoWhileStatement<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_do_while_statement_impl(stmt, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_function<'a>(function: &Function<'a>, p: &mut Codegen, ctx: Context) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_function_impl(function, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_function_body<'a>(
    body: &FunctionBody<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_function_body_impl(body, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_empty_statement(stmt: &EmptyStatement, p: &mut Codegen) {
    let p: &mut Codegen<'_> = unsafe { cast_codegen(p) };
    emit_empty_statement_impl(stmt, p);
}

#[inline(always)]
pub(crate) unsafe fn emit_continue_statement<'a>(
    stmt: &ContinueStatement<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_continue_statement_impl(stmt, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_break_statement<'a>(
    stmt: &BreakStatement<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_break_statement_impl(stmt, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_switch_statement<'a>(
    stmt: &SwitchStatement<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_switch_statement_impl(stmt, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_return_statement<'a>(stmt: &ReturnStatement<'a>, p: &mut Codegen) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_return_statement_impl(stmt, p);
}

#[inline(always)]
pub(crate) unsafe fn emit_formal_parameter<'a>(
    param: &FormalParameter<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_formal_parameter_impl(param, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_formal_parameters<'a>(
    params: &FormalParameters<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_formal_parameters_impl(params, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_arrow_function_expression<'a>(
    expr: &ArrowFunctionExpression<'a>,
    p: &mut Codegen,
    precedence: Precedence,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_arrow_function_expression_impl(expr, p, precedence, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_parenthesized_expression<'a>(
    expr: &ParenthesizedExpression<'a>,
    p: &mut Codegen,
    precedence: Precedence,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_parenthesized_expression_impl(expr, p, precedence, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_identifier_reference<'a>(
    ident: &IdentifierReference<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_identifier_reference_impl(ident, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_identifier_name<'a>(
    ident: &IdentifierName<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_identifier_name_impl(ident, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_binding_identifier<'a>(
    ident: &BindingIdentifier<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_binding_identifier_impl(ident, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_label_identifier<'a>(
    ident: &LabelIdentifier<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_label_identifier_impl(ident, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_boolean_literal(lit: &BooleanLiteral, p: &mut Codegen, ctx: Context) {
    let p: &mut Codegen<'_> = unsafe { cast_codegen(p) };
    emit_boolean_literal_impl(lit, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_null_literal(lit: &NullLiteral, p: &mut Codegen, ctx: Context) {
    let p: &mut Codegen<'_> = unsafe { cast_codegen(p) };
    emit_null_literal_impl(lit, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_numeric_literal<'a>(
    lit: &NumericLiteral<'a>,
    p: &mut Codegen,
    precedence: Precedence,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_numeric_literal_impl(lit, p, precedence, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_bigint_literal<'a>(
    lit: &BigIntLiteral<'a>,
    p: &mut Codegen,
    precedence: Precedence,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_bigint_literal_impl(lit, p, precedence, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_reg_exp_literal<'a>(
    lit: &RegExpLiteral<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_reg_exp_literal_impl(lit, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_string_literal<'a>(
    lit: &StringLiteral<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_string_literal_impl(lit, p, ctx);
}

#[inline(always)]
#[expect(clippy::extra_unused_lifetimes)]
pub(crate) unsafe fn emit_this_expression<'a>(
    expr: &ThisExpression,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_this_expression_impl(expr, p, ctx);
}

#[inline(always)]
#[expect(clippy::extra_unused_lifetimes)]
pub(crate) unsafe fn emit_super<'a>(super_expr: &Super, p: &mut Codegen, ctx: Context) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_super_impl(super_expr, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_computed_member_expression<'a>(
    expr: &ComputedMemberExpression<'a>,
    p: &mut Codegen,
    precedence: Precedence,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_computed_member_expression_impl(expr, p, precedence, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_static_member_expression<'a>(
    expr: &StaticMemberExpression<'a>,
    p: &mut Codegen,
    precedence: Precedence,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_static_member_expression_impl(expr, p, precedence, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_private_field_expression<'a>(
    expr: &PrivateFieldExpression<'a>,
    p: &mut Codegen,
    precedence: Precedence,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_private_field_expression_impl(expr, p, precedence, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_yield_expression<'a>(
    expr: &YieldExpression<'a>,
    p: &mut Codegen,
    precedence: Precedence,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_yield_expression_impl(expr, p, precedence, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_update_expression<'a>(
    expr: &UpdateExpression<'a>,
    p: &mut Codegen,
    precedence: Precedence,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_update_expression_impl(expr, p, precedence, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_unary_expression<'a>(
    expr: &UnaryExpression<'a>,
    p: &mut Codegen,
    precedence: Precedence,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_unary_expression_impl(expr, p, precedence, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_binary_expression<'a>(
    expr: &BinaryExpression<'a>,
    p: &mut Codegen,
    precedence: Precedence,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_binary_expression_impl(expr, p, precedence, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_private_in_expression<'a>(
    expr: &PrivateInExpression<'a>,
    p: &mut Codegen,
    precedence: Precedence,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_private_in_expression_impl(expr, p, precedence, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_logical_expression<'a>(
    expr: &LogicalExpression<'a>,
    p: &mut Codegen,
    precedence: Precedence,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_logical_expression_impl(expr, p, precedence, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_conditional_expression<'a>(
    expr: &ConditionalExpression<'a>,
    p: &mut Codegen,
    precedence: Precedence,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_conditional_expression_impl(expr, p, precedence, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_assignment_expression<'a>(
    expr: &AssignmentExpression<'a>,
    p: &mut Codegen,
    precedence: Precedence,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_assignment_expression_impl(expr, p, precedence, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_assignment_target<'a>(
    target: &AssignmentTarget<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_assignment_target_impl(target, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_assignment_target_pattern<'a>(
    pattern: &AssignmentTargetPattern<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_assignment_target_pattern_impl(pattern, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_array_assignment_target<'a>(
    target: &ArrayAssignmentTarget<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_array_assignment_target_impl(target, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_object_assignment_target<'a>(
    target: &ObjectAssignmentTarget<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_object_assignment_target_impl(target, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_assignment_target_maybe_default<'a>(
    target: &AssignmentTargetMaybeDefault<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_assignment_target_maybe_default_impl(target, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_assignment_target_with_default<'a>(
    target: &AssignmentTargetWithDefault<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_assignment_target_with_default_impl(target, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_assignment_target_property<'a>(
    prop: &AssignmentTargetProperty<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_assignment_target_property_impl(prop, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_assignment_target_property_identifier<'a>(
    prop: &AssignmentTargetPropertyIdentifier<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_assignment_target_property_identifier_impl(prop, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_assignment_target_property_property<'a>(
    prop: &AssignmentTargetPropertyProperty<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_assignment_target_property_property_impl(prop, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_assignment_target_rest<'a>(
    rest: &AssignmentTargetRest<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_assignment_target_rest_impl(rest, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_assignment_pattern<'a>(
    pattern: &AssignmentPattern<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_assignment_pattern_impl(pattern, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_simple_assignment_target<'a>(
    target: &SimpleAssignmentTarget<'a>,
    p: &mut Codegen,
    precedence: Precedence,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_simple_assignment_target_impl(target, p, precedence, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_sequence_expression<'a>(
    expr: &SequenceExpression<'a>,
    p: &mut Codegen,
    precedence: Precedence,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_sequence_expression_impl(expr, p, precedence, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_import_expression<'a>(
    expr: &ImportExpression<'a>,
    p: &mut Codegen,
    precedence: Precedence,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_import_expression_impl(expr, p, precedence, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_await_expression<'a>(
    expr: &AwaitExpression<'a>,
    p: &mut Codegen,
    precedence: Precedence,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_await_expression_impl(expr, p, precedence, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_chain_expression<'a>(
    expr: &ChainExpression<'a>,
    p: &mut Codegen,
    precedence: Precedence,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_chain_expression_impl(expr, p, precedence, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_new_expression<'a>(
    expr: &NewExpression<'a>,
    p: &mut Codegen,
    precedence: Precedence,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_new_expression_impl(expr, p, precedence, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_as_expression<'a>(
    expr: &TSAsExpression<'a>,
    p: &mut Codegen,
    precedence: Precedence,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_as_expression_impl(expr, p, precedence, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_satisfies_expression<'a>(
    expr: &TSSatisfiesExpression<'a>,
    p: &mut Codegen,
    precedence: Precedence,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_satisfies_expression_impl(expr, p, precedence, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_non_null_expression<'a>(
    expr: &TSNonNullExpression<'a>,
    p: &mut Codegen,
    precedence: Precedence,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_non_null_expression_impl(expr, p, precedence, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_instantiation_expression<'a>(
    expr: &TSInstantiationExpression<'a>,
    p: &mut Codegen,
    precedence: Precedence,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_instantiation_expression_impl(expr, p, precedence, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_type_assertion<'a>(
    expr: &TSTypeAssertion<'a>,
    p: &mut Codegen,
    precedence: Precedence,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_type_assertion_impl(expr, p, precedence, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_v8_intrinsic_expression<'a>(
    expr: &V8IntrinsicExpression<'a>,
    p: &mut Codegen,
    precedence: Precedence,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_v8_intrinsic_expression_impl(expr, p, precedence, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_switch_case<'a>(case: &SwitchCase<'a>, p: &mut Codegen, ctx: Context) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_switch_case_impl(case, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_catch_clause<'a>(
    clause: &CatchClause<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_catch_clause_impl(clause, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_variable_declarator<'a>(
    declarator: &VariableDeclarator<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_variable_declarator_impl(declarator, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_call_expression<'a>(
    expr: &CallExpression<'a>,
    p: &mut Codegen,
    precedence: Precedence,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_call_expression_impl(expr, p, precedence, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_argument<'a>(arg: &Argument<'a>, p: &mut Codegen, ctx: Context) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_argument_impl(arg, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_array_expression_element<'a>(
    element: &ArrayExpressionElement<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_array_expression_element_impl(element, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_spread_element<'a>(
    elem: &SpreadElement<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_spread_element_impl(elem, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_array_expression<'a>(
    expr: &ArrayExpression<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_array_expression_impl(expr, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_object_expression<'a>(
    expr: &ObjectExpression<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_object_expression_impl(expr, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_object_property_kind<'a>(
    kind: &ObjectPropertyKind<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_object_property_kind_impl(kind, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_object_property<'a>(
    prop: &ObjectProperty<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_object_property_impl(prop, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_property_key<'a>(key: &PropertyKey<'a>, p: &mut Codegen, ctx: Context) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_property_key_impl(key, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_with_statement<'a>(
    stmt: &WithStatement<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_with_statement_impl(stmt, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_debugger_statement(stmt: &DebuggerStatement, p: &mut Codegen) {
    let p: &mut Codegen<'_> = unsafe { cast_codegen(p) };
    emit_debugger_statement_impl(stmt, p);
}

#[inline(always)]
pub(crate) unsafe fn emit_import_declaration<'a>(
    decl: &ImportDeclaration<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_import_declaration_impl(decl, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_with_clause<'a>(clause: &WithClause<'a>, p: &mut Codegen, ctx: Context) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_with_clause_impl(clause, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_import_attribute<'a>(
    attr: &ImportAttribute<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_import_attribute_impl(attr, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_export_named_declaration<'a>(
    decl: &ExportNamedDeclaration<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_export_named_declaration_impl(decl, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_export_all_declaration<'a>(
    decl: &ExportAllDeclaration<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_export_all_declaration_impl(decl, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_export_default_declaration<'a>(
    decl: &ExportDefaultDeclaration<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_export_default_declaration_impl(decl, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_export_default_declaration_kind<'a>(
    kind: &ExportDefaultDeclarationKind<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_export_default_declaration_kind_impl(kind, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_export_specifier<'a>(
    spec: &ExportSpecifier<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_export_specifier_impl(spec, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_module_export_name<'a>(
    name: &ModuleExportName<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_module_export_name_impl(name, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_export_assignment<'a>(
    decl: &TSExportAssignment<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_export_assignment_impl(decl, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_namespace_export_declaration<'a>(
    decl: &TSNamespaceExportDeclaration<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_namespace_export_declaration_impl(decl, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_import_equals_declaration<'a>(
    decl: &TSImportEqualsDeclaration<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_import_equals_declaration_impl(decl, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_import_equals_module_reference<'a>(
    reference: &TSModuleReference<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_import_equals_module_reference_impl(reference, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_decorator<'a>(decorator: &Decorator<'a>, p: &mut Codegen, ctx: Context) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_decorator_impl(decorator, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_class_implements<'a>(
    implements: &TSClassImplements<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_class_implements_impl(implements, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_type_parameter_declaration<'a>(
    decl: &TSTypeParameterDeclaration<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_type_parameter_declaration_impl(decl, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_type_annotation<'a>(
    annotation: &TSTypeAnnotation<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_type_annotation_impl(annotation, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_type<'a>(ty: &TSType<'a>, p: &mut Codegen, ctx: Context) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_type_impl(ty, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_array_type<'a>(ty: &TSArrayType<'a>, p: &mut Codegen, ctx: Context) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_array_type_impl(ty, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_tuple_type<'a>(ty: &TSTupleType<'a>, p: &mut Codegen, ctx: Context) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_tuple_type_impl(ty, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_union_type<'a>(ty: &TSUnionType<'a>, p: &mut Codegen, ctx: Context) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_union_type_impl(ty, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_parenthesized_type<'a>(
    ty: &TSParenthesizedType<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_parenthesized_type_impl(ty, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_intersection_type<'a>(
    ty: &TSIntersectionType<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_intersection_type_impl(ty, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_conditional_type<'a>(
    ty: &TSConditionalType<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_conditional_type_impl(ty, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_infer_type<'a>(ty: &TSInferType<'a>, p: &mut Codegen, ctx: Context) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_infer_type_impl(ty, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_indexed_access_type<'a>(
    ty: &TSIndexedAccessType<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_indexed_access_type_impl(ty, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_mapped_type<'a>(ty: &TSMappedType<'a>, p: &mut Codegen, ctx: Context) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_mapped_type_impl(ty, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_qualified_name<'a>(
    name: &TSQualifiedName<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_qualified_name_impl(name, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_type_operator<'a>(
    ty: &TSTypeOperator<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_type_operator_impl(ty, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_type_predicate<'a>(
    ty: &TSTypePredicate<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_type_predicate_impl(ty, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_type_reference<'a>(
    ty: &TSTypeReference<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_type_reference_impl(ty, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_type_literal<'a>(
    ty: &TSTypeLiteral<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_type_literal_impl(ty, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_type_name<'a>(ty: &TSTypeName<'a>, p: &mut Codegen, ctx: Context) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_type_name_impl(ty, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_jsdoc_nullable_type<'a>(
    ty: &JSDocNullableType<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_jsdoc_nullable_type_impl(ty, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_jsdoc_non_nullable_type<'a>(
    ty: &JSDocNonNullableType<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_jsdoc_non_nullable_type_impl(ty, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_template_literal_type<'a>(
    ty: &TSTemplateLiteralType<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_template_literal_type_impl(ty, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_signature<'a>(
    signature: &TSSignature<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_signature_impl(signature, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_literal<'a>(literal: &TSLiteral<'a>, p: &mut Codegen, ctx: Context) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_literal_impl(literal, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_type_parameter<'a>(
    param: &TSTypeParameter<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_type_parameter_impl(param, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_property_signature<'a>(
    signature: &TSPropertySignature<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_property_signature_impl(signature, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_type_query<'a>(
    query: &TSTypeQuery<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_type_query_impl(query, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_type_query_expr_name<'a>(
    name: &TSTypeQueryExprName<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_type_query_expr_name_impl(name, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_import_type<'a>(
    import: &TSImportType<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_import_type_impl(import, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_import_type_qualifier<'a>(
    qualifier: &TSImportTypeQualifier<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_import_type_qualifier_impl(qualifier, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_import_type_qualified_name<'a>(
    name: &TSImportTypeQualifiedName<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_import_type_qualified_name_impl(name, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_type_parameter_instantiation<'a>(
    instantiation: &TSTypeParameterInstantiation<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_type_parameter_instantiation_impl(instantiation, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_index_signature<'a>(
    signature: &TSIndexSignature<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_index_signature_impl(signature, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_tuple_element<'a>(
    element: &TSTupleElement<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_tuple_element_impl(element, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_named_tuple_member<'a>(
    member: &TSNamedTupleMember<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_named_tuple_member_impl(member, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_function_type<'a>(
    ty: &TSFunctionType<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_function_type_impl(ty, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_this_parameter<'a>(
    param: &TSThisParameter<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_this_parameter_impl(param, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_constructor_type<'a>(
    ty: &TSConstructorType<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_constructor_type_impl(ty, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_module_declaration<'a>(
    decl: &TSModuleDeclaration<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_module_declaration_impl(decl, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_module_block<'a>(
    block: &TSModuleBlock<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_module_block_impl(block, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_module_declaration_name<'a>(
    name: &TSModuleDeclarationName<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_module_declaration_name_impl(name, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_type_alias_declaration<'a>(
    decl: &TSTypeAliasDeclaration<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_type_alias_declaration_impl(decl, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_interface_declaration<'a>(
    decl: &TSInterfaceDeclaration<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_interface_declaration_impl(decl, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_interface_heritage<'a>(
    heritage: &TSInterfaceHeritage<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_interface_heritage_impl(heritage, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_enum_declaration<'a>(
    decl: &TSEnumDeclaration<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_enum_declaration_impl(decl, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_enum_body<'a>(body: &TSEnumBody<'a>, p: &mut Codegen, ctx: Context) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_enum_body_impl(body, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_ts_enum_member<'a>(
    member: &TSEnumMember<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_ts_enum_member_impl(member, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_labeled_statement<'a>(
    stmt: &LabeledStatement<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_labeled_statement_impl(stmt, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_try_statement<'a>(
    stmt: &TryStatement<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_try_statement_impl(stmt, p, ctx);
}

#[inline(always)]
pub(crate) unsafe fn emit_throw_statement<'a>(stmt: &ThrowStatement<'a>, p: &mut Codegen) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_throw_statement_impl(stmt, p);
}

#[inline(always)]
pub(crate) unsafe fn emit_variable_declaration_for_head<'a>(
    decl: &VariableDeclaration<'a>,
    p: &mut Codegen,
    ctx: Context,
) {
    let p: &mut Codegen<'a> = unsafe { cast_codegen(p) };
    emit_variable_declaration_inner(p, decl, ctx);
}

#[inline(always)]
fn emit_program_impl<'a>(program: &Program<'a>, p: &mut Codegen<'a>, ctx: Context) {
    p.is_jsx = program.source_type.is_jsx();

    p.print_comments_at(0);
    if let Some(hashbang) = &program.hashbang {
        emit_hashbang_impl(hashbang, p);
    }

    StatementEmitter { p, ctx }.emit_top_level(&program.directives, &program.body);
    p.print_semicolon_if_needed();
    p.print_comments_at(program.span.end);
}

#[inline(always)]
fn emit_hashbang_impl(hashbang: &Hashbang<'_>, p: &mut Codegen<'_>) {
    p.print_keyword(b"#!");
    p.print_str(hashbang.value.as_str());
    p.print_hard_newline();
}

#[inline(always)]
fn emit_directive_impl(directive: &Directive<'_>, p: &mut Codegen<'_>) {
    p.add_source_mapping(directive.span);
    p.print_indent();

    let directive_text = directive.directive.as_str();
    let mut bytes = directive_text.as_bytes().iter();
    let mut quote = p.quote;

    while let Some(&b) = bytes.next() {
        match b {
            b'"' => {
                quote = Quote::Single;
                break;
            }
            b'\'' => {
                quote = Quote::Double;
                break;
            }
            b'\\' => {
                let _ = bytes.next();
            }
            _ => {}
        }
    }

    quote.print(p);
    p.print_str(directive_text);
    quote.print(p);
    p.print_ascii_byte(b';');
    p.print_soft_newline();
}

fn emit_statement_impl<'a>(stmt: &Statement<'a>, p: &mut Codegen<'a>, ctx: Context) {
    StatementEmitter { p, ctx }.emit_statement(stmt);
}

#[inline(always)]
fn emit_expression_impl<'a>(
    expr: &Expression<'a>,
    p: &mut Codegen<'a>,
    precedence: Precedence,
    ctx: Context,
) {
    match expr {
        // Most common expressions first (identifiers, member access, calls)
        Expression::Identifier(ident) => emit_identifier_reference_impl(ident, p, ctx),
        Expression::StaticMemberExpression(member) => {
            emit_static_member_expression_impl(member, p, precedence, ctx);
        }
        Expression::ComputedMemberExpression(member) => {
            emit_computed_member_expression_impl(member, p, precedence, ctx);
        }
        Expression::CallExpression(call) => emit_call_expression_impl(call, p, precedence, ctx),
        // Literals (very common)
        Expression::NumericLiteral(lit) => emit_numeric_literal_impl(lit, p, precedence, ctx),
        Expression::StringLiteral(lit) => emit_string_literal_impl(lit, p, ctx),
        Expression::BooleanLiteral(lit) => emit_boolean_literal_impl(lit, p, ctx),
        Expression::NullLiteral(lit) => emit_null_literal_impl(lit, p, ctx),
        // Binary and logical operations (common)
        Expression::BinaryExpression(binary) => {
            emit_binary_expression_impl(binary, p, precedence, ctx);
        }
        Expression::LogicalExpression(logical) => {
            emit_logical_expression_impl(logical, p, precedence, ctx);
        }
        // Object and array literals (common)
        Expression::ObjectExpression(object) => emit_object_expression_impl(object, p, ctx),
        Expression::ArrayExpression(array) => emit_array_expression_impl(array, p, ctx),
        // Assignment and update (common)
        Expression::AssignmentExpression(assign) => {
            emit_assignment_expression_impl(assign, p, precedence, ctx);
        }
        Expression::UpdateExpression(update) => {
            emit_update_expression_impl(update, p, precedence, ctx);
        }
        Expression::UnaryExpression(unary) => emit_unary_expression_impl(unary, p, precedence, ctx),
        // Conditional and sequence
        Expression::ConditionalExpression(cond) => {
            emit_conditional_expression_impl(cond, p, precedence, ctx);
        }
        Expression::SequenceExpression(seq) => {
            emit_sequence_expression_impl(seq, p, precedence, ctx);
        }
        // Function expressions
        Expression::ArrowFunctionExpression(func) => {
            if func.pure && p.options.print_annotation_comment() {
                p.print_str(NO_SIDE_EFFECTS_COMMENT);
            }
            emit_arrow_function_expression_impl(func, p, precedence, ctx);
        }
        Expression::FunctionExpression(func) => {
            if func.pure && p.options.print_annotation_comment() {
                p.print_str(NO_SIDE_EFFECTS_COMMENT);
            }
            emit_function_impl(func, p, ctx);
        }
        // This and super
        Expression::ThisExpression(this_expr) => emit_this_expression_impl(this_expr, p, ctx),
        Expression::Super(super_expr) => emit_super_impl(super_expr, p, ctx),
        // New expression
        Expression::NewExpression(new_expr) => {
            emit_new_expression_impl(new_expr, p, precedence, ctx);
        }
        // Template literals
        Expression::TemplateLiteral(template) => emit_template_literal_impl(template, p),
        Expression::TaggedTemplateExpression(tagged) => {
            emit_tagged_template_expression_impl(tagged, p, ctx);
        }
        // Other literals
        Expression::RegExpLiteral(regex) => emit_reg_exp_literal_impl(regex, p, ctx),
        Expression::BigIntLiteral(bigint) => emit_bigint_literal_impl(bigint, p, precedence, ctx),
        // Class expression
        Expression::ClassExpression(class) => emit_class_impl(class, p, ctx),
        // Async/await and yield
        Expression::AwaitExpression(await_expr) => {
            emit_await_expression_impl(await_expr, p, precedence, ctx);
        }
        Expression::YieldExpression(yield_expr) => {
            emit_yield_expression_impl(yield_expr, p, precedence, ctx);
        }
        // Import expression
        Expression::ImportExpression(import_expr) => {
            emit_import_expression_impl(import_expr, p, precedence, ctx);
        }
        // Meta property
        Expression::MetaProperty(meta) => emit_meta_property_impl(meta, p, ctx),
        // Chain expression
        Expression::ChainExpression(chain) => emit_chain_expression_impl(chain, p, precedence, ctx),
        // Private field
        Expression::PrivateFieldExpression(private_field) => {
            emit_private_field_expression_impl(private_field, p, precedence, ctx);
        }
        Expression::PrivateInExpression(private_in) => {
            emit_private_in_expression_impl(private_in, p, precedence, ctx);
        }
        // Parenthesized
        Expression::ParenthesizedExpression(expr) => {
            emit_parenthesized_expression_impl(expr, p, precedence, ctx);
        }
        // JSX (less common in typical JS code)
        Expression::JSXElement(element) => emit_jsx_element_impl(element, p, ctx),
        Expression::JSXFragment(fragment) => emit_jsx_fragment_impl(fragment, p, ctx),
        // TypeScript (less common in runtime)
        Expression::TSAsExpression(ts_expr) => {
            emit_ts_as_expression_impl(ts_expr, p, precedence, ctx);
        }
        Expression::TSSatisfiesExpression(ts_expr) => {
            emit_ts_satisfies_expression_impl(ts_expr, p, precedence, ctx);
        }
        Expression::TSTypeAssertion(ts_expr) => {
            emit_ts_type_assertion_impl(ts_expr, p, precedence, ctx);
        }
        Expression::TSNonNullExpression(ts_expr) => {
            emit_ts_non_null_expression_impl(ts_expr, p, precedence, ctx);
        }
        Expression::TSInstantiationExpression(ts_expr) => {
            emit_ts_instantiation_expression_impl(ts_expr, p, precedence, ctx);
        }
        // V8 intrinsics (rare)
        Expression::V8IntrinsicExpression(intrinsic) => {
            emit_v8_intrinsic_expression_impl(intrinsic, p, precedence, ctx);
        }
    }
}

#[inline(always)]
fn emit_parenthesized_expression_impl<'a>(
    expr: &ParenthesizedExpression<'a>,
    p: &mut Codegen<'a>,
    precedence: Precedence,
    ctx: Context,
) {
    emit_expression_impl(&expr.expression, p, precedence, ctx);
}

#[inline(always)]
fn emit_identifier_reference_impl<'a>(
    ident: &IdentifierReference<'a>,
    p: &mut Codegen<'a>,
    _ctx: Context,
) {
    let name = p.get_identifier_reference_name(ident);
    p.print_space_before_identifier();
    p.add_source_mapping_for_name(ident.span, name);
    p.print_str(name);
}

#[inline(always)]
fn emit_identifier_name_impl<'a>(ident: &IdentifierName<'a>, p: &mut Codegen<'a>, _ctx: Context) {
    p.print_space_before_identifier();
    p.add_source_mapping_for_name(ident.span, &ident.name);
    p.print_str(ident.name.as_str());
}

#[inline(always)]
fn emit_binding_identifier_impl<'a>(
    ident: &BindingIdentifier<'a>,
    p: &mut Codegen<'a>,
    _ctx: Context,
) {
    let name = p.get_binding_identifier_name(ident);
    p.print_space_before_identifier();
    p.add_source_mapping_for_name(ident.span, name);
    p.print_str(name);
}

#[inline(always)]
fn emit_label_identifier_impl<'a>(ident: &LabelIdentifier<'a>, p: &mut Codegen<'a>, _ctx: Context) {
    p.print_space_before_identifier();
    p.add_source_mapping_for_name(ident.span, &ident.name);
    p.print_str(ident.name.as_str());
}

#[inline(always)]
fn emit_boolean_literal_impl(lit: &BooleanLiteral, p: &mut Codegen, _ctx: Context) {
    p.add_source_mapping(lit.span);
    p.print_space_before_identifier();
    p.print_str(lit.as_str());
}

#[inline(always)]
fn emit_null_literal_impl(lit: &NullLiteral, p: &mut Codegen, _ctx: Context) {
    p.print_space_before_identifier();
    p.add_source_mapping(lit.span);
    p.print_keyword(b"null");
}

#[inline(always)]
fn emit_numeric_literal_impl<'a>(
    lit: &NumericLiteral<'a>,
    p: &mut Codegen<'a>,
    precedence: Precedence,
    ctx: Context,
) {
    p.add_source_mapping(lit.span);
    let value = lit.value;
    if ctx.contains(Context::TYPESCRIPT) {
        p.print_str(&lit.raw_str());
    } else if value.is_nan() {
        p.print_space_before_identifier();
        p.print_keyword(b"NaN");
    } else if value.is_infinite() {
        let wrap = (p.options.minify && precedence >= Precedence::Multiply)
            || (value.is_sign_negative() && precedence >= Precedence::Prefix);
        p.wrap(wrap, |p| {
            if value.is_sign_negative() {
                p.print_space_before_operator(Operator::Unary(UnaryOperator::UnaryNegation));
                p.print_ascii_byte(b'-');
            } else {
                p.print_space_before_identifier();
            }
            if p.options.minify {
                p.print_keyword(b"1/0");
            } else {
                p.print_keyword(b"Infinity");
            }
        });
    } else if value.is_sign_positive() {
        p.print_space_before_identifier();
        p.print_non_negative_float(value);
    } else if precedence >= Precedence::Prefix {
        p.print_keyword(b"(-");
        p.print_non_negative_float(value.abs());
        p.print_ascii_byte(b')');
    } else {
        p.print_space_before_operator(Operator::Unary(UnaryOperator::UnaryNegation));
        p.print_ascii_byte(b'-');
        p.print_non_negative_float(value.abs());
    }
}

#[inline(always)]
fn emit_bigint_literal_impl<'a>(
    lit: &BigIntLiteral<'a>,
    p: &mut Codegen<'a>,
    precedence: Precedence,
    _ctx: Context,
) {
    p.print_space_before_identifier();
    p.add_source_mapping(lit.span);
    let value = lit.value.as_str();
    if value.starts_with('-') && precedence >= Precedence::Prefix {
        p.print_ascii_byte(b'(');
        p.print_str(value);
        p.print_keyword(b"n)");
    } else {
        p.print_str(value);
        p.print_ascii_byte(b'n');
    }
}

#[inline(always)]
fn emit_reg_exp_literal_impl<'a>(lit: &RegExpLiteral<'a>, p: &mut Codegen<'a>, _ctx: Context) {
    p.add_source_mapping(lit.span);
    let last = p.last_byte();
    if last == Some(b'/')
        || (last == Some(b'<')
            && lit
                .regex
                .pattern
                .text
                .get(..6)
                .is_some_and(|first_six| first_six.cow_to_ascii_lowercase() == "script"))
    {
        p.print_hard_space();
    }
    p.print_ascii_byte(b'/');
    p.print_str(lit.regex.pattern.text.as_str());
    p.print_ascii_byte(b'/');
    p.print_str(lit.regex.flags.to_inline_string().as_str());
    p.prev_reg_exp_end = p.code().len();
}

#[inline(always)]
fn emit_string_literal_impl<'a>(lit: &StringLiteral<'a>, p: &mut Codegen<'a>, _ctx: Context) {
    p.print_string_literal(lit, true);
}

#[inline(always)]
fn emit_this_expression_impl(expr: &ThisExpression, p: &mut Codegen<'_>, _ctx: Context) {
    p.print_space_before_identifier();
    p.add_source_mapping(expr.span);
    p.print_keyword(b"this");
}

#[inline(always)]
fn emit_super_impl(expr: &Super, p: &mut Codegen<'_>, _ctx: Context) {
    p.print_space_before_identifier();
    p.add_source_mapping(expr.span);
    p.print_keyword(b"super");
}

#[inline(always)]
fn emit_computed_member_expression_impl<'a>(
    expr: &ComputedMemberExpression<'a>,
    p: &mut Codegen<'a>,
    _precedence: Precedence,
    ctx: Context,
) {
    let wrap = expr.object.get_identifier_reference().is_some_and(|r| r.name == "let");
    p.wrap(wrap, |p| {
        expr.object.print_expr(p, Precedence::Postfix, ctx.intersection(Context::FORBID_CALL));
    });
    if expr.optional {
        p.print_keyword(b"?.");
    }
    p.print_ascii_byte(b'[');
    expr.expression.print_expr(p, Precedence::Lowest, Context::empty());
    p.print_ascii_byte(b']');
}

#[inline(always)]
fn emit_static_member_expression_impl<'a>(
    expr: &StaticMemberExpression<'a>,
    p: &mut Codegen<'a>,
    _precedence: Precedence,
    ctx: Context,
) {
    expr.object.print_expr(p, Precedence::Postfix, ctx.intersection(Context::FORBID_CALL));
    if expr.optional {
        p.print_ascii_byte(b'?');
    } else if p.need_space_before_dot == p.code_len() {
        p.print_hard_space();
    }
    p.print_ascii_byte(b'.');
    expr.property.print(p, ctx);
}

#[inline(always)]
fn emit_private_field_expression_impl<'a>(
    expr: &PrivateFieldExpression<'a>,
    p: &mut Codegen<'a>,
    _precedence: Precedence,
    ctx: Context,
) {
    expr.object.print_expr(p, Precedence::Postfix, ctx.intersection(Context::FORBID_CALL));
    if expr.optional {
        p.print_keyword(b"?");
    }
    p.print_ascii_byte(b'.');
    expr.field.print(p, ctx);
}

#[inline(always)]
fn emit_yield_expression_impl<'a>(
    expr: &YieldExpression<'a>,
    p: &mut Codegen<'a>,
    precedence: Precedence,
    _ctx: Context,
) {
    p.wrap(precedence >= Precedence::Assign, |p| {
        p.print_space_before_identifier();
        p.add_source_mapping(expr.span);
        p.print_keyword(b"yield");
        if expr.delegate {
            p.print_ascii_byte(b'*');
        }
        if let Some(argument) = expr.argument.as_ref() {
            p.print_soft_space();
            argument.print_expr(p, Precedence::Yield, Context::empty());
        }
    });
}

#[inline(always)]
fn emit_update_expression_impl<'a>(
    expr: &UpdateExpression<'a>,
    p: &mut Codegen<'a>,
    precedence: Precedence,
    ctx: Context,
) {
    let operator = expr.operator.as_str();
    p.wrap(precedence >= expr.precedence(), |p| {
        if expr.prefix {
            p.print_space_before_operator(expr.operator.into());
            p.add_source_mapping(expr.span);
            p.print_str(operator);
            p.prev_op = Some(expr.operator.into());
            p.prev_op_end = p.code().len();
            expr.argument.print_expr(p, Precedence::Prefix, ctx);
        } else {
            p.print_space_before_operator(expr.operator.into());
            p.add_source_mapping(expr.span);
            expr.argument.print_expr(p, Precedence::Postfix, ctx);
            p.print_str(operator);
            p.prev_op = Some(expr.operator.into());
            p.prev_op_end = p.code().len();
        }
    });
}

#[inline(always)]
fn emit_unary_expression_impl<'a>(
    expr: &UnaryExpression<'a>,
    p: &mut Codegen<'a>,
    precedence: Precedence,
    ctx: Context,
) {
    p.wrap(precedence >= expr.precedence(), |p| {
        let operator = expr.operator.as_str();
        if expr.operator.is_keyword() {
            p.print_space_before_identifier();
            p.add_source_mapping(expr.span);
            p.print_str(operator);
            p.print_soft_space();
        } else {
            p.print_space_before_operator(expr.operator.into());
            p.add_source_mapping(expr.span);
            p.print_str(operator);
            p.prev_op = Some(expr.operator.into());
            p.prev_op_end = p.code().len();
        }
        let is_delete_infinity = expr.operator == UnaryOperator::Delete
            && !p.options.minify
            && matches!(&expr.argument, Expression::NumericLiteral(lit) if lit.value.is_sign_positive() && lit.value.is_infinite());
        if is_delete_infinity {
            p.print_keyword(b"(0,");
            p.print_soft_space();
        }
        expr.argument
            .print_expr(p, Precedence::Exponentiation, ctx);
        if is_delete_infinity {
            p.print_ascii_byte(b')');
        }
    });
}

#[inline(always)]
fn emit_binary_expression_impl<'a>(
    expr: &BinaryExpression<'a>,
    p: &mut Codegen<'a>,
    precedence: Precedence,
    ctx: Context,
) {
    let visitor = BinaryExpressionVisitor {
        // SAFETY: the expression lives for 'a and the visitor consumes it immediately.
        e: Binaryish::Binary(unsafe {
            std::mem::transmute::<&BinaryExpression<'_>, &BinaryExpression<'_>>(expr)
        }),
        precedence,
        ctx,
        left_precedence: Precedence::Lowest,
        operator: BinaryishOperator::Binary(expr.operator),
        wrap: false,
        right_precedence: Precedence::Lowest,
    };
    BinaryExpressionVisitor::gen_expr(visitor, p);
}

#[inline(always)]
fn emit_private_in_expression_impl<'a>(
    expr: &PrivateInExpression<'a>,
    p: &mut Codegen<'a>,
    precedence: Precedence,
    ctx: Context,
) {
    p.wrap(precedence >= Precedence::Compare, |p| {
        p.add_source_mapping(expr.span);
        expr.left.print(p, ctx);
        p.print_keyword(b" in ");
        expr.right.print_expr(p, Precedence::Equals, Context::FORBID_IN);
    });
}

#[inline(always)]
fn emit_logical_expression_impl<'a>(
    expr: &LogicalExpression<'a>,
    p: &mut Codegen<'a>,
    precedence: Precedence,
    ctx: Context,
) {
    let visitor = BinaryExpressionVisitor {
        e: Binaryish::Logical(unsafe {
            std::mem::transmute::<&LogicalExpression<'_>, &LogicalExpression<'_>>(expr)
        }),
        precedence,
        ctx,
        left_precedence: Precedence::Lowest,
        operator: BinaryishOperator::Logical(expr.operator),
        wrap: false,
        right_precedence: Precedence::Lowest,
    };
    BinaryExpressionVisitor::gen_expr(visitor, p);
}

#[inline(always)]
fn emit_conditional_expression_impl<'a>(
    expr: &ConditionalExpression<'a>,
    p: &mut Codegen<'a>,
    precedence: Precedence,
    mut ctx: Context,
) {
    let wrap = precedence >= expr.precedence();
    if wrap {
        ctx &= Context::FORBID_IN.not();
    }
    p.wrap(wrap, |p| {
        expr.test.print_expr(p, Precedence::Conditional, ctx & Context::FORBID_IN);
        p.print_soft_space();
        p.print_ascii_byte(b'?');
        p.print_soft_space();
        expr.consequent.print_expr(p, Precedence::Yield, Context::empty());
        p.print_soft_space();
        p.print_colon();
        p.print_soft_space();
        expr.alternate.print_expr(p, Precedence::Yield, ctx & Context::FORBID_IN);
    });
}

#[inline(always)]
fn emit_assignment_expression_impl<'a>(
    expr: &AssignmentExpression<'a>,
    p: &mut Codegen<'a>,
    precedence: Precedence,
    ctx: Context,
) {
    let n = p.code_len();
    let wrap = (p.start_of_stmt == n || p.start_of_arrow_expr == n)
        && matches!(expr.left, AssignmentTarget::ObjectAssignmentTarget(_));
    p.wrap(wrap || precedence >= expr.precedence(), |p| {
        p.add_source_mapping(expr.span);
        expr.left.print(p, ctx);
        p.print_soft_space();
        p.print_str(expr.operator.as_str());
        p.print_soft_space();
        expr.right.print_expr(p, Precedence::Comma, ctx);
    });
}

#[inline(always)]
fn emit_assignment_target_impl<'a>(
    target: &AssignmentTarget<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    match target {
        match_simple_assignment_target!(AssignmentTarget) => {
            target.to_simple_assignment_target().print_expr(p, Precedence::Comma, Context::empty());
        }
        match_assignment_target_pattern!(AssignmentTarget) => {
            target.to_assignment_target_pattern().print(p, ctx);
        }
    }
}

#[inline(always)]
fn emit_assignment_target_pattern_impl<'a>(
    pattern: &AssignmentTargetPattern<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    match pattern {
        AssignmentTargetPattern::ArrayAssignmentTarget(target) => {
            emit_array_assignment_target_impl(target, p, ctx);
        }
        AssignmentTargetPattern::ObjectAssignmentTarget(target) => {
            emit_object_assignment_target_impl(target, p, ctx);
        }
    }
}

#[inline(always)]
fn emit_array_assignment_target_impl<'a>(
    target: &ArrayAssignmentTarget<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    p.add_source_mapping(target.span);
    p.print_ascii_byte(b'[');
    for (index, element) in target.elements.iter().enumerate() {
        if index != 0 {
            p.print_comma();
            p.print_soft_space();
        }
        if let Some(item) = element {
            item.print(p, ctx);
        }
        if index + 1 == target.elements.len() && (element.is_none() || target.rest.is_some()) {
            p.print_comma();
        }
    }
    if let Some(rest) = &target.rest {
        if !target.elements.is_empty() {
            p.print_soft_space();
        }
        rest.print(p, ctx);
    }
    p.print_ascii_byte(b']');
}

#[inline(always)]
fn emit_object_assignment_target_impl<'a>(
    target: &ObjectAssignmentTarget<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    p.add_source_mapping(target.span);
    p.print_ascii_byte(b'{');
    p.print_list(&target.properties, ctx);
    if let Some(rest) = &target.rest {
        if !target.properties.is_empty() {
            p.print_comma();
            p.print_soft_space();
        }
        rest.print(p, ctx);
    }
    p.print_ascii_byte(b'}');
}

#[inline(always)]
fn emit_assignment_target_maybe_default_impl<'a>(
    target: &AssignmentTargetMaybeDefault<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    match target {
        match_assignment_target!(AssignmentTargetMaybeDefault) => {
            target.to_assignment_target().print(p, ctx);
        }
        AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(with_default) => {
            emit_assignment_target_with_default_impl(with_default, p, ctx);
        }
    }
}

#[inline(always)]
fn emit_assignment_target_with_default_impl<'a>(
    target: &AssignmentTargetWithDefault<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    target.binding.print(p, ctx);
    p.print_soft_space();
    p.print_equal();
    p.print_soft_space();
    target.init.print_expr(p, Precedence::Comma, Context::empty());
}

#[inline(always)]
fn emit_assignment_target_property_impl<'a>(
    prop: &AssignmentTargetProperty<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    match prop {
        AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(identifier) => {
            emit_assignment_target_property_identifier_impl(identifier, p, ctx);
        }
        AssignmentTargetProperty::AssignmentTargetPropertyProperty(property) => {
            emit_assignment_target_property_property_impl(property, p, ctx);
        }
    }
}

#[inline(always)]
fn emit_assignment_target_property_identifier_impl<'a>(
    prop: &AssignmentTargetPropertyIdentifier<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    let ident_name = p.get_identifier_reference_name(&prop.binding);
    if ident_name == prop.binding.name.as_str() {
        prop.binding.print(p, ctx);
    } else {
        p.print_str(prop.binding.name.as_str());
        p.print_colon();
        p.print_soft_space();
        p.print_str(ident_name);
    }
    if let Some(expr) = &prop.init {
        p.print_soft_space();
        p.print_equal();
        p.print_soft_space();
        expr.print_expr(p, Precedence::Comma, Context::empty());
    }
}

#[inline(always)]
fn emit_assignment_target_property_property_impl<'a>(
    prop: &AssignmentTargetPropertyProperty<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    let omit_key = if p.options.minify {
        let key_name = match &prop.name {
            PropertyKey::StaticIdentifier(ident) => Some(&ident.name),
            _ => None,
        };
        let value_name = prop.binding.identifier().map(|id| p.get_identifier_reference_name(id));
        matches!((key_name, value_name), (Some(key), Some(value)) if key == value)
    } else {
        false
    };
    if !omit_key {
        match &prop.name {
            PropertyKey::StaticIdentifier(ident) => {
                ident.print(p, ctx);
            }
            PropertyKey::PrivateIdentifier(ident) => {
                ident.print(p, ctx);
            }
            PropertyKey::StringLiteral(literal) => {
                if prop.computed {
                    p.print_ascii_byte(b'[');
                }
                p.print_string_literal(literal, false);
                if prop.computed {
                    p.print_ascii_byte(b']');
                }
            }
            key => {
                if prop.computed {
                    p.print_ascii_byte(b'[');
                }
                key.to_expression().print_expr(p, Precedence::Comma, Context::empty());
                if prop.computed {
                    p.print_ascii_byte(b']');
                }
            }
        }
        p.print_colon();
        p.print_soft_space();
    }
    prop.binding.print(p, ctx);
}

#[inline(always)]
fn emit_assignment_target_rest_impl<'a>(
    rest: &AssignmentTargetRest<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    p.print_ellipsis();
    rest.target.print(p, ctx);
}

#[inline(always)]
fn emit_assignment_pattern_impl<'a>(
    pattern: &AssignmentPattern<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    pattern.left.print(p, ctx);
    p.print_soft_space();
    p.print_equal();
    p.print_soft_space();
    pattern.right.print_expr(p, Precedence::Comma, Context::empty());
}

#[inline(always)]
fn emit_simple_assignment_target_impl<'a>(
    target: &SimpleAssignmentTarget<'a>,
    p: &mut Codegen<'a>,
    precedence: Precedence,
    ctx: Context,
) {
    match target {
        SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) => ident.print(p, ctx),
        SimpleAssignmentTarget::ComputedMemberExpression(expr) => {
            expr.print_expr(p, precedence, ctx);
        }
        SimpleAssignmentTarget::StaticMemberExpression(expr) => expr.print_expr(p, precedence, ctx),
        SimpleAssignmentTarget::PrivateFieldExpression(expr) => expr.print_expr(p, precedence, ctx),
        SimpleAssignmentTarget::TSAsExpression(expr) => expr.print_expr(p, precedence, ctx),
        SimpleAssignmentTarget::TSSatisfiesExpression(expr) => expr.print_expr(p, precedence, ctx),
        SimpleAssignmentTarget::TSNonNullExpression(expr) => expr.print_expr(p, precedence, ctx),
        SimpleAssignmentTarget::TSTypeAssertion(expr) => expr.print_expr(p, precedence, ctx),
    }
}

#[inline(always)]
fn emit_sequence_expression_impl<'a>(
    expr: &SequenceExpression<'a>,
    p: &mut Codegen<'a>,
    precedence: Precedence,
    ctx: Context,
) {
    p.wrap(precedence >= expr.precedence(), |p| {
        p.print_expressions(&expr.expressions, Precedence::Lowest, ctx.and_forbid_call(false));
    });
}

#[inline(always)]
fn emit_import_expression_impl<'a>(
    expr: &ImportExpression<'a>,
    p: &mut Codegen<'a>,
    precedence: Precedence,
    ctx: Context,
) {
    let wrap = precedence >= Precedence::New || ctx.intersects(Context::FORBID_CALL);
    let has_comment_before_right_paren = p.options.print_annotation_comment()
        && expr.span.end > 0
        && p.has_comment(expr.span.end - 1);
    let has_comment = p.options.print_annotation_comment()
        && (has_comment_before_right_paren
            || p.has_comment(expr.source.span().start)
            || expr.options.as_ref().is_some_and(|options| p.has_comment(options.span().start)));

    p.wrap(wrap, |p| {
        p.print_space_before_identifier();
        p.add_source_mapping(expr.span);
        p.print_keyword(b"import");
        if let Some(phase) = expr.phase {
            p.print_ascii_byte(b'.');
            p.print_str(phase.as_str());
        }
        p.print_ascii_byte(b'(');
        if has_comment {
            p.indent();
        }
        if p.print_expr_comments(expr.source.span().start) {
            p.print_indent();
        } else if has_comment {
            p.print_soft_newline();
            p.print_indent();
        }
        expr.source.print_expr(p, Precedence::Comma, Context::empty());
        if let Some(options) = &expr.options {
            p.print_comma();
            if has_comment {
                p.print_soft_newline();
                p.print_indent();
            } else {
                p.print_soft_space();
            }
            options.gen_expr(p, Precedence::Comma, Context::empty());
        }
        if has_comment {
            if !has_comment_before_right_paren || !p.print_expr_comments(expr.span.end - 1) {
                p.print_soft_newline();
            }
            p.dedent();
        }
        p.print_ascii_byte(b')');
    });
}

#[inline(always)]
fn emit_await_expression_impl<'a>(
    expr: &AwaitExpression<'a>,
    p: &mut Codegen<'a>,
    precedence: Precedence,
    ctx: Context,
) {
    p.wrap(precedence >= expr.precedence(), |p| {
        p.print_space_before_identifier();
        p.add_source_mapping(expr.span);
        p.print_keyword(b"await");
        p.print_soft_space();
        expr.argument.print_expr(p, Precedence::Exponentiation, ctx);
    });
}

#[inline(always)]
fn emit_chain_expression_impl<'a>(
    expr: &ChainExpression<'a>,
    p: &mut Codegen<'a>,
    precedence: Precedence,
    ctx: Context,
) {
    p.wrap(precedence >= Precedence::Postfix, |p| match &expr.expression {
        ChainElement::CallExpression(call) => call.print_expr(p, precedence, ctx),
        ChainElement::TSNonNullExpression(ts) => ts.print_expr(p, precedence, ctx),
        ChainElement::ComputedMemberExpression(member) => member.print_expr(p, precedence, ctx),
        ChainElement::StaticMemberExpression(member) => member.print_expr(p, precedence, ctx),
        ChainElement::PrivateFieldExpression(member) => member.print_expr(p, precedence, ctx),
    });
}

#[inline(always)]
fn emit_new_expression_impl<'a>(
    expr: &NewExpression<'a>,
    p: &mut Codegen<'a>,
    precedence: Precedence,
    ctx: Context,
) {
    let mut wrap = precedence >= expr.precedence();
    let pure = expr.pure && p.options.print_annotation_comment();
    if precedence >= Precedence::Postfix && pure {
        wrap = true;
    }
    p.wrap(wrap, |p| {
        if pure {
            p.print_str(PURE_COMMENT);
        }
        p.print_space_before_identifier();
        p.add_source_mapping(expr.span);
        p.print_keyword(b"new");
        p.print_soft_space();
        expr.callee.print_expr(p, Precedence::New, Context::FORBID_CALL);
        if !p.options.minify || !expr.arguments.is_empty() || precedence >= Precedence::Postfix {
            p.print_arguments(expr.span, &expr.arguments, ctx);
        }
    });
}

#[inline(always)]
fn emit_meta_property_impl<'a>(meta: &MetaProperty<'a>, p: &mut Codegen<'a>, ctx: Context) {
    p.print_space_before_identifier();
    p.add_source_mapping(meta.span);
    meta.meta.print(p, ctx);
    p.print_ascii_byte(b'.');
    meta.property.print(p, ctx);
}

#[inline(always)]
fn emit_ts_as_expression_impl<'a>(
    expr: &TSAsExpression<'a>,
    p: &mut Codegen<'a>,
    precedence: Precedence,
    ctx: Context,
) {
    let wrap = precedence >= Precedence::Shift;
    p.wrap(wrap, |p| {
        expr.expression.print_expr(p, Precedence::Exponentiation, ctx);
        p.print_keyword(b" as ");
        expr.type_annotation.print(p, ctx);
    });
}

#[inline(always)]
fn emit_ts_satisfies_expression_impl<'a>(
    expr: &TSSatisfiesExpression<'a>,
    p: &mut Codegen<'a>,
    precedence: Precedence,
    ctx: Context,
) {
    p.print_ascii_byte(b'(');
    let should_wrap =
        if let Expression::FunctionExpression(func) = &expr.expression.without_parentheses() {
            !func.pife
        } else {
            true
        };
    p.wrap(should_wrap, |p| {
        expr.expression.print_expr(p, precedence, Context::default());
    });
    p.print_keyword(b" satisfies ");
    expr.type_annotation.print(p, ctx);
    p.print_ascii_byte(b')');
}

#[inline(always)]
fn emit_ts_non_null_expression_impl<'a>(
    expr: &TSNonNullExpression<'a>,
    p: &mut Codegen<'a>,
    precedence: Precedence,
    ctx: Context,
) {
    p.wrap(matches!(expr.expression, Expression::ParenthesizedExpression(_)), |p| {
        expr.expression.print_expr(p, precedence, ctx);
    });
    p.print_ascii_byte(b'!');
    if p.options.minify {
        p.print_hard_space();
    }
}

#[inline(always)]
fn emit_ts_instantiation_expression_impl<'a>(
    expr: &TSInstantiationExpression<'a>,
    p: &mut Codegen<'a>,
    precedence: Precedence,
    ctx: Context,
) {
    expr.expression.print_expr(p, precedence, ctx);
    expr.type_arguments.print(p, ctx);
    if p.options.minify {
        p.print_hard_space();
    }
}

#[inline(always)]
fn emit_ts_type_assertion_impl<'a>(
    expr: &TSTypeAssertion<'a>,
    p: &mut Codegen<'a>,
    precedence: Precedence,
    ctx: Context,
) {
    p.wrap(precedence >= expr.precedence(), |p| {
        p.print_keyword(b"<");
        if matches!(expr.type_annotation, TSType::TSFunctionType(_)) {
            p.print_hard_space();
        }
        expr.type_annotation.print(p, ctx);
        p.print_keyword(b">");
        expr.expression.print_expr(p, Precedence::Member, ctx);
    });
}

#[inline(always)]
fn emit_v8_intrinsic_expression_impl<'a>(
    expr: &V8IntrinsicExpression<'a>,
    p: &mut Codegen<'a>,
    precedence: Precedence,
    ctx: Context,
) {
    let is_statement = p.start_of_stmt == p.code_len();
    let is_export_default = p.start_of_default_export == p.code_len();
    let mut wrap = precedence >= Precedence::New || ctx.intersects(Context::FORBID_CALL);
    if precedence >= Precedence::Postfix {
        wrap = true;
    }
    p.wrap(wrap, |p| {
        if is_export_default {
            p.start_of_default_export = p.code_len();
        } else if is_statement {
            p.start_of_stmt = p.code_len();
        }
        p.add_source_mapping(expr.span);
        p.print_ascii_byte(b'%');
        expr.name.print(p, Context::empty());
        p.print_arguments(expr.span, &expr.arguments, ctx);
    });
}

#[inline(always)]
fn emit_call_expression_impl<'a>(
    expr: &CallExpression<'a>,
    p: &mut Codegen<'a>,
    precedence: Precedence,
    ctx: Context,
) {
    let is_statement = p.start_of_stmt == p.code_len();
    let is_export_default = p.start_of_default_export == p.code_len();
    let mut wrap = precedence >= Precedence::New || ctx.intersects(Context::FORBID_CALL);
    let pure = expr.pure && p.options.print_annotation_comment();
    if !wrap && pure && precedence >= Precedence::Postfix {
        wrap = true;
    }

    p.wrap(wrap, |p| {
        if pure {
            p.add_source_mapping(expr.span);
            p.print_str(PURE_COMMENT);
        }
        if is_export_default {
            p.start_of_default_export = p.code_len();
        } else if is_statement {
            p.start_of_stmt = p.code_len();
        }
        expr.callee.print_expr(p, Precedence::Postfix, Context::empty());
        if expr.optional {
            p.print_keyword(b"?.");
        }
        if let Some(type_parameters) = &expr.type_arguments {
            type_parameters.print(p, ctx);
        }
        p.print_arguments(expr.span, &expr.arguments, ctx);
    });
}

#[inline(always)]
fn emit_argument_impl<'a>(arg: &Argument<'a>, p: &mut Codegen<'a>, _ctx: Context) {
    match arg {
        Argument::SpreadElement(elem) => emit_spread_element_impl(elem, p, Context::empty()),
        _ => arg.to_expression().print_expr(p, Precedence::Comma, Context::empty()),
    }
}

#[inline(always)]
fn emit_array_expression_element_impl<'a>(
    elem: &ArrayExpressionElement<'a>,
    p: &mut Codegen<'a>,
    _ctx: Context,
) {
    match elem {
        ArrayExpressionElement::SpreadElement(elem) => {
            emit_spread_element_impl(elem, p, Context::empty());
        }
        ArrayExpressionElement::Elision(_) => {}
        _ => elem.to_expression().print_expr(p, Precedence::Comma, Context::empty()),
    }
}

#[inline(always)]
fn emit_spread_element_impl<'a>(elem: &SpreadElement<'a>, p: &mut Codegen<'a>, _ctx: Context) {
    p.add_source_mapping(elem.span);
    p.print_ellipsis();
    elem.argument.print_expr(p, Precedence::Comma, Context::empty());
}

#[inline(always)]
fn emit_array_expression_impl<'a>(expr: &ArrayExpression<'a>, p: &mut Codegen<'a>, ctx: Context) {
    let is_multi_line = expr.elements.len() > 2;
    p.add_source_mapping(expr.span);
    p.print_ascii_byte(b'[');
    if is_multi_line {
        p.indent();
    }
    for (i, item) in expr.elements.iter().enumerate() {
        if i != 0 {
            p.print_comma();
        }
        if is_multi_line {
            p.print_soft_newline();
            p.print_indent();
        } else if i != 0 {
            p.print_soft_space();
        }
        emit_array_expression_element_impl(item, p, ctx);
        if i == expr.elements.len() - 1 && matches!(item, ArrayExpressionElement::Elision(_)) {
            p.print_comma();
        }
    }
    if is_multi_line {
        p.print_soft_newline();
        p.dedent();
        p.print_indent();
    }
    p.print_ascii_byte(b']');
    p.add_source_mapping_end(expr.span);
}

#[inline(always)]
fn emit_object_expression_impl<'a>(expr: &ObjectExpression<'a>, p: &mut Codegen<'a>, ctx: Context) {
    let n = p.code_len();
    let len = expr.properties.len();
    let is_multi_line = len > 1;
    let has_comment = p.has_comment(expr.span.start);
    let wrap = has_comment || p.start_of_stmt == n || p.start_of_arrow_expr == n;
    p.wrap(wrap, |p| {
        if has_comment {
            p.print_leading_comments(expr.span.start);
            p.print_indent();
        }
        p.add_source_mapping(expr.span);
        p.print_ascii_byte(b'{');
        if is_multi_line {
            p.indent();
        }
        for (i, item) in expr.properties.iter().enumerate() {
            if i != 0 {
                p.print_comma();
            }
            if is_multi_line {
                p.print_soft_newline();
                p.print_indent();
            } else {
                p.print_soft_space();
            }
            emit_object_property_kind_impl(item, p, ctx);
        }
        if is_multi_line {
            p.print_soft_newline();
            p.dedent();
            p.print_indent();
        } else if len > 0 {
            p.print_soft_space();
        }
        p.print_ascii_byte(b'}');
        p.add_source_mapping_end(expr.span);
    });
}

#[inline(always)]
fn emit_object_property_kind_impl<'a>(
    kind: &ObjectPropertyKind<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    match kind {
        ObjectPropertyKind::ObjectProperty(prop) => emit_object_property_impl(prop, p, ctx),
        ObjectPropertyKind::SpreadProperty(elem) => emit_spread_element_impl(elem, p, ctx),
    }
}

#[inline(always)]
fn emit_object_property_impl<'a>(prop: &ObjectProperty<'a>, p: &mut Codegen<'a>, ctx: Context) {
    if let Expression::FunctionExpression(func) = &prop.value {
        p.add_source_mapping(prop.span);
        let is_accessor = match &prop.kind {
            PropertyKind::Init => false,
            PropertyKind::Get => {
                p.print_keyword(b"get");
                p.print_soft_space();
                true
            }
            PropertyKind::Set => {
                p.print_keyword(b"set");
                p.print_soft_space();
                true
            }
        };
        if prop.method || is_accessor {
            if func.r#async {
                p.print_space_before_identifier();
                p.print_keyword(b"async");
                p.print_soft_space();
            }
            if func.generator {
                p.print_keyword(b"*");
            }
            if prop.computed {
                p.print_ascii_byte(b'[');
            }
            prop.key.print(p, ctx);
            if prop.computed {
                p.print_ascii_byte(b']');
            }
            if let Some(type_parameters) = &func.type_parameters {
                type_parameters.print(p, ctx);
            }
            p.print_ascii_byte(b'(');
            func.params.print(p, ctx);
            p.print_ascii_byte(b')');
            if let Some(return_type) = &func.return_type {
                p.print_colon();
                p.print_soft_space();
                return_type.print(p, ctx);
            }
            if let Some(body) = &func.body {
                p.print_soft_space();
                body.print(p, ctx);
            }
            return;
        }
    }

    let mut shorthand = false;
    if let PropertyKey::StaticIdentifier(key) = &prop.key {
        if key.name == "__proto__" {
            shorthand = prop.shorthand;
        } else if let Expression::Identifier(ident) = prop.value.without_parentheses()
            && key.name == p.get_identifier_reference_name(ident)
        {
            shorthand = true;
        }
    }

    let mut computed = prop.computed;
    if !computed
        && let Some(Expression::NumericLiteral(n)) = prop.key.as_expression()
        && (n.value.is_sign_negative() || n.value.is_infinite())
    {
        computed = true;
    }

    if !shorthand {
        if computed {
            p.print_ascii_byte(b'[');
        }
        prop.key.print(p, ctx);
        if computed {
            p.print_ascii_byte(b']');
        }
        p.print_colon();
        p.print_soft_space();
    }

    prop.value.print_expr(p, Precedence::Comma, Context::empty());
}

#[inline(always)]
fn emit_property_key_impl<'a>(key: &PropertyKey<'a>, p: &mut Codegen<'a>, ctx: Context) {
    match key {
        PropertyKey::StaticIdentifier(ident) => ident.print(p, ctx),
        PropertyKey::PrivateIdentifier(ident) => ident.print(p, ctx),
        PropertyKey::StringLiteral(s) => p.print_string_literal(s, false),
        _ => key.to_expression().print_expr(p, Precedence::Comma, Context::empty()),
    }
}

struct StatementEmitter<'a, 'gc> {
    p: &'gc mut Codegen<'a>,
    ctx: Context,
}

impl<'a> StatementEmitter<'a, '_> {
    #[inline(always)]
    fn emit_top_level(mut self, directives: &[Directive<'a>], body: &[Statement<'a>]) {
        for directive in directives {
            emit_directive_impl(directive, self.p);
        }

        let mut iter = body.iter();
        let Some(first) = iter.next() else {
            return;
        };

        if directives.is_empty()
            && !self.p.options.minify
            && matches!(first, Statement::ExpressionStatement(stmt)
                if matches!(stmt.expression.without_parentheses(), Expression::StringLiteral(_)))
        {
            let Statement::ExpressionStatement(expr_stmt) = first else { unreachable!() };
            emit_string_expression_directive(self.p, expr_stmt, self.ctx);
        } else {
            self.emit_statement(first);
        }

        for stmt in iter {
            self.p.print_semicolon_if_needed();
            self.emit_statement(stmt);
        }
    }

    #[inline(always)]
    fn emit_statement(&mut self, stmt: &Statement<'a>) {
        match stmt {
            Statement::BlockStatement(block) => {
                self.p.print_comments_at(block.span.start);
                block.print(self.p, self.ctx);
            }
            Statement::ExpressionStatement(stmt) => emit_expression_statement_impl(stmt, self.p),
            Statement::VariableDeclaration(decl) => {
                emit_variable_declaration_statement(self.p, decl, self.ctx);
            }
            Statement::IfStatement(stmt) => emit_if_statement_impl(stmt, self.p, self.ctx),
            Statement::ReturnStatement(stmt) => emit_return_statement_impl(stmt, self.p),
            Statement::FunctionDeclaration(decl) => {
                emit_function_declaration(self.p, decl, self.ctx);
            }
            Statement::ForStatement(stmt) => emit_for_statement_impl(stmt, self.p, self.ctx),
            Statement::WhileStatement(stmt) => emit_while_statement_impl(stmt, self.p, self.ctx),
            Statement::DoWhileStatement(stmt) => {
                emit_do_while_statement_impl(stmt, self.p, self.ctx);
            }
            Statement::SwitchStatement(stmt) => emit_switch_statement_impl(stmt, self.p, self.ctx),
            Statement::BreakStatement(stmt) => emit_break_statement_impl(stmt, self.p, self.ctx),
            Statement::ContinueStatement(stmt) => {
                emit_continue_statement_impl(stmt, self.p, self.ctx);
            }
            Statement::TryStatement(stmt) => emit_try_statement_impl(stmt, self.p, self.ctx),
            Statement::ThrowStatement(stmt) => emit_throw_statement_impl(stmt, self.p),
            Statement::ForInStatement(stmt) => emit_for_in_statement_impl(stmt, self.p, self.ctx),
            Statement::ForOfStatement(stmt) => emit_for_of_statement_impl(stmt, self.p, self.ctx),
            Statement::ClassDeclaration(decl) => emit_class_declaration(self.p, decl, self.ctx),
            Statement::LabeledStatement(stmt) => {
                emit_labeled_statement_impl(stmt, self.p, self.ctx);
            }
            Statement::EmptyStatement(stmt) => emit_empty_statement_impl(stmt, self.p),
            Statement::ImportDeclaration(decl) => {
                emit_import_declaration_impl(decl, self.p, self.ctx);
            }
            Statement::ExportNamedDeclaration(decl) => {
                emit_export_named_declaration_impl(decl, self.p, self.ctx);
            }
            Statement::ExportDefaultDeclaration(decl) => {
                emit_export_default_declaration_impl(decl, self.p, self.ctx);
            }
            Statement::ExportAllDeclaration(decl) => {
                emit_export_all_declaration_impl(decl, self.p, self.ctx);
            }
            Statement::WithStatement(stmt) => emit_with_statement_impl(stmt, self.p, self.ctx),
            Statement::DebuggerStatement(stmt) => emit_debugger_statement_impl(stmt, self.p),
            Statement::TSModuleDeclaration(decl) => {
                self.p.print_comments_at(decl.span.start);
                self.p.print_indent();
                emit_ts_module_declaration_impl(decl, self.p, self.ctx);
                self.p.print_soft_newline();
            }
            Statement::TSTypeAliasDeclaration(decl) => {
                self.p.print_indent();
                self.p.print_comments_at(decl.span.start);
                emit_ts_type_alias_declaration_impl(decl, self.p, self.ctx);
                self.p.print_semicolon_after_statement();
            }
            Statement::TSInterfaceDeclaration(decl) => {
                self.p.print_indent();
                self.p.print_comments_at(decl.span.start);
                emit_ts_interface_declaration_impl(decl, self.p, self.ctx);
                self.p.print_soft_newline();
            }
            Statement::TSEnumDeclaration(decl) => {
                self.p.print_indent();
                self.p.print_comments_at(decl.span.start);
                emit_ts_enum_declaration_impl(decl, self.p, self.ctx);
                self.p.print_soft_newline();
            }
            Statement::TSExportAssignment(decl) => {
                emit_ts_export_assignment_impl(decl, self.p, self.ctx);
            }
            Statement::TSNamespaceExportDeclaration(decl) => {
                emit_ts_namespace_export_declaration_impl(decl, self.p, self.ctx);
            }
            Statement::TSImportEqualsDeclaration(decl) => {
                self.p.print_indent();
                self.p.print_comments_at(decl.span.start);
                emit_ts_import_equals_declaration_impl(decl, self.p, self.ctx);
                self.p.print_semicolon_after_statement();
            }
        }
    }
}

#[inline(always)]
fn emit_string_expression_directive<'a>(
    p: &mut Codegen<'a>,
    stmt: &ExpressionStatement<'a>,
    ctx: Context,
) {
    let expr = stmt.expression.without_parentheses();
    p.print_ascii_byte(b'(');
    expr.print_expr(p, Precedence::Lowest, ctx);
    p.print_ascii_byte(b')');
    p.print_semicolon_after_statement();
}

#[inline(always)]
fn emit_expression_statement_impl<'a>(stmt: &ExpressionStatement<'a>, p: &mut Codegen<'a>) {
    p.print_comments_at(stmt.span.start);
    if !p.options.minify && (p.indent > 0 || p.print_next_indent_as_space) {
        p.add_source_mapping(stmt.span);
        p.print_indent();
    }
    p.start_of_stmt = p.code_len();
    p.print_expression(&stmt.expression);
    p.print_semicolon_after_statement();
}

#[inline(always)]
fn emit_variable_declaration_statement<'a>(
    p: &mut Codegen<'a>,
    decl: &VariableDeclaration<'a>,
    ctx: Context,
) {
    p.print_comments_at(decl.span.start);
    p.print_indent();
    emit_variable_declaration_inner(p, decl, ctx);
    p.print_semicolon_after_statement();
}

#[inline(always)]
fn emit_if_statement_impl<'a>(stmt: &IfStatement<'a>, p: &mut Codegen<'a>, ctx: Context) {
    p.print_comments_at(stmt.span.start);
    p.add_source_mapping(stmt.span);
    p.print_indent();
    print_if(p, stmt, ctx);
}

#[inline(always)]
fn emit_return_statement_impl<'a>(stmt: &ReturnStatement<'a>, p: &mut Codegen<'a>) {
    p.print_comments_at(stmt.span.start);
    p.add_source_mapping(stmt.span);
    p.print_indent();
    p.print_space_before_identifier();
    p.print_keyword(b"return");
    if let Some(argument) = &stmt.argument {
        p.print_soft_space();
        p.print_expression(argument);
    }
    p.print_semicolon_after_statement();
}

#[inline(always)]
fn emit_block_statement_impl<'a>(block: &BlockStatement<'a>, p: &mut Codegen<'a>, ctx: Context) {
    p.print_indent();
    p.print_block_statement(block, ctx);
    p.print_soft_newline();
}

#[inline(always)]
fn emit_function_impl<'a>(function: &Function<'a>, p: &mut Codegen<'a>, ctx: Context) {
    let n = p.code_len();
    let wrap = function.is_expression()
        && ((p.start_of_stmt == n || p.start_of_default_export == n) || function.pife);
    let ctx = ctx.and_forbid_call(false);
    p.wrap(wrap, |p| {
        p.print_space_before_identifier();
        p.add_source_mapping(function.span);
        if function.declare {
            p.print_keyword(b"declare ");
        }
        if function.r#async {
            p.print_keyword(b"async ");
        }
        p.print_keyword(b"function");
        if function.generator {
            p.print_ascii_byte(b'*');
            p.print_soft_space();
        }
        if let Some(id) = &function.id {
            p.print_space_before_identifier();
            id.print(p, ctx);
        }
        if let Some(type_parameters) = &function.type_parameters {
            type_parameters.print(p, ctx);
        }
        p.print_ascii_byte(b'(');
        if let Some(this_param) = &function.this_param {
            this_param.print(p, ctx);
            if !function.params.is_empty() || function.params.rest.is_some() {
                p.print_keyword(b",");
                p.print_soft_space();
            }
        }
        function.params.print(p, ctx);
        p.print_ascii_byte(b')');
        if let Some(return_type) = &function.return_type {
            p.print_keyword(b": ");
            return_type.print(p, ctx);
        }
        if let Some(body) = &function.body {
            p.print_soft_space();
            body.print(p, ctx);
        } else {
            p.print_semicolon();
        }
    });
}

#[inline(always)]
fn emit_function_body_impl<'a>(body: &FunctionBody<'a>, p: &mut Codegen<'a>, ctx: Context) {
    let span_end = body.span.end;
    let trailing_comments_key = span_end.checked_sub(1);
    let mut single_line = body.is_empty();
    if single_line
        && let Some(key) = trailing_comments_key
        && let Some(block) = p.peek_comments(key)
    {
        single_line = block.iter().all(|c| !c.preceded_by_newline() && !c.followed_by_newline());
    }
    p.print_curly_braces(body.span, single_line, |p| {
        p.print_directives_and_statements(&body.directives, &body.statements, ctx);
        if let Some(key) = trailing_comments_key
            && let Some(comments) = p.take_comments(key)
        {
            p.print_comments(comments);
            p.print_next_indent_as_space = false;
        }
    });
    p.needs_semicolon = false;
}

#[inline(always)]
fn emit_formal_parameter_impl<'a>(param: &FormalParameter<'a>, p: &mut Codegen<'a>, ctx: Context) {
    p.add_source_mapping(param.span);
    p.print_decorators(&param.decorators, ctx);
    if let Some(accessibility) = param.accessibility {
        p.print_space_before_identifier();
        p.print_str(accessibility.as_str());
        p.print_soft_space();
    }
    if param.r#override {
        p.print_space_before_identifier();
        p.print_keyword(b"override");
        p.print_soft_space();
    }
    if param.readonly {
        p.print_space_before_identifier();
        p.print_keyword(b"readonly");
        p.print_soft_space();
    }
    param.pattern.print(p, ctx);
}

#[inline(always)]
fn emit_formal_parameters_impl<'a>(
    params: &FormalParameters<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    p.print_list(&params.items, ctx);
    if let Some(rest) = &params.rest {
        if !params.items.is_empty() {
            p.print_comma();
            p.print_soft_space();
        }
        rest.print(p, ctx);
    }
}

#[inline(always)]
fn emit_arrow_function_expression_impl<'a>(
    expr: &ArrowFunctionExpression<'a>,
    p: &mut Codegen<'a>,
    precedence: Precedence,
    ctx: Context,
) {
    p.wrap(precedence >= Precedence::Assign || expr.pife, |p| {
        if expr.r#async {
            p.print_space_before_identifier();
            p.add_source_mapping(expr.span);
            p.print_keyword(b"async");
            p.print_soft_space();
        }
        if let Some(type_parameters) = &expr.type_parameters {
            type_parameters.print(p, ctx);
        }
        let remove_params_wrap = p.options.minify
            && expr.params.items.len() == 1
            && expr.params.rest.is_none()
            && expr.type_parameters.is_none()
            && expr.return_type.is_none()
            && {
                let param = &expr.params.items[0];
                param.decorators.is_empty()
                    && !param.has_modifier()
                    && param.pattern.kind.is_binding_identifier()
                    && param.pattern.type_annotation.is_none()
                    && !param.pattern.optional
            };
        p.wrap(!remove_params_wrap, |p| {
            expr.params.print(p, ctx);
        });
        if let Some(return_type) = &expr.return_type {
            p.print_keyword(b":");
            p.print_soft_space();
            return_type.print(p, ctx);
        }
        p.print_soft_space();
        p.print_keyword(b"=>");
        p.print_soft_space();
        if expr.expression {
            if let Some(Statement::ExpressionStatement(stmt)) = expr.body.statements.first() {
                p.start_of_arrow_expr = p.code_len();
                stmt.expression.print_expr(p, Precedence::Comma, ctx);
            }
        } else {
            expr.body.print(p, ctx);
        }
    });
}

#[inline(always)]
fn emit_function_declaration<'a>(p: &mut Codegen<'a>, decl: &Function<'a>, ctx: Context) {
    p.print_comments_at(decl.span.start);
    if decl.pure && p.options.print_annotation_comment() {
        p.print_indent();
        p.print_str(NO_SIDE_EFFECTS_NEW_LINE_COMMENT);
    }
    p.print_indent();
    decl.print(p, ctx);
    p.print_soft_newline();
}

#[inline(always)]
fn emit_for_statement_impl<'a>(stmt: &ForStatement<'a>, p: &mut Codegen<'a>, ctx: Context) {
    p.print_comments_at(stmt.span.start);
    p.add_source_mapping(stmt.span);
    p.print_indent();
    p.print_space_before_identifier();
    p.print_keyword(b"for");
    p.print_soft_space();
    p.print_ascii_byte(b'(');

    if let Some(init) = &stmt.init {
        init.print(p, Context::FORBID_IN);
    }

    p.print_semicolon();

    if let Some(test) = stmt.test.as_ref() {
        p.print_soft_space();
        p.print_expression(test);
    }

    p.print_semicolon();

    if let Some(update) = stmt.update.as_ref() {
        p.print_soft_space();
        p.print_expression(update);
    }

    p.print_ascii_byte(b')');
    p.print_body(&stmt.body, false, ctx);
}

#[inline(always)]
fn emit_for_statement_init_impl<'a>(
    init: &ForStatementInit<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    match init {
        ForStatementInit::VariableDeclaration(var) => var.print(p, ctx),
        _ => init.to_expression().print_expr(p, Precedence::Lowest, ctx),
    }
}

#[inline(always)]
fn emit_for_statement_left_impl<'a>(
    left: &ForStatementLeft<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    match left {
        ForStatementLeft::VariableDeclaration(var) => var.print(p, ctx),
        ForStatementLeft::AssignmentTargetIdentifier(identifier) => {
            let wrap = identifier.name == "async";
            p.wrap(wrap, |p| left.to_assignment_target().print(p, ctx));
        }
        match_assignment_target!(ForStatementLeft) => {
            p.wrap(false, |p| left.to_assignment_target().print(p, ctx));
        }
    }
}

#[inline(always)]
fn emit_while_statement_impl<'a>(stmt: &WhileStatement<'a>, p: &mut Codegen<'a>, ctx: Context) {
    p.print_comments_at(stmt.span.start);
    p.add_source_mapping(stmt.span);
    p.print_indent();
    p.print_space_before_identifier();
    p.print_keyword(b"while");
    p.print_soft_space();
    p.print_ascii_byte(b'(');
    p.print_expression(&stmt.test);
    p.print_ascii_byte(b')');
    p.print_body(&stmt.body, false, ctx);
}

#[inline(always)]
fn emit_do_while_statement_impl<'a>(
    stmt: &DoWhileStatement<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    p.print_comments_at(stmt.span.start);
    p.add_source_mapping(stmt.span);
    p.print_indent();
    p.print_space_before_identifier();
    p.print_keyword(b"do");
    match &stmt.body {
        Statement::BlockStatement(block) => {
            p.print_soft_space();
            p.print_block_statement(block, ctx);
            p.print_soft_space();
        }
        Statement::EmptyStatement(s) => s.print(p, ctx),
        _ => {
            p.print_soft_newline();
            p.indent();
            stmt.body.print(p, ctx);
            p.print_semicolon_if_needed();
            p.dedent();
            p.print_indent();
        }
    }
    p.print_keyword(b"while");
    p.print_soft_space();
    p.print_ascii_byte(b'(');
    p.print_expression(&stmt.test);
    p.print_ascii_byte(b')');
    p.print_semicolon_after_statement();
}

#[inline(always)]
fn emit_switch_statement_impl<'a>(stmt: &SwitchStatement<'a>, p: &mut Codegen<'a>, ctx: Context) {
    p.print_comments_at(stmt.span.start);
    p.add_source_mapping(stmt.span);
    p.print_indent();
    p.print_space_before_identifier();
    p.print_keyword(b"switch");
    p.print_soft_space();
    p.print_ascii_byte(b'(');
    p.print_expression(&stmt.discriminant);
    p.print_ascii_byte(b')');
    p.print_soft_space();
    p.print_curly_braces(stmt.span, stmt.cases.is_empty(), |p| {
        for case in &stmt.cases {
            emit_switch_case_impl(case, p, ctx);
        }
    });
    p.print_soft_newline();
    p.needs_semicolon = false;
}

#[inline(always)]
fn emit_switch_case_impl<'a>(case: &SwitchCase<'a>, p: &mut Codegen<'a>, ctx: Context) {
    p.print_semicolon_if_needed();
    p.print_indent();
    p.add_source_mapping(case.span);
    match &case.test {
        Some(test) => {
            p.print_keyword(b"case");
            p.print_soft_space();
            emit_expression_impl(test, p, Precedence::Lowest, ctx);
        }
        None => p.print_keyword(b"default"),
    }
    p.print_colon();

    if case.consequent.len() == 1 {
        let stmt = &case.consequent[0];
        p.print_body(stmt, false, ctx);
        return;
    }

    p.print_soft_newline();
    p.indent();
    for item in &case.consequent {
        p.print_semicolon_if_needed();
        emit_statement_impl(item, p, ctx);
    }
    p.dedent();
}

#[inline(always)]
fn emit_break_statement_impl<'a>(stmt: &BreakStatement<'a>, p: &mut Codegen<'a>, ctx: Context) {
    p.print_comments_at(stmt.span.start);
    p.add_source_mapping(stmt.span);
    p.print_indent();
    p.print_space_before_identifier();
    p.print_keyword(b"break");
    if let Some(label) = &stmt.label {
        p.print_soft_space();
        label.print(p, ctx);
    }
    p.print_semicolon_after_statement();
}

#[inline(always)]
fn emit_continue_statement_impl<'a>(
    stmt: &ContinueStatement<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    p.print_comments_at(stmt.span.start);
    p.add_source_mapping(stmt.span);
    p.print_indent();
    p.print_space_before_identifier();
    p.print_keyword(b"continue");
    if let Some(label) = &stmt.label {
        p.print_soft_space();
        label.print(p, ctx);
    }
    p.print_semicolon_after_statement();
}

#[inline(always)]
fn emit_try_statement_impl<'a>(stmt: &TryStatement<'a>, p: &mut Codegen<'a>, ctx: Context) {
    p.print_comments_at(stmt.span.start);
    p.add_source_mapping(stmt.span);
    p.print_indent();
    p.print_space_before_identifier();
    p.print_keyword(b"try");
    p.print_soft_space();
    p.print_block_statement(&stmt.block, ctx);
    if let Some(handler) = &stmt.handler {
        emit_catch_clause_impl(handler, p, ctx);
    }
    if let Some(finalizer) = &stmt.finalizer {
        p.print_soft_space();
        p.print_keyword(b"finally");
        p.print_soft_space();
        p.print_block_statement(finalizer, ctx);
    }
    p.print_soft_newline();
}

#[inline(always)]
fn emit_catch_clause_impl<'a>(clause: &CatchClause<'a>, p: &mut Codegen<'a>, ctx: Context) {
    p.print_soft_space();
    p.print_comments_at(clause.span.start);
    p.print_keyword(b"catch");
    if let Some(param) = &clause.param {
        p.print_soft_space();
        p.print_ascii_byte(b'(');
        param.pattern.print(p, ctx);
        p.print_ascii_byte(b')');
    }
    p.print_soft_space();
    p.print_block_statement(&clause.body, ctx);
}

#[inline(always)]
fn emit_throw_statement_impl<'a>(stmt: &ThrowStatement<'a>, p: &mut Codegen<'a>) {
    p.print_comments_at(stmt.span.start);
    p.add_source_mapping(stmt.span);
    p.print_indent();
    p.print_space_before_identifier();
    p.print_keyword(b"throw");
    p.print_soft_space();
    p.print_expression(&stmt.argument);
    p.print_semicolon_after_statement();
}

#[inline(always)]
fn emit_with_statement_impl<'a>(stmt: &WithStatement<'a>, p: &mut Codegen<'a>, ctx: Context) {
    p.print_comments_at(stmt.span.start);
    p.add_source_mapping(stmt.span);
    p.print_indent();
    p.print_space_before_identifier();
    p.print_keyword(b"with");
    p.print_ascii_byte(b'(');
    p.print_expression(&stmt.object);
    p.print_ascii_byte(b')');
    p.print_body(&stmt.body, false, ctx);
}

#[inline(always)]
fn emit_debugger_statement_impl(stmt: &DebuggerStatement, p: &mut Codegen<'_>) {
    p.print_comments_at(stmt.span.start);
    p.add_source_mapping(stmt.span);
    p.print_indent();
    p.print_space_before_identifier();
    p.print_keyword(b"debugger");
    p.print_semicolon_after_statement();
}

#[inline(always)]
fn emit_for_in_statement_impl<'a>(stmt: &ForInStatement<'a>, p: &mut Codegen<'a>, ctx: Context) {
    p.print_comments_at(stmt.span.start);
    p.add_source_mapping(stmt.span);
    p.print_indent();
    p.print_space_before_identifier();
    p.print_keyword(b"for");
    p.print_soft_space();
    p.print_ascii_byte(b'(');
    stmt.left.print(p, Context::FORBID_IN);
    p.print_soft_space();
    p.print_space_before_identifier();
    p.print_keyword(b"in");
    p.print_soft_space();
    p.print_expression(&stmt.right);
    p.print_ascii_byte(b')');
    p.print_body(&stmt.body, false, ctx);
}

#[inline(always)]
fn emit_for_of_statement_impl<'a>(stmt: &ForOfStatement<'a>, p: &mut Codegen<'a>, ctx: Context) {
    p.print_comments_at(stmt.span.start);
    p.add_source_mapping(stmt.span);
    p.print_indent();
    p.print_space_before_identifier();
    p.print_keyword(b"for");
    if stmt.r#await {
        p.print_keyword(b" await");
    }
    p.print_soft_space();
    p.print_ascii_byte(b'(');
    stmt.left.print(p, ctx);
    p.print_soft_space();
    p.print_space_before_identifier();
    p.print_keyword(b"of");
    p.print_soft_space();
    stmt.right.print_expr(p, Precedence::Comma, Context::empty());
    p.print_ascii_byte(b')');
    p.print_body(&stmt.body, false, ctx);
}

#[inline(always)]
fn emit_class_declaration<'a>(p: &mut Codegen<'a>, decl: &Class<'a>, ctx: Context) {
    p.print_comments_at(decl.span.start);
    p.print_indent();
    emit_class_impl(decl, p, ctx);
    p.print_soft_newline();
}

#[inline(always)]
fn emit_class_impl<'a>(class: &Class<'a>, p: &mut Codegen<'a>, ctx: Context) {
    let n = p.code_len();
    let wrap = class.is_expression() && (p.start_of_stmt == n || p.start_of_default_export == n);
    let ctx = ctx.and_forbid_call(false);
    p.wrap(wrap, |p| {
        p.enter_class();
        p.print_decorators(&class.decorators, ctx);
        p.print_space_before_identifier();
        p.add_source_mapping(class.span);
        if class.declare {
            p.print_keyword(b"declare ");
        }
        if class.r#abstract {
            p.print_keyword(b"abstract ");
        }
        p.print_keyword(b"class");
        if let Some(id) = &class.id {
            p.print_hard_space();
            id.print(p, ctx);
            if let Some(type_parameters) = class.type_parameters.as_ref() {
                type_parameters.print(p, ctx);
            }
        }
        if let Some(super_class) = class.super_class.as_ref() {
            p.print_keyword(b" extends ");
            super_class.print_expr(p, Precedence::Postfix, Context::empty());
            if let Some(super_type_parameters) = &class.super_type_arguments {
                super_type_parameters.print(p, ctx);
            }
        }
        if !class.implements.is_empty() {
            p.print_keyword(b" implements ");
            p.print_list(&class.implements, ctx);
        }
        p.print_soft_space();
        emit_class_body_impl(&class.body, p, ctx);
        p.needs_semicolon = false;
        p.exit_class();
    });
}

#[inline(always)]
fn emit_class_body_impl<'a>(body: &ClassBody<'a>, p: &mut Codegen<'a>, ctx: Context) {
    p.print_curly_braces(body.span, body.body.is_empty(), |p| {
        for item in &body.body {
            p.print_semicolon_if_needed();
            p.print_leading_comments(item.span().start);
            p.print_indent();
            emit_class_element_impl(item, p, ctx);
        }
    });
}

#[inline(always)]
fn emit_class_element_impl<'a>(elem: &ClassElement<'a>, p: &mut Codegen<'a>, ctx: Context) {
    match elem {
        ClassElement::StaticBlock(block) => {
            emit_static_block_impl(block, p, ctx);
            p.print_soft_newline();
        }
        ClassElement::MethodDefinition(method) => {
            emit_method_definition_impl(method, p, ctx);
            p.print_soft_newline();
        }
        ClassElement::PropertyDefinition(prop) => {
            emit_property_definition_impl(prop, p, ctx);
            p.print_semicolon_after_statement();
        }
        ClassElement::AccessorProperty(accessor) => {
            emit_accessor_property_impl(accessor, p, ctx);
            p.print_semicolon_after_statement();
        }
        ClassElement::TSIndexSignature(sig) => {
            sig.print(p, ctx);
            p.print_semicolon_after_statement();
        }
    }
}

#[inline(always)]
fn emit_static_block_impl<'a>(block: &StaticBlock<'a>, p: &mut Codegen<'a>, ctx: Context) {
    p.add_source_mapping(block.span);
    p.print_keyword(b"static");
    p.print_soft_space();
    p.print_curly_braces(block.span, block.body.is_empty(), |p| {
        for stmt in &block.body {
            p.print_semicolon_if_needed();
            emit_statement_impl(stmt, p, ctx);
        }
    });
    p.needs_semicolon = false;
}

#[inline(always)]
fn emit_template_literal_impl<'a>(template: &TemplateLiteral<'a>, p: &mut Codegen<'a>) {
    p.add_source_mapping(template.span);
    p.print_ascii_byte(b'`');
    debug_assert_eq!(template.quasis.len(), template.expressions.len() + 1);
    let (first_quasi, remaining_quasis) = template.quasis.split_first().unwrap();
    p.print_str_escaping_script_close_tag(first_quasi.value.raw.as_str());
    for (expr, quasi) in template.expressions.iter().zip(remaining_quasis) {
        p.print_keyword(b"${");
        p.print_expression(expr);
        p.print_ascii_byte(b'}');
        p.add_source_mapping(quasi.span);
        p.print_str_escaping_script_close_tag(quasi.value.raw.as_str());
    }
    p.print_ascii_byte(b'`');
}

#[inline(always)]
fn emit_tagged_template_expression_impl<'a>(
    expr: &TaggedTemplateExpression<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    p.add_source_mapping(expr.span);
    expr.tag.print_expr(p, Precedence::Postfix, Context::empty());
    if let Some(type_parameters) = &expr.type_arguments {
        type_parameters.print(p, ctx);
    }
    emit_template_literal_impl(&expr.quasi, p);
}

#[inline(always)]
fn emit_method_definition_impl<'a>(
    method: &MethodDefinition<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    p.add_source_mapping(method.span);
    p.print_decorators(&method.decorators, ctx);
    if let Some(accessibility) = &method.accessibility {
        p.print_space_before_identifier();
        p.print_str(accessibility.as_str());
        p.print_soft_space();
    }
    if method.r#type == MethodDefinitionType::TSAbstractMethodDefinition {
        p.print_space_before_identifier();
        p.print_keyword(b"abstract");
        p.print_soft_space();
    }
    if method.r#static {
        p.print_space_before_identifier();
        p.print_keyword(b"static");
        p.print_soft_space();
    }
    match &method.kind {
        MethodDefinitionKind::Constructor | MethodDefinitionKind::Method => {}
        MethodDefinitionKind::Get => {
            p.print_space_before_identifier();
            p.print_keyword(b"get");
            p.print_soft_space();
        }
        MethodDefinitionKind::Set => {
            p.print_space_before_identifier();
            p.print_keyword(b"set");
            p.print_soft_space();
        }
    }
    if method.value.r#async {
        p.print_space_before_identifier();
        p.print_keyword(b"async");
        p.print_soft_space();
    }
    if method.value.generator {
        p.print_keyword(b"*");
    }
    if method.computed {
        p.print_ascii_byte(b'[');
    }
    method.key.print(p, ctx);
    if method.computed {
        p.print_ascii_byte(b']');
    }
    if method.optional {
        p.print_ascii_byte(b'?');
    }
    if let Some(type_parameters) = method.value.type_parameters.as_ref() {
        type_parameters.print(p, ctx);
    }
    p.print_ascii_byte(b'(');
    if let Some(this_param) = &method.value.this_param {
        this_param.print(p, ctx);
        if !method.value.params.is_empty() || method.value.params.rest.is_some() {
            p.print_keyword(b",");
            p.print_soft_space();
        }
    }
    method.value.params.print(p, ctx);
    p.print_ascii_byte(b')');
    if let Some(return_type) = &method.value.return_type {
        p.print_colon();
        p.print_soft_space();
        return_type.print(p, ctx);
    }
    if let Some(body) = &method.value.body {
        p.print_soft_space();
        body.print(p, ctx);
    } else {
        p.print_semicolon();
    }
}

#[inline(always)]
fn emit_property_definition_impl<'a>(
    property: &PropertyDefinition<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    p.add_source_mapping(property.span);
    p.print_decorators(&property.decorators, ctx);
    if property.declare {
        p.print_space_before_identifier();
        p.print_keyword(b"declare");
        p.print_soft_space();
    }
    if let Some(accessibility) = property.accessibility {
        p.print_space_before_identifier();
        p.print_str(accessibility.as_str());
        p.print_soft_space();
    }
    if property.r#type == PropertyDefinitionType::TSAbstractPropertyDefinition {
        p.print_space_before_identifier();
        p.print_keyword(b"abstract");
        p.print_soft_space();
    }
    if property.r#static {
        p.print_space_before_identifier();
        p.print_keyword(b"static");
        p.print_soft_space();
    }
    if property.readonly {
        p.print_space_before_identifier();
        p.print_keyword(b"readonly");
        p.print_soft_space();
    }
    if property.computed {
        p.print_ascii_byte(b'[');
    }
    property.key.print(p, ctx);
    if property.computed {
        p.print_ascii_byte(b']');
    }
    if property.optional {
        p.print_keyword(b"?");
    }
    if let Some(type_annotation) = &property.type_annotation {
        p.print_colon();
        p.print_soft_space();
        type_annotation.print(p, ctx);
    }
    if let Some(value) = &property.value {
        p.print_soft_space();
        p.print_equal();
        p.print_soft_space();
        value.print_expr(p, Precedence::Comma, Context::empty());
    }
}

#[inline(always)]
fn emit_accessor_property_impl<'a>(
    property: &AccessorProperty<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    p.add_source_mapping(property.span);
    p.print_decorators(&property.decorators, ctx);
    if property.r#type.is_abstract() {
        p.print_space_before_identifier();
        p.print_keyword(b"abstract");
        p.print_soft_space();
    }
    if let Some(accessibility) = property.accessibility {
        p.print_space_before_identifier();
        p.print_str(accessibility.as_str());
        p.print_soft_space();
    }
    if property.r#static {
        p.print_space_before_identifier();
        p.print_keyword(b"static");
        p.print_soft_space();
    }
    if property.r#override {
        p.print_space_before_identifier();
        p.print_keyword(b"override");
        p.print_soft_space();
    }
    p.print_space_before_identifier();
    p.print_keyword(b"accessor");
    if property.computed {
        p.print_soft_space();
        p.print_ascii_byte(b'[');
    } else {
        p.print_hard_space();
    }
    property.key.print(p, ctx);
    if property.computed {
        p.print_ascii_byte(b']');
    }
    if let Some(type_annotation) = &property.type_annotation {
        p.print_colon();
        p.print_soft_space();
        type_annotation.print(p, ctx);
    }
    if let Some(value) = &property.value {
        p.print_soft_space();
        p.print_equal();
        p.print_soft_space();
        value.print_expr(p, Precedence::Comma, Context::empty());
    }
}

#[inline(always)]
fn emit_private_identifier_impl<'a>(ident: &PrivateIdentifier<'a>, p: &mut Codegen<'a>) {
    let name = if let Some(private_member_mappings) = &p.private_member_mappings
        && let Some(mangled) = p.current_class_ids().find_map(|class_id| {
            private_member_mappings.get(class_id).and_then(|m| m.get(ident.name.as_str()))
        }) {
        (*mangled).clone()
    } else {
        ident.name.into_compact_str()
    };

    p.print_ascii_byte(b'#');
    p.add_source_mapping_for_name(ident.span, &ident.name);
    p.print_str(name.as_str());
}

#[inline(always)]
fn emit_binding_pattern_impl<'a>(pattern: &BindingPattern<'a>, p: &mut Codegen<'a>, ctx: Context) {
    emit_binding_pattern_kind_impl(&pattern.kind, p, ctx);
    if pattern.optional {
        p.print_keyword(b"?");
    }
    if let Some(type_annotation) = &pattern.type_annotation {
        p.print_colon();
        p.print_soft_space();
        type_annotation.print(p, ctx);
    }
}

#[inline(always)]
fn emit_binding_pattern_kind_impl<'a>(
    pattern: &BindingPatternKind<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    match pattern {
        BindingPatternKind::BindingIdentifier(ident) => ident.print(p, ctx),
        BindingPatternKind::ObjectPattern(pattern) => emit_object_pattern_impl(pattern, p, ctx),
        BindingPatternKind::ArrayPattern(pattern) => emit_array_pattern_impl(pattern, p, ctx),
        BindingPatternKind::AssignmentPattern(pattern) => pattern.print(p, ctx),
    }
}

#[inline(always)]
fn emit_object_pattern_impl<'a>(pattern: &ObjectPattern<'a>, p: &mut Codegen<'a>, ctx: Context) {
    p.add_source_mapping(pattern.span);
    p.print_ascii_byte(b'{');
    if !pattern.is_empty() {
        p.print_soft_space();
    }
    p.print_list(&pattern.properties, ctx);
    if let Some(rest) = &pattern.rest {
        if !pattern.properties.is_empty() {
            p.print_comma();
        }
        rest.print(p, ctx);
    }
    if !pattern.is_empty() {
        p.print_soft_space();
    }
    p.print_ascii_byte(b'}');
}

#[inline(always)]
fn emit_binding_property_impl<'a>(
    property: &BindingProperty<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    let mut shorthand = false;
    if let PropertyKey::StaticIdentifier(key) = &property.key {
        match &property.value.kind {
            BindingPatternKind::BindingIdentifier(ident)
                if key.name == p.get_binding_identifier_name(ident) =>
            {
                shorthand = true;
            }
            BindingPatternKind::AssignmentPattern(assignment_pattern) => {
                if let BindingPatternKind::BindingIdentifier(ident) = &assignment_pattern.left.kind
                    && key.name == p.get_binding_identifier_name(ident)
                {
                    shorthand = true;
                }
            }
            _ => {}
        }
    }

    if !shorthand {
        if property.computed {
            p.print_ascii_byte(b'[');
        }
        property.key.print(p, ctx);
        if property.computed {
            p.print_ascii_byte(b']');
        }
        p.print_colon();
        p.print_soft_space();
    }

    emit_binding_pattern_impl(&property.value, p, ctx);
}

#[inline(always)]
fn emit_binding_rest_element_impl<'a>(
    element: &BindingRestElement<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    p.add_source_mapping(element.span);
    p.print_ellipsis();
    emit_binding_pattern_impl(&element.argument, p, ctx);
}

#[inline(always)]
fn emit_array_pattern_impl<'a>(pattern: &ArrayPattern<'a>, p: &mut Codegen<'a>, ctx: Context) {
    p.add_source_mapping(pattern.span);
    p.print_ascii_byte(b'[');
    for (index, item) in pattern.elements.iter().enumerate() {
        if index != 0 {
            p.print_comma();
            p.print_soft_space();
        }
        if let Some(item) = item {
            item.print(p, ctx);
        }
        if index == pattern.elements.len() - 1 && (item.is_none() || pattern.rest.is_some()) {
            p.print_comma();
        }
    }
    if let Some(rest) = &pattern.rest {
        p.print_soft_space();
        rest.print(p, ctx);
    }
    p.print_ascii_byte(b']');
}

#[inline(always)]
fn emit_jsx_identifier_impl<'a>(ident: &JSXIdentifier<'a>, p: &mut Codegen<'a>, _ctx: Context) {
    p.add_source_mapping_for_name(ident.span, &ident.name);
    p.print_str(ident.name.as_str());
}

#[inline(always)]
fn emit_jsx_member_expression_object_impl<'a>(
    object: &JSXMemberExpressionObject<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    match object {
        JSXMemberExpressionObject::IdentifierReference(ident) => ident.print(p, ctx),
        JSXMemberExpressionObject::MemberExpression(member_expr) => member_expr.print(p, ctx),
        JSXMemberExpressionObject::ThisExpression(expr) => expr.print(p, ctx),
    }
}

#[inline(always)]
fn emit_jsx_member_expression_impl<'a>(
    expr: &JSXMemberExpression<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    emit_jsx_member_expression_object_impl(&expr.object, p, ctx);
    p.print_ascii_byte(b'.');
    expr.property.print(p, ctx);
}

#[inline(always)]
fn emit_jsx_element_name_impl<'a>(name: &JSXElementName<'a>, p: &mut Codegen<'a>, ctx: Context) {
    match name {
        JSXElementName::Identifier(identifier) => emit_jsx_identifier_impl(identifier, p, ctx),
        JSXElementName::IdentifierReference(identifier) => identifier.print(p, ctx),
        JSXElementName::NamespacedName(namespaced_name) => {
            emit_jsx_namespaced_name_impl(namespaced_name, p, ctx);
        }
        JSXElementName::MemberExpression(member_expr) => {
            emit_jsx_member_expression_impl(member_expr, p, ctx);
        }
        JSXElementName::ThisExpression(expr) => expr.print(p, ctx),
    }
}

#[inline(always)]
fn emit_jsx_namespaced_name_impl<'a>(
    name: &JSXNamespacedName<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    emit_jsx_identifier_impl(&name.namespace, p, ctx);
    p.print_colon();
    emit_jsx_identifier_impl(&name.name, p, ctx);
}

#[inline(always)]
fn emit_jsx_attribute_name_impl<'a>(
    name: &JSXAttributeName<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    match name {
        JSXAttributeName::Identifier(ident) => emit_jsx_identifier_impl(ident, p, ctx),
        JSXAttributeName::NamespacedName(namespaced_name) => {
            emit_jsx_namespaced_name_impl(namespaced_name, p, ctx);
        }
    }
}

#[inline(always)]
fn emit_jsx_attribute_impl<'a>(attribute: &JSXAttribute<'a>, p: &mut Codegen<'a>, ctx: Context) {
    emit_jsx_attribute_name_impl(&attribute.name, p, ctx);
    if let Some(value) = &attribute.value {
        p.print_equal();
        emit_jsx_attribute_value_impl(value, p, ctx);
    }
}

#[inline(always)]
fn emit_jsx_empty_expression_impl(expr: &JSXEmptyExpression, p: &mut Codegen<'_>, _ctx: Context) {
    p.print_comments_at(expr.span.end);
}

#[inline(always)]
fn emit_jsx_expression_impl<'a>(expr: &JSXExpression<'a>, p: &mut Codegen<'a>, ctx: Context) {
    match expr {
        JSXExpression::EmptyExpression(empty) => emit_jsx_empty_expression_impl(empty, p, ctx),
        _ => p.print_expression(expr.to_expression()),
    }
}

#[inline(always)]
fn emit_jsx_expression_container_impl<'a>(
    expr: &JSXExpressionContainer<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    p.print_ascii_byte(b'{');
    emit_jsx_expression_impl(&expr.expression, p, ctx);
    p.print_ascii_byte(b'}');
}

#[inline(always)]
fn emit_jsx_attribute_value_impl<'a>(
    value: &JSXAttributeValue<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    match value {
        JSXAttributeValue::Fragment(fragment) => emit_jsx_fragment_impl(fragment, p, ctx),
        JSXAttributeValue::Element(el) => emit_jsx_element_impl(el, p, ctx),
        JSXAttributeValue::StringLiteral(lit) => {
            let quote = if lit.value.contains('"') { b'\'' } else { b'"' };
            p.print_ascii_byte(quote);
            p.print_str(&lit.value);
            p.print_ascii_byte(quote);
        }
        JSXAttributeValue::ExpressionContainer(expr_container) => {
            emit_jsx_expression_container_impl(expr_container, p, ctx);
        }
    }
}

#[inline(always)]
fn emit_jsx_spread_attribute_impl<'a>(
    attr: &JSXSpreadAttribute<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    let _ = ctx;
    p.print_keyword(b"{...");
    attr.argument.print_expr(p, Precedence::Comma, Context::empty());
    p.print_ascii_byte(b'}');
}

#[inline(always)]
fn emit_jsx_attribute_item_impl<'a>(
    item: &JSXAttributeItem<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    match item {
        JSXAttributeItem::Attribute(attr) => emit_jsx_attribute_impl(attr, p, ctx),
        JSXAttributeItem::SpreadAttribute(spread_attr) => {
            emit_jsx_spread_attribute_impl(spread_attr, p, ctx);
        }
    }
}

#[inline(always)]
fn emit_jsx_element_impl<'a>(element: &JSXElement<'a>, p: &mut Codegen<'a>, ctx: Context) {
    p.add_source_mapping(element.opening_element.span);
    p.print_ascii_byte(b'<');
    emit_jsx_element_name_impl(&element.opening_element.name, p, ctx);
    for attr in &element.opening_element.attributes {
        match attr {
            JSXAttributeItem::Attribute(_) => {
                p.print_hard_space();
            }
            JSXAttributeItem::SpreadAttribute(_) => {
                p.print_soft_space();
            }
        }
        emit_jsx_attribute_item_impl(attr, p, ctx);
    }
    if element.closing_element.is_none() {
        p.print_soft_space();
        p.print_keyword(b"/");
    }
    p.print_ascii_byte(b'>');

    for child in &element.children {
        emit_jsx_child_impl(child, p, ctx);
    }

    if let Some(closing_element) = &element.closing_element {
        p.add_source_mapping(closing_element.span);
        p.print_keyword(b"</");
        emit_jsx_element_name_impl(&closing_element.name, p, ctx);
        p.print_ascii_byte(b'>');
    }
}

#[inline(always)]
fn emit_jsx_opening_fragment_impl(
    fragment: &JSXOpeningFragment,
    p: &mut Codegen<'_>,
    _ctx: Context,
) {
    p.add_source_mapping(fragment.span);
    p.print_keyword(b"<>");
}

#[inline(always)]
fn emit_jsx_closing_fragment_impl(
    fragment: &JSXClosingFragment,
    p: &mut Codegen<'_>,
    _ctx: Context,
) {
    p.add_source_mapping(fragment.span);
    p.print_keyword(b"</>");
}

#[inline(always)]
fn emit_jsx_text_impl(text: &JSXText<'_>, p: &mut Codegen<'_>, _ctx: Context) {
    p.add_source_mapping(text.span);
    p.print_str(text.value.as_str());
}

#[inline(always)]
fn emit_jsx_spread_child_impl(child: &JSXSpreadChild<'_>, p: &mut Codegen<'_>, _ctx: Context) {
    p.print_keyword(b"{...");
    p.print_expression(&child.expression);
    p.print_ascii_byte(b'}');
}

#[inline(always)]
fn emit_jsx_child_impl<'a>(child: &JSXChild<'a>, p: &mut Codegen<'a>, ctx: Context) {
    match child {
        JSXChild::Fragment(fragment) => emit_jsx_fragment_impl(fragment, p, ctx),
        JSXChild::Element(el) => emit_jsx_element_impl(el, p, ctx),
        JSXChild::Spread(spread) => emit_jsx_spread_child_impl(spread, p, ctx),
        JSXChild::ExpressionContainer(expr_container) => {
            emit_jsx_expression_container_impl(expr_container, p, ctx);
        }
        JSXChild::Text(text) => emit_jsx_text_impl(text, p, ctx),
    }
}

#[inline(always)]
fn emit_jsx_fragment_impl<'a>(fragment: &JSXFragment<'a>, p: &mut Codegen<'a>, ctx: Context) {
    emit_jsx_opening_fragment_impl(&fragment.opening_fragment, p, ctx);
    for child in &fragment.children {
        emit_jsx_child_impl(child, p, ctx);
    }
    emit_jsx_closing_fragment_impl(&fragment.closing_fragment, p, ctx);
}

#[inline(always)]
fn emit_labeled_statement_impl<'a>(stmt: &LabeledStatement<'a>, p: &mut Codegen<'a>, ctx: Context) {
    p.print_comments_at(stmt.span.start);
    if !p.options.minify && (p.indent > 0 || p.print_next_indent_as_space) {
        p.add_source_mapping(stmt.span);
        p.print_indent();
    }
    p.print_space_before_identifier();
    stmt.label.print(p, ctx);
    p.print_colon();
    p.print_body(&stmt.body, false, ctx);
}

#[inline(always)]
fn emit_empty_statement_impl(stmt: &EmptyStatement, p: &mut Codegen<'_>) {
    p.print_comments_at(stmt.span.start);
    p.add_source_mapping(stmt.span);
    p.print_indent();
    p.print_semicolon();
    p.print_soft_newline();
}

#[inline(always)]
fn print_if<'a>(p: &mut Codegen<'a>, if_stmt: &IfStatement<'a>, ctx: Context) {
    p.print_space_before_identifier();
    p.print_keyword(b"if");
    p.print_soft_space();
    p.print_ascii_byte(b'(');
    p.print_expression(&if_stmt.test);
    p.print_ascii_byte(b')');

    match &if_stmt.consequent {
        Statement::BlockStatement(block) => {
            p.print_soft_space();
            p.print_block_statement(block, ctx);
            if if_stmt.alternate.is_some() {
                p.print_soft_space();
            } else {
                p.print_soft_newline();
            }
        }
        stmt if wrap_to_avoid_ambiguous_else(stmt) => {
            p.print_soft_space();
            p.print_block_start(stmt.span());
            stmt.print(p, ctx);
            p.needs_semicolon = false;
            p.print_block_end(stmt.span());
            if if_stmt.alternate.is_some() {
                p.print_soft_space();
            } else {
                p.print_soft_newline();
            }
        }
        stmt => {
            p.print_body(stmt, false, ctx);
            if if_stmt.alternate.is_some() {
                p.print_indent();
            }
        }
    }
    if let Some(alternate) = if_stmt.alternate.as_ref() {
        p.print_semicolon_if_needed();
        p.print_space_before_identifier();
        p.print_keyword(b"else");
        match alternate {
            Statement::BlockStatement(block) => {
                p.print_soft_space();
                p.print_block_statement(block, ctx);
                p.print_soft_newline();
            }
            Statement::IfStatement(if_stmt) => {
                p.print_hard_space();
                print_if(p, if_stmt, ctx);
            }
            stmt => p.print_body(stmt, true, ctx),
        }
    }
}

#[inline(always)]
fn wrap_to_avoid_ambiguous_else(stmt: &Statement<'_>) -> bool {
    let mut current = stmt;
    loop {
        current = match current {
            Statement::IfStatement(if_stmt) => {
                if let Some(stmt) = &if_stmt.alternate {
                    stmt
                } else {
                    return true;
                }
            }
            Statement::ForStatement(for_stmt) => &for_stmt.body,
            Statement::ForOfStatement(for_of_stmt) => &for_of_stmt.body,
            Statement::ForInStatement(for_in_stmt) => &for_in_stmt.body,
            Statement::WhileStatement(while_stmt) => &while_stmt.body,
            Statement::WithStatement(with_stmt) => &with_stmt.body,
            Statement::LabeledStatement(labeled_stmt) => &labeled_stmt.body,
            _ => return false,
        };
    }
}

#[inline(always)]
fn emit_variable_declaration_inner<'a>(
    p: &mut Codegen<'a>,
    decl: &VariableDeclaration<'a>,
    ctx: Context,
) {
    p.print_space_before_identifier();
    if decl.declare {
        p.print_keyword(b"declare ");
    }

    match decl.kind {
        VariableDeclarationKind::Const => p.print_keyword(b"const"),
        VariableDeclarationKind::Let => p.print_keyword(b"let"),
        VariableDeclarationKind::Var => p.print_keyword(b"var"),
        VariableDeclarationKind::Using => p.print_keyword(b"using"),
        VariableDeclarationKind::AwaitUsing => {
            p.print_keyword(b"await");
            p.print_hard_space();
            p.print_keyword(b"using");
        }
    }
    if !decl.declarations.is_empty() {
        p.print_soft_space();
    }
    p.print_list(&decl.declarations, ctx);
}

#[inline(always)]
fn emit_variable_declarator_impl<'a>(
    decl: &VariableDeclarator<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    decl.id.kind.print(p, ctx);
    if decl.definite {
        p.print_ascii_byte(b'!');
    }
    if decl.id.optional {
        p.print_keyword(b"?");
    }
    if let Some(type_annotation) = &decl.id.type_annotation {
        p.print_colon();
        p.print_soft_space();
        type_annotation.print(p, ctx);
    }
    if let Some(init) = &decl.init {
        p.print_soft_space();
        p.print_equal();
        p.print_soft_space();
        init.print_expr(p, Precedence::Comma, ctx);
    }
}

#[inline(always)]
fn emit_import_declaration_impl<'a>(
    decl: &ImportDeclaration<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    p.print_comments_at(decl.span.start);
    p.add_source_mapping(decl.span);
    p.print_indent();
    p.print_space_before_identifier();
    p.print_keyword(b"import");
    if decl.import_kind.is_type() {
        p.print_keyword(b" type");
    }
    if let Some(phase) = decl.phase {
        p.print_hard_space();
        p.print_str(phase.as_str());
    }
    if let Some(specifiers) = &decl.specifiers {
        if specifiers.is_empty() {
            p.print_soft_space();
            p.print_keyword(b"{}");
            p.print_soft_space();
            p.print_keyword(b"from");
            p.print_soft_space();
            p.print_ascii_byte(b'"');
            p.print_str(decl.source.value.as_str());
            p.print_ascii_byte(b'"');
            if let Some(with_clause) = &decl.with_clause {
                p.print_hard_space();
                emit_with_clause_impl(with_clause, p, ctx);
            }
            p.print_semicolon_after_statement();
            return;
        }

        let mut in_block = false;
        for (index, specifier) in specifiers.iter().enumerate() {
            match specifier {
                ImportDeclarationSpecifier::ImportDefaultSpecifier(spec) => {
                    if in_block {
                        p.print_soft_space();
                        p.print_keyword(b"},");
                        in_block = false;
                    } else if index == 0 {
                        p.print_hard_space();
                    } else {
                        p.print_comma();
                        p.print_soft_space();
                    }
                    spec.local.print(p, ctx);
                    if index == specifiers.len() - 1 {
                        p.print_hard_space();
                    }
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(spec) => {
                    if in_block {
                        p.print_soft_space();
                        p.print_keyword(b"},");
                        in_block = false;
                    } else if index == 0 {
                        p.print_soft_space();
                    } else {
                        p.print_comma();
                        p.print_soft_space();
                    }
                    p.print_ascii_byte(b'*');
                    p.print_soft_space();
                    p.print_keyword(b"as ");
                    spec.local.print(p, ctx);
                    p.print_hard_space();
                }
                ImportDeclarationSpecifier::ImportSpecifier(spec) => {
                    if in_block {
                        p.print_comma();
                        p.print_soft_space();
                    } else {
                        if index != 0 {
                            p.print_comma();
                        }
                        in_block = true;
                        p.print_soft_space();
                        p.print_ascii_byte(b'{');
                        p.print_soft_space();
                    }

                    if spec.import_kind.is_type() {
                        p.print_keyword(b"type ");
                    }

                    spec.imported.print(p, ctx);
                    let local_name = p.get_binding_identifier_name(&spec.local);
                    let imported_name = get_module_export_name(&spec.imported, p);
                    if imported_name != local_name {
                        p.print_keyword(b" as ");
                        spec.local.print(p, ctx);
                    }
                }
            }
        }
        if in_block {
            p.print_soft_space();
            p.print_ascii_byte(b'}');
            p.print_soft_space();
        }
        p.print_keyword(b"from");
    }
    p.print_soft_space();
    p.print_string_literal(&decl.source, false);
    if let Some(with_clause) = &decl.with_clause {
        p.print_soft_space();
        emit_with_clause_impl(with_clause, p, ctx);
    }
    p.print_semicolon_after_statement();
}

#[inline(always)]
fn emit_with_clause_impl<'a>(clause: &WithClause<'a>, p: &mut Codegen<'a>, ctx: Context) {
    p.add_source_mapping(clause.span);
    p.print_str(match clause.keyword {
        WithClauseKeyword::With => "with",
        WithClauseKeyword::Assert => "assert",
    });
    p.print_soft_space();
    p.add_source_mapping(clause.span);
    p.print_ascii_byte(b'{');
    if !clause.with_entries.is_empty() {
        p.print_soft_space();
        p.print_list(&clause.with_entries, ctx);
        p.print_soft_space();
    }
    p.print_ascii_byte(b'}');
}

#[inline(always)]
fn emit_import_attribute_impl<'a>(attr: &ImportAttribute<'a>, p: &mut Codegen<'a>, _ctx: Context) {
    match &attr.key {
        ImportAttributeKey::Identifier(identifier) => {
            p.print_str(identifier.name.as_str());
        }
        ImportAttributeKey::StringLiteral(literal) => {
            p.print_string_literal(literal, false);
        }
    }
    p.print_colon();
    p.print_soft_space();
    p.print_string_literal(&attr.value, false);
}

#[inline(always)]
fn emit_export_named_declaration_impl<'a>(
    decl: &ExportNamedDeclaration<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    p.print_comments_at(decl.span.start);
    if let Some(Declaration::FunctionDeclaration(func)) = &decl.declaration
        && func.pure
        && p.options.print_annotation_comment()
    {
        p.print_str(NO_SIDE_EFFECTS_NEW_LINE_COMMENT);
    }
    p.add_source_mapping(decl.span);
    p.print_indent();
    p.print_keyword(b"export");
    if let Some(decl_item) = &decl.declaration {
        p.print_hard_space();
        match decl_item {
            Declaration::VariableDeclaration(decl) => decl.print(p, ctx),
            Declaration::FunctionDeclaration(decl) => decl.print(p, ctx),
            Declaration::ClassDeclaration(decl) => decl.print(p, ctx),
            Declaration::TSModuleDeclaration(decl) => emit_ts_module_declaration_impl(decl, p, ctx),
            Declaration::TSTypeAliasDeclaration(decl) => {
                emit_ts_type_alias_declaration_impl(decl, p, ctx);
            }
            Declaration::TSInterfaceDeclaration(decl) => {
                emit_ts_interface_declaration_impl(decl, p, ctx);
            }
            Declaration::TSEnumDeclaration(decl) => emit_ts_enum_declaration_impl(decl, p, ctx),
            Declaration::TSImportEqualsDeclaration(decl) => decl.print(p, ctx),
        }
        if matches!(
            decl_item,
            Declaration::VariableDeclaration(_)
                | Declaration::TSTypeAliasDeclaration(_)
                | Declaration::TSImportEqualsDeclaration(_)
        ) {
            p.print_semicolon_after_statement();
        } else {
            p.print_soft_newline();
            p.needs_semicolon = false;
        }
    } else {
        if decl.export_kind.is_type() {
            p.print_hard_space();
            p.print_keyword(b"type");
        }
        p.print_soft_space();
        p.print_ascii_byte(b'{');
        if !decl.specifiers.is_empty() {
            p.print_soft_space();
            p.print_list(&decl.specifiers, ctx);
            p.print_soft_space();
        }
        p.print_ascii_byte(b'}');
        if let Some(source) = &decl.source {
            p.print_soft_space();
            p.print_keyword(b"from");
            p.print_soft_space();
            p.print_string_literal(source, false);
        }
        p.print_semicolon_after_statement();
    }
}

#[inline(always)]
fn emit_ts_export_assignment_impl<'a>(
    decl: &TSExportAssignment<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    p.print_indent();
    p.print_comments_at(decl.span.start);
    p.print_keyword(b"export = ");
    decl.expression.print_expr(p, Precedence::Lowest, ctx);
    p.print_semicolon_after_statement();
}

#[inline(always)]
fn emit_ts_namespace_export_declaration_impl<'a>(
    decl: &TSNamespaceExportDeclaration<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    p.print_indent();
    p.print_comments_at(decl.span.start);
    p.print_keyword(b"export as namespace ");
    decl.id.print(p, ctx);
    p.print_semicolon_after_statement();
}

#[inline(always)]
fn emit_export_specifier_impl<'a>(spec: &ExportSpecifier<'a>, p: &mut Codegen<'a>, ctx: Context) {
    if spec.export_kind.is_type() {
        p.print_keyword(b"type ");
    }
    spec.local.print(p, ctx);
    let local_name = get_module_export_name(&spec.local, p);
    let exported_name = get_module_export_name(&spec.exported, p);
    if local_name != exported_name {
        p.print_keyword(b" as ");
        spec.exported.print(p, ctx);
    }
}

#[inline(always)]
fn get_module_export_name<'a>(
    module_export_name: &ModuleExportName<'a>,
    p: &Codegen<'a>,
) -> &'a str {
    match module_export_name {
        ModuleExportName::IdentifierName(ident) => ident.name.as_str(),
        ModuleExportName::IdentifierReference(ident) => p.get_identifier_reference_name(ident),
        ModuleExportName::StringLiteral(s) => s.value.as_str(),
    }
}

#[inline(always)]
fn emit_module_export_name_impl<'a>(
    name: &ModuleExportName<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    match name {
        ModuleExportName::IdentifierName(ident) => ident.print(p, ctx),
        ModuleExportName::IdentifierReference(ident) => ident.print(p, ctx),
        ModuleExportName::StringLiteral(literal) => p.print_string_literal(literal, false),
    }
}

#[inline(always)]
fn emit_export_all_declaration_impl<'a>(
    decl: &ExportAllDeclaration<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    p.print_comments_at(decl.span.start);
    p.add_source_mapping(decl.span);
    p.print_indent();
    p.print_keyword(b"export");
    if decl.export_kind.is_type() {
        p.print_keyword(b" type ");
    } else {
        p.print_soft_space();
    }
    p.print_ascii_byte(b'*');

    if let Some(exported) = &decl.exported {
        p.print_soft_space();
        p.print_keyword(b"as ");
        exported.print(p, ctx);
        p.print_hard_space();
    } else {
        p.print_soft_space();
    }

    p.print_keyword(b"from");
    p.print_soft_space();
    p.print_string_literal(&decl.source, false);
    if let Some(with_clause) = &decl.with_clause {
        p.print_hard_space();
        emit_with_clause_impl(with_clause, p, ctx);
    }
    p.print_semicolon_after_statement();
}

#[inline(always)]
fn emit_export_default_declaration_impl<'a>(
    decl: &ExportDefaultDeclaration<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    p.print_comments_at(decl.span.start);
    if let ExportDefaultDeclarationKind::FunctionDeclaration(func) = &decl.declaration
        && func.pure
        && p.options.print_annotation_comment()
    {
        p.print_str(NO_SIDE_EFFECTS_NEW_LINE_COMMENT);
    }
    p.add_source_mapping(decl.span);
    p.print_indent();
    p.print_keyword(b"export default ");
    decl.declaration.print(p, ctx);
}

#[inline(always)]
fn emit_export_default_declaration_kind_impl<'a>(
    kind: &ExportDefaultDeclarationKind<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    match kind {
        ExportDefaultDeclarationKind::FunctionDeclaration(func) => {
            func.print(p, ctx);
            p.print_soft_newline();
        }
        ExportDefaultDeclarationKind::ClassDeclaration(class) => {
            class.print(p, ctx);
            p.print_soft_newline();
        }
        ExportDefaultDeclarationKind::TSInterfaceDeclaration(interface) => interface.print(p, ctx),
        _ => {
            p.start_of_default_export = p.code_len();
            kind.to_expression().print_expr(p, Precedence::Comma, Context::empty());
            p.print_semicolon_after_statement();
        }
    }
}

#[inline(always)]
fn emit_ts_import_equals_declaration_impl<'a>(
    decl: &TSImportEqualsDeclaration<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    p.print_keyword(b"import ");
    decl.id.print(p, ctx);
    p.print_keyword(b" = ");
    decl.module_reference.print(p, ctx);
}

#[inline(always)]
fn emit_ts_import_equals_module_reference_impl<'a>(
    reference: &TSModuleReference<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    match reference {
        TSModuleReference::ExternalModuleReference(decl) => {
            p.print_keyword(b"require(");
            p.print_string_literal(&decl.expression, false);
            p.print_keyword(b")");
        }
        match_ts_type_name!(TSModuleReference) => reference.to_ts_type_name().print(p, ctx),
    }
}

#[inline(always)]
fn emit_decorator_impl<'a>(decorator: &Decorator<'a>, p: &mut Codegen<'a>, _ctx: Context) {
    fn need_wrap(expr: &Expression) -> bool {
        match expr {
            Expression::Identifier(_)
            | Expression::StaticMemberExpression(_)
            | Expression::PrivateFieldExpression(_) => false,
            Expression::CallExpression(call_expr) => need_wrap(&call_expr.callee),
            _ => true,
        }
    }

    p.add_source_mapping(decorator.span);
    p.print_ascii_byte(b'@');
    let wrap = need_wrap(&decorator.expression);
    p.wrap(wrap, |p| {
        decorator.expression.print_expr(p, Precedence::Lowest, Context::empty());
    });
}

#[inline(always)]
fn emit_ts_class_implements_impl<'a>(
    implements: &TSClassImplements<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    implements.expression.print(p, ctx);
    if let Some(type_parameters) = implements.type_arguments.as_ref() {
        type_parameters.print(p, ctx);
    }
}

#[inline(always)]
fn emit_ts_type_parameter_declaration_impl<'a>(
    decl: &TSTypeParameterDeclaration<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    let is_multi_line = decl.params.len() >= 2;
    p.print_ascii_byte(b'<');
    if is_multi_line {
        p.indent();
    }
    for (index, item) in decl.params.iter().enumerate() {
        if index != 0 {
            p.print_comma();
        }
        if is_multi_line {
            p.print_soft_newline();
            p.print_indent();
        } else if index != 0 {
            p.print_soft_space();
        }
        item.print(p, ctx);
    }
    if is_multi_line {
        p.print_soft_newline();
        p.dedent();
        p.print_indent();
    } else if p.is_jsx {
        p.print_keyword(b",");
    }
    p.print_ascii_byte(b'>');
}

#[inline(always)]
fn emit_ts_type_annotation_impl<'a>(
    annotation: &TSTypeAnnotation<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    annotation.type_annotation.print(p, ctx);
}

#[inline(always)]
fn emit_ts_type_impl<'a>(ty: &TSType<'a>, p: &mut Codegen<'a>, ctx: Context) {
    let ctx = ctx.with_typescript();
    match ty {
        TSType::TSFunctionType(ty) => emit_ts_function_type_impl(ty, p, ctx),
        TSType::TSConstructorType(ty) => emit_ts_constructor_type_impl(ty, p, ctx),
        TSType::TSArrayType(ty) => emit_ts_array_type_impl(ty, p, ctx),
        TSType::TSTupleType(ty) => emit_ts_tuple_type_impl(ty, p, ctx),
        TSType::TSUnionType(ty) => emit_ts_union_type_impl(ty, p, ctx),
        TSType::TSParenthesizedType(ty) => emit_ts_parenthesized_type_impl(ty, p, ctx),
        TSType::TSIntersectionType(ty) => emit_ts_intersection_type_impl(ty, p, ctx),
        TSType::TSConditionalType(ty) => emit_ts_conditional_type_impl(ty, p, ctx),
        TSType::TSInferType(ty) => emit_ts_infer_type_impl(ty, p, ctx),
        TSType::TSIndexedAccessType(ty) => emit_ts_indexed_access_type_impl(ty, p, ctx),
        TSType::TSMappedType(ty) => emit_ts_mapped_type_impl(ty, p, ctx),
        TSType::TSNamedTupleMember(ty) => emit_ts_named_tuple_member_impl(ty, p, ctx),
        TSType::TSLiteralType(ty) => ty.literal.print(p, ctx),
        TSType::TSImportType(ty) => emit_ts_import_type_impl(ty, p, ctx),
        TSType::TSAnyKeyword(_) => p.print_keyword(b"any"),
        TSType::TSBigIntKeyword(_) => p.print_keyword(b"bigint"),
        TSType::TSBooleanKeyword(_) => p.print_keyword(b"boolean"),
        TSType::TSIntrinsicKeyword(_) => p.print_keyword(b"intrinsic"),
        TSType::TSNeverKeyword(_) => p.print_keyword(b"never"),
        TSType::TSNullKeyword(_) => p.print_keyword(b"null"),
        TSType::TSNumberKeyword(_) => p.print_keyword(b"number"),
        TSType::TSObjectKeyword(_) => p.print_keyword(b"object"),
        TSType::TSStringKeyword(_) => p.print_keyword(b"string"),
        TSType::TSSymbolKeyword(_) => p.print_keyword(b"symbol"),
        TSType::TSThisType(_) => p.print_keyword(b"this"),
        TSType::TSUndefinedKeyword(_) => p.print_keyword(b"undefined"),
        TSType::TSUnknownKeyword(_) | TSType::JSDocUnknownType(_) => p.print_keyword(b"unknown"),
        TSType::TSVoidKeyword(_) => p.print_keyword(b"void"),
        TSType::TSTemplateLiteralType(ty) => emit_ts_template_literal_type_impl(ty, p, ctx),
        TSType::TSTypeLiteral(ty) => emit_ts_type_literal_impl(ty, p, ctx),
        TSType::TSTypeOperatorType(ty) => emit_ts_type_operator_impl(ty, p, ctx),
        TSType::TSTypePredicate(ty) => emit_ts_type_predicate_impl(ty, p, ctx),
        TSType::TSTypeQuery(ty) => emit_ts_type_query_impl(ty, p, ctx),
        TSType::TSTypeReference(ty) => emit_ts_type_reference_impl(ty, p, ctx),
        TSType::JSDocNullableType(ty) => emit_jsdoc_nullable_type_impl(ty, p, ctx),
        TSType::JSDocNonNullableType(ty) => emit_jsdoc_non_nullable_type_impl(ty, p, ctx),
    }
}

#[inline(always)]
fn emit_ts_array_type_impl<'a>(ty: &TSArrayType<'a>, p: &mut Codegen<'a>, ctx: Context) {
    ty.element_type.print(p, ctx);
    p.print_keyword(b"[]");
}

#[inline(always)]
fn emit_ts_tuple_type_impl<'a>(ty: &TSTupleType<'a>, p: &mut Codegen<'a>, ctx: Context) {
    p.print_keyword(b"[");
    p.print_list(&ty.element_types, ctx);
    p.print_keyword(b"]");
}

#[inline(always)]
fn parenthesize_check_type_of_conditional_type(ty: &TSType<'_>) -> bool {
    matches!(
        ty,
        TSType::TSFunctionType(_) | TSType::TSConstructorType(_) | TSType::TSConditionalType(_)
    )
}

#[inline(always)]
fn emit_ts_union_type_impl<'a>(ty: &TSUnionType<'a>, p: &mut Codegen<'a>, ctx: Context) {
    let Some((first, rest)) = ty.types.split_first() else {
        return;
    };
    p.wrap(parenthesize_check_type_of_conditional_type(first), |p| {
        first.print(p, ctx);
    });
    for item in rest {
        p.print_soft_space();
        p.print_keyword(b"|");
        p.print_soft_space();
        p.wrap(parenthesize_check_type_of_conditional_type(item), |p| {
            item.print(p, ctx);
        });
    }
}

#[inline(always)]
fn emit_ts_parenthesized_type_impl<'a>(
    ty: &TSParenthesizedType<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    p.print_ascii_byte(b'(');
    ty.type_annotation.print(p, ctx);
    p.print_ascii_byte(b')');
}

#[inline(always)]
fn emit_ts_intersection_type_impl<'a>(
    ty: &TSIntersectionType<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    let Some((first, rest)) = ty.types.split_first() else {
        return;
    };
    first.print(p, ctx);
    for item in rest {
        p.print_soft_space();
        p.print_keyword(b"&");
        p.print_soft_space();
        item.print(p, ctx);
    }
}

#[inline(always)]
fn emit_ts_conditional_type_impl<'a>(
    ty: &TSConditionalType<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    ty.check_type.print(p, ctx);
    p.print_keyword(b" extends ");
    ty.extends_type.print(p, ctx);
    p.print_keyword(b" ? ");
    ty.true_type.print(p, ctx);
    p.print_keyword(b" : ");
    ty.false_type.print(p, ctx);
}

#[inline(always)]
fn emit_ts_infer_type_impl<'a>(ty: &TSInferType<'a>, p: &mut Codegen<'a>, ctx: Context) {
    p.print_keyword(b"infer ");
    ty.type_parameter.print(p, ctx);
}

#[inline(always)]
fn emit_ts_indexed_access_type_impl<'a>(
    ty: &TSIndexedAccessType<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    ty.object_type.print(p, ctx);
    p.print_keyword(b"[");
    ty.index_type.print(p, ctx);
    p.print_keyword(b"]");
}

#[inline(always)]
fn emit_ts_mapped_type_impl<'a>(ty: &TSMappedType<'a>, p: &mut Codegen<'a>, ctx: Context) {
    p.print_keyword(b"{");
    p.print_soft_space();
    match ty.readonly {
        Some(TSMappedTypeModifierOperator::True) => p.print_keyword(b"readonly "),
        Some(TSMappedTypeModifierOperator::Plus) => p.print_keyword(b"+readonly "),
        Some(TSMappedTypeModifierOperator::Minus) => p.print_keyword(b"-readonly "),
        None => {}
    }
    p.print_keyword(b"[");
    ty.type_parameter.name.print(p, ctx);
    if let Some(constraint) = &ty.type_parameter.constraint {
        p.print_keyword(b" in ");
        constraint.print(p, ctx);
    }
    if let Some(default) = &ty.type_parameter.default {
        p.print_keyword(b" = ");
        default.print(p, ctx);
    }
    if let Some(name_type) = &ty.name_type {
        p.print_keyword(b" as ");
        name_type.print(p, ctx);
    }
    p.print_keyword(b"]");
    match ty.optional {
        Some(TSMappedTypeModifierOperator::True) => p.print_keyword(b"?"),
        Some(TSMappedTypeModifierOperator::Plus) => p.print_keyword(b"+?"),
        Some(TSMappedTypeModifierOperator::Minus) => p.print_keyword(b"-?"),
        None => {}
    }
    p.print_soft_space();
    if let Some(type_annotation) = &ty.type_annotation {
        p.print_keyword(b":");
        p.print_soft_space();
        type_annotation.print(p, ctx);
    }
    p.print_soft_space();
    p.print_keyword(b"}");
}

#[inline(always)]
fn emit_ts_qualified_name_impl<'a>(name: &TSQualifiedName<'a>, p: &mut Codegen<'a>, ctx: Context) {
    name.left.print(p, ctx);
    p.print_keyword(b".");
    name.right.print(p, ctx);
}

#[inline(always)]
fn emit_ts_type_operator_impl<'a>(ty: &TSTypeOperator<'a>, p: &mut Codegen<'a>, ctx: Context) {
    p.print_str(ty.operator.to_str());
    p.print_hard_space();
    ty.type_annotation.print(p, ctx);
}

#[inline(always)]
fn emit_ts_type_predicate_impl<'a>(ty: &TSTypePredicate<'a>, p: &mut Codegen<'a>, ctx: Context) {
    if ty.asserts {
        p.print_keyword(b"asserts ");
    }
    match &ty.parameter_name {
        TSTypePredicateName::Identifier(ident) => ident.print(p, ctx),
        TSTypePredicateName::This(_) => p.print_keyword(b"this"),
    }
    if let Some(type_annotation) = &ty.type_annotation {
        p.print_keyword(b" is ");
        type_annotation.print(p, ctx);
    }
}

#[inline(always)]
fn emit_ts_type_reference_impl<'a>(ty: &TSTypeReference<'a>, p: &mut Codegen<'a>, ctx: Context) {
    emit_ts_type_name_impl(&ty.type_name, p, ctx);
    if let Some(type_parameters) = &ty.type_arguments {
        type_parameters.print(p, ctx);
    }
}

#[inline(always)]
fn emit_ts_type_literal_impl<'a>(ty: &TSTypeLiteral<'a>, p: &mut Codegen<'a>, ctx: Context) {
    p.print_curly_braces(ty.span, ty.members.is_empty(), |p| {
        for item in &ty.members {
            p.print_leading_comments(item.span().start);
            p.print_indent();
            item.print(p, ctx);
            p.print_semicolon();
            p.print_soft_newline();
        }
    });
}

#[inline(always)]
fn emit_ts_type_name_impl<'a>(ty: &TSTypeName<'a>, p: &mut Codegen<'a>, ctx: Context) {
    match ty {
        TSTypeName::IdentifierReference(ident) => ident.print(p, ctx),
        TSTypeName::QualifiedName(name) => {
            name.left.print(p, ctx);
            p.print_keyword(b".");
            name.right.print(p, ctx);
        }
        TSTypeName::ThisExpression(expr) => expr.print(p, ctx),
    }
}

#[inline(always)]
fn emit_jsdoc_nullable_type_impl<'a>(
    ty: &JSDocNullableType<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    if ty.postfix {
        ty.type_annotation.print(p, ctx);
        p.print_keyword(b"?");
    } else {
        p.print_keyword(b"?");
        ty.type_annotation.print(p, ctx);
    }
}

#[inline(always)]
fn emit_jsdoc_non_nullable_type_impl<'a>(
    ty: &JSDocNonNullableType<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    if ty.postfix {
        ty.type_annotation.print(p, ctx);
        p.print_keyword(b"!");
    } else {
        p.print_keyword(b"!");
        ty.type_annotation.print(p, ctx);
    }
}

#[inline(always)]
fn emit_ts_template_literal_type_impl<'a>(
    ty: &TSTemplateLiteralType<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    p.print_keyword(b"`");
    for (index, item) in ty.quasis.iter().enumerate() {
        if index != 0
            && let Some(types) = ty.types.get(index - 1)
        {
            p.print_keyword(b"${");
            types.print(p, ctx);
            p.print_keyword(b"}");
        }
        p.print_str(item.value.raw.as_str());
    }
    p.print_keyword(b"`");
}

#[inline(always)]
fn emit_ts_signature_impl<'a>(signature: &TSSignature<'a>, p: &mut Codegen<'a>, ctx: Context) {
    match signature {
        TSSignature::TSIndexSignature(signature) => emit_ts_index_signature_impl(signature, p, ctx),
        TSSignature::TSPropertySignature(signature) => {
            emit_ts_property_signature_impl(signature, p, ctx);
        }
        TSSignature::TSCallSignatureDeclaration(signature) => {
            if let Some(type_parameters) = signature.type_parameters.as_ref() {
                type_parameters.print(p, ctx);
            }
            p.print_keyword(b"(");
            if let Some(this_param) = &signature.this_param {
                emit_ts_this_parameter_impl(this_param, p, ctx);
                if !signature.params.is_empty() || signature.params.rest.is_some() {
                    p.print_keyword(b",");
                    p.print_soft_space();
                }
            }
            signature.params.print(p, ctx);
            p.print_keyword(b")");
            if let Some(return_type) = &signature.return_type {
                p.print_colon();
                p.print_soft_space();
                return_type.print(p, ctx);
            }
        }
        TSSignature::TSConstructSignatureDeclaration(signature) => {
            p.print_keyword(b"new ");
            if let Some(type_parameters) = signature.type_parameters.as_ref() {
                type_parameters.print(p, ctx);
            }
            p.print_keyword(b"(");
            signature.params.print(p, ctx);
            p.print_keyword(b")");
            if let Some(return_type) = &signature.return_type {
                p.print_colon();
                p.print_soft_space();
                return_type.print(p, ctx);
            }
        }
        TSSignature::TSMethodSignature(signature) => {
            match signature.kind {
                TSMethodSignatureKind::Method => {}
                TSMethodSignatureKind::Get => p.print_keyword(b"get "),
                TSMethodSignatureKind::Set => p.print_keyword(b"set "),
            }
            if signature.computed {
                p.print_ascii_byte(b'[');
                signature.key.print(p, ctx);
                p.print_ascii_byte(b']');
            } else {
                match &signature.key {
                    PropertyKey::StaticIdentifier(key) => key.print(p, ctx),
                    PropertyKey::PrivateIdentifier(key) => p.print_str(key.name.as_str()),
                    PropertyKey::StringLiteral(key) => p.print_string_literal(key, false),
                    key => key.to_expression().print_expr(p, Precedence::Comma, ctx),
                }
            }
            if signature.optional {
                p.print_keyword(b"?");
            }
            if let Some(type_parameters) = &signature.type_parameters {
                type_parameters.print(p, ctx);
            }
            p.print_keyword(b"(");
            if let Some(this_param) = &signature.this_param {
                emit_ts_this_parameter_impl(this_param, p, ctx);
                if !signature.params.is_empty() || signature.params.rest.is_some() {
                    p.print_keyword(b",");
                    p.print_soft_space();
                }
            }
            signature.params.print(p, ctx);
            p.print_keyword(b")");
            if let Some(return_type) = &signature.return_type {
                p.print_colon();
                p.print_soft_space();
                return_type.print(p, ctx);
            }
        }
    }
}

#[inline(always)]
fn emit_ts_literal_impl<'a>(literal: &TSLiteral<'a>, p: &mut Codegen<'a>, ctx: Context) {
    match literal {
        TSLiteral::BooleanLiteral(decl) => decl.print(p, ctx),
        TSLiteral::NumericLiteral(decl) => decl.print_expr(p, Precedence::Lowest, ctx),
        TSLiteral::BigIntLiteral(decl) => decl.print_expr(p, Precedence::Lowest, ctx),
        TSLiteral::StringLiteral(decl) => decl.print(p, ctx),
        TSLiteral::TemplateLiteral(decl) => decl.print(p, ctx),
        TSLiteral::UnaryExpression(decl) => decl.print_expr(p, Precedence::Comma, ctx),
    }
}

#[inline(always)]
fn emit_ts_type_parameter_impl<'a>(param: &TSTypeParameter<'a>, p: &mut Codegen<'a>, ctx: Context) {
    if param.r#const {
        p.print_keyword(b"const ");
    }
    if param.r#in {
        p.print_keyword(b"in ");
    }
    if param.out {
        p.print_keyword(b"out ");
    }
    param.name.print(p, ctx);
    if let Some(constraint) = &param.constraint {
        p.print_keyword(b" extends ");
        constraint.print(p, ctx);
    }
    if let Some(default) = &param.default {
        p.print_keyword(b" = ");
        default.print(p, ctx);
    }
}

#[inline(always)]
fn emit_ts_property_signature_impl<'a>(
    signature: &TSPropertySignature<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    if signature.readonly {
        p.print_keyword(b"readonly ");
    }
    if signature.computed {
        p.print_ascii_byte(b'[');
        signature.key.print(p, ctx);
        p.print_ascii_byte(b']');
    } else {
        match &signature.key {
            PropertyKey::StaticIdentifier(key) => key.print(p, ctx),
            PropertyKey::PrivateIdentifier(key) => p.print_str(key.name.as_str()),
            PropertyKey::StringLiteral(key) => p.print_string_literal(key, false),
            key => key.to_expression().print_expr(p, Precedence::Comma, ctx),
        }
    }
    if signature.optional {
        p.print_keyword(b"?");
    }
    if let Some(type_annotation) = &signature.type_annotation {
        p.print_colon();
        p.print_soft_space();
        type_annotation.print(p, ctx);
    }
}

#[inline(always)]
fn emit_ts_type_query_impl<'a>(query: &TSTypeQuery<'a>, p: &mut Codegen<'a>, ctx: Context) {
    p.print_keyword(b"typeof ");
    query.expr_name.print(p, ctx);
    if let Some(type_params) = &query.type_arguments {
        type_params.print(p, ctx);
    }
}

#[inline(always)]
fn emit_ts_type_query_expr_name_impl<'a>(
    name: &TSTypeQueryExprName<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    match name {
        match_ts_type_name!(TSTypeQueryExprName) => name.to_ts_type_name().print(p, ctx),
        TSTypeQueryExprName::TSImportType(decl) => emit_ts_import_type_impl(decl, p, ctx),
    }
}

#[inline(always)]
fn emit_ts_import_type_impl<'a>(import: &TSImportType<'a>, p: &mut Codegen<'a>, ctx: Context) {
    p.print_keyword(b"import(");
    import.argument.print(p, ctx);
    if let Some(options) = &import.options {
        p.print_keyword(b", ");
        options.print_expr(p, Precedence::Lowest, ctx);
    }
    p.print_keyword(b")");
    if let Some(qualifier) = &import.qualifier {
        p.print_ascii_byte(b'.');
        emit_ts_import_type_qualifier_impl(qualifier, p, ctx);
    }
    if let Some(type_parameters) = &import.type_arguments {
        type_parameters.print(p, ctx);
    }
}

#[inline(always)]
fn emit_ts_import_type_qualifier_impl<'a>(
    qualifier: &TSImportTypeQualifier<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    match qualifier {
        TSImportTypeQualifier::Identifier(ident) => p.print_str(ident.name.as_str()),
        TSImportTypeQualifier::QualifiedName(qualified) => {
            emit_ts_import_type_qualified_name_impl(qualified, p, ctx);
        }
    }
}

#[inline(always)]
fn emit_ts_import_type_qualified_name_impl<'a>(
    name: &TSImportTypeQualifiedName<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    name.left.print(p, ctx);
    p.print_ascii_byte(b'.');
    p.print_str(name.right.name.as_str());
}

#[inline(always)]
fn emit_ts_type_parameter_instantiation_impl<'a>(
    instantiation: &TSTypeParameterInstantiation<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    p.print_keyword(b"<");
    p.print_list(&instantiation.params, ctx);
    p.print_keyword(b">");
}

#[inline(always)]
fn emit_ts_index_signature_impl<'a>(
    signature: &TSIndexSignature<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    if signature.readonly {
        p.print_keyword(b"readonly ");
    }
    p.print_keyword(b"[");
    for (index, parameter) in signature.parameters.iter().enumerate() {
        if index != 0 {
            p.print_keyword(b",");
            p.print_soft_space();
        }
        p.print_str(parameter.name.as_str());
        p.print_colon();
        p.print_soft_space();
        parameter.type_annotation.print(p, ctx);
    }
    p.print_keyword(b"]");
    p.print_colon();
    p.print_soft_space();
    signature.type_annotation.print(p, ctx);
}

#[inline(always)]
fn emit_ts_tuple_element_impl<'a>(element: &TSTupleElement<'a>, p: &mut Codegen<'a>, ctx: Context) {
    match element {
        match_ts_type!(TSTupleElement) => element.to_ts_type().print(p, ctx),
        TSTupleElement::TSOptionalType(ts_type) => {
            ts_type.type_annotation.print(p, ctx);
            p.print_keyword(b"?");
        }
        TSTupleElement::TSRestType(ts_type) => {
            p.print_keyword(b"...");
            ts_type.type_annotation.print(p, ctx);
        }
    }
}

#[inline(always)]
fn emit_ts_named_tuple_member_impl<'a>(
    member: &TSNamedTupleMember<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    member.label.print(p, ctx);
    if member.optional {
        p.print_keyword(b"?");
    }
    p.print_keyword(b":");
    p.print_soft_space();
    member.element_type.print(p, ctx);
}

#[inline(always)]
fn emit_ts_function_type_impl<'a>(ty: &TSFunctionType<'a>, p: &mut Codegen<'a>, ctx: Context) {
    if let Some(type_parameters) = &ty.type_parameters {
        type_parameters.print(p, ctx);
    }
    p.print_keyword(b"(");
    if let Some(this_param) = &ty.this_param {
        emit_ts_this_parameter_impl(this_param, p, ctx);
        if !ty.params.is_empty() || ty.params.rest.is_some() {
            p.print_keyword(b",");
            p.print_soft_space();
        }
    }
    ty.params.print(p, ctx);
    p.print_keyword(b")");
    p.print_soft_space();
    p.print_keyword(b"=>");
    p.print_soft_space();
    ty.return_type.print(p, ctx);
}

#[inline(always)]
fn emit_ts_this_parameter_impl<'a>(param: &TSThisParameter<'a>, p: &mut Codegen<'a>, ctx: Context) {
    p.print_keyword(b"this");
    if let Some(type_annotation) = &param.type_annotation {
        p.print_keyword(b": ");
        type_annotation.print(p, ctx);
    }
}

#[inline(always)]
fn emit_ts_constructor_type_impl<'a>(
    ty: &TSConstructorType<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    if ty.r#abstract {
        p.print_keyword(b"abstract ");
    }
    p.print_keyword(b"new ");
    if let Some(type_parameters) = &ty.type_parameters {
        type_parameters.print(p, ctx);
    }
    p.print_keyword(b"(");
    ty.params.print(p, ctx);
    p.print_keyword(b")");
    p.print_soft_space();
    p.print_keyword(b"=>");
    p.print_soft_space();
    ty.return_type.print(p, ctx);
}

#[inline(always)]
fn emit_ts_module_declaration_impl<'a>(
    decl: &TSModuleDeclaration<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    if decl.declare {
        p.print_keyword(b"declare ");
    }
    p.print_str(decl.kind.as_str());
    if !decl.kind.is_global() {
        p.print_space_before_identifier();
        decl.id.print(p, ctx);
    }

    if let Some(body) = &decl.body {
        let mut body = body;
        loop {
            match body {
                TSModuleDeclarationBody::TSModuleDeclaration(nested) => {
                    p.print_ascii_byte(b'.');
                    nested.id.print(p, ctx);
                    if let Some(next) = &nested.body {
                        body = next;
                    } else {
                        break;
                    }
                }
                TSModuleDeclarationBody::TSModuleBlock(block) => {
                    p.print_soft_space();
                    emit_ts_module_block_impl(block, p, ctx);
                    break;
                }
            }
        }
    } else {
        p.print_semicolon();
    }
    p.needs_semicolon = false;
}

#[inline(always)]
fn emit_ts_module_block_impl<'a>(block: &TSModuleBlock<'a>, p: &mut Codegen<'a>, ctx: Context) {
    let is_empty = block.directives.is_empty() && block.body.is_empty();
    p.print_curly_braces(block.span, is_empty, |p| {
        p.print_directives_and_statements(&block.directives, &block.body, ctx);
    });
    p.needs_semicolon = false;
}

#[inline(always)]
fn emit_ts_module_declaration_name_impl<'a>(
    name: &TSModuleDeclarationName<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    match name {
        TSModuleDeclarationName::Identifier(ident) => ident.print(p, ctx),
        TSModuleDeclarationName::StringLiteral(lit) => p.print_string_literal(lit, false),
    }
}

#[inline(always)]
fn emit_ts_type_alias_declaration_impl<'a>(
    decl: &TSTypeAliasDeclaration<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    if decl.declare {
        p.print_keyword(b"declare ");
    }
    p.print_keyword(b"type");
    p.print_space_before_identifier();
    decl.id.print(p, ctx);
    if let Some(params) = &decl.type_parameters {
        params.print(p, ctx);
    }
    p.print_soft_space();
    p.print_keyword(b"=");
    p.print_soft_space();
    decl.type_annotation.print(p, ctx);
}

#[inline(always)]
fn emit_ts_interface_declaration_impl<'a>(
    decl: &TSInterfaceDeclaration<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    p.print_keyword(b"interface");
    p.print_hard_space();
    decl.id.print(p, ctx);
    if let Some(params) = &decl.type_parameters {
        params.print(p, ctx);
    }
    if !decl.extends.is_empty() {
        p.print_keyword(b" extends ");
        p.print_list(&decl.extends, ctx);
    }
    p.print_soft_space();
    p.print_curly_braces(decl.body.span, decl.body.body.is_empty(), |p| {
        for item in &decl.body.body {
            p.print_leading_comments(item.span().start);
            p.print_indent();
            item.print(p, ctx);
            p.print_semicolon();
            p.print_soft_newline();
        }
    });
}

#[inline(always)]
fn emit_ts_interface_heritage_impl<'a>(
    heritage: &TSInterfaceHeritage<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    heritage.expression.print_expr(p, Precedence::Call, ctx);
    if let Some(args) = &heritage.type_arguments {
        args.print(p, ctx);
    }
}

#[inline(always)]
fn emit_ts_enum_declaration_impl<'a>(
    decl: &TSEnumDeclaration<'a>,
    p: &mut Codegen<'a>,
    ctx: Context,
) {
    p.print_indent();
    if decl.declare {
        p.print_keyword(b"declare ");
    }
    if decl.r#const {
        p.print_keyword(b"const ");
    }
    p.print_space_before_identifier();
    p.print_keyword(b"enum ");
    decl.id.print(p, ctx);
    p.print_space_before_identifier();
    emit_ts_enum_body_impl(&decl.body, p, ctx);
}

#[inline(always)]
fn emit_ts_enum_body_impl<'a>(body: &TSEnumBody<'a>, p: &mut Codegen<'a>, ctx: Context) {
    p.print_curly_braces(body.span, body.members.is_empty(), |p| {
        for (index, member) in body.members.iter().enumerate() {
            p.print_leading_comments(member.span().start);
            p.print_indent();
            emit_ts_enum_member_impl(member, p, ctx);
            if index != body.members.len() - 1 {
                p.print_comma();
            }
            p.print_soft_newline();
        }
    });
}

#[inline(always)]
fn emit_ts_enum_member_impl<'a>(member: &TSEnumMember<'a>, p: &mut Codegen<'a>, ctx: Context) {
    match &member.id {
        TSEnumMemberName::Identifier(ident) => ident.print(p, ctx),
        TSEnumMemberName::String(lit) => p.print_string_literal(lit, false),
        TSEnumMemberName::ComputedString(lit) => {
            p.print_ascii_byte(b'[');
            p.print_string_literal(lit, false);
            p.print_ascii_byte(b']');
        }
        TSEnumMemberName::ComputedTemplateString(template) => {
            let quasi = template.quasis.first().unwrap();
            p.add_source_mapping(quasi.span);
            p.print_keyword(b"[`");
            p.print_str(quasi.value.raw.as_str());
            p.print_keyword(b"`]");
        }
    }
    if let Some(initializer) = &member.initializer {
        p.print_soft_space();
        p.print_equal();
        p.print_soft_space();
        initializer.print_expr(p, Precedence::Lowest, ctx);
    }
}
