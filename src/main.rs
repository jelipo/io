use anyhow::Result;

use crate::aio::my_aio;
use crate::epoll::my_epoll;
use crate::io_uring::my_io_uring;

mod poll;
mod aio;
mod epoll;
mod io_uring;

fn main() -> Result<()> {
    let listen_addr = "0.0.0.0:10086";
    //my_poll(listen_addr)?;
    // my_epoll(listen_addr)?;
    // my_aio()?;
    my_io_uring(listen_addr)?;
    Ok(())
}
