pub mod digit;
pub mod text;
pub mod token;

#[macro_export]
macro_rules! eat {
    ($i:ident, $last:expr, $r:expr) => {
        if let Ok($i) = $last.drop($i) {
            return Ok(($i, $r));
        }
    };
    ($i:ident, $last:expr) => {
        if let Ok($i) = $last.drop($i) {
            return Ok($i);
        }
    };
}

pub trait Eat<Input, Error, Data>
where
    Self: Sized,
{
    fn eat(i: Input, data: Data) -> Result<(Input, Self), Error>;
}

pub trait EatMany<Input, Error, Data>
where
    Self: Sized,
{
    fn eat_many(i: Input, data: Data) -> (Input, Vec<Self>);
}

impl<Input, Error, Data, T> EatMany<Input, Error, Data> for T
where
    T: Eat<Input, Error, Data>,
    Data: Clone,
    Input: Clone,
{
    fn eat_many(mut i: Input, data: Data) -> (Input, Vec<T>) {
        let mut results = vec![];
        while let Ok((new_i, item)) = T::eat(i.clone(), data.clone()) {
            i = new_i;
            results.push(item)
        }
        (i, results)
    }
}

pub trait EatLen<Input, Error, Data>
where
    Self: Sized,
{
    fn eat_len(i: Input, data: Data, len: usize) -> Result<(Input, Vec<Self>), Error>;
}

impl<'a, T> EatLen<&'a [u8], (), ()> for T
where
    T: Eat<&'a [u8], (), ()>,
{
    fn eat_len(mut i: &'a [u8], _data: (), len: usize) -> Result<(&'a [u8], Vec<T>), ()> {
        let mut r = Vec::new();
        for _ in 0..len {
            let (new_i, x) = T::eat(i, ())?;
            i = new_i;
            r.push(x);
        }
        Ok((i, r))
    }
}

pub trait Drop<Input, Error>
where
    Self: Sized,
{
    fn drop(self, i: Input) -> Result<Input, Error>;
}

pub trait DropMany<Input, Error>
where
    Self: Sized,
{
    fn drop_many(self, i: Input) -> (Input, usize);
}

impl<Input, Error, T> DropMany<Input, Error> for T
where
    T: Drop<Input, Error> + Copy,
    Input: Copy,
{
    fn drop_many(self, mut i: Input) -> (Input, usize) {
        let mut count = 0;
        while let Ok(new_i) = self.drop(i) {
            i = new_i;
            count += 1;
        }
        (i, count)
    }
}

pub trait DropLen<Input, Error>
where
    Self: Sized,
{
    fn drop_len(self, i: Input, len: usize) -> Result<Input, Error>;
}

impl<'a, T> DropLen<&'a [u8], ()> for &'a T
where
    &'a T: Drop<&'a [u8], ()>,
{
    fn drop_len(self, mut i: &'a [u8], len: usize) -> Result<&'a [u8], ()> {
        for _ in 0..len {
            i = self.drop(i)?;
        }
        Ok(i)
    }
}

impl Eat<&[u8], (), ()> for u32 {
    fn eat(i: &[u8], _data: ()) -> Result<(&[u8], Self), ()> {
        let (i, b0) = u8::eat(i, ())?;
        let (i, b1) = u8::eat(i, ())?;
        let (i, b2) = u8::eat(i, ())?;
        let (i, b3) = u8::eat(i, ())?;
        let x = u32::from_be_bytes([b0, b1, b2, b3]);
        Ok((i, x))
    }
}

impl<'a, A, B> Eat<&'a [u8], (), ()> for (A, B)
where
    A: Eat<&'a [u8], (), ()>,
    B: Eat<&'a [u8], (), ()>,
{
    fn eat(i: &'a [u8], _data: ()) -> Result<(&'a [u8], Self), ()> {
        let (i, a) = A::eat(i, ())?;
        let (i, b) = B::eat(i, ())?;
        Ok((i, (a, b)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f32() {
        let input = "21.37";
        let result = f32::eat(input, ());
        assert!(result.is_ok());
        let (rest, value) = result.unwrap();
        assert!(rest.is_empty());
        assert_eq!(value, 21.37);
    }

    #[test]
    fn test_f32_no_decimal_point() {
        let input = "+1";
        let result = f32::eat(input, ());
        assert!(result.is_ok());
        let (rest, value) = result.unwrap();
        assert!(rest.is_empty());
        assert_eq!(value, 1.);
    }
}
