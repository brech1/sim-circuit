use circom_2_arithc::compiler::AGateType;

use crate::number::{Number, NumberError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumberU32(pub u32);

impl Number for NumberU32 {
    fn zero() -> Self {
        NumberU32(0)
    }

    fn from_str(s: &str) -> Result<Self, NumberError> {
        match s.parse() {
            Ok(n) => Ok(NumberU32(n)),
            Err(_) => Err(NumberError::ParseError),
        }
    }

    fn to_string(&self) -> String {
        self.0.to_string()
    }

    fn infix_op(&self, op: AGateType, rhs: &Self) -> Result<Self, NumberError> {
        Ok(NumberU32(match op {
            AGateType::AAdd => self.0 + rhs.0,
            AGateType::ADiv => return Err(NumberError::UnsupportedOperation(AGateType::ADiv)),
            AGateType::AEq => (self.0 == rhs.0) as u32,
            AGateType::AGEq => (self.0 >= rhs.0) as u32,
            AGateType::AGt => (self.0 > rhs.0) as u32,
            AGateType::ALEq => (self.0 <= rhs.0) as u32,
            AGateType::ALt => (self.0 < rhs.0) as u32,
            AGateType::AMul => self.0 * rhs.0,
            AGateType::ANeq => (self.0 != rhs.0) as u32,
            AGateType::ASub => self.0 - rhs.0,
            AGateType::AXor => self.0 ^ rhs.0,
            AGateType::APow => self.0.pow(rhs.0),
            AGateType::AIntDiv => {
                if rhs.0 == 0 {
                    return Err(NumberError::DivisionByZero);
                }
                self.0 / rhs.0
            }
            AGateType::AMod => {
                if rhs.0 == 0 {
                    return Err(NumberError::DivisionByZero);
                }
                self.0 % rhs.0
            }
            AGateType::AShiftL => self.0 << rhs.0,
            AGateType::AShiftR => self.0 >> rhs.0,
            AGateType::ABoolOr => (self.0 != 0 || rhs.0 != 0) as u32,
            AGateType::ABoolAnd => (self.0 != 0 && rhs.0 != 0) as u32,
            AGateType::ABitOr => self.0 | rhs.0,
            AGateType::ABitAnd => self.0 & rhs.0,
        }))
    }
}
