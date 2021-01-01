pub mod error;
mod socket;

use error::UdtError;
use udt_sys;

use futures::{
    future::Future,
    io::{AsyncRead, AsyncWrite},
    task::{Context, Poll},
};

use std::{
    io::{self, Read, Write},
    net::{SocketAddr, ToSocketAddrs},
    ops::Drop,
    os::raw::c_int,
    pin::Pin,
    ptr, thread, time,
};

pub use socket::{UdtSocket, UdtStatus};

type Result<T> = std::result::Result<T, UdtError>;

pub fn startup() -> Result<()> {
    let result = unsafe { udt_sys::udt_startup() };
    if result == unsafe { udt_sys::UDT_ERROR } {
        error::get_error(())
    } else {
        Ok(())
    }
}

pub fn cleanup() -> Result<()> {
    let result = unsafe { udt_sys::udt_cleanup() };
    if result == unsafe { udt_sys::UDT_ERROR } {
        error::get_error(())
    } else {
        Ok(())
    }
}

pub fn builder() -> UdtBuilder {
    UdtBuilder {
        opt_vec: Vec::new(),
    }
}

pub fn async_builder() -> UdtAsyncBuilder {
    let opt_vec = [UdtSockOpt::RcvSyn(false), UdtSockOpt::SndSyn(false)].to_vec();
    UdtAsyncBuilder { opt_vec }
}

pub struct UdtListener {
    socket: UdtSocket,
}

impl UdtListener {
    pub fn accept(&self) -> Result<(UdtStream, SocketAddr)> {
        let (socket, addr) = self.socket.accept()?;
        Ok((UdtStream { socket }, addr))
    }
    pub fn local_addr(&self) -> Result<SocketAddr> {
        self.socket.local_addr()
    }
}

impl Drop for UdtListener {
    fn drop(&mut self) {
        if let Err(_) = self.socket.close() {}
    }
}

pub struct UdtStream {
    socket: UdtSocket,
}

impl UdtStream {
    pub fn local_addr(&self) -> Result<SocketAddr> {
        self.socket.local_addr()
    }
    pub fn peer_addr(&self) -> Result<SocketAddr> {
        self.socket.peer_addr()
    }
    pub fn close(self) -> Result<()> {
        self.socket.close()
    }
}

impl Read for UdtStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        Ok(self.socket.recv(buf)?)
    }
}

impl Write for UdtStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(self.socket.send(buf)?)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Drop for UdtStream {
    fn drop(&mut self) {
        if let Err(_) = self.socket.close() {}
    }
}

pub struct UdtBoundSocket {
    socket: UdtSocket,
}

impl UdtBoundSocket {
    pub fn connect<A: ToSocketAddrs>(self, remote: A) -> Result<UdtStream> {
        self.socket.connect(remote)?;
        Ok(UdtStream {
            socket: self.socket,
        })
    }
    pub fn local_addr(&self) -> Result<SocketAddr> {
        self.socket.local_addr()
    }
}

pub struct UdtBuilder {
    opt_vec: Vec<UdtSockOpt>,
}

