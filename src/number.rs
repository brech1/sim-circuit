use circom_2_arithc::compiler::AGateType;

pub trait Number: Sized + Clone + PartialEq + Eq {
    fn zero() -> Self;
    fn infix_op(&self, op: AGateType, rhs: &Self) -> Result<Self, NumberError>;
    fn from_str(s: &str) -> Result<Self, NumberError>;
    fn to_string(&self) -> String;
}

#[derive(Debug, Clone)]
pub enum NumberError {
    DivisionByZero,
    UnsupportedOperation(AGateType),
    MissingInput(String),
    ParseError,
}
