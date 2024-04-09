use kanit_common::error::Result;
use kanit_executor::block;

pub fn teardown_rc() -> Result<()> {
    block(kanit_rc::teardown())?;

    Ok(())
}

pub fn initialize_rc() -> Result<()> {
    block(kanit_rc::start())?;

    Ok(())
}

#[inline]
#[cfg(not(feature = "testing"))]
pub async fn event_rc(ev: Vec<u8>) -> Result<()> {
    kanit_rc::event(ev).await
}
