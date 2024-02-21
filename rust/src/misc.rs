use std::str::FromStr;
use derive_more::Display;

#[derive(Debug, PartialEq, Display)]
pub struct U5(u8);

impl FromStr for U5 {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = s.parse().map_err(|_| "Invalid number")?;
        if value < 32 {
            Ok(U5(value))
        } else {
            Err("Number must be less than 32")
        }
    }
}

impl From<U5> for i32 {
    fn from(u5: U5) -> i32 {
        u5.0 as i32
    }
}

impl From<&U5> for i32 {
    fn from(u5: &U5) -> i32 {
        u5.0 as i32
    }
}