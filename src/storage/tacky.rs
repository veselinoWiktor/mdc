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
///  | Unary(unary_operator, val src, val dst)
///  | Binary(binary_operator, val src1, val src2, val dst)
/// ```
/// `dst` should be Val::Var
#[derive(Debug, PartialEq, Clone)]
pub enum Instruction {
    Return(Val),
    Unary(UnaryOp, Val, Val),
    Binary(BinaryOp, Val, Val, Val)
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
    Negate
}

/// ```binary_operator = Add | Subtract | Multiply | Divide | Remainder```
#[derive(Debug, PartialEq, Clone)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder
}