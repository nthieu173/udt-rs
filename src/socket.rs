use crate::error;

use error::UdtError;
use os_socketaddr::{self, OsSocketAddr};
use udt_sys::{self, sockaddr};

use std::{
    convert::TryInto,
    ffi::c_void,
    mem,
    net::{SocketAddr, ToSocketAddrs},
    os::raw::{c_char, c_int},
};

#[cfg(target_family = "unix")]
use libc::{linger, AF_INET, AF_INET6, SOCK_STREAM};

#[cfg(target_os = "windows")]
use winapi::{
    shared::ws2def::{AF_INET, AF_INET6},
    um::winsock2::{linger, SOCK_STREAM},
};

type Result<T> = std::result::Result<T, UdtError>;

#[derive(Copy, Clone, Debug)]
pub enum UdtStatus {
    Init,
    Opened,
    Listening,
    Connecting,
    Connected,
    Broken,
    Closing,
    Closed,
    NonExist,
}

#[derive(Copy, Clone, Debug)]
pub struct UdtSocket {
    pub id: i32,
}

//General methods
impl UdtSocket {
    pub fn new_ipv4() -> Result<Self> {
        let sock = unsafe { udt_sys::udt_socket(AF_INET, SOCK_STREAM, 0) };
        if sock == unsafe { udt_sys::UDT_INVALID_SOCK } {
            error::get_error(Self { id: 0 })
        } else {
            Ok(Self { id: sock })
        }
    }
    pub fn new_ipv6() -> Result<Self> {
        let sock = unsafe { udt_sys::udt_socket(AF_INET6, SOCK_STREAM, 0) };
        if sock == unsafe { udt_sys::UDT_INVALID_SOCK } {
            error::get_error(Self { id: 0 })
        } else {
            Ok(Self { id: sock })
        }
    }
    pub fn bind(self, addr: SocketAddr) -> Result<Self> {
        let os_addr: OsSocketAddr = addr.into();
        let result = unsafe {
            udt_sys::udt_bind(
                self.id,
                os_addr.as_ptr() as *const sockaddr,
                os_addr.len() as i32,
            )
        };
        if result == unsafe { udt_sys::UDT_ERROR } {
            return error::get_error(self);
        } else {
            return Ok(self);
        }
    }
    pub fn connect<A: ToSocketAddrs>(&self, addrs: A) -> Result<()> {
        if let Ok(addrs) = addrs.to_socket_addrs() {
            for addr in addrs {
                let os_target: OsSocketAddr = addr.into();
                let result = unsafe {
                    udt_sys::udt_connect(
                        self.id,
                        os_target.as_ptr() as *const sockaddr,
                        os_target.len() as i32,
                    )
                };
                if result == unsafe { udt_sys::UDT_ERROR } {
                    return error::get_error(());
                } else {
                    return Ok(());
                }
            }
        }
        Err(UdtError::ConnFail("invalid address".to_string()))
    }
    pub fn listen(&self, backlog: i32) -> Result<()> {
        let result = unsafe { udt_sys::udt_listen(self.id, backlog) };
        if result == unsafe { udt_sys::UDT_ERROR } {
            return error::get_error(());
        } else {
            return Ok(());
        }
    }
}