impl UdtBuilder {
    pub fn bind_ipv4<A: ToSocketAddrs>(self, local: A) -> Result<UdtBoundSocket> {
        let socket = UdtSocket::new_ipv4()?;
        self.config_socket(&socket)?;
        let socket = socket.bind(local)?;
        Ok(UdtBoundSocket { socket })
    }
    pub fn bind_ipv6<A: ToSocketAddrs>(self, local: A) -> Result<UdtBoundSocket> {
        let socket = UdtSocket::new_ipv6()?;
        self.config_socket(&socket)?;
        let socket = socket.bind(local)?;
        Ok(UdtBoundSocket { socket })
    }
    pub fn connect_ipv4<A: ToSocketAddrs>(self, remote: A) -> Result<UdtStream> {
        let socket = UdtSocket::new_ipv4()?;
        self.config_socket(&socket)?;
        socket.connect(remote)?;
        Ok(UdtStream { socket })
    }
    pub fn connect_ipv6<A: ToSocketAddrs>(self, remote: A) -> Result<UdtStream> {
        let socket = UdtSocket::new_ipv6()?;
        self.config_socket(&socket)?;
        socket.connect(remote)?;
        Ok(UdtStream { socket })
    }
    pub fn listen_ipv4<A: ToSocketAddrs>(self, addr: A, backlog: i32) -> Result<UdtListener> {
        let socket = UdtSocket::new_ipv4()?;
        self.config_socket(&socket)?;
        let socket = socket.bind(addr)?;
        socket.listen(backlog)?;
        Ok(UdtListener { socket })
    }
    pub fn listen_ipv6<A: ToSocketAddrs>(self, addr: A, backlog: i32) -> Result<UdtListener> {
        let socket = UdtSocket::new_ipv6()?;
        self.config_socket(&socket)?;
        let socket = socket.bind(addr)?;
        socket.listen(backlog)?;
        Ok(UdtListener { socket })
    }
}

impl UdtBuilder {
    pub fn set_mss(mut self, val: i32) -> Self {
        self.opt_vec.push(UdtSockOpt::Mss(val));
        self
    }
    pub fn set_snd_syn(mut self, val: bool) -> Self {
        self.opt_vec.push(UdtSockOpt::SndSyn(val));
        self
    }
    pub fn set_rcv_syn(mut self, val: bool) -> Self {
        self.opt_vec.push(UdtSockOpt::RcvSyn(val));
        self
    }
    pub fn set_fc(mut self, val: i32) -> Self {
        self.opt_vec.push(UdtSockOpt::Fc(val));
        self
    }
    pub fn set_snd_fuf(mut self, val: i32) -> Self {
        self.opt_vec.push(UdtSockOpt::SndBuf(val));
        self
    }
    pub fn set_rcv_buf(mut self, val: i32) -> Self {
        self.opt_vec.push(UdtSockOpt::RcvBuf(val));
        self
    }
    pub fn set_linger(mut self, val: i32) -> Self {
        self.opt_vec.push(UdtSockOpt::Linger(val));
        self
    }
    pub fn set_rendezvous(mut self, val: bool) -> Self {
        self.opt_vec.push(UdtSockOpt::Rendezvous(val));
        self
    }
    pub fn set_snd_timeo(mut self, val: i32) -> Self {
        self.opt_vec.push(UdtSockOpt::SndTimeo(val));
        self
    }
    pub fn set_rcv_timeo(mut self, val: i32) -> Self {
        self.opt_vec.push(UdtSockOpt::RcvTimeo(val));
        self
    }
    pub fn set_reuse_addr(mut self, val: bool) -> Self {
        self.opt_vec.push(UdtSockOpt::ReuseAddr(val));
        self
    }
    pub fn set_max_bw(mut self, val: i64) -> Self {
        self.opt_vec.push(UdtSockOpt::MaxBW(val));
        self
    }
    fn config_socket(self, socket: &UdtSocket) -> Result<()> {
        for opt in self.opt_vec {
            match opt {
                UdtSockOpt::Mss(val) => socket.set_mss(val)?,
                UdtSockOpt::SndSyn(val) => socket.set_sndsyn(val)?,
                UdtSockOpt::RcvSyn(val) => socket.set_rcvsyn(val)?,
                UdtSockOpt::Fc(val) => socket.set_fc(val)?,
                UdtSockOpt::SndBuf(val) => socket.set_sndbuf(val)?,
                UdtSockOpt::RcvBuf(val) => socket.set_rcvbuf(val)?,
                UdtSockOpt::Linger(val) => socket.set_linger(val)?,
                UdtSockOpt::Rendezvous(val) => socket.set_rendezvous(val)?,
                UdtSockOpt::SndTimeo(val) => socket.set_sndtimeo(val)?,
                UdtSockOpt::RcvTimeo(val) => socket.set_rcvtimeo(val)?,
                UdtSockOpt::ReuseAddr(val) => socket.set_reuseaddr(val)?,
                UdtSockOpt::MaxBW(val) => socket.set_maxbw(val)?,
            }
        }
        Ok(())
    }
}

