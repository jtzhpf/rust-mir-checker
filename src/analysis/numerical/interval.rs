use rug::Integer;
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt;
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Shl, Shr, Sub};

/// Either `∞`, `-∞`, or an arbitrary precision integer
#[derive(Clone, Eq, PartialEq)]
pub enum Bound {
    INF,          // Positive infinity
    Int(Integer), // Arbitrary precision integer
    NINF,         // Negative infinity
}

pub enum Sign {
    POSITIVE,
    NEGATIVE,
    ZERO,
}

use Bound::*;

impl Bound {
    pub fn is_finite(&self) -> bool {
        matches!(self, Int(_))
    }
}

impl fmt::Debug for Bound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            INF => String::from("∞"),
            NINF => String::from("-∞"),
            Int(n) => n.to_string(),
        };
        write!(f, "{}", value)
    }
}

impl Ord for Bound {
    fn cmp(&self, other: &Self) -> Ordering {
        if self == other {
            Ordering::Equal
        } else {
            match (self, other) {
                (INF, _) | (_, NINF) => Ordering::Greater,
                (NINF, _) | (_, INF) => Ordering::Less,
                (Int(a), Int(b)) => a.cmp(b),
            }
        }
    }
}

impl PartialOrd for Bound {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl From<Integer> for Bound {
    fn from(n: Integer) -> Self {
        Bound::Int(n)
    }
}

impl From<i128> for Bound {
    fn from(n: i128) -> Self {
        Bound::Int(Integer::from(n))
    }
}

impl From<u128> for Bound {
    fn from(n: u128) -> Self {
        Bound::Int(Integer::from(n))
    }
}

impl Add for Bound {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match (self, other) {
            // FIXME: Here, `INF + NINF` = INF, does it matter?
            (INF, _) | (_, INF) => Self::INF,
            (NINF, _) | (_, NINF) => Self::NINF,
            (Int(a), Int(b)) => Self::Int(a + b),
        }
    }
}

impl Sub for Bound {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        match (self, other) {
            // FIXME: Here `INF - INF = INF`, `NINF - NINF = INF`, does it matter?
            (INF, _) | (_, NINF) => Self::INF,
            (NINF, _) | (_, INF) => Self::NINF,
            (Int(a), Int(b)) => Self::Int(a - b),
        }
    }
}

impl Mul for Bound {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        use Sign::*;

        let sign_lhs = match self {
            INF => POSITIVE,
            NINF => NEGATIVE,
            Int(ref n) if *n > 0 => POSITIVE,
            Int(ref n) if *n < 0 => NEGATIVE,
            Int(_) => ZERO,
        };
        let sign_rhs = match rhs {
            INF => POSITIVE,
            NINF => NEGATIVE,
            Int(ref n) if *n > 0 => POSITIVE,
            Int(ref n) if *n < 0 => NEGATIVE,
            Int(_) => ZERO,
        };
        let sign = match (sign_lhs, sign_rhs) {
            (ZERO, _) | (_, ZERO) => ZERO,
            (POSITIVE, POSITIVE) | (NEGATIVE, NEGATIVE) => POSITIVE,
            (POSITIVE, NEGATIVE) | (NEGATIVE, POSITIVE) => NEGATIVE,
        };
        match (self, rhs) {
            (INF, _) | (_, INF) | (NINF, _) | (_, NINF) => match sign {
                POSITIVE => INF,
                NEGATIVE => NINF,
                ZERO => Int(Integer::from(0)),
            },
            (Int(a), Int(b)) => Self::Int(a * b),
        }
    }
}

impl Div for Bound {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        use Sign::*;

        let sign_lhs = match self {
            INF => POSITIVE,
            NINF => NEGATIVE,
            Int(ref n) if *n > 0 => POSITIVE,
            Int(ref n) if *n < 0 => NEGATIVE,
            Int(_) => ZERO,
        };
        let sign_rhs = match rhs {
            INF => POSITIVE,
            NINF => NEGATIVE,
            Int(ref n) if *n > 0 => POSITIVE,
            Int(ref n) if *n < 0 => NEGATIVE,
            Int(_) => ZERO,
        };
        let sign = match (sign_lhs, sign_rhs) {
            (ZERO, _) => ZERO,
            (POSITIVE, POSITIVE) | (NEGATIVE, NEGATIVE) => POSITIVE,
            (POSITIVE, NEGATIVE) | (NEGATIVE, POSITIVE) => NEGATIVE,
            (_, ZERO) => panic!("Division by zero"),
        };
        match (self, rhs) {
            (INF, _) | (NINF, _) => match sign {
                POSITIVE => INF,
                NEGATIVE => NINF,
                ZERO => Int(Integer::from(0)),
            },
            (_, INF) | (_, NINF) => Int(Integer::from(0)),
            (Int(a), Int(b)) => Self::Int(a / b),
        }
    }
}

