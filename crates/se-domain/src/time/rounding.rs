//! Rounding of integer divisions, used when a time falls between two frames.

/// How a time that falls between two frames is turned into a frame count.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RoundingMode {
    /// Toward negative infinity: the frame being displayed at that instant.
    Floor,
    /// Toward positive infinity: the first frame starting at or after that instant.
    Ceil,
    /// To the closest frame; exact halves go away from zero (like `f64::round`).
    Nearest,
}

impl RoundingMode {
    /// Divides `dividend` by a strictly positive `divisor`, rounding the quotient with this mode.
    ///
    /// Callers guarantee `divisor > 0` (frame durations are always positive) and values far from
    /// `i64` bounds (flicks stay within ±2^53), so no overflow can happen.
    pub(crate) const fn divide(self, dividend: i64, divisor: i64) -> i64 {
        let floor = dividend.div_euclid(divisor);
        let remainder = dividend.rem_euclid(divisor);
        match self {
            Self::Floor => floor,
            Self::Ceil => {
                if remainder == 0 {
                    floor
                } else {
                    floor + 1
                }
            }
            Self::Nearest => {
                // `remainder` is in [0, divisor): round up past the half, and exactly at the half
                // only for non-negative quotients (away from zero).
                let doubled = remainder * 2;
                if doubled > divisor || (doubled == divisor && floor >= 0) {
                    floor + 1
                } else {
                    floor
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RoundingMode::{Ceil, Floor, Nearest};

    #[test]
    fn floor_and_ceil_round_toward_their_infinity() {
        assert_eq!(Floor.divide(7, 2), 3);
        assert_eq!(Floor.divide(-7, 2), -4);
        assert_eq!(Ceil.divide(7, 2), 4);
        assert_eq!(Ceil.divide(-7, 2), -3);
        assert_eq!(Ceil.divide(8, 2), 4);
    }

    #[test]
    fn nearest_sends_halves_away_from_zero() {
        assert_eq!(Nearest.divide(5, 2), 3);
        assert_eq!(Nearest.divide(-5, 2), -3);
        assert_eq!(Nearest.divide(4, 3), 1);
        assert_eq!(Nearest.divide(5, 3), 2);
        assert_eq!(Nearest.divide(-4, 3), -1);
        assert_eq!(Nearest.divide(-5, 3), -2);
        assert_eq!(Nearest.divide(0, 3), 0);
    }
}