pub struct UdtAsyncStream {
    socket: UdtSocket,
}

impl UdtAsyncStream {
    pub fn local_addr(&self) -> Result<SocketAddr> {
        self.socket.local_addr()
    }
    pub fn peer_addr(&self) -> Result<SocketAddr> {
        self.socket.peer_addr()
    }
}

impl AsyncRead for UdtAsyncStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::result::Result<usize, io::Error>> {
        match self.socket.recv(buf) {
            Ok(s) => Poll::Ready(Ok(s)),
            Err(e) => match e {
                UdtError::AsyncRcv(_) => {
                    let waker = cx.waker().clone();
                    let mut epoll = Epoll::new()?;
                    epoll.add(&self.socket, &udt_sys::EPOLLOpt::UDT_EPOLL_IN)?;
                    thread::spawn(move || {
                        if let Ok(_) = epoll.wait(-1) {
                            waker.wake();
                        }
                    });
                    Poll::Pending
                }
                e => Poll::Ready(Err(e.into())),
            },
        }
    }
}

impl AsyncWrite for UdtAsyncStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::result::Result<usize, io::Error>> {
        match self.socket.send(buf) {
            Ok(s) => Poll::Ready(Ok(s)),
            Err(e) => match e {
                UdtError::AsyncSnd(_) => match self.socket.get_snddata() {
                    Ok(bytes) => {
                        if bytes == 0 {
                            Poll::Ready(Ok(0))
                        } else {
                            let waker = cx.waker().clone();
                            let mut epoll = Epoll::new()?;
                            epoll.add(&self.socket, &udt_sys::EPOLLOpt::UDT_EPOLL_OUT)?;
                            thread::spawn(move || {
                                if let Ok(_) = epoll.wait(-1) {
                                    waker.wake();
                                }
                            });
                            Poll::Pending
                        }
                    }
                    Err(e) => Poll::Ready(Err(e.into())),
                },
                e => Poll::Ready(Err(e.into())),
            },
        }
    }
    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), io::Error>> {
        match self.socket.get_snddata() {
            Ok(bytes) => {
                if bytes == 0 {
                    Poll::Ready(Ok(()))
                } else {
                    let waker = cx.waker().clone();
                    let mut epoll = Epoll::new()?;
                    epoll.add(&self.socket, &udt_sys::EPOLLOpt::UDT_EPOLL_OUT)?;
                    thread::spawn(move || {
                        if let Ok(_) = epoll.wait(-1) {
                            waker.wake();
                        }
                    });
                    Poll::Pending
                }
            }
            Err(e) => Poll::Ready(Err(e.into())),
        }
    }
    fn poll_close(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), io::Error>> {
        match self.socket.get_snddata() {
            Ok(bytes) => {
                if bytes == 0 {
                    Poll::Ready(match self.socket.close() {
                        Ok(()) => Ok(()),
                        Err(e) => Err(e.into()),
                    })
                } else {
                    let waker = cx.waker().clone();
                    let mut epoll = Epoll::new()?;
                    epoll.add(&self.socket, &udt_sys::EPOLLOpt::UDT_EPOLL_OUT)?;
                    thread::spawn(move || {
                        if let Ok(_) = epoll.wait(-1) {
                            waker.wake();
                        }
                    });
                    Poll::Pending
                }
            }
            Err(e) => Poll::Ready(Err(e.into())),
        }
    }
}

impl Drop for UdtAsyncStream {
    fn drop(&mut self) {
        if let Err(_) = self.socket.close() {}
    }
}

pub struct UdtAsyncListener {
    socket: UdtSocket,
}

