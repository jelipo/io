use std::os::unix::io::RawFd;

use anyhow::Result;
use nix::errno::Errno;
use nix::fcntl::{OFlag, open};
use nix::sys::aio::*;
use nix::sys::signal::SigevNotify;
use nix::unistd::close;

pub fn my_aio() -> Result<()> {
    let path = "/home/me/.bashrc";

    // 打开文件，准备写入
    let fd: RawFd = open(path, OFlag::O_RDONLY, nix::sys::stat::Mode::S_IRWXU)?;
    let mut buffer = vec![0u8; 512];

    // 等同于 aio_read
    let mut aior = Box::pin(
        AioRead::new(fd, 0, &mut buffer, 0, SigevNotify::SigevNone)
    );
    // 等同于 io_submit
    aior.as_mut().submit()?;
    // 这里假装没有就绪的时候就处理别的任务，直到没有error
    while (aior.as_mut().error() == Err(Errno::EINPROGRESS)) {
        println!("未就绪");
    }
    // aio_return 查看读取了多少
    let size = aior.as_mut().aio_return()?;
    println!("已经就绪，读取了{}个字节", size);
    // 清理资源
    close(fd)?;

    Ok(())
}