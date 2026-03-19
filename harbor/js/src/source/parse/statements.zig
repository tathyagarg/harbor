/// NOTE: This file was made through a more thought out approach than expressions.zig,
/// so the code is more organized and easier to read.
pub const EXTERN_UNION = @import("mod.zig").EXTERN_UNION;
pub const MAYBE = @import("mod.zig").MAYBE;

pub const IdentifierNameData = @import("../text.zig").IdentifierNameData;

pub const expr = @import("expressions.zig");

pub const MaybeIdentifier = MAYBE(IdentifierNameData);
pub const MaybeStatement = MAYBE(Statement);
pub const MaybeExpression = MAYBE(expr.Expression);
pub const MaybeAssignmentExpression = MAYBE(expr.AssignmentExpression);
pub const MaybeBlock = MAYBE(BlockStatement);

pub const Statement = EXTERN_UNION(
    extern union {
        block_statement: *BlockStatement,
        var_statement: *VarStatement,
        empty_statement: *void,
        expr_statement: *expr.Expression,
        if_statement: *IfStatement,
        breakable_statement: *BreakableStatement,
        continue_statement: *MaybeIdentifier,
        break_statement: *MaybeIdentifier,
        return_statement: *MaybeExpression,
        with_statement: *WithStatement,
        labelled_statement: *LabelledStatement,
        throw_statement: *expr.Expression,
        try_statement: *TryStatement,
        debugger_statement: *void,
    },
);

pub const STATEMENT_BLOCK_STATEMENT = 0;
pub const STATEMENT_VAR_STATEMENT = 1;
pub const STATEMENT_EMPTY_STATEMENT = 2;
pub const STATEMENT_EXPR_STATEMENT = 3;
pub const STATEMENT_IF_STATEMENT = 4;
pub const STATEMENT_BREAKABLE_STATEMENT = 5;
pub const STATEMENT_CONTINUE_STATEMENT = 6;
pub const STATEMENT_BREAK_STATEMENT = 7;
pub const STATEMENT_RETURN_STATEMENT = 8;
pub const STATEMENT_WITH_STATEMENT = 9;
pub const STATEMENT_LABELLED_STATEMENT = 10;
pub const STATEMENT_THROW_STATEMENT = 11;
pub const STATEMENT_TRY_STATEMENT = 12;
pub const STATEMENT_DEBUGGER_STATEMENT = 13;

pub const StatementOrDeclaration = EXTERN_UNION(
    extern union {
        statement: *Statement,
        declaration: *Declaration,
    },
);

pub const STATEMENT_OR_DECLARATION_STATEMENT = 0;
pub const STATEMENT_OR_DECLARATION_DECLARATION = 1;

pub const Declaration = EXTERN_UNION(
    extern union {
        // hoistable
        function_declaration: *HoistableDeclaration,
        generator_declaration: *HoistableDeclaration,
        async_function_declaration: *HoistableDeclaration,
        async_generator_declaration: *HoistableDeclaration,

        class_declaration: *ClassDeclaration,

        lexical_declaration: *LexicalDeclaration,
    },
);

pub const DECLARATION_FUNCTION_DECLARATION = 0;
pub const DECLARATION_GENERATOR_DECLARATION = 1;
pub const DECLARATION_ASYNC_FUNCTION_DECLARATION = 2;
pub const DECLARATION_ASYNC_GENERATOR_DECLARATION = 3;
pub const DECLARATION_CLASS_DECLARATION = 4;
pub const DECLARATION_LEXICAL_DECLARATION = 5;

pub const FormalParameter = extern struct {
    name: *IdentifierNameData,
    initializer: *MaybeAssignmentExpression,
    is_rest: *bool,
};

pub const HoistableDeclaration = extern struct {
    name: *MaybeIdentifier,
    params: []FormalParameter,
    body: *BlockStatement,
};

// TODO: ClassDeclaration
pub const ClassDeclaration = extern struct {};

pub const LexicalBinding = extern struct {
    name: *IdentifierNameData,
    initializer: *MaybeAssignmentExpression,
};

pub const LexicalDeclaration = extern struct {
    is_const: *bool,
    declarations: []LexicalBinding,
};

pub const BlockStatement = extern struct {
    body: []StatementOrDeclaration,
};

pub const VarStatement = extern struct {
    declarations: []LexicalBinding,
};

pub const IfStatement = extern struct {
    test_: *expr.Expression,
    consequent: *Statement,
    alternate: *MaybeStatement,
};

pub const WhileStatement = extern struct {
    test_: *expr.Expression,
    body: *Statement,
};

pub const ForStatementInit = EXTERN_UNION(
    extern union {
        var_statement: *VarStatement,
        decl: *LexicalDeclaration,

        expr: *expr.Expression,
    },
);

pub const FOR_STATEMENT_INIT_VAR_STATEMENT = 0;
pub const FOR_STATEMENT_INIT_DECL = 1;
pub const FOR_STATEMENT_INIT_EXPR = 2;

pub const MaybeForStatementInit = MAYBE(ForStatementInit);

pub const ForStatement = extern struct {
    init: *MaybeForStatementInit,
    test_: *MaybeExpression,
    update: *MaybeExpression,
    body: *Statement,
};

pub const BreakableStatement = EXTERN_UNION(
    extern union {
        // iteration
        do_while: *WhileStatement,
        while_: *WhileStatement,
        for_: *ForStatement,

        // TODO: ForInOf, SwitchStatement
    },
);

pub const BREAKABLE_STATEMENT_DO_WHILE = 0;
pub const BREAKABLE_STATEMENT_WHILE = 1;
pub const BREAKABLE_STATEMENT_FOR = 2;

// WARN: This is LEGACY
pub const WithStatement = extern struct {
    object: *expr.Expression,
    body: *Statement,
};

pub const LabelledStatement = extern struct {
    label: *IdentifierNameData,
    body: *Statement,
};

pub const CatchClause = extern struct {
    param: *MaybeIdentifier,
    body: *BlockStatement,
};

pub const MaybeCatchClause = MAYBE(CatchClause);

pub const TryStatement = extern struct {
    block: *BlockStatement,
    catch_clause: *MaybeCatchClause,
    finally: *MaybeBlock,
};
