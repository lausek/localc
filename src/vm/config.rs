pub struct VmConfig
{
    optimize: bool,
}

impl VmConfig
{
    pub fn new() -> Self
    {
        Self { optimize: true }
    }

    pub fn is_optimizing(&self) -> bool
    {
        self.optimize
    }
}