/// Abstract value that represents an interval
/// When `low` <= `high`, it is a normal interval `[low, high]`
/// When `low` == `NINF` && `high` == `INF`, it is `[-∞, ∞]`
/// When `low` == `INF` && `high` == `NINF`, it is `⊥`
#[derive(Clone, PartialEq)]
pub struct Interval {
    pub high: Bound,
    pub low: Bound,
}

impl fmt::Debug for Interval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_bottom() {
            write!(f, "⊥")
        } else {
            write!(f, "[{:?}, {:?}]", self.low, self.high)
        }
    }
}

impl Interval {
    const INF: Bound = Bound::INF;
    const NINF: Bound = Bound::NINF;

    pub fn new(low: Bound, high: Bound) -> Self {
        Interval { high: high, low: low }
    }

    pub fn top() -> Self {
        Interval {
            high: Self::INF,
            low: Self::NINF,
        }
    }

    pub fn bottom() -> Self {
        Interval {
            high: Self::NINF,
            low: Self::INF,
        }
    }

    pub fn is_top(&self) -> bool {
        self.high == Self::INF && self.low == Self::NINF
    }

    pub fn is_bottom(&self) -> bool {
        self.high < self.low
    }

    // FIXME: When will use this? What's the purpose of these code?
    pub fn less_than(&self, other: &Interval) -> Option<bool> {
        if self.is_bottom() || self.is_top() || other.is_bottom() || other.is_top() {
            None
        } else if self.high < other.low {
            Some(true)
        } else if other.high <= self.low {
            Some(false)
        } else {
            None
        }
    }

    pub fn less_equal(&self, other: &Interval) -> Option<bool> {
        if self.is_bottom() || self.is_top() || other.is_bottom() || other.is_top() {
            None
        } else if self.high <= other.low {
            Some(true)
        } else if other.high < self.low {
            Some(false)
        } else {
            None
        }
    }

    pub fn greater_equal(&self, other: &Interval) -> Option<bool> {
        if self.is_bottom() || self.is_top() || other.is_bottom() || other.is_top() {
            None
        } else if self.low >= other.high {
            Some(true)
        } else if other.low > self.high {
            Some(false)
        } else {
            None
        }
    }

    pub fn greater_than(&self, other: &Interval) -> Option<bool> {
        if self.is_bottom() || self.is_top() || other.is_bottom() || other.is_top() {
            None
        } else if self.low > other.high {
            Some(true)
        } else if other.low >= self.high {
            Some(false)
        } else {
            None
        }
    }

    pub fn equal_to(&self, other: &Interval) -> Option<bool> {
        if let (Ok(v1), Ok(v2)) = (Integer::try_from(self), Integer::try_from(other)) {
            if v1 == v2 {
                return Some(true);
            } else {
                return Some(false);
            }
        }
        None
    }

    pub fn not_equal_to(&self, other: &Interval) -> Option<bool> {
        if let Some(bool_value) = self.equal_to(other) {
            return Some(!bool_value);
        }
        None
    }
    // until here

    fn is_zero(&self) -> bool {
        self.low == Bound::Int(Integer::from(0)) && self.high == Bound::Int(Integer::from(0))
    }

    fn all_ones(&self) -> bool {
        self.low == Bound::Int(Integer::from(-1)) && self.high == Bound::Int(Integer::from(-1))
    }
}

impl TryFrom<Interval> for Integer {
    type Error = &'static str;
    fn try_from(value: Interval) -> Result<Self, Self::Error> {
        if let (Bound::Int(high), Bound::Int(low)) = (value.high, value.low) {
            if high == low {
                return Ok(high);
            }
        }
        Err("interval is not a integer")
    }
}

