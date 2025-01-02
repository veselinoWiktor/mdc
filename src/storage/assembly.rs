#[derive(Debug, PartialEq)]
pub enum AssemblyProgram {
    Program(AssemblyFunctionDefinition)
}

#[derive(Debug, PartialEq)]
pub enum AssemblyFunctionDefinition {
    Function(String, Vec<AssemblyInstruction>)
}

#[derive(Debug, PartialEq)]
pub enum AssemblyInstruction {
    Mov(AssemblyOperand, AssemblyOperand),
    Unary(AssemblyUnaryOp, AssemblyOperand),
    AllocateStack(i32),
    Ret
}

#[derive(Debug, PartialEq)]
pub enum AssemblyUnaryOp {
    Neg,
    Not
}

#[derive(Debug, PartialEq)]
pub enum AssemblyOperand {
    Imm(i32),
    Reg(AssemblyRegister),
    PseudoReg(String),
    Stack(i32)
}

#[derive(Debug, PartialEq)]
pub enum AssemblyRegister {
    AX,
    R10
}