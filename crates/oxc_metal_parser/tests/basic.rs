use oxc_allocator::Allocator;
use oxc_span::SourceType;

#[test]
fn let_decl_basic() {
    let alloc = Allocator::new();
    let src = "let x = 1;";
    let program = oxc_metal_parser::parse_program(&alloc, src, SourceType::mjs());
    assert_eq!(program.body.len(), 1);
}

#[test]
fn expr_stmt_string() {
    let alloc = Allocator::new();
    let src = "'hi';";
    let program = oxc_metal_parser::parse_program(&alloc, src, SourceType::mjs());
    assert_eq!(program.body.len(), 1);
}