impl UdtAsyncListener {
    pub fn accept(&self) -> AcceptFuture {
        AcceptFuture {
            socket: self.socket,
        }
    }
    pub fn local_addr(&self) -> Result<SocketAddr> {
        self.socket.local_addr()
    }
}

impl Drop for UdtAsyncListener {
    fn drop(&mut self) {
        if let Err(_) = self.socket.close() {}
    }
}

pub struct AcceptFuture {
    socket: UdtSocket,
}

impl Future for AcceptFuture {
    type Output = Result<(UdtAsyncStream, SocketAddr)>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.socket.accept() {
            Ok((socket, addr)) => {
                let r_b = socket.set_rcvsyn(false);
                let s_b = socket.set_sndsyn(false);
                if r_b.is_err() {
                    Poll::Ready(Err(r_b.expect_err("unreachable")))
                } else if s_b.is_err() {
                    Poll::Ready(Err(s_b.expect_err("unreachable")))
                } else {
                    Poll::Ready(Ok((UdtAsyncStream { socket }, addr)))
                }
            }
            Err(e) => match e {
                UdtError::AsyncRcv(_) => {
                    let waker = cx.waker().clone();
                    let mut epoll = Epoll::new()?;
                    epoll.add(&self.socket, &udt_sys::EPOLLOpt::UDT_EPOLL_IN)?;
                    thread::spawn(move || {
                        if let Ok(_) = epoll.wait(-1) {
                            waker.wake();
                        }
                    });
                    Poll::Pending
                }
                e => Poll::Ready(Err(e)),
            },
        }
    }
}

pub struct ConnectFuture {
    socket: UdtSocket,
}

impl Future for ConnectFuture {
    type Output = Result<UdtAsyncStream>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.socket.get_state() {
            UdtStatus::Connecting => {
                let waker = cx.waker().clone();
                thread::spawn(move || {
                    thread::sleep(time::Duration::from_millis(500));
                    waker.wake();
                });
                Poll::Pending
            }
            UdtStatus::Connected => Poll::Ready(Ok(UdtAsyncStream {
                socket: self.socket,
            })),
            UdtStatus::Broken => {
                Poll::Ready(Err(UdtError::ConnLost("connection broken".to_string())))
            }
            UdtStatus::Init => {
                Poll::Ready(Err(UdtError::UnboundSock("socket not bound".to_string())))
            }
            UdtStatus::Opened => Poll::Ready(Err(UdtError::InvOp("already connected".to_string()))),
            UdtStatus::Listening => {
                Poll::Ready(Err(UdtError::InvOp("socket is listening".to_string())))
            }
            UdtStatus::Closing => {
                Poll::Ready(Err(UdtError::InvSock("socket is being closed".to_string())))
            }
            UdtStatus::Closed => {
                Poll::Ready(Err(UdtError::InvSock("socket already closed".to_string())))
            }
            UdtStatus::NonExist => {
                Poll::Ready(Err(UdtError::InvSock("socket do not exist".to_string())))
            }
        }
    }
}

pub struct UdtBoundAsyncSocket {
    socket: UdtSocket,
}

impl UdtBoundAsyncSocket {
    pub fn connect<A: ToSocketAddrs>(self, remote: A) -> Result<ConnectFuture> {
        self.socket.connect(remote)?;
        Ok(ConnectFuture {
            socket: self.socket,
        })
    }
    pub fn local_addr(&self) -> Result<SocketAddr> {
        self.socket.local_addr()
    }
}

pub struct UdtAsyncBuilder {
    opt_vec: Vec<UdtSockOpt>,
}

