use std::fmt::{Debug, Display};

use crate::js::expr::{AssignmentExpression, Expression, IdentifierNameTokenData, Seq};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MaybeIdentifier {
    pub has_value: bool,
    pub value: MaybeIdentifierValue,
}

impl Display for MaybeIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.has_value {
            write!(f, "{:?}", unsafe { self.value.value })
        } else {
            write!(f, "None")
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union MaybeIdentifierValue {
    pub value: IdentifierNameTokenData,
    pub none: (),
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MaybeStatement {
    pub has_value: bool,
    pub value: MaybeStatementValue,
}

impl Display for MaybeStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.has_value {
            write!(f, "{}", unsafe { self.value.value })
        } else {
            write!(f, "None")
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union MaybeStatementValue {
    pub value: Statement,
    pub none: (),
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MaybeExpression {
    pub has_value: bool,
    pub value: MaybeExpressionValue,
}

impl Display for MaybeExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.has_value {
            write!(f, "{:?}", unsafe { self.value.value })
        } else {
            write!(f, "None")
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union MaybeExpressionValue {
    pub value: Expression,
    pub none: (),
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MaybeAssignmentExpression {
    pub has_value: bool,
    pub value: MaybeAssignmentExpressionValue,
}

impl Display for MaybeAssignmentExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.has_value {
            write!(f, "{:?}", unsafe { self.value.value })
        } else {
            write!(f, "None")
        }
    }
}

impl Debug for MaybeAssignmentExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.has_value {
            write!(f, "{:?}", unsafe { self.value.value })
        } else {
            write!(f, "None")
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union MaybeAssignmentExpressionValue {
    pub value: AssignmentExpression,
    pub none: (),
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MaybeBlock {
    pub has_value: bool,
    pub value: MaybeBlockValue,
}

impl Display for MaybeBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.has_value {
            write!(f, "{}", unsafe { self.value.value })
        } else {
            write!(f, "None")
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union MaybeBlockValue {
    pub value: BlockStatement,
    pub none: (),
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SeqStatementOrDeclaration {
    pub items: *const StatementOrDeclaration,
    pub len: usize,
}

impl Seq for SeqStatementOrDeclaration {
    type Item = StatementOrDeclaration;

    fn len(&self) -> usize {
        self.len
    }

    fn data(&self) -> *const Self::Item {
        self.items
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Script {
    pub body: SeqStatementOrDeclaration,
}

impl Display for Script {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let items = unsafe { std::slice::from_raw_parts(self.body.items, self.body.len) };
        write!(f, "Script {{\n")?;
        for item in items {
            write!(f, "  {},\n", item)?;
        }
        write!(f, "}}")
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Statement {
    pub tag: u8,
    pub data: StatementData,
}

impl Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.tag {
            STATEMENT_BLOCK_STATEMENT => write!(f, "BlockStatement {{ {} }}", unsafe {
                self.data.block.as_ref().unwrap()
            },),
            STATEMENT_VAR_STATEMENT => write!(f, "VariableStatement {{ {} }}", unsafe {
                self.data.var.as_ref().unwrap()
            },),
            STATEMENT_EMPTY_STATEMENT => write!(f, "EmptyStatement"),
            STATEMENT_EXPR_STATEMENT => write!(f, "ExpressionStatement {{ {:?} }}", unsafe {
                self.data.expression.as_ref().unwrap()
            },),
            STATEMENT_IF_STATEMENT => write!(f, "IfStatement {{ {} }}", unsafe {
                self.data.if_stmt.as_ref().unwrap()
            },),
            STATEMENT_CONTINUE_STATEMENT => write!(f, "ContinueStatement {{ {} }}", unsafe {
                self.data.continue_.as_ref().unwrap()
            },),
            STATEMENT_BREAK_STATEMENT => write!(f, "BreakStatement {{ {} }}", unsafe {
                self.data.break_.as_ref().unwrap()
            },),
            STATEMENT_RETURN_STATEMENT => write!(f, "ReturnStatement {{ {} }}", unsafe {
                self.data.return_.as_ref().unwrap()
            },),
            STATEMENT_WITH_STATEMENT => write!(f, "WithStatement {{ }}"),
            STATEMENT_THROW_STATEMENT => write!(f, "ThrowStatement {{ {:?} }}", unsafe {
                self.data.throw.as_ref().unwrap()
            },),
            STATEMENT_TRY_STATEMENT => write!(f, "TryStatement {{ {} }}", unsafe {
                self.data.try_.as_ref().unwrap()
            },),
            STATEMENT_DEBUGGER_STATEMENT => write!(f, "DebuggerStatement"),
            STATEMENT_DO_WHILE => write!(f, "DoWhileStatement {{ {} }}", unsafe {
                self.data.do_while.as_ref().unwrap()
            },),
            STATEMENT_WHILE => write!(f, "WhileStatement {{ {} }}", unsafe {
                self.data.while_.as_ref().unwrap()
            },),
            STATEMENT_FOR => write!(f, "ForStatement {{ }}"),
            _ => write!(f, "Unknown"),
        }
    }
}

impl Debug for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self, f)
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union StatementData {
    pub block: *const BlockStatement,
    pub var: *const VariableStatement,
    pub empty: (),
    pub expression: *const Expression,
    pub if_stmt: *const IfStatement,
    pub continue_: *const MaybeIdentifier,
    pub break_: *const MaybeIdentifier,
    pub return_: *const MaybeExpression,
    pub with: *const WithStatement,
    pub throw: *const Expression,
    pub try_: *const TryStatement,
    pub debugger: (),

    pub do_while: *const WhileStatement,
    pub while_: *const WhileStatement,
    pub for_: *const ForStatement,
}

pub const STATEMENT_BLOCK_STATEMENT: u8 = 0;
pub const STATEMENT_VAR_STATEMENT: u8 = 1;
pub const STATEMENT_EMPTY_STATEMENT: u8 = 2;
pub const STATEMENT_EXPR_STATEMENT: u8 = 3;
pub const STATEMENT_IF_STATEMENT: u8 = 4;
// WARN: deprecated
pub const STATEMENT_BREAKABLE_STATEMENT: u8 = 5;
pub const STATEMENT_CONTINUE_STATEMENT: u8 = 6;
pub const STATEMENT_BREAK_STATEMENT: u8 = 7;
pub const STATEMENT_RETURN_STATEMENT: u8 = 8;
pub const STATEMENT_WITH_STATEMENT: u8 = 9;
pub const STATEMENT_LABELLED_STATEMENT: u8 = 10;
pub const STATEMENT_THROW_STATEMENT: u8 = 11;
pub const STATEMENT_TRY_STATEMENT: u8 = 12;
pub const STATEMENT_DEBUGGER_STATEMENT: u8 = 13;
pub const STATEMENT_DO_WHILE: u8 = 14;
pub const STATEMENT_WHILE: u8 = 15;
pub const STATEMENT_FOR: u8 = 16;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct StatementOrDeclaration {
    pub tag: u8,
    pub data: StatementOrDeclarationData,
}

impl Display for StatementOrDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.tag {
            STATEMENT_OR_DECLARATION_STATEMENT => write!(f, "Statement { }", unsafe {
                self.data.statement.as_ref().unwrap()
            },),
            STATEMENT_OR_DECLARATION_DECLARATION => write!(f, "Declaration { }", unsafe {
                self.data.declaration.as_ref().unwrap()
            },),
            _ => write!(f, "Unknown"),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union StatementOrDeclarationData {
    pub statement: *const Statement,
    pub declaration: *const Declaration,
}

pub const STATEMENT_OR_DECLARATION_STATEMENT: u8 = 0;
pub const STATEMENT_OR_DECLARATION_DECLARATION: u8 = 1;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Declaration {
    pub tag: u8,
    pub data: DeclarationData,
}

impl Display for Declaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.tag {
            DECLARATION_FUNCTION_DECLARATION => write!(f, "FunctionDeclaration {{ }}"),
            DECLARATION_GENERATOR_DECLARATION => write!(f, "GeneratorDeclaration {{ }}"),
            DECLARATION_ASYNC_FUNCTION_DECLARATION => {
                write!(f, "AsyncFunctionDeclaration {{ }}")
            }
            DECLARATION_ASYNC_GENERATOR_DECLARATION => {
                write!(f, "AsyncGeneratorDeclaration {{ }}")
            }
            DECLARATION_CLASS_DECLARATION => write!(f, "ClassDeclaration {{}}"),
            DECLARATION_LEXICAL_DECLARATION => write!(f, "LexicalDeclaration { }", unsafe {
                self.data.lex_decl.as_ref().unwrap()
            },),
            _ => write!(f, "Unknown"),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union DeclarationData {
    pub function: *const HoistableDeclaration,
    pub generator: *const HoistableDeclaration,
    pub async_function: *const HoistableDeclaration,
    pub async_generator: *const HoistableDeclaration,

    pub class_declaration: *const ClassDeclaration,

    pub lex_decl: *const LexicalDeclaration,
}

pub const DECLARATION_FUNCTION_DECLARATION: u8 = 0;
pub const DECLARATION_GENERATOR_DECLARATION: u8 = 1;
pub const DECLARATION_ASYNC_FUNCTION_DECLARATION: u8 = 2;
pub const DECLARATION_ASYNC_GENERATOR_DECLARATION: u8 = 3;
pub const DECLARATION_CLASS_DECLARATION: u8 = 4;
pub const DECLARATION_LEXICAL_DECLARATION: u8 = 5;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FormalParameter {
    pub name: *const IdentifierNameTokenData,
    pub initializer: *const MaybeAssignmentExpression,
    pub is_rest: bool,
}

impl Debug for FormalParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FormalParameter")
            .field("name", unsafe { self.name.as_ref().unwrap() })
            .field("initializer", unsafe { self.initializer.as_ref().unwrap() })
            .field("is_rest", &self.is_rest)
            .finish()
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SeqFormalParameter {
    pub items: *const FormalParameter,
    pub len: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HoistableDeclaration {
    pub name: *const MaybeIdentifier,
    pub params: SeqFormalParameter,
    pub body: *const BlockStatement,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ClassDeclaration {
    abc: (),
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct LexicalBinding {
    pub name: *const IdentifierNameTokenData,
    pub initializer: *const MaybeAssignmentExpression,
}

impl Display for LexicalBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "LexicalBinding {{ name: {:?}, initializer: {} }}",
            unsafe { self.name.as_ref().unwrap() },
            unsafe { self.initializer.as_ref().unwrap() }
        )
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SeqLexicalBinding {
    pub items: *const LexicalBinding,
    pub len: usize,
}

impl Seq for SeqLexicalBinding {
    type Item = LexicalBinding;

    fn len(&self) -> usize {
        self.len
    }

    fn data(&self) -> *const Self::Item {
        self.items
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct LexicalDeclaration {
    pub is_const: bool,
    pub bindings: SeqLexicalBinding,
}

impl Display for LexicalDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bindings =
            unsafe { std::slice::from_raw_parts(self.bindings.items, self.bindings.len) };

        write!(
            f,
            "LexicalDeclaration {{ is_const: {}, len: {}, bindings: [\n",
            self.is_const, self.bindings.len
        )?;
        for binding in bindings {
            write!(f, "  {},\n", binding)?;
        }
        write!(f, "]}}")
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct BlockStatement {
    pub body: SeqStatementOrDeclaration,
}

impl Display for BlockStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let items = unsafe { std::slice::from_raw_parts(self.body.items, self.body.len) };
        write!(f, "BlockStatement {{\n")?;
        for item in items {
            write!(f, "  {},\n", item)?;
        }
        write!(f, "}}")
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SeqLexicalDeclaration {
    pub items: *const LexicalDeclaration,
    pub len: usize,
}

impl Seq for SeqLexicalDeclaration {
    type Item = LexicalDeclaration;

    fn len(&self) -> usize {
        self.len
    }

    fn data(&self) -> *const Self::Item {
        self.items
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct VariableStatement {
    pub declarations: SeqLexicalDeclaration,
}

impl Display for VariableStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let decls =
            unsafe { std::slice::from_raw_parts(self.declarations.items, self.declarations.len) };
        write!(f, "VariableStatement {{\n")?;
        for decl in decls {
            write!(f, "  {},\n", decl)?;
        }
        write!(f, "}}")
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct IfStatement {
    pub test: *const Expression,
    pub consequent: *const Statement,
    pub alternate: *const MaybeStatement,
}

impl Display for IfStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "IfStatement {{\n  test: {:?},\n  consequent: {},\n  alternate: {}\n}}",
            unsafe { self.test.as_ref().unwrap() },
            unsafe { self.consequent.as_ref().unwrap() },
            unsafe { self.alternate.as_ref().unwrap() }
        )
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WhileStatement {
    pub test: *const Expression,
    pub body: *const Statement,
}

impl Display for WhileStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "WhileStatement {{\n  test: {:?},\n  body: {}\n}}",
            unsafe { self.test.as_ref().unwrap() },
            unsafe { self.body.as_ref().unwrap() }
        )
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ForStatementInit {
    pub tag: u8,
    pub data: ForStatementInitData,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union ForStatementInitData {
    pub var: *const VariableStatement,
    pub lexical: *const LexicalDeclaration,
    pub expr: *const Expression,
}

pub const FOR_STATEMENT_INIT_VAR: u8 = 0;
pub const FOR_STATEMENT_INIT_LEXICAL: u8 = 1;
pub const FOR_STATEMENT_INIT_EXPR: u8 = 2;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MaybeForStatementInit {
    pub has_value: bool,
    pub value: MaybeForStatementInitValue,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union MaybeForStatementInitValue {
    pub value: ForStatementInit,
    pub none: (),
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ForStatement {
    pub init: MaybeForStatementInit,
    pub test: *const MaybeExpression,
    pub update: *const MaybeExpression,
    pub body: *const Statement,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WithStatement {
    pub object: *const Expression,
    pub body: *const Statement,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CatchClause {
    pub param: *const MaybeIdentifier,
    pub body: *const BlockStatement,
}

impl Display for CatchClause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CatchClause {{\n  param: {},\n  body: {}\n}}",
            unsafe { self.param.as_ref().unwrap() },
            unsafe { self.body.as_ref().unwrap() }
        )
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MaybeCatchClause {
    pub has_value: bool,
    pub value: MaybeCatchClauseValue,
}

impl Display for MaybeCatchClause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.has_value {
            write!(f, "{}", unsafe { self.value.value })
        } else {
            write!(f, "None")
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union MaybeCatchClauseValue {
    pub value: CatchClause,
    pub none: (),
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TryStatement {
    pub block: *const BlockStatement,
    pub handler: *const MaybeCatchClause,
    pub finalizer: *const MaybeBlock,
}

impl Display for TryStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TryStatement {{\n  block: {},\n  handler: {},\n  finalizer: {}\n}}",
            unsafe { self.block.as_ref().unwrap() },
            unsafe { self.handler.as_ref().unwrap() },
            unsafe { self.finalizer.as_ref().unwrap() }
        )
    }
}
