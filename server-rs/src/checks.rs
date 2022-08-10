use std::fmt::Debug;

use anyhow::anyhow;

pub fn require_msg(cond: bool, msg: &str) -> anyhow::Result<()> {
    if !cond {
        Err(anyhow!("{}", msg))
    } else {
        Ok(())
    }
}

pub fn require_eq<T>(lhs: &T, rhs: &T) -> anyhow::Result<()>
where
    T: Debug + PartialEq,
{
    if lhs != rhs {
        Err(anyhow!("Requirement failed: {:?} != {:?}", lhs, rhs))
    } else {
        Ok(())
    }
}

pub fn require(cond: bool) -> anyhow::Result<()> {
    if !cond {
        Err(anyhow!("Requirement failed"))
    } else {
        Ok(())
    }
}