impl UdtAsyncBuilder {
    pub fn bind_ipv4<A: ToSocketAddrs>(self, local: A) -> Result<UdtBoundAsyncSocket> {
        let socket = UdtSocket::new_ipv4()?;
        self.config_socket(&socket)?;
        let socket = socket.bind(local)?;
        Ok(UdtBoundAsyncSocket { socket })
    }
    pub fn bind_ipv6<A: ToSocketAddrs>(self, local: A) -> Result<UdtBoundAsyncSocket> {
        let socket = UdtSocket::new_ipv6()?;
        self.config_socket(&socket)?;
        let socket = socket.bind(local)?;
        Ok(UdtBoundAsyncSocket { socket })
    }
    pub fn connect_ipv4<A: ToSocketAddrs>(self, remote: A) -> Result<ConnectFuture> {
        let socket = UdtSocket::new_ipv4()?;
        self.config_socket(&socket)?;
        socket.connect(remote)?;
        Ok(ConnectFuture { socket })
    }
    pub fn connect_ipv6<A: ToSocketAddrs>(self, remote: A) -> Result<ConnectFuture> {
        let socket = UdtSocket::new_ipv6()?;
        self.config_socket(&socket)?;
        socket.connect(remote)?;
        Ok(ConnectFuture { socket })
    }
    pub fn listen_ipv4<A: ToSocketAddrs>(self, addr: A, backlog: i32) -> Result<UdtAsyncListener> {
        let socket = UdtSocket::new_ipv4()?;
        self.config_socket(&socket)?;
        let socket = socket.bind(addr)?;
        socket.listen(backlog)?; // Still synchronous
        Ok(UdtAsyncListener { socket })
    }
    pub fn listen_ipv6<A: ToSocketAddrs>(self, addr: A, backlog: i32) -> Result<UdtAsyncListener> {
        let socket = UdtSocket::new_ipv6()?;
        self.config_socket(&socket)?;
        let socket = socket.bind(addr)?;
        socket.listen(backlog)?; // Still synchronous
        Ok(UdtAsyncListener { socket })
    }
}

impl UdtAsyncBuilder {
    pub fn set_mss(mut self, val: i32) -> Self {
        self.opt_vec.push(UdtSockOpt::Mss(val));
        self
    }
    pub fn set_fc(mut self, val: i32) -> Self {
        self.opt_vec.push(UdtSockOpt::Fc(val));
        self
    }
    pub fn set_snd_fuf(mut self, val: i32) -> Self {
        self.opt_vec.push(UdtSockOpt::SndBuf(val));
        self
    }
    pub fn set_rcv_buf(mut self, val: i32) -> Self {
        self.opt_vec.push(UdtSockOpt::RcvBuf(val));
        self
    }
    pub fn set_linger(mut self, val: i32) -> Self {
        self.opt_vec.push(UdtSockOpt::Linger(val));
        self
    }
    pub fn set_rendezvous(mut self, val: bool) -> Self {
        self.opt_vec.push(UdtSockOpt::Rendezvous(val));
        self
    }
    pub fn set_snd_timeo(mut self, val: i32) -> Self {
        self.opt_vec.push(UdtSockOpt::SndTimeo(val));
        self
    }
    pub fn set_rcv_timeo(mut self, val: i32) -> Self {
        self.opt_vec.push(UdtSockOpt::RcvTimeo(val));
        self
    }
    pub fn set_reuse_addr(mut self, val: bool) -> Self {
        self.opt_vec.push(UdtSockOpt::ReuseAddr(val));
        self
    }
    pub fn set_max_bw(mut self, val: i64) -> Self {
        self.opt_vec.push(UdtSockOpt::MaxBW(val));
        self
    }
    fn config_socket(self, socket: &UdtSocket) -> Result<()> {
        for opt in self.opt_vec {
            match opt {
                UdtSockOpt::Mss(val) => socket.set_mss(val)?,
                UdtSockOpt::SndSyn(val) => socket.set_sndsyn(val)?,
                UdtSockOpt::RcvSyn(val) => socket.set_rcvsyn(val)?,
                UdtSockOpt::Fc(val) => socket.set_fc(val)?,
                UdtSockOpt::SndBuf(val) => socket.set_sndbuf(val)?,
                UdtSockOpt::RcvBuf(val) => socket.set_rcvbuf(val)?,
                UdtSockOpt::Linger(val) => socket.set_linger(val)?,
                UdtSockOpt::Rendezvous(val) => socket.set_rendezvous(val)?,
                UdtSockOpt::SndTimeo(val) => socket.set_sndtimeo(val)?,
                UdtSockOpt::RcvTimeo(val) => socket.set_rcvtimeo(val)?,
                UdtSockOpt::ReuseAddr(val) => socket.set_reuseaddr(val)?,
                UdtSockOpt::MaxBW(val) => socket.set_maxbw(val)?,
            }
        }
        Ok(())
    }
}

