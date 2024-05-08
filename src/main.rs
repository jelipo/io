mod poll;

use anyhow::Result;
use crate::poll::my_select;

fn main() -> Result<()> {
    let listen_addr = "0.0.0.0:10086";
    my_poll(listen_addr)?;
    Ok(())
}