// FIXME: Never used code
impl TryFrom<&Interval> for Integer {
    type Error = &'static str;
    fn try_from(value: &Interval) -> Result<Self, Self::Error> {
        if let (Bound::Int(high), Bound::Int(low)) = (value.high.clone(), value.low.clone()) {
            if high == low {
                return Ok(high);
            }
        }
        Err("interval is not a integer")
    }
}

impl TryFrom<Interval> for bool {
    type Error = &'static str;
    fn try_from(value: Interval) -> Result<Self, Self::Error> {
        if value.high == Bound::Int(Integer::from(1)) && value.low == Bound::Int(Integer::from(1)) {
            Ok(true)
        } else if value.high == Bound::Int(Integer::from(0))
            && value.low == Bound::Int(Integer::from(0))
        {
            Ok(false)
        } else {
            Err("interval is not a bool")
        }
    }
}
// until here

impl From<bool> for Interval {
    fn from(b: bool) -> Self {
        let v;
        if b {
            v = Bound::Int(Integer::from(1));
        } else {
            v = Bound::Int(Integer::from(0));
        }
        Interval {
            high: v.clone(),
            low: v,
        }
    }
}

// FIXME: unused code
impl Add for Interval {
    type Output = Interval;

    fn add(self, other: Interval) -> Interval {
        let low = self.low + other.low;
        let high = self.high + other.high;
        Interval::new(low, high)
    }
}

impl Sub for Interval {
    type Output = Interval;

    fn sub(self, other: Interval) -> Interval {
        let low = self.low - other.high;
        let high = self.high - other.low;
        Interval::new(low, high)
    }
}

impl Mul for Interval {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let a = self.low.clone() * rhs.low.clone();
        let b = self.low.clone() * rhs.high.clone();
        let c = self.high.clone() * rhs.low.clone();
        let d = self.high * rhs.high;
        let low = [a.clone(), b.clone(), c.clone(), d.clone()]
            .iter()
            .min()
            .unwrap()
            .clone();
        let high = [a, b, c, d].iter().max().unwrap().clone();
        Interval::new(low, high)
    }
}

impl Div for Interval {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        let a = self.low.clone() / rhs.low.clone();
        let b = self.low.clone() / rhs.high.clone();
        let c = self.high.clone() / rhs.low.clone();
        let d = self.high / rhs.high;
        let low = [a.clone(), b.clone(), c.clone(), d.clone()]
            .iter()
            .min()
            .unwrap()
            .clone();
        let high = [a, b, c, d].iter().max().unwrap().clone();
        Interval::new(low, high)
    }
}
// until here

impl BitAnd for Interval {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        if self.is_bottom() || rhs.is_bottom() {
            Self::bottom()
        } else if self.is_top() || rhs.is_top() {
            Self::top()
        } else if self.is_zero() || rhs.is_zero() {
            Self::new(Bound::from(0u128), Bound::from(0u128))
        } else if self.all_ones() {
            rhs
        } else if rhs.all_ones() {
            self
        } else if let (Ok(lval), Ok(rval)) = (Integer::try_from(self), Integer::try_from(rhs)) {
            let and_val = lval & rval;
            Self::new(Bound::from(and_val.clone()), Bound::from(and_val))
        } else {
            Self::top()
        }
    }
}

impl BitOr for Interval {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        if self.is_bottom() || rhs.is_bottom() {
            Self::bottom()
        } else if self.is_top() || rhs.is_top() {
            Self::top()
        } else if self.all_ones() || rhs.all_ones() {
            Self::new(Bound::from(-1i128), Bound::from(-1i128))
        } else if self.is_zero() {
            rhs
        } else if rhs.is_zero() {
            self
        } else if let (Ok(lval), Ok(rval)) = (Integer::try_from(self), Integer::try_from(rhs)) {
            let or_val = lval | rval;
            Self::new(Bound::from(or_val.clone()), Bound::from(or_val))
        } else {
            Self::top()
        }
    }
}

impl BitXor for Interval {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        if self.is_bottom() || rhs.is_bottom() {
            Self::bottom()
        } else if self.is_top() || rhs.is_top() {
            Self::top()
        } else if self.is_zero() {
            rhs
        } else if rhs.is_zero() {
            self
        } else if let (Ok(lval), Ok(rval)) = (Integer::try_from(self), Integer::try_from(rhs)) {
            let xor_val = lval ^ rval;
            Self::new(Bound::from(xor_val.clone()), Bound::from(xor_val))
        } else {
            Self::top()
        }
    }
}