#[derive(Clone)]
enum UdtSockOpt {
    Mss(i32),
    SndSyn(bool),
    RcvSyn(bool),
    Fc(i32),
    SndBuf(i32),
    RcvBuf(i32),
    Linger(i32),
    Rendezvous(bool),
    SndTimeo(i32),
    RcvTimeo(i32),
    ReuseAddr(bool),
    MaxBW(i64),
}

struct Epoll {
    id: i32,
    num_rd_sock: usize,
    num_wr_sock: usize,
}

impl Epoll {
    fn new() -> Result<Self> {
        let result = unsafe { udt_sys::udt_epoll_create() };
        if result == unsafe { udt_sys::UDT_ERROR } {
            error::get_error(Self {
                id: 0,
                num_rd_sock: 0,
                num_wr_sock: 0,
            })
        } else {
            Ok(Self {
                id: result,
                num_rd_sock: 0,
                num_wr_sock: 0,
            })
        }
    }
    fn add(&mut self, socket: &UdtSocket, event: &udt_sys::EPOLLOpt) -> Result<()> {
        let udt_sys::EPOLLOpt(ev) = event;
        let ev = *ev as i32;
        let result =
            unsafe { udt_sys::udt_epoll_add_usock(self.id, socket.id, &ev as &i32 as *const i32) };
        if result == unsafe { udt_sys::UDT_ERROR } {
            error::get_error(())
        } else {
            if *event & udt_sys::EPOLLOpt::UDT_EPOLL_IN == udt_sys::EPOLLOpt::UDT_EPOLL_IN {
                self.num_rd_sock += 1;
            }

            if *event & udt_sys::EPOLLOpt::UDT_EPOLL_OUT == udt_sys::EPOLLOpt::UDT_EPOLL_OUT {
                self.num_wr_sock += 1;
            }
            Ok(())
        }
    }
    #[allow(dead_code)]
    fn remove(&mut self, socket: &UdtSocket) -> Result<()> {
        let event = socket.get_event()?;
        let result = unsafe { udt_sys::udt_epoll_remove_usock(self.id, socket.id) };
        if result == unsafe { udt_sys::UDT_ERROR } {
            error::get_error(())
        } else {
            if event & udt_sys::EPOLLOpt::UDT_EPOLL_IN == udt_sys::EPOLLOpt::UDT_EPOLL_IN {
                self.num_rd_sock -= 1;
            }

            if event & udt_sys::EPOLLOpt::UDT_EPOLL_OUT == udt_sys::EPOLLOpt::UDT_EPOLL_OUT {
                self.num_wr_sock -= 1;
            }
            Ok(())
        }
    }
    fn wait(&self, timeout: i64) -> Result<(Vec<udt_sys::UDTSOCKET>, Vec<udt_sys::UDTSOCKET>)> {
        let mut rd_array = vec![unsafe { udt_sys::UDT_INVALID_SOCK }; self.num_rd_sock];
        let mut rd_len = rd_array.len() as c_int;
        let mut wr_array = vec![unsafe { udt_sys::UDT_INVALID_SOCK }; self.num_wr_sock];
        let mut wr_len = wr_array.len() as c_int;
        let result = unsafe {
            udt_sys::udt_epoll_wait2(
                self.id,
                rd_array[..].as_mut_ptr() as *mut udt_sys::UDTSOCKET,
                &mut rd_len as *mut i32,
                wr_array[..].as_mut_ptr() as *mut udt_sys::UDTSOCKET,
                &mut wr_len as *mut i32,
                timeout,
                ptr::null::<udt_sys::SYSSOCKET> as *mut udt_sys::SYSSOCKET,
                ptr::null::<c_int> as *mut i32,
                ptr::null::<udt_sys::SYSSOCKET> as *mut udt_sys::SYSSOCKET,
                ptr::null::<c_int> as *mut i32,
            )
        };
        if result == unsafe { udt_sys::UDT_ERROR } {
            error::get_error((Vec::new(), Vec::new()))
        } else {
            rd_array.truncate(rd_len as usize);
            wr_array.truncate(wr_len as usize);
            Ok((rd_array, wr_array))
        }
    }
    #[allow(dead_code)]
    fn release(self) -> Result<()> {
        let result = unsafe { udt_sys::udt_epoll_release(self.id) };
        if result == unsafe { udt_sys::UDT_ERROR } {
            error::get_error(())
        } else {
            Ok(())
        }
    }
}