//Public operational methods
impl UdtSocket {
    pub fn local_addr(&self) -> Result<SocketAddr> {
        let mut addr = OsSocketAddr::new();
        let mut addrlen: c_int = addr.capacity() as i32;
        let result = unsafe {
            udt_sys::udt_getsockname(
                self.id,
                addr.as_mut_ptr() as *mut sockaddr,
                &mut addrlen as *mut c_int,
            )
        };
        if result == unsafe { udt_sys::UDT_ERROR } {
            error::get_error("0.0.0.0:0".parse().unwrap())
        } else {
            Ok(addr.into_addr().unwrap())
        }
    }
    pub fn peer_addr(&self) -> Result<SocketAddr> {
        let mut addr = OsSocketAddr::new();
        let mut addrlen: c_int = addr.capacity() as i32;
        let result = unsafe {
            udt_sys::udt_getpeername(
                self.id,
                addr.as_mut_ptr() as *mut sockaddr,
                &mut addrlen as *mut c_int,
            )
        };
        if result == unsafe { udt_sys::UDT_ERROR } {
            error::get_error("0.0.0.0:0".parse().unwrap())
        } else {
            Ok(addr.into_addr().unwrap())
        }
    }
    pub fn accept(&self) -> Result<(Self, SocketAddr)> {
        let mut addr = OsSocketAddr::new();
        let mut _addrlen: c_int = addr.capacity() as i32;
        let result = unsafe {
            udt_sys::udt_accept(
                self.id,
                addr.as_mut_ptr() as *mut sockaddr,
                &mut _addrlen as *mut c_int,
            )
        };
        if result == unsafe { udt_sys::UDT_INVALID_SOCK } {
            error::get_error((Self { id: result }, "0.0.0.0:0".parse().unwrap()))
        } else {
            Ok((Self { id: result }, addr.into_addr().unwrap()))
        }
    }
    pub fn close(self) -> Result<()> {
        let result = unsafe { udt_sys::udt_close(self.id) };
        if result == unsafe { udt_sys::UDT_ERROR } {
            error::get_error(())
        } else {
            Ok(())
        }
    }
    pub fn send(&self, buf: &[u8]) -> Result<usize> {
        let result = unsafe {
            udt_sys::udt_send(
                self.id,
                buf as *const [u8] as *const c_char,
                buf.len() as i32,
                0,
            )
        };
        if result == unsafe { udt_sys::UDT_ERROR } {
            error::get_error(0)
        } else {
            Ok(result as usize)
        }
    }
    pub fn recv(&self, buf: &mut [u8]) -> Result<usize> {
        let result = unsafe {
            udt_sys::udt_recv(
                self.id,
                buf as *mut [u8] as *mut c_char,
                buf.len() as i32,
                0,
            )
        };
        if result == unsafe { udt_sys::UDT_ERROR } {
            error::get_error(0)
        } else {
            Ok(result as usize)
        }
    }
}
//Get opt methods
impl UdtSocket {
    pub fn get_mss(&self) -> Result<i32> {
        let mut val = 0;
        let mut val_len = mem::size_of_val(&val) as i32;
        let result = unsafe {
            udt_sys::udt_getsockopt(
                self.id,
                0,
                udt_sys::UDTOpt::UDT_MSS,
                &mut val as *mut i32 as *mut c_void,
                &mut val_len as *mut i32,
            )
        };
        if result == unsafe { udt_sys::UDT_ERROR } {
            error::get_error(val)
        } else {
            Ok(val)
        }
    }
    pub fn get_sndsyn(&self) -> Result<bool> {
        let mut val = true;
        let mut val_len = mem::size_of_val(&val) as i32;
        let result = unsafe {
            udt_sys::udt_getsockopt(
                self.id,
                0,
                udt_sys::UDTOpt::UDT_SNDSYN,
                &mut val as *mut bool as *mut c_void,
                &mut val_len as *mut i32,
            )
        };
        if result == unsafe { udt_sys::UDT_ERROR } {
            error::get_error(val)
        } else {
            Ok(val)
        }
    }
    pub fn get_rcvsyn(&self) -> Result<bool> {
        let mut val = true;
        let mut val_len = mem::size_of_val(&val) as i32;
        let result = unsafe {
            udt_sys::udt_getsockopt(
                self.id,
                0,
                udt_sys::UDTOpt::UDT_RCVSYN,
                &mut val as *mut bool as *mut c_void,
                &mut val_len as *mut i32,
            )
        };
        if result == unsafe { udt_sys::UDT_ERROR } {
            error::get_error(val)
        } else {
            Ok(val)
        }
    }
    pub fn get_fc(&self) -> Result<i32> {
        let mut val = 0;
        let mut val_len = mem::size_of_val(&val) as i32;
        let result = unsafe {
            udt_sys::udt_getsockopt(
                self.id,
                0,
                udt_sys::UDTOpt::UDT_FC,
                &mut val as *mut i32 as *mut c_void,
                &mut val_len as *mut i32,
            )
        };
        if result == unsafe { udt_sys::UDT_ERROR } {
            error::get_error(val)
        } else {
            Ok(val)
        }
    }
    pub fn get_sndbuf(&self) -> Result<i32> {
        let mut val = 0;
        let mut val_len = mem::size_of_val(&val) as i32;
        let result = unsafe {
            udt_sys::udt_getsockopt(
                self.id,
                0,
                udt_sys::UDTOpt::UDT_SNDBUF,
                &mut val as *mut i32 as *mut c_void,
                &mut val_len as *mut i32,
            )
        };
        if result == unsafe { udt_sys::UDT_ERROR } {
            error::get_error(val)
        } else {
            Ok(val)
        }
    }
    pub fn get_rcvbuf(&self) -> Result<i32> {
        let mut val = 0;
        let mut val_len = mem::size_of_val(&val) as i32;
        let result = unsafe {
            udt_sys::udt_getsockopt(
                self.id,
                0,
                udt_sys::UDTOpt::UDT_RCVBUF,
                &mut val as *mut i32 as *mut c_void,
                &mut val_len as *mut i32,
            )
        };
        if result == unsafe { udt_sys::UDT_ERROR } {
            error::get_error(val)
        } else {
            Ok(val)
        }
    }
    pub fn get_udp_sndbuf(&self) -> Result<i32> {
        let mut val = 0;
        let mut val_len = mem::size_of_val(&val) as i32;
        let result = unsafe {
            udt_sys::udt_getsockopt(
                self.id,
                0,
                udt_sys::UDTOpt::UDP_SNDBUF,
                &mut val as *mut i32 as *mut c_void,
                &mut val_len as *mut i32,
            )
        };
        if result == unsafe { udt_sys::UDT_ERROR } {
            error::get_error(val)
        } else {
            Ok(val)
        }
    }
    pub fn get_udp_rcvbuf(&self) -> Result<i32> {
        let mut val = 0;
        let mut val_len = mem::size_of_val(&val) as i32;
        let result = unsafe {
            udt_sys::udt_getsockopt(
                self.id,
                0,
                udt_sys::UDTOpt::UDP_RCVBUF,
                &mut val as *mut i32 as *mut c_void,
                &mut val_len as *mut i32,
            )
        };
        if result == unsafe { udt_sys::UDT_ERROR } {
            error::get_error(val)
        } else {
            Ok(val)
        }
    }
    pub fn get_linger(&self) -> Result<i32> {
        let mut val = linger {
            l_onoff: 0,
            l_linger: 0,
        };
        let mut val_len = mem::size_of_val(&val) as i32;
        let result = unsafe {
            udt_sys::udt_getsockopt(
                self.id,
                0,
                udt_sys::UDTOpt::UDT_LINGER,
                &mut val as *mut linger as *mut c_void,
                &mut val_len as *mut c_int,
            )
        };
        if result == unsafe { udt_sys::UDT_ERROR } {
            error::get_error(val.l_linger.into())
        } else {
            Ok(val.l_linger.into())
        }
    }
    pub fn get_rendezvous(&self) -> Result<bool> {
        let mut val = true;
        let mut val_len = mem::size_of_val(&val) as i32;
        let result = unsafe {
            udt_sys::udt_getsockopt(
                self.id,
                0,
                udt_sys::UDTOpt::UDT_RENDEZVOUS,
                &mut val as *mut bool as *mut c_void,
                &mut val_len as *mut i32,
            )
        };
        if result == unsafe { udt_sys::UDT_ERROR } {
            error::get_error(val)
        } else {
            Ok(val)
        }
    }
    pub fn get_sndtimeo(&self) -> Result<i32> {
        let mut val = 0;
        let mut val_len = mem::size_of_val(&val) as i32;
        let result = unsafe {
            udt_sys::udt_getsockopt(
                self.id,
                0,
                udt_sys::UDTOpt::UDT_SNDTIMEO,
                &mut val as *mut i32 as *mut c_void,
                &mut val_len as *mut i32,
            )
        };
        if result == unsafe { udt_sys::UDT_ERROR } {
            error::get_error(val)
        } else {
            Ok(val)
        }
    }
    pub fn get_rcvtimeo(&self) -> Result<i32> {
        let mut val = 0;
        let mut val_len = mem::size_of_val(&val) as i32;
        let result = unsafe {
            udt_sys::udt_getsockopt(
                self.id,
                0,
                udt_sys::UDTOpt::UDT_RCVTIMEO,
                &mut val as *mut i32 as *mut c_void,
                &mut val_len as *mut i32,
            )
        };
        if result == unsafe { udt_sys::UDT_ERROR } {
            error::get_error(val)
        } else {
            Ok(val)
        }
    }
    pub fn get_reuseaddr(&self) -> Result<bool> {
        let mut val = true;
        let mut val_len = mem::size_of_val(&val) as i32;
        let result = unsafe {
            udt_sys::udt_getsockopt(
                self.id,
                0,
                udt_sys::UDTOpt::UDT_REUSEADDR,
                &mut val as *mut bool as *mut c_void,
                &mut val_len as *mut i32,
            )
        };
        if result == unsafe { udt_sys::UDT_ERROR } {
            error::get_error(val)
        } else {
            Ok(val)
        }
    }
    pub fn get_maxbw(&self) -> Result<i64> {
        let mut val = 0;
        let mut val_len = mem::size_of_val(&val) as i32;
        let result = unsafe {
            udt_sys::udt_getsockopt(
                self.id,
                0,
                udt_sys::UDTOpt::UDT_MAXBW,
                &mut val as *mut i64 as *mut c_void,
                &mut val_len as *mut i32,
            )
        };
        if result == unsafe { udt_sys::UDT_ERROR } {
            error::get_error(val)
        } else {
            Ok(val)
        }
    }
    pub fn get_state(&self) -> UdtStatus {
        let result = unsafe { udt_sys::udt_getsockstate(self.id) };
        match result {
            udt_sys::UDTSTATUS::INIT => UdtStatus::Init,
            udt_sys::UDTSTATUS::OPENED => UdtStatus::Opened,
            udt_sys::UDTSTATUS::LISTENING => UdtStatus::Listening,
            udt_sys::UDTSTATUS::CONNECTING => UdtStatus::Connecting,
            udt_sys::UDTSTATUS::CONNECTED => UdtStatus::Connected,
            udt_sys::UDTSTATUS::BROKEN => UdtStatus::Broken,
            udt_sys::UDTSTATUS::CLOSING => UdtStatus::Closing,
            udt_sys::UDTSTATUS::CLOSED => UdtStatus::Closed,
            udt_sys::UDTSTATUS::NONEXIST => UdtStatus::NonExist,
            _ => unreachable!("unrecognized udt status"),
        }
    }
    pub fn get_event(&self) -> Result<udt_sys::EPOLLOpt> {
        let mut val = 0;
        let mut val_len = mem::size_of_val(&val) as i32;
        let result = unsafe {
            udt_sys::udt_getsockopt(
                self.id,
                0,
                udt_sys::UDTOpt::UDT_EVENT,
                &mut val as *mut i32 as *mut c_void,
                &mut val_len as *mut i32,
            )
        };
        if result == unsafe { udt_sys::UDT_ERROR } {
            error::get_error(udt_sys::EPOLLOpt(0))
        } else {
            Ok(udt_sys::EPOLLOpt(val as u32))
        }
    }
    pub fn get_snddata(&self) -> Result<i32> {
        let mut val = 0;
        let mut val_len = mem::size_of_val(&val) as i32;
        let result = unsafe {
            udt_sys::udt_getsockopt(
                self.id,
                0,
                udt_sys::UDTOpt::UDT_SNDDATA,
                &mut val as *mut i32 as *mut c_void,
                &mut val_len as *mut i32,
            )
        };
        if result == unsafe { udt_sys::UDT_ERROR } {
            error::get_error(val)
        } else {
            Ok(val)
        }
    }
    pub fn get_rcvdata(&self) -> Result<i32> {
        let mut val = 0;
        let mut val_len = mem::size_of_val(&val) as i32;
        let result = unsafe {
            udt_sys::udt_getsockopt(
                self.id,
                0,
                udt_sys::UDTOpt::UDT_RCVDATA,
                &mut val as *mut i32 as *mut c_void,
                &mut val_len as *mut i32,
            )
        };
        if result == unsafe { udt_sys::UDT_ERROR } {
            error::get_error(val)
        } else {
            Ok(val)
        }
    }
}
//Set opt methods
impl UdtSocket {
    /*
        Maximum packet size (bytes).
        Including all UDT, UDP, and IP headers. Default 1500 bytes.
    */
    pub fn set_mss(&self, mss: i32) -> Result<()> {
        let result = unsafe {
            udt_sys::udt_setsockopt(
                self.id,
                0,
                udt_sys::UDTOpt::UDT_MSS,
                &mss as *const i32 as *const c_void,
                mem::size_of_val(&mss) as i32,
            )
        };
        if result == unsafe { udt_sys::UDT_ERROR } {
            error::get_error(())
        } else {
            Ok(())
        }
    }
    /*
        Synchronization mode of data sending.
        true for blocking sending; false for non-blocking sending. Default true.
    */
    pub fn set_sndsyn(&self, blocking: bool) -> Result<()> {
        let result = unsafe {
            udt_sys::udt_setsockopt(
                self.id,
                0,
                udt_sys::UDTOpt::UDT_SNDSYN,
                &blocking as *const bool as *const c_void,
                mem::size_of_val(&blocking) as i32,
            )
        };
        if result == unsafe { udt_sys::UDT_ERROR } {
            error::get_error(())
        } else {
            Ok(())
        }
    }
    /*
        Synchronization mode for receiving.
        true for blocking receiving; false for non-blocking receiving. Default true.
    */
    pub fn set_rcvsyn(&self, blocking: bool) -> Result<()> {
        let result = unsafe {
            udt_sys::udt_setsockopt(
                self.id,
                0,
                udt_sys::UDTOpt::UDT_RCVSYN,
                &blocking as *const bool as *const c_void,
                mem::size_of_val(&blocking) as i32,
            )
        };
        if result == unsafe { udt_sys::UDT_ERROR } {
            error::get_error(())
        } else {
            Ok(())
        }
    }
    /*
        Maximum window size (packets).
        Do NOT change this unless you know what you are doing. Must change this before modifying the buffer sizes. Default 25600.
    */
    pub fn set_fc(&self, fc: i32) -> Result<()> {
        let result = unsafe {
            udt_sys::udt_setsockopt(
                self.id,
                0,
                udt_sys::UDTOpt::UDT_FC,
                &fc as *const i32 as *const c_void,
                mem::size_of_val(&fc) as i32,
            )
        };
        if result == unsafe { udt_sys::UDT_ERROR } {
            error::get_error(())
        } else {
            Ok(())
        }
    }
    /*
        UDT sender buffer size limit (bytes).
        Default 10MB (10240000).
    */
    pub fn set_sndbuf(&self, size: i32) -> Result<()> {
        let result = unsafe {
            udt_sys::udt_setsockopt(
                self.id,
                0,
                udt_sys::UDTOpt::UDT_SNDBUF,
                &size as *const i32 as *const c_void,
                mem::size_of_val(&size) as i32,
            )
        };
        if result == unsafe { udt_sys::UDT_ERROR } {
            error::get_error(())
        } else {
            Ok(())
        }
    }
    /*
        UDT receiver buffer size limit (bytes).
        Default 10MB (10240000).
    */
    pub fn set_rcvbuf(&self, size: i32) -> Result<()> {
        let result = unsafe {
            udt_sys::udt_setsockopt(
                self.id,
                0,
                udt_sys::UDTOpt::UDT_RCVBUF,
                &size as *const i32 as *const c_void,
                mem::size_of_val(&size) as i32,
            )
        };
        if result == unsafe { udt_sys::UDT_ERROR } {
            error::get_error(())
        } else {
            Ok(())
        }
    }
    /*
        UDP socket sender buffer size (bytes).
        Default 1MB (1024000).
    */
    pub fn set_udp_sndbuf(&self, size: i32) -> Result<()> {
        let result = unsafe {
            udt_sys::udt_setsockopt(
                self.id,
                0,
                udt_sys::UDTOpt::UDP_SNDBUF,
                &size as *const i32 as *const c_void,
                mem::size_of_val(&size) as i32,
            )
        };
        if result == unsafe { udt_sys::UDT_ERROR } {
            error::get_error(())
        } else {
            Ok(())
        }
    }
    /*
        UDP socket receiver buffer size (bytes).
        Default 1MB (1024000).
    */
    pub fn set_udp_rcvbuf(&self, size: i32) -> Result<()> {
        let result = unsafe {
            udt_sys::udt_setsockopt(
                self.id,
                0,
                udt_sys::UDTOpt::UDP_RCVBUF,
                &size as *const i32 as *const c_void,
                mem::size_of_val(&size) as i32,
            )
        };
        if result == unsafe { udt_sys::UDT_ERROR } {
            error::get_error(())
        } else {
            Ok(())
        }
    }
    /*
        Linger time on close().
        Default 180 seconds.
    */
    pub fn set_linger(&self, time: i32) -> Result<()> {
        let linger = linger {
            l_onoff: if time <= 0 { 0 } else { 1 },
            l_linger: time.try_into().expect("linger time out of scope"),
        };
        let result = unsafe {
            udt_sys::udt_setsockopt(
                self.id,
                0,
                udt_sys::UDTOpt::UDT_LINGER,
                &linger as *const linger as *const c_void,
                mem::size_of_val(&linger) as i32,
            )
        };
        if result == unsafe { udt_sys::UDT_ERROR } {
            error::get_error(())
        } else {
            Ok(())
        }
    }

