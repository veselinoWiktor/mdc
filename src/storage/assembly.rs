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
    Ret
}

#[derive(Debug, PartialEq)]
pub enum AssemblyOperand {
    Imm(i32),
    Register()
}