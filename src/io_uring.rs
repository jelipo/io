use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::os::fd::{AsFd, AsRawFd, FromRawFd, RawFd};
use std::thread::sleep;
use std::time::Duration;

use anyhow::{anyhow, Error, Result};
use io_uring::{IoUring, opcode, types};

pub fn my_io_uring(addr: &str) -> Result<()> {
    let listener = TcpListener::bind(addr)?;
    let (mut stream, _) = listener.accept()?;
    let stream_clone = stream.try_clone()?;
    let fd = stream_clone.as_fd();

    println!("连入连接");

    // 创建 io_uring 实例，指定队列深度为 4，等同于io_uring_setup
    let mut ring = IoUring::new(4)?;
    // 准备缓冲区来接收数据
    let mut buf = vec![0u8; 1024];  // 读取最多 1024 字节

    // 创建一个读取操作
    let read_e = opcode::Read::new(types::Fd(fd.as_raw_fd()), buf.as_mut_ptr(), buf.len() as _)
        .build()
        .user_data(fd.as_raw_fd() as u64);  // 用户数据，用于追踪请求

    unsafe {
        // 获取提交队列的可变引用
        let mut sq = ring.submission();
        sq.push(&read_e).expect("提交读取请求失败");
    }

    // 提交请求并等待至少一个事件完成，等同于 io_uring_enter 
    let submit_size = ring.submit()?;
    println!("提交了{}个", submit_size);

    loop {
        // 这里的sleep，假装自己去处理别的数据了，不阻塞等着网络
        sleep(Duration::from_secs(1));
        let mut cq = ring.completion();
        match cq.next() {
            None => {
                println!("没有就绪的");
                continue;
            }
            Some(cqe) => {
                // 检查操作结果，如果是负数,出现错误
                if cqe.result() < 0 {
                    println!("错误");
                    break;
                }
                // 说明到了末尾
                if cqe.result() == 0 {
                    println!("结尾");
                    break;
                }
                // 确认读取的字节数
                let fd = cqe.user_data() as RawFd;
                // 可以根据fd找到对应的TCP
                let stream = unsafe { TcpStream::from_raw_fd(fd) };
                // 如果是正数，说明是读到的字节数
                println!("读取到了{}字节", cqe.result() as usize);
                // 通知内核 CQE 已经处理
                cq.sync();
            }
        }
    }
    Ok(())
}