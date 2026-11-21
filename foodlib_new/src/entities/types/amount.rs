use bigdecimal::{BigDecimal, FromPrimitive, ParseBigDecimalError};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Serialize, Deserialize, Default, sqlx::Type)]
#[sqlx(transparent)]
pub struct Amount(BigDecimal);

impl From<BigDecimal> for Amount {
    fn from(value: BigDecimal) -> Self {
        Amount(value)
    }
}

impl From<i32> for Amount {
    fn from(value: i32) -> Self {
        Amount(BigDecimal::from(value))
    }
}

impl From<i64> for Amount {
    fn from(value: i64) -> Self {
        Amount(BigDecimal::from(value))
    }
}

impl From<u64> for Amount {
    fn from(value: u64) -> Self {
        Amount(BigDecimal::from(value))
    }
}

impl From<usize> for Amount {
    fn from(value: usize) -> Self {
        Amount(BigDecimal::from(value as u64))
    }
}
impl TryFrom<f32> for Amount {
    type Error = &'static str;
    fn try_from(value: f32) -> Result<Self, Self::Error> {
        let decimal = BigDecimal::from_f32(value)
            .ok_or("cannot convert float to decimal")?;

        Ok(Amount(decimal))
    }
}

impl FromStr for Amount {
    type Err = ParseBigDecimalError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let decimal = BigDecimal::from_str(s)?;
        Ok(Amount(decimal))
    }
}

impl Amount {
    pub fn get(&self) -> &BigDecimal {
        &self.0
    }

    /// Constructs from a raw numeric value — only for boundary code
    /// (DB plumbing, external imports, tests).
    pub fn from_raw(value: BigDecimal) -> Self {
        Amount(value)
    }

    /// Consumes into the raw numeric value — only for DB plumbing /
    /// formatting at the boundaries.
    pub fn into_raw(self) -> BigDecimal {
        self.0
    }

    /// The additive identity.
    pub fn zero() -> Self {
        Amount(BigDecimal::from(0))
    }
}

impl std::fmt::Display for Amount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Forward the precision specifier so `{:.2}` keeps working through the
        // newtype (BigDecimal honors it).
        match f.precision() {
            Some(p) => write!(f, "{:.p$}", self.0, p = p),
            None => write!(f, "{}", self.0),
        }
    }
}

impl std::ops::AddAssign for Amount {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl std::ops::SubAssign for Amount {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl std::ops::Neg for Amount {
    type Output = Self;
    fn neg(self) -> Self {
        Amount(-self.0)
    }
}

impl std::iter::Sum for Amount {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        Amount(iter.map(|a| a.0).sum())
    }
}

/// Numeric conversions delegate to the inner `BigDecimal`, so code that
/// needs an `f64`/`i64` view (PDF/typst export, frontend rendering) keeps
/// working through `ToPrimitive`/`FromPrimitive`.
impl bigdecimal::ToPrimitive for Amount {
    fn to_i64(&self) -> Option<i64> {
        self.0.to_i64()
    }
    fn to_u64(&self) -> Option<u64> {
        self.0.to_u64()
    }
    fn to_f64(&self) -> Option<f64> {
        self.0.to_f64()
    }
}

impl bigdecimal::FromPrimitive for Amount {
    fn from_i64(n: i64) -> Option<Self> {
        BigDecimal::from_i64(n).map(Self)
    }
    fn from_u64(n: u64) -> Option<Self> {
        BigDecimal::from_u64(n).map(Self)
    }
    fn from_f64(n: f64) -> Option<Self> {
        BigDecimal::from_f64(n).map(Self)
    }
}


use std::ops::{Add, Div, Mul, Sub};

macro_rules! impl_amount_op {
    ($trait:ident, $method:ident) => {
        impl $trait for Amount {
            type Output = Amount;

            fn $method(self, rhs: Amount) -> Self::Output {
                Amount($trait::$method(self.0, rhs.0))
            }
        }
    };
}

impl_amount_op!(Add, add);
impl_amount_op!(Sub, sub);
impl_amount_op!(Mul, mul);
impl_amount_op!(Div, div);


/// Scaling by an unscaled raw factor (unit conversions, portion counts,
/// derived ratios). Multiplying two scaled quantities is meaningless, so only
/// the raw numeric type is accepted here.
impl std::ops::Mul<BigDecimal> for Amount {
    type Output = Self;
    fn mul(self, rhs: BigDecimal) -> Self {
        Amount(self.0 * rhs)
    }
}

impl std::ops::Div<BigDecimal> for Amount {
    type Output = Self;
    fn div(self, rhs: BigDecimal) -> Self {
        Amount(self.0 / rhs)
    }
}

impl std::ops::Div<i64> for Amount {
    type Output = Self;
    fn div(self, rhs: i64) -> Self {
        Amount(self.0 / BigDecimal::from(rhs))
    }
}

impl std::ops::Mul<i64> for Amount {
    type Output = Self;
    fn mul(self, rhs: i64) -> Self {
        Amount(self.0 * rhs)
    }
}

impl Amount {
    /// Rounds to `scale` decimal places (delegates to `BigDecimal::round`).
    pub fn round(&self, scale: i64) -> Self {
        Amount(self.0.round(scale))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amount_from_i64() {
        let amount = Amount::from(42);
        assert_eq!(amount.0, BigDecimal::from(42));
    }

    #[test]
    fn test_amount_from_str() {
        let amount: Amount = "12345678901234567890".parse().unwrap();
        assert_eq!(amount.0, BigDecimal::from_str("12345678901234567890").unwrap());
    }
}