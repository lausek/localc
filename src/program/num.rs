use std::num::ParseFloatError;

type InternalNumType = f64;

#[derive(Clone, Copy, Debug)]
pub struct Num {
    value: InternalNumType,
}

impl Num 
{
    pub const fn new(init: InternalNumType)
        -> Self
    {
        Num {
            value: init,
        }
    }

    pub const fn pi()
        -> Self
    {
        Num {
            value: std::f64::consts::PI,
        }
    }

    pub const fn e()
        -> Self
    {
        Num {
            value: std::f64::consts::E,
        }
    }

    // FIXME: replace this with trait impl?
    pub fn powf(self, other: Self)
        -> Self
    {
        Self {
            value: self.value.powf(other.value),
        }
    }

    // FIXME: replace this with trait impl?
    pub fn log(self, other: Self)
        -> Self
    {
        Self {
            value: self.value.log(other.value),
        }
    }
}

impl std::ops::Add for Num 
{
    type Output = Self;

    fn add(mut self, other: Self)
        -> Self
    {
        self.value += other.value; 
        self
    }
}

impl std::ops::Sub for Num 
{
    type Output = Self;

    fn sub(mut self, other: Self)
        -> Self
    {
        self.value -= other.value; 
        self
    }
}

impl std::ops::Mul for Num 
{
    type Output = Self;

    fn mul(mut self, other: Self)
        -> Self
    {
        self.value *= other.value; 
        self
    }
}

impl std::ops::Div for Num
{
    type Output = Self;

    fn div(mut self, other: Self)
        -> Self
    {
        self.value /= other.value; 
        self
    }
}

impl std::cmp::PartialEq for Num
{
    fn eq(&self, other: &Self)
        -> bool
    {
        self.value == other.value
    }
}

impl std::convert::From<InternalNumType> for Num
{
    fn from(from: InternalNumType)
        -> Self
    {
        Self {
            value: from,
        }
    }
}

impl std::convert::From<Num> for InternalNumType
{
    fn from(from: Num)
        -> Self
    {
        from.value
    }
}

impl std::str::FromStr for Num
{
    type Err = ParseFloatError;
    
    fn from_str(from: &str)
        -> Result<Self, Self::Err>
    {
        let value = from.parse::<InternalNumType>()?;
        Ok(Self {
            value,
        })
    }
}

impl Default for Num 
{
    fn default() -> Self
    {
        Num {
            value: InternalNumType::from(0),
        }
    }
}

impl std::fmt::Display for Num 
{
    fn fmt(&self, f: &mut std::fmt::Formatter)
        -> std::fmt::Result
    {
        write!(f, "{}", self.value);
        Ok(())
    }
}