impl Shl for Interval {
    type Output = Self;

    fn shl(self, rhs: Self) -> Self::Output {
        if self.is_bottom() || rhs.is_bottom() {
            Self::bottom()
        } else if self.is_top() || rhs.is_top() {
            Self::top()
        } else if let (Ok(lval), Ok(rval)) = (Integer::try_from(self), Integer::try_from(rhs)) {
            if let Some(u32val) = rval.to_u32() {
                let val = lval << u32val;
                Self::new(Bound::from(val.clone()), Bound::from(val))
            } else {
                Self::top()
            }
        } else {
            Self::top()
        }
    }
}

impl Shr for Interval {
    type Output = Self;

    fn shr(self, rhs: Self) -> Self::Output {
        if self.is_bottom() || rhs.is_bottom() {
            Self::bottom()
        } else if self.is_top() || rhs.is_top() {
            Self::top()
        } else if let (Ok(lval), Ok(rval)) = (Integer::try_from(self), Integer::try_from(rhs)) {
            if let Some(u32val) = rval.to_u32() {
                let val = lval >> u32val;
                Self::new(Bound::from(val.clone()), Bound::from(val))
            } else {
                Self::top()
            }
        } else {
            Self::top()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integer_cmp() {
        let ninf = Bound::NINF;
        let a = Bound::from(-1 as i128);
        let b = Bound::from(0 as i128);
        let c = Bound::from(1 as i128);
        let inf = Bound::INF;
        assert!(ninf < a && a < b && b < c && c < inf);
    }

    #[test]
    fn test_is_finite() {
        let inf = Bound::INF;
        let ninf = Bound::NINF;
        let int = Bound::from(42i128);

        assert!(!inf.is_finite());
        assert!(!ninf.is_finite());
        assert!(int.is_finite());
    }

    #[test]
    fn test_debug_fmt() {
        let inf = Bound::INF;
        let ninf = Bound::NINF;
        let int = Bound::from(42i128);
    
        assert_eq!(format!("{:?}", inf), "∞");
        assert_eq!(format!("{:?}", ninf), "-∞");
        assert_eq!(format!("{:?}", int), "42");
    }

    #[test]
    fn test_from() {
        let int_bound = Bound::from(42i128);
        assert_eq!(int_bound, Bound::Int(Integer::from(42)));

        let i128_bound = Bound::from(-9223372036854775808i128);
        assert_eq!(i128_bound, Bound::Int(Integer::from(-9223372036854775808i128)));

        let u128_bound = Bound::from(18446744073709551615u128);
        assert_eq!(u128_bound, Bound::Int(Integer::from(18446744073709551615u128)));
    }

    #[test]
    fn test_bound_addition() {
        let ninf = Bound::NINF;
        let a = Bound::from(-1i128);
        let b = Bound::from(0i128);
        let c = Bound::from(1i128);
        let inf = Bound::INF;
        assert_eq!(ninf.clone() + ninf.clone(), ninf);
        assert_eq!(ninf.clone() + a.clone(), ninf);
        assert_eq!(ninf.clone() + b.clone(), ninf);
        assert_eq!(ninf.clone() + c.clone(), ninf);
        assert_eq!(ninf.clone() + inf.clone(), inf);
        assert_eq!(a.clone() + ninf.clone(), ninf);
        assert_eq!(a.clone() + a.clone(), Bound::from(-2i128));
        assert_eq!(a.clone() + b.clone(), Bound::from(-1i128));
        assert_eq!(a.clone() + c.clone(), Bound::from(0i128));
        assert_eq!(a.clone() + inf.clone(), inf);
        assert_eq!(b.clone() + ninf.clone(), ninf);
        assert_eq!(b.clone() + a.clone(), Bound::from(-1i128));
        assert_eq!(b.clone() + b.clone(), b);
        assert_eq!(b.clone() + c.clone(), Bound::from(1i128));
        assert_eq!(b.clone() + inf.clone(), inf);
        assert_eq!(c.clone() + ninf.clone(), ninf);
        assert_eq!(c.clone() + a.clone(), Bound::from(0i128));
        assert_eq!(c.clone() + b.clone(), Bound::from(1i128));
        assert_eq!(c.clone() + c.clone(), Bound::from(2i128));
        assert_eq!(c.clone() + inf.clone(), inf);
        assert_eq!(inf.clone() + ninf.clone(), inf);
        assert_eq!(inf.clone() + a.clone(), inf);
        assert_eq!(inf.clone() + b.clone(), inf);
        assert_eq!(inf.clone() + c.clone(), inf);
        assert_eq!(inf.clone() + inf.clone(), inf);
    }

    #[test]
    fn test_bound_subtraction() {
        let ninf = Bound::NINF;
        let a = Bound::from(-1i128);
        let b = Bound::from(0i128);
        let c = Bound::from(1i128);
        let inf = Bound::INF;
        assert_eq!(ninf.clone() - ninf.clone(), inf);
        assert_eq!(ninf.clone() - a.clone(), ninf);
        assert_eq!(ninf.clone() - b.clone(), ninf);
        assert_eq!(ninf.clone() - c.clone(), ninf);
        assert_eq!(ninf.clone() - inf.clone(), ninf);
        assert_eq!(a.clone() - ninf.clone(), inf);
        assert_eq!(a.clone() - a.clone(), b);
        assert_eq!(a.clone() - b.clone(), a);
        assert_eq!(a.clone() - c.clone(), Bound::from(-2i128));
        assert_eq!(a.clone() - inf.clone(), ninf);
        assert_eq!(b.clone() - ninf.clone(), inf);
        assert_eq!(b.clone() - a.clone(), Bound::from(1i128));
        assert_eq!(b.clone() - b.clone(), b);
        assert_eq!(b.clone() - c.clone(), Bound::from(-1i128));
        assert_eq!(b.clone() - inf.clone(), ninf);
        assert_eq!(c.clone() - ninf.clone(), inf);
        assert_eq!(c.clone() - a.clone(), Bound::from(2i128));
        assert_eq!(c.clone() - b.clone(), Bound::from(1i128));
        assert_eq!(c.clone() - c.clone(), b);
        assert_eq!(c.clone() - inf.clone(), ninf);
        assert_eq!(inf.clone() - ninf.clone(), inf);
        assert_eq!(inf.clone() - a.clone(), inf);
        assert_eq!(inf.clone() - b.clone(), inf);
        assert_eq!(inf.clone() - c.clone(), inf);
        assert_eq!(inf.clone() - inf.clone(), inf);
    }

    #[test]
    fn test_bound_multiplication() {
        let ninf = Bound::NINF;
        let a = Bound::from(-1i128);
        let b = Bound::from(0i128);
        let c = Bound::from(1i128);
        let inf = Bound::INF;
        assert_eq!(ninf.clone() * ninf.clone(), inf);
        assert_eq!(ninf.clone() * a.clone(), inf);
        assert_eq!(ninf.clone() * b.clone(), b);
        assert_eq!(ninf.clone() * c.clone(), ninf);
        assert_eq!(ninf.clone() * inf.clone(), ninf);
        assert_eq!(a.clone() * ninf.clone(), inf);
        assert_eq!(a.clone() * a.clone(), Bound::from(1i128));
        assert_eq!(a.clone() * b.clone(), b);
        assert_eq!(a.clone() * c.clone(), Bound::from(-1i128));
        assert_eq!(a.clone() * inf.clone(), ninf);
        assert_eq!(b.clone() * ninf.clone(), b);
        assert_eq!(b.clone() * a.clone(), b);
        assert_eq!(b.clone() * b.clone(), b);
        assert_eq!(b.clone() * c.clone(), b);
        assert_eq!(b.clone() * inf.clone(), b);
        assert_eq!(c.clone() * ninf.clone(), ninf);
        assert_eq!(c.clone() * a.clone(), Bound::from(-1i128));
        assert_eq!(c.clone() * b.clone(), b);
        assert_eq!(c.clone() * c.clone(), Bound::from(1i128));
        assert_eq!(c.clone() * inf.clone(), inf);
        assert_eq!(inf.clone() * ninf.clone(), ninf);
        assert_eq!(inf.clone() * a.clone(), ninf);
        assert_eq!(inf.clone() * b.clone(), b);
        assert_eq!(inf.clone() * c.clone(), inf);
        assert_eq!(inf.clone() * inf.clone(), inf);
    }
}