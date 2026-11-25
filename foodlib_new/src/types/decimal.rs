//! Shared codegen for the BigDecimal-backed unit newtypes (`Amount`,
//! `Money`).
//!
//! Both types are exact-decimal wrappers with the same mechanical surface:
//! integer/`BigDecimal` conversions, string parsing, transparent serde+sqlx
//! wrapping, precision-forwarding `Display`, `Add`/`SubAssign`, `Neg`, `Sum`,
//! primitive conversions and scaling bridges against raw factors. Only the
//! *type-specific semantics* differ (which self-ops are allowed, cross-type
//! bridges), and those stay in the concrete files.
//!
//! The macro is a plain `macro_rules!` re-exported via `pub(crate) use`; all
//! paths inside the expansion are fully qualified (`::serde`, `::bigdecimal`,
//! …) so it works in any sibling module regardless of local imports.

/// Generates the common BigDecimal-newtype boilerplate.
///
/// `self_ops` lists the operations that make sense *between two values of the
/// type itself* (`Add::add`, `Sub::sub`, …). Anything semantically dubious
/// (e.g. `Money * Money`) is simply left out of the list, which keeps the
/// "meaningless operation" rule enforced by the type system.
macro_rules! decimal_newtype {
    (
        $(#[$meta:meta])*
        $name:ident,
        self_ops { $($op_trait:ident :: $op_method:ident),* $(,)? }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, ::serde::Serialize, ::serde::Deserialize, Default, ::sqlx::Type)]
        #[sqlx(transparent)]
        pub struct $name(::bigdecimal::BigDecimal);

        impl ::std::convert::From<::bigdecimal::BigDecimal> for $name {
            fn from(value: ::bigdecimal::BigDecimal) -> Self {
                $name(value)
            }
        }

        impl ::std::convert::From<i32> for $name {
            fn from(value: i32) -> Self {
                $name(::bigdecimal::BigDecimal::from(value))
            }
        }

        impl ::std::convert::From<i64> for $name {
            fn from(value: i64) -> Self {
                $name(::bigdecimal::BigDecimal::from(value))
            }
        }

        impl ::std::convert::From<u64> for $name {
            fn from(value: u64) -> Self {
                $name(::bigdecimal::BigDecimal::from(value))
            }
        }

        impl ::std::convert::From<usize> for $name {
            fn from(value: usize) -> Self {
                $name(::bigdecimal::BigDecimal::from(value as u64))
            }
        }

        impl ::std::str::FromStr for $name {
            type Err = ::bigdecimal::ParseBigDecimalError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok($name(::std::str::FromStr::from_str(s)?))
            }
        }

        impl $name {
            /// The inner exact decimal value.
            pub fn get(&self) -> &::bigdecimal::BigDecimal {
                &self.0
            }

            /// Consumes into the raw numeric value — only for DB plumbing /
            /// formatting at the boundaries.
            pub fn into_raw(self) -> ::bigdecimal::BigDecimal {
                self.0
            }

            /// The additive identity.
            pub fn zero() -> Self {
                $name(::bigdecimal::BigDecimal::from(0))
            }

            /// True when the value is exactly zero.
            pub fn is_zero(&self) -> bool {
                self.0 == ::bigdecimal::BigDecimal::from(0)
            }

            /// Rounds to `scale` decimal places (delegates to
            /// `BigDecimal::round`).
            pub fn round(&self, scale: i64) -> Self {
                $name(self.0.round(scale))
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                // Forward the precision specifier so `{:.2}` keeps working
                // through the newtype (BigDecimal honors it).
                match f.precision() {
                    ::std::option::Option::Some(p) => ::std::write!(f, "{:.p$}", self.0, p = p),
                    ::std::option::Option::None => ::std::write!(f, "{}", self.0),
                }
            }
        }

        impl ::core::ops::AddAssign for $name {
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }

        impl ::core::ops::SubAssign for $name {
            fn sub_assign(&mut self, rhs: Self) {
                self.0 -= rhs.0;
            }
        }

        impl ::core::ops::Neg for $name {
            type Output = Self;
            fn neg(self) -> Self {
                $name(-self.0)
            }
        }

        impl ::core::iter::Sum for $name {
            fn sum<I: ::std::iter::Iterator<Item = Self>>(iter: I) -> Self {
                $name(iter.map(|v| v.0).sum())
            }
        }

        /// Numeric conversions delegate to the inner `BigDecimal`, so code
        /// that needs an `f64`/`i64` view keeps working through
        /// `ToPrimitive`/`FromPrimitive`.
        impl ::bigdecimal::ToPrimitive for $name {
            fn to_i64(&self) -> ::std::option::Option<i64> {
                self.0.to_i64()
            }
            fn to_u64(&self) -> ::std::option::Option<u64> {
                self.0.to_u64()
            }
            fn to_f64(&self) -> ::std::option::Option<f64> {
                self.0.to_f64()
            }
        }

        impl ::bigdecimal::FromPrimitive for $name {
            fn from_i64(n: i64) -> ::std::option::Option<Self> {
                ::bigdecimal::BigDecimal::from_i64(n).map(Self)
            }
            fn from_u64(n: u64) -> ::std::option::Option<Self> {
                ::bigdecimal::BigDecimal::from_u64(n).map(Self)
            }
            fn from_f64(n: f64) -> ::std::option::Option<Self> {
                ::bigdecimal::BigDecimal::from_f64(n).map(Self)
            }
        }

        /// Scaling by an unscaled raw factor (unit conversions, portion
        /// counts, derived ratios). Multiplying two scaled quantities is
        /// meaningless, so only the raw numeric type is accepted here.
        impl ::core::ops::Mul<::bigdecimal::BigDecimal> for $name {
            type Output = Self;
            fn mul(self, rhs: ::bigdecimal::BigDecimal) -> Self {
                $name(self.0 * rhs)
            }
        }

        impl ::core::ops::Div<::bigdecimal::BigDecimal> for $name {
            type Output = Self;
            fn div(self, rhs: ::bigdecimal::BigDecimal) -> Self {
                $name(self.0 / rhs)
            }
        }

        impl ::core::ops::Div<i64> for $name {
            type Output = Self;
            fn div(self, rhs: i64) -> Self {
                $name(self.0 / ::bigdecimal::BigDecimal::from(rhs))
            }
        }

        impl ::core::ops::Mul<i64> for $name {
            type Output = Self;
            fn mul(self, rhs: i64) -> Self {
                $name(self.0 * rhs)
            }
        }

        $(
            impl ::core::ops::$op_trait<$name> for $name {
                type Output = $name;
                #[inline]
                fn $op_method(self, rhs: $name) -> $name {
                    $name(::core::ops::$op_trait::$op_method(self.0, rhs.0))
                }
            }

            impl ::core::ops::$op_trait<&$name> for $name {
                type Output = $name;
                #[inline]
                fn $op_method(self, rhs: &$name) -> $name {
                    $name(::core::ops::$op_trait::$op_method(self.0, &rhs.0))
                }
            }

            impl ::core::ops::$op_trait<$name> for &$name {
                type Output = $name;
                #[inline]
                fn $op_method(self, rhs: $name) -> $name {
                    $name(::core::ops::$op_trait::$op_method(&self.0, rhs.0))
                }
            }

            impl ::core::ops::$op_trait<&$name> for &$name {
                type Output = $name;
                #[inline]
                fn $op_method(self, rhs: &$name) -> $name {
                    $name(::core::ops::$op_trait::$op_method(&self.0, &rhs.0))
                }
            }
        )*

        #[cfg(test)]
        mod decimal_newtype_tests {
            use super::*;

            #[test]
            fn from_i64_wraps_the_exact_decimal() {
                let value = $name::from(42i64);
                assert_eq!(value.0, ::bigdecimal::BigDecimal::from(42));
            }

            #[test]
            fn parse_round_trips_large_integers() {
                let value: $name = "12345678901234567890".parse().unwrap();
                assert_eq!(
                    value.0,
                    "12345678901234567890".parse::<::bigdecimal::BigDecimal>().unwrap()
                );
            }
        }
    };
}

pub(crate) use decimal_newtype;
