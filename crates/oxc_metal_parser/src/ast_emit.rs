use oxc_allocator::Allocator;
use oxc_ast::AstBuilder;
use oxc_ast::ast::*;
use oxc_span::{Span, SPAN, SourceType};

pub struct Emitter<'a> { pub b: AstBuilder<'a> }

impl<'a> Emitter<'a> {
    pub fn new(alloc: &'a Allocator) -> Self { Self { b: AstBuilder::new(alloc) } }

    pub fn empty_program(&self, source_text: &'a str, source_type: SourceType) -> Program<'a> {
        self.b.program(SPAN, source_type, source_text, self.b.vec(), None, self.b.vec(), self.b.vec())
    }

    pub fn push_stmt(body: &mut oxc_allocator::Vec<'a, Statement<'a>>, stmt: Statement<'a>) { body.push(stmt); }

    pub fn stmt_expr(&self, span: Span, expr: Expression<'a>) -> Statement<'a> {
        Statement::ExpressionStatement(self.b.alloc(ExpressionStatement { span, expression: expr }))
    }

    pub fn stmt_empty(&self, span: Span) -> Statement<'a> {
        Statement::EmptyStatement(self.b.alloc(EmptyStatement { span }))
    }

    pub fn expr_ident(&self, span: Span, name: &'a str) -> Expression<'a> {
        let id = IdentifierReference { span, name: self.b.atom(name), reference_id: Default::default() };
        Expression::Identifier(self.b.alloc(id))
    }

    pub fn expr_string(&self, span: Span, value: &'a str) -> Expression<'a> {
        self.b.expression_string_literal(span, self.b.atom(value), None)
    }

    pub fn expr_number(&self, span: Span, value: f64) -> Expression<'a> {
        self.b.expression_numeric_literal(span, value, None, oxc_syntax::number::NumberBase::Decimal)
    }

    pub fn var_decl_simple(&self, span: Span, name: &'a str, init: Expression<'a>) -> Statement<'a> {
        let id_name = IdentifierName { span, name: self.b.atom(name) };
        let binding = BindingIdentifier { span, name: id_name.name, symbol_id: Default::default() };
        let pat = BindingPatternKind::BindingIdentifier(self.b.alloc(binding));
        let kind = VariableDeclarationKind::Let;
        let binding_pat = BindingPattern { kind: pat, type_annotation: None, optional: false };
        let decl = VariableDeclarator { span, kind, id: binding_pat, init: Some(init), definite: false };
        let list = self.b.vec1(decl);
        let var = VariableDeclaration { span, kind, declarations: list, declare: false };
        Statement::VariableDeclaration(self.b.alloc(var))
    }
}
