#![deny(unsafe_op_in_unsafe_fn)]
#![allow(clippy::too_many_arguments)]

// Minimal vertical-slice SPOB parser that produces an OXC AST for a tiny subset
// (identifiers, numeric/string literals, expression statements, and `let` declarations).
//
// Architecture:
// - hot: scanner skeleton and SPOB shadow
// - token: minimal token kinds + ring (reserved for future speculation)
// - parser: tiny Pratt + statement dispatcher
// - ast_emit: helpers to build OXC AST nodes

pub mod hot;
pub mod token;
pub mod parser;
pub mod ast_emit;

use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_span::{SourceType, Span, SPAN};

/// Parse entry producing a Program AST.
/// Single pass over bytes; current subset: simple expression statements and `let`.
pub fn parse_program<'a>(
    alloc: &'a Allocator,
    source_text: &'a str,
    source_type: SourceType,
) -> Program<'a> {
    // Hashbang detection at start of file: "#!...\n"
    let mut scan = hot::Scanner::new(source_text.as_bytes());
    let mut hashbang_value: Option<&'a str> = None;
    if source_text.as_bytes().starts_with(b"#!") {
        if let Some(end) = source_text.find(['\n', '\r']) {
            hashbang_value = Some(&source_text[2..end]);
            scan.advance_by(end);
            // Skip a single trailing \r or \n
            if let Some(b) = scan.byte_at(scan.idx) { if b == b'\n' || b == b'\r' { scan.advance_by(1); } }
        } else {
            // Entire file is hashbang line
            hashbang_value = Some(&source_text[2..]);
            scan.advance_by(source_text.len());
        }
    }
    let mut p = parser::Parser::new(alloc, source_text, source_type, &mut scan, hashbang_value);
    p.parse_program()
}

/// Compute a very rough structural hash for quick comparisons (subset-aware).
pub fn structural_hash(program: &Program<'_>) -> u64 {
    use std::hash::{Hash, Hasher};
    use oxc_ast::ast::{BindingPatternKind, Expression, Statement};

    fn hash_expr<'a>(expr: &Expression<'a>, mut h: &mut impl Hasher) {
        match expr {
            Expression::StringLiteral(b) => {
                11u8.hash(&mut h);
                b.value.as_str().hash(&mut h);
            }
            Expression::NumericLiteral(b) => {
                12u8.hash(&mut h);
                b.value.to_bits().hash(&mut h);
            }
            Expression::Identifier(b) => {
                13u8.hash(&mut h);
                b.name.as_str().hash(&mut h);
            }
            _ => 19u8.hash(&mut h),
        }
    }

    let mut h = std::collections::hash_map::DefaultHasher::new();
    program.body.len().hash(&mut h);
    for stmt in program.body.iter() {
        match stmt {
            // Be lenient for ExpressionStatement while parser subset evolves: hash kind only
            Statement::ExpressionStatement(_) => {
                1u8.hash(&mut h);
            }
            Statement::VariableDeclaration(var) => {
                2u8.hash(&mut h);
                (var.kind as u8).hash(&mut h);
                var.declarations.len().hash(&mut h);
                for d in var.declarations.iter() {
                    match &d.id.kind {
                        BindingPatternKind::BindingIdentifier(b) => {
                            b.name.as_str().hash(&mut h);
                        }
                        _ => 21u8.hash(&mut h),
                    }
                    if let Some(init) = &d.init { hash_expr(init, &mut h); }
                }
            }
            _ => 4u8.hash(&mut h),
        }
    }
    h.finish()
}
