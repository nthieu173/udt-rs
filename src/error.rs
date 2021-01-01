use udt_sys;

use std::{
    convert::From,
    error::Error,
    ffi::CStr,
    fmt::{self, Display, Formatter},
    io::{self, ErrorKind},
};

pub fn get_error<T>(ok: T) -> Result<T, UdtError> {
    let err_code = unsafe { udt_sys::udt_getlasterror_code() };
    match UdtError::from(err_code) {
        UdtError::Success(_) => Ok(ok),
        e => Err(e),
    }
}

#[derive(Clone, Debug)]
pub enum UdtError {
    Success(String),
    ConnSetup(String),
    NoServer(String),
    ConnRej(String),
    SockFail(String),
    SecFail(String),
    ConnFail(String),
    ConnLost(String),
    NoConn(String),
    Resource(String),
    Thread(String),
    NoBuf(String),
    File(String),
    InvRdOff(String),
    RdPerm(String),
    InvWrOff(String),
    WrPerm(String),
    InvOp(String),
    BoundSock(String),
    ConnSock(String),
    InvParam(String),
    InvSock(String),
    UnboundSock(String),
    NoListen(String),
    RdvNoServ(String),
    RdvUnbound(String),
    StreamIll(String),
    DgramIll(String),
    DupListen(String),
    LargeMsg(String),
    AsyncFail(String),
    AsyncSnd(String),
    AsyncRcv(String),
    Timeout(String),
    PeerErr(String),
}

impl From<i32> for UdtError {
    fn from(code: i32) -> Self {
        match code {
            0 => UdtError::Success(get_error_desc()),
            1000 => UdtError::ConnSetup(get_error_desc()),
            1001 => UdtError::NoServer(get_error_desc()),
            1002 => UdtError::ConnRej(get_error_desc()),
            1003 => UdtError::SockFail(get_error_desc()),
            1004 => UdtError::SecFail(get_error_desc()),
            2000 => UdtError::ConnFail(get_error_desc()),
            2001 => UdtError::ConnLost(get_error_desc()),
            2002 => UdtError::NoConn(get_error_desc()),
            3000 => UdtError::Resource(get_error_desc()),
            3001 => UdtError::Thread(get_error_desc()),
            3002 => UdtError::NoBuf(get_error_desc()),
            4000 => UdtError::File(get_error_desc()),
            4001 => UdtError::InvRdOff(get_error_desc()),
            4002 => UdtError::RdPerm(get_error_desc()),
            4003 => UdtError::InvWrOff(get_error_desc()),
            4004 => UdtError::WrPerm(get_error_desc()),
            5000 => UdtError::InvOp(get_error_desc()),
            5001 => UdtError::BoundSock(get_error_desc()),
            5002 => UdtError::ConnSock(get_error_desc()),
            5003 => UdtError::InvParam(get_error_desc()),
            5004 => UdtError::InvSock(get_error_desc()),
            5005 => UdtError::UnboundSock(get_error_desc()),
            5006 => UdtError::NoListen(get_error_desc()),
            5007 => UdtError::RdvNoServ(get_error_desc()),
            5008 => UdtError::RdvUnbound(get_error_desc()),
            5009 => UdtError::StreamIll(get_error_desc()),
            5010 => UdtError::DgramIll(get_error_desc()),
            5011 => UdtError::DupListen(get_error_desc()),
            5012 => UdtError::LargeMsg(get_error_desc()),
            6000 => UdtError::AsyncFail(get_error_desc()),
            6001 => UdtError::AsyncSnd(get_error_desc()),
            6002 => UdtError::AsyncRcv(get_error_desc()),
            6003 => UdtError::Timeout(get_error_desc()),
            7000 => UdtError::PeerErr(get_error_desc()),
            _ => unreachable!(format!("unrecognized error code {}", code)),
        }
    }
}

