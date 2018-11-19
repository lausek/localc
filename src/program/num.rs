use std::num::ParseFloatError;

type InternalNumType = f64;

#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
pub struct Num(InternalNumType);

impl Num
{
    pub const fn new(init: InternalNumType) -> Self
    {
        Num(init)
    }

    pub const fn pi() -> Self
    {
        Self::new(std::f64::consts::PI)
    }

    pub const fn e() -> Self
    {
        Self::new(std::f64::consts::E)
    }

    pub fn as_usize(&self) -> usize
    {
        self.0 as usize
    }

    // FIXME: replace this with trait impl?
    pub fn powf(self, other: Self) -> Self
    {
        Self(self.0.powf(other.0))
    }

    // FIXME: replace this with trait impl?
    pub fn log(self, other: Self) -> Self
    {
        Self(self.0.log(other.0))
    }
}

impl std::ops::Add for Num
{
    type Output = Self;

    fn add(mut self, other: Self) -> Self
    {
        self.0 += other.0;
        self
    }
}

impl std::ops::Sub for Num
{
    type Output = Self;

    fn sub(mut self, other: Self) -> Self
    {
        self.0 -= other.0;
        self
    }
}

impl std::ops::Mul for Num
{
    type Output = Self;

    fn mul(mut self, other: Self) -> Self
    {
        self.0 *= other.0;
        self
    }
}

impl std::ops::Div for Num
{
    type Output = Self;

    fn div(mut self, other: Self) -> Self
    {
        self.0 /= other.0;
        self
    }
}

impl std::ops::Rem for Num
{
    type Output = Self;

    fn rem(mut self, other: Self) -> Self
    {
        self.0 %= other.0;
        self
    }
}

impl std::convert::From<InternalNumType> for Num
{
    fn from(from: InternalNumType) -> Self
    {
        Self(from)
    }
}

impl std::convert::From<Num> for InternalNumType
{
    fn from(from: Num) -> Self
    {
        from.0
    }
}

impl std::str::FromStr for Num
{
    type Err = ParseFloatError;

    fn from_str(from: &str) -> Result<Self, Self::Err>
    {
        let value = from.parse::<InternalNumType>()?;
        Ok(Self(value))
    }
}

impl Default for Num
{
    fn default() -> Self
    {
        Num(InternalNumType::from(0))
    }
}

impl std::fmt::Display for Num
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        write!(f, "{}", self.0)?;
        Ok(())
    }
}
