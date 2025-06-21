use super::{Drop, Eat, EatMany};

pub struct Digit(u32);

impl Eat<&str, (), ()> for Digit {
    fn eat(i: &str, _data: ()) -> Result<(&str, Self), ()> {
        let (i, c) = char::eat(i, ())?;
        let digit = c.to_digit(10).ok_or(())?;
        Ok((i, Digit(digit)))
    }
}

impl Eat<&str, (), ()> for u32 {
    fn eat(i: &str, _data: ()) -> Result<(&str, Self), ()> {
        let (i, digits) = Digit::eat_many(i, ());
        if digits.is_empty() {
            return Err(());
        }
        let n = digits.into_iter().fold(0, |r, digit| r * 10 + digit.0);
        Ok((i, n))
    }
}

impl Eat<&str, (), ()> for u64 {
    fn eat(i: &str, _data: ()) -> Result<(&str, Self), ()> {
        let (i, digits) = Digit::eat_many(i, ());
        if digits.is_empty() {
            return Err(());
        }
        let n = digits
            .into_iter()
            .fold(0, |r, digit| r * 10 + digit.0 as u64);
        Ok((i, n))
    }
}

pub struct Sign(char);

impl Eat<&str, (), ()> for Sign {
    fn eat(i: &str, _data: ()) -> Result<(&str, Self), ()> {
        let (i, s) = {
            if let Ok(i) = '-'.drop(i) {
                (i, '-')
            } else if let Ok(i) = '+'.drop(i) {
                (i, '+')
            } else {
                (i, '+')
            }
        };
        Ok((i, Sign(s)))
    }
}

pub struct Float(String);

impl Eat<&str, (), ()> for Float {
    fn eat(i: &str, _data: ()) -> Result<(&str, Self), ()> {
        let (i, sign) = Sign::eat(i, ())?;
        let (i, digits) = Digit::eat_many(i, ());
        let before: String = digits
            .iter()
            .map(|d| std::char::from_digit(d.0, 10).unwrap())
            .collect();
        let (i, after) = if let Ok(i) = '.'.drop(i) {
            let (i, digits) = Digit::eat_many(i, ());
            let after: String = digits
                .iter()
                .map(|d| std::char::from_digit(d.0, 10).unwrap())
                .collect();
            (i, after)
        } else {
            (i, "0".to_string())
        };
        let f = format!("{}{}.{}", sign.0, before, after);
        Ok((i, Float(f)))
    }
}

impl Eat<&str, (), ()> for f32 {
    fn eat(i: &str, _data: ()) -> Result<(&str, Self), ()> {
        let (i, f) = Float::eat(i, ())?;
        let f = f.0.parse::<f32>().map_err(|_| ())?;
        Ok((i, f))
    }
}

impl Eat<&str, (), ()> for f64 {
    fn eat(i: &str, _data: ()) -> Result<(&str, Self), ()> {
        let (i, f) = Float::eat(i, ())?;
        let f = f.0.parse::<f64>().map_err(|_| ())?;
        Ok((i, f))
    }
}
