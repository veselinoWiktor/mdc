//! Three-address code(TAC)-KY makes it fancier.
//!
//! TACKY ASDL definition:
//! ```
//! program = Program(function_definition)
//! function_definition = Function(identifier, instruction* body)
//! instruction = Return(val)
//!  | Unary(unary_operator, val src, val dst)
//!  | Binary(binary_operator, val src1, val src2, val dst)
//! val = Constant(int) | Var(identifier)
//! unary_operator = Complement | Negate
//! binary_operator = Add | Subtract | Multiply | Divide | Remainder
//! ```

/// ```program = Program(function_definition)```
#[derive(Debug, PartialEq, Clone)]
pub enum Program {
    Program(FunctionDefinition)
}

/// ```function_definition = Function(identifier, instruction* body)```
#[derive(Debug, PartialEq, Clone)]
pub enum FunctionDefinition {
    Function(String, Vec<Instruction>)
}

/// ```
/// instruction = Return(val)
///             | Unary(unary_operator, val src, val dst)
///             | Binary(binary_operator, val src1, val src2, val dst)
///             | Copy(val src, val dst)
///             | Jump(identifier target)
///             | JumpIfZero(val condition, identifier target)
///             | JumpIfNotZero(val condition, identifier target)
///             | Label(identifier)
/// ```
/// `dst` should be Val::Var
#[derive(Debug, PartialEq, Clone)]
pub enum Instruction {
    Return(Val),
    Unary(UnaryOp, Val, Val),
    Binary(BinaryOp, Val, Val, Val),
    Copy(Val, Val),
    Jump(String),
    JumpIfZero(Val, String),
    JumpIfNotZero(Val, String),
    Label(String)
}

/// ```val = Constant(int) | Var(identifier)```
#[derive(Debug, PartialEq, Clone)]
pub enum Val {
    Constant(i32),
    Var(String)
}

/// ```unary_operator = Complement | Negate```
#[derive(Debug, PartialEq, Clone)]
pub enum UnaryOp {
    Complement,
    Negate,
    Not
}

/// ```binary_operator = Add | Subtract | Multiply | Divide | Remainder```
#[derive(Debug, PartialEq, Clone)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    Equal,
    NotEqual,
    LessThan,
    LessOrEqual,
    GreaterThan,
    GreaterOrEqual,
}