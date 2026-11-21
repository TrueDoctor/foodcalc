use crate::entities::types::amount::Amount;

use bigdecimal::{BigDecimal, FromPrimitive, ParseBigDecimalError, ToPrimitive};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Serialize, Deserialize, Default, sqlx::Type)]
#[sqlx(transparent)]
pub struct Money(BigDecimal);

impl From<BigDecimal> for Money {
    fn from(value: BigDecimal) -> Self {
        Money(value)
    }
}

impl From<i32> for Money {
    fn from(value: i32) -> Self {
        Money(BigDecimal::from(value))
    }
}

impl From<i64> for Money {
    fn from(value: i64) -> Self {
        Money(BigDecimal::from(value))
    }
}

impl From<u64> for Money {
    fn from(value: u64) -> Self {
        Money(BigDecimal::from(value))
    }
}

impl From<usize> for Money {
    fn from(value: usize) -> Self {
        Money(BigDecimal::from(value as u64))
    }
}

impl FromStr for Money {
    type Err = ParseBigDecimalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Money(BigDecimal::from_str(s)?))
    }
}


use std::ops::{Add, Div, Mul, Sub};

macro_rules! impl_money_op {
    ($trait:ident, $method:ident) => {
        impl $trait<Money> for Money {
            type Output = Money;

            #[inline]
            fn $method(self, rhs: Money) -> Money {
                Money($trait::$method(self.0, rhs.0))
            }
        }

        impl $trait<&Money> for Money {
            type Output = Money;

            #[inline]
            fn $method(self, rhs: &Money) -> Money {
                Money($trait::$method(self.0, &rhs.0))
            }
        }

        impl $trait<Money> for &Money {
            type Output = Money;

            #[inline]
            fn $method(self, rhs: Money) -> Money {
                Money($trait::$method(&self.0, rhs.0))
            }
        }

        impl $trait<&Money> for &Money {
            type Output = Money;

            #[inline]
            fn $method(self, rhs: &Money) -> Money {
                Money($trait::$method(&self.0, &rhs.0))
            }
        }
    };
}

impl_money_op!(Add, add);
impl_money_op!(Sub, sub);
impl_money_op!(Mul, mul);
impl_money_op!(Div, div);
impl Money {
    pub fn get(&self) -> &BigDecimal {
        &self.0
    }

    /// Constructs from a raw numeric value — only for boundary code
    /// (DB plumbing, external imports, tests).
    pub fn from_raw(value: BigDecimal) -> Self {
        Money(value)
    }

    /// Consumes into the raw numeric value — only for DB plumbing /
    /// formatting at the boundaries.
    pub fn into_raw(self) -> BigDecimal {
        self.0
    }

    /// The additive identity.
    pub fn zero() -> Self {
        Money(BigDecimal::from(0))
    }
}

impl std::fmt::Display for Money {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::ops::AddAssign for Money {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl std::ops::SubAssign for Money {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl std::ops::Neg for Money {
    type Output = Self;
    fn neg(self) -> Self {
        Money(-self.0)
    }
}

impl std::iter::Sum for Money {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        Money(iter.map(|m| m.0).sum())
    }
}

/// Numeric conversions delegate to the inner `BigDecimal`, so code that
/// needs an `f64`/`i64` view keeps working through
/// `ToPrimitive`/`FromPrimitive`.
impl ToPrimitive for Money {
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

impl FromPrimitive for Money {
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



/// Scaling by an unscaled raw factor (unit conversions, portion counts,
/// derived ratios). Multiplying two scaled quantities is meaningless, so only
/// the raw numeric type is accepted here.
impl std::ops::Mul<BigDecimal> for Money {
    type Output = Self;
    fn mul(self, rhs: BigDecimal) -> Self {
        Money(self.0 * rhs)
    }
}

impl std::ops::Div<BigDecimal> for Money {
    type Output = Self;
    fn div(self, rhs: BigDecimal) -> Self {
        Money(self.0 / rhs)
    }
}

impl std::ops::Div<i64> for Money {
    type Output = Self;
    fn div(self, rhs: i64) -> Self {
        Money(self.0 / BigDecimal::from(rhs))
    }
}

/// Scaling a unit price (€/kg) by a physical quantity (kg) yields money.
/// This is the one deliberate bridge between the two types.
impl std::ops::Mul<&Amount> for Money {
    type Output = Money;
    fn mul(self, rhs: &Amount) -> Money {
        Money(self.0 * rhs.get().clone())
    }
}

impl std::ops::Mul<&Amount> for &Money {
    type Output = Money;
    fn mul(self, rhs: &Amount) -> Money {
        Money(self.0.clone() * rhs.get().clone())
    }
}

impl std::ops::Mul<i64> for Money {
    type Output = Self;
    fn mul(self, rhs: i64) -> Self {
        Money(self.0 * rhs)
    }
}

impl Money {
    /// Rounds to `scale` decimal places (delegates to `BigDecimal::round`).
    pub fn round(&self, scale: i64) -> Self {
        Money(self.0.round(scale))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_money_from_i64() {
        let money = Money::from(100);
        assert_eq!(money.0, BigDecimal::from(100));
    }

    #[test]
    fn test_money_from_str() {
        let money: Money = "12345678901234567890".parse().unwrap();
        assert_eq!(money.0, BigDecimal::parse_bytes(b"12345678901234567890", 10).unwrap());
    }
}

