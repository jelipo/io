use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::os::fd::{AsFd, AsRawFd, FromRawFd, RawFd};
use std::thread::sleep;
use std::time::Duration;
use anyhow::Result;
use nix::sys::epoll::{Epoll, EpollCreateFlags, EpollEvent, EpollFlags, EpollTimeout};

pub fn my_epoll(addr: &str) -> Result<()> {
    // 创建 socket
    let listener = TcpListener::bind(addr)?;
    let (mut stream, _) = listener.accept()?;
    let stream_clone = stream.try_clone()?;
    let fd = stream_clone.as_fd();

    // 对应Linux的epoll_create1
    let epoll = Epoll::new(EpollCreateFlags::EPOLL_CLOEXEC)?;

    // 添加 socket 到 epoll 监控
    let mut event = EpollEvent::new(EpollFlags::EPOLLIN, fd.as_raw_fd() as u64);
    // 对应 epoll_ctl
    epoll.add(fd, event)?;

    let mut event_buf = [EpollEvent::empty(); 10];  // 处理最多 10 个事件

    let mut buffer = vec![0u8; 4096];

    loop {

        // 这里的sleep，假装自己去处理别的数据了，不阻塞等着网络
        sleep(Duration::from_secs(1));
        
        // 不阻塞等待
        let num_events = epoll.wait(&mut event_buf, EpollTimeout::ZERO)?;
        if num_events == 0 {
            println!("没有事件");
            continue;
        }
        for i in 0..num_events {
            let event = event_buf[i];
            let fd_data = event.data();
            // data可以放任何u64数据，这里放了fd，然后恢复成TcpStream
            let mut stream_new = unsafe { TcpStream::from_raw_fd(fd_data as RawFd) };
            match stream_new.read(&mut buffer) {
                Ok(0) => {
                    println!("连接关闭");
                    return Ok(());
                }
                Ok(n) => {
                    println!("读取到了{}字节", n);
                }
                Err(_) => {
                    println!("退出");
                    return Ok(());
                }
            }
        }
    }

    Ok(())
}