impl Drop for Epoll {
    fn drop(&mut self) {
        unsafe {
            udt_sys::udt_epoll_release(self.id);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate as udt;
    use futures::{
        executor::block_on,
        future,
        io::{AsyncReadExt, AsyncWriteExt},
    };
    use std::{
        io::{Read, Write},
        net::SocketAddr,
        sync::mpsc,
        thread,
    };

    #[test]
    fn test_ipv6() {
        udt::startup().expect("failed startup");
        test_ipv6_connect_accept();
        block_on(test_ipv6_connect_accept_async());
        udt::cleanup().expect("failed cleanup()");
    }
    fn test_ipv6_connect_accept() {
        let (tx, rx) = mpsc::channel::<SocketAddr>();
        thread::spawn(move || {
            let listen = udt::builder()
                .set_reuse_addr(false)
                .listen_ipv6("[::1]:0", 1)
                .expect("fail listen()");
            let local = listen.local_addr().expect("fail local_addr()");
            tx.send(local).expect("fail send through mpsc channel");
            let (mut peer, _peer_addr) = listen.accept().expect("fail accep()");
            peer.write_all(b"testing").expect("fail write()");
            assert!(peer.close().is_ok());
        });
        let addr = rx.recv().expect("fail recv through mpsc channel");
        println!("{}", addr);
        let mut connect = udt::builder()
            .set_reuse_addr(false)
            .connect_ipv6(addr)
            .expect("fail connect()");
        let mut buf = [0; 7];
        connect.read_exact(&mut buf).expect("fail read()");
        assert_eq!(
            std::str::from_utf8(&buf).expect("malformed message"),
            "testing"
        );
        assert!(connect.close().is_ok());
    }
    async fn test_ipv6_connect_accept_async() {
        let (tx, rx) = mpsc::channel::<SocketAddr>();
        let listen_task = async move {
            let listen = udt::async_builder()
                .set_reuse_addr(false)
                .listen_ipv6("[::1]:0", 1)
                .expect("fail listen()");
            let local = listen.local_addr().expect("fail local_addr()");
            tx.send(local).expect("fail send through mpsc channel");
            let (mut peer, _peer_addr) = listen.accept().await.expect("fail accep()");
            peer.write_all(b"testing").await.expect("fail write()");
            assert!(peer.close().await.is_ok());
        };
        let connect_task = async move {
            let addr = rx.recv().expect("fail recv through mpsc channel");
            let mut connect = udt::async_builder()
                .set_reuse_addr(false)
                .connect_ipv6(addr)
                .expect("fail start connect")
                .await
                .expect("fail connect");
            let mut buf = [0; 7];
            connect.read_exact(&mut buf).await.expect("fail read()");
            assert_eq!(
                std::str::from_utf8(&buf).expect("malformed message"),
                "testing"
            );
            assert!(connect.close().await.is_ok());
        };
        future::join(listen_task, connect_task).await;
    }
    #[test]
    fn test_ipv4_connect_accept() {
        udt::startup().expect("failed startup");
        let (tx, rx) = mpsc::channel::<SocketAddr>();
        thread::spawn(move || {
            let listen = udt::builder()
                .set_reuse_addr(false)
                .listen_ipv4("127.0.0.1:0", 1)
                .expect("fail listen()");
            let local = listen.local_addr().expect("fail local_addr()");
            tx.send(local).expect("fail send through mpsc channel");
            let (mut peer, _peer_addr) = listen.accept().expect("fail accep()");
            peer.write_all(b"testing").expect("fail write()");
            assert!(peer.close().is_ok());
        });
        let addr = rx.recv().expect("fail recv through mpsc channel");
        let mut connect = udt::builder()
            .set_reuse_addr(false)
            .connect_ipv4(addr)
            .expect("fail connect()");
        let mut buf = [0; 7];
        connect.read_exact(&mut buf).expect("fail read()");
        assert_eq!(
            std::str::from_utf8(&buf).expect("malformed message"),
            "testing"
        );
        assert!(connect.close().is_ok());
        udt::cleanup().expect("failed cleanup()");
    }
    #[test]
    fn test_ipv4_connect_accept_async() {
        udt::startup().expect("failed startup");
        let (tx, rx) = mpsc::channel::<SocketAddr>();
        let listen_task = async move {
            let listen = udt::async_builder()
                .set_reuse_addr(false)
                .listen_ipv4("127.0.0.1:0", 1)
                .expect("fail listen()");
            let local = listen.local_addr().expect("fail local_addr()");
            tx.send(local).expect("fail send through mpsc channel");
            let (mut peer, _peer_addr) = listen.accept().await.expect("fail accep()");
            peer.write_all(b"testing").await.expect("fail write()");
            assert!(peer.close().await.is_ok());
        };
        let connect_task = async move {
            let addr = rx.recv().expect("fail recv through mpsc channel");
            let mut connect = udt::async_builder()
                .set_reuse_addr(false)
                .connect_ipv4(addr)
                .expect("fail start connect")
                .await
                .expect("fail connect");
            let mut buf = [0; 7];
            connect.read_exact(&mut buf).await.expect("fail read()");
            assert_eq!(
                std::str::from_utf8(&buf).expect("malformed message"),
                "testing"
            );
            assert!(connect.close().await.is_ok());
        };
        block_on(future::join(listen_task, connect_task));
        udt::cleanup().expect("failed cleanup()");
    }

    #[test]
    fn test_ipv4_rendezvous() {
        udt::startup().expect("failed startup");
        let (tx_1, rx_1) = mpsc::channel::<SocketAddr>();
        let (tx_2, rx_2) = mpsc::channel::<SocketAddr>();
        thread::spawn(move || {
            let one = udt::builder()
                .set_reuse_addr(false)
                .set_rendezvous(true)
                .bind_ipv4("127.0.0.1:0")
                .expect("fail bind()");
            let local = one.local_addr().expect("fail local_addr()");
            tx_1.send(local).expect("fail send through mpsc channel");
            let addr = rx_2.recv().expect("fail recv through mpsc channel");
            let mut one = one.connect(addr).expect("fail connect()");
            one.write_all(b"testing").expect("fail write()");
            assert!(one.close().is_ok());
        });
        let two = udt::builder()
            .set_reuse_addr(false)
            .set_rendezvous(true)
            .bind_ipv4("127.0.0.2:0")
            .expect("fail bind()");
        let local = two.local_addr().expect("fail local_addr()");
        tx_2.send(local).expect("fail send through mpsc channel");
        let addr = rx_1.recv().expect("fail recv through mpsc channel");
        let mut two = two.connect(addr).expect("fail connect()");
        let mut buf = [0; 7];
        two.read_exact(&mut buf).expect("fail read()");
        assert_eq!(
            std::str::from_utf8(&buf).expect("malformed message"),
            "testing"
        );
        assert!(two.close().is_ok());
        udt::cleanup().expect("failed cleanup");
    }
}
