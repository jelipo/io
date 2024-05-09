use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::os::fd::{AsFd, AsRawFd, FromRawFd};
use std::thread::sleep;
use std::time::Duration;

use anyhow::Result;
use nix::poll::{poll, PollFd, PollFlags, PollTimeout};

pub fn my_poll(addr: &str) -> Result<()> {
    let listener = TcpListener::bind(addr)?;
    let (mut stream, _) = listener.accept()?;
    let stream_clone = stream.try_clone()?;
    let fd = stream_clone.as_fd();

    loop {
        // 这里的sleep，假装自己去处理别的数据了，不阻塞等着网络
        sleep(Duration::from_secs(1));
        // 要把批量的fd复制到数组中
        let mut poll_fds = [PollFd::new(fd, PollFlags::POLLIN)];
        // 执行poll
        let i = poll(&mut poll_fds, PollTimeout::ZERO)?;
        if i == 0 {
            // 说明没有新数据
            println!("没有新数据");
            continue;
        }

        // 遍历fd数组，查看哪个有数据
        for poll_fd in &poll_fds {
            if poll_fd.revents().unwrap().contains(PollFlags::POLLIN) {
                // 根据fd恢复对应的TCP
                let mut stream_new = unsafe { TcpStream::from_raw_fd(poll_fd.as_fd().as_raw_fd()) };
                let mut buffer = [0; 1024];
                match stream_new.read(&mut buffer) {
                    Ok(0) => {
                        println!("连接关闭");
                        break;
                    }
                    Ok(n) => println!("读取到了 {} 字节", n),
                    Err(_) => {
                        println!("连接出错");
                        break;
                    }
                };
            }
        }
    }
}