impl Display for UdtError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let msg = match self {
            UdtError::Success(msg) => msg,
            UdtError::ConnSetup(msg) => msg,
            UdtError::NoServer(msg) => msg,
            UdtError::ConnRej(msg) => msg,
            UdtError::SockFail(msg) => msg,
            UdtError::SecFail(msg) => msg,
            UdtError::ConnFail(msg) => msg,
            UdtError::ConnLost(msg) => msg,
            UdtError::NoConn(msg) => msg,
            UdtError::Resource(msg) => msg,
            UdtError::Thread(msg) => msg,
            UdtError::NoBuf(msg) => msg,
            UdtError::File(msg) => msg,
            UdtError::InvRdOff(msg) => msg,
            UdtError::RdPerm(msg) => msg,
            UdtError::InvWrOff(msg) => msg,
            UdtError::WrPerm(msg) => msg,
            UdtError::InvOp(msg) => msg,
            UdtError::BoundSock(msg) => msg,
            UdtError::ConnSock(msg) => msg,
            UdtError::InvParam(msg) => msg,
            UdtError::InvSock(msg) => msg,
            UdtError::UnboundSock(msg) => msg,
            UdtError::NoListen(msg) => msg,
            UdtError::RdvNoServ(msg) => msg,
            UdtError::RdvUnbound(msg) => msg,
            UdtError::StreamIll(msg) => msg,
            UdtError::DgramIll(msg) => msg,
            UdtError::DupListen(msg) => msg,
            UdtError::LargeMsg(msg) => msg,
            UdtError::AsyncFail(msg) => msg,
            UdtError::AsyncSnd(msg) => msg,
            UdtError::AsyncRcv(msg) => msg,
            UdtError::Timeout(msg) => msg,
            UdtError::PeerErr(msg) => msg,
        };
        write!(f, "{}", msg)
    }
}

impl Error for UdtError {}

impl From<UdtError> for io::Error {
    fn from(e: UdtError) -> Self {
        io::Error::new(
            match e {
                UdtError::Success(_) => ErrorKind::Other,
                UdtError::ConnSetup(_) => ErrorKind::ConnectionRefused,
                UdtError::NoServer(_) => ErrorKind::ConnectionRefused,
                UdtError::ConnRej(_) => ErrorKind::ConnectionRefused,
                UdtError::SockFail(_) => ErrorKind::AddrNotAvailable,
                UdtError::SecFail(_) => ErrorKind::ConnectionRefused,
                UdtError::ConnFail(_) => ErrorKind::ConnectionRefused,
                UdtError::ConnLost(_) => ErrorKind::ConnectionAborted,
                UdtError::NoConn(_) => ErrorKind::NotConnected,
                UdtError::Resource(_) => ErrorKind::Other,
                UdtError::Thread(_) => ErrorKind::Other,
                UdtError::NoBuf(_) => ErrorKind::Other,
                UdtError::File(_) => ErrorKind::NotFound,
                UdtError::InvRdOff(_) => ErrorKind::InvalidInput,
                UdtError::RdPerm(_) => ErrorKind::PermissionDenied,
                UdtError::InvWrOff(_) => ErrorKind::InvalidInput,
                UdtError::WrPerm(_) => ErrorKind::PermissionDenied,
                UdtError::InvOp(_) => ErrorKind::InvalidInput,
                UdtError::BoundSock(_) => ErrorKind::AddrInUse,
                UdtError::ConnSock(_) => ErrorKind::AddrInUse,
                UdtError::InvParam(_) => ErrorKind::InvalidInput,
                UdtError::InvSock(_) => ErrorKind::AddrNotAvailable,
                UdtError::UnboundSock(_) => ErrorKind::NotConnected,
                UdtError::NoListen(_) => ErrorKind::InvalidInput,
                UdtError::RdvNoServ(_) => ErrorKind::ConnectionRefused,
                UdtError::RdvUnbound(_) => ErrorKind::ConnectionRefused,
                UdtError::StreamIll(_) => ErrorKind::InvalidInput,
                UdtError::DgramIll(_) => ErrorKind::InvalidInput,
                UdtError::DupListen(_) => ErrorKind::AddrInUse,
                UdtError::LargeMsg(_) => ErrorKind::Other,
                UdtError::AsyncFail(_) => ErrorKind::WouldBlock,
                UdtError::AsyncSnd(_) => ErrorKind::WouldBlock,
                UdtError::AsyncRcv(_) => ErrorKind::WouldBlock,
                UdtError::Timeout(_) => ErrorKind::TimedOut,
                UdtError::PeerErr(_) => ErrorKind::Other,
            },
            e,
        )
    }
}

fn get_error_desc() -> String {
    unsafe {
        return CStr::from_ptr(udt_sys::udt_getlasterror_desc())
            .to_str()
            .unwrap()
            .to_string();
    };
}
