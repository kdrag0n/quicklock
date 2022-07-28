use anyhow::anyhow;

pub fn require_msg(cond: bool, msg: &str) -> anyhow::Result<()> {
    if !cond {
        Err(anyhow!("{}", msg))
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
