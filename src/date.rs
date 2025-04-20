//! The [`Date`] type represents in-game dates.

use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;

#[cfg(feature = "hoi4")]
use crate::game::Game;
use crate::token::Token;

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct Date {
    year: i16,
    month: i8,
    day: i8,
    #[cfg(feature = "hoi4")]
    hour: i8,
}

impl Date {
    pub fn new(year: i16, month: i8, day: i8) -> Self {
        Date {
            year,
            month,
            day,
            #[cfg(feature = "hoi4")]
            hour: 1,
        }
    }
}

impl FromStr for Date {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Trailing whitespace is accepted by the engine
        let s = s.trim_end();

        let mut splits = s.split('.');
        let year = splits.next().ok_or(Error)?;
        let month = splits.next().unwrap_or("1");
        let mut day = splits.next().unwrap_or("1");
        #[cfg(feature = "hoi4")]
        let mut hour = splits.next().unwrap_or("1");
        // Error if there is another field, but do allow a trailing dot
        if let Some(next) = splits.next() {
            if !next.is_empty() {
                return Err(Error);
            }
        }
        if day.is_empty() {
            day = "1";
        }
        #[cfg(feature = "hoi4")]
        if hour.is_empty() {
            hour = "1";
        }
        Ok(Date {
            year: year.parse().map_err(|_| Error)?,
            month: month.parse().map_err(|_| Error)?,
            day: day.parse().map_err(|_| Error)?,
            #[cfg(feature = "hoi4")]
            hour: hour.parse().map_err(|_| Error)?,
        })
    }
}

impl TryFrom<&Token> for Date {
    type Error = Error;

    fn try_from(value: &Token) -> Result<Self, Self::Error> {
        value.as_str().parse()
    }
}

impl Display for Date {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        #[cfg(feature = "hoi4")]
        if Game::is_hoi4() {
            return write!(f, "{}.{}.{}.{}", self.year, self.month, self.day, self.hour);
        }
        write!(f, "{}.{}.{}", self.year, self.month, self.day)
    }
}