    /*
        Rendezvous connection setup.
        Default false (no rendezvous mode).
    */
    pub fn set_rendezvous(&self, rendezvous: bool) -> Result<()> {
        let result = unsafe {
            udt_sys::udt_setsockopt(
                self.id,
                0,
                udt_sys::UDTOpt::UDT_RENDEZVOUS,
                &rendezvous as *const bool as *const c_void,
                mem::size_of_val(&rendezvous) as i32,
            )
        };
        if result == unsafe { udt_sys::UDT_ERROR } {
            error::get_error(())
        } else {
            Ok(())
        }
    }
    /*
        Sending call timeout (milliseconds).
        Default -1 (infinite).
    */
    pub fn set_sndtimeo(&self, timeout: i32) -> Result<()> {
        let result = unsafe {
            udt_sys::udt_setsockopt(
                self.id,
                0,
                udt_sys::UDTOpt::UDT_SNDTIMEO,
                &timeout as *const i32 as *const c_void,
                mem::size_of_val(&timeout) as i32,
            )
        };
        if result == unsafe { udt_sys::UDT_ERROR } {
            error::get_error(())
        } else {
            Ok(())
        }
    }
    /*
        Receiving call timeout (milliseconds).
        Default -1 (infinite).
    */
    pub fn set_rcvtimeo(&self, timeout: i32) -> Result<()> {
        let result = unsafe {
            udt_sys::udt_setsockopt(
                self.id,
                0,
                udt_sys::UDTOpt::UDT_RCVTIMEO,
                &timeout as *const i32 as *const c_void,
                mem::size_of_val(&timeout) as i32,
            )
        };
        if result == unsafe { udt_sys::UDT_ERROR } {
            error::get_error(())
        } else {
            Ok(())
        }
    }
    /*
        Reuse an existing address or create a new one.
        Default true (reuse).
    */
    pub fn set_reuseaddr(&self, reuse: bool) -> Result<()> {
        let result = unsafe {
            udt_sys::udt_setsockopt(
                self.id,
                0,
                udt_sys::UDTOpt::UDT_REUSEADDR,
                &reuse as *const bool as *const c_void,
                mem::size_of_val(&reuse) as i32,
            )
        };
        if result == unsafe { udt_sys::UDT_ERROR } {
            error::get_error(())
        } else {
            Ok(())
        }
    }
    /*
        Maximum bandwidth that one single UDT connection can use (bytes per second).
        Default -1 (no upper limit).
    */
    pub fn set_maxbw(&self, maxbw: i64) -> Result<()> {
        let result = unsafe {
            udt_sys::udt_setsockopt(
                self.id,
                0,
                udt_sys::UDTOpt::UDT_MAXBW,
                &maxbw as *const i64 as *const c_void,
                mem::size_of_val(&maxbw) as i32,
            )
        };
        if result == unsafe { udt_sys::UDT_ERROR } {
            error::get_error(())
        } else {
            Ok(())
        }
    }
}
