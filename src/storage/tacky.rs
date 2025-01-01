//! Three-address code(TAC)-KY makes it fancier.
//!
//! TACKY ASDL definition:
//! ```
//! program = Program(function_definition)
//! function_definition = Function(identifier, instruction* body)
//! instruction = Return(val) | Unary(unary_operator, val src, val dst)
//! val = Constant(int) | Var(identifier)
//! unary_operator = Complement | Negate
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

/// ```instruction = Return(val) | Unary(unary_operator, val src, val dst)```
/// `dst` should be Val::Var
#[derive(Debug, PartialEq, Clone)]
pub enum Instruction {
    Return(Val),
    Unary(UnaryOp, Val, Val)
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