// Convert Win32 TCP row structures to Rust types

use std::net::{IpAddr, SocketAddr};
use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::E_NOTIMPL;
use windows::Win32::NetworkManagement::IpHelper::{
    MIB_TCP6ROW, MIB_TCPROW_LH, MIB_TCPROW_LH_0, MIB_TCP_STATE,
};
use windows::Win32::Networking::WinSock::{IN6_ADDR, IN6_ADDR_0};

#[derive(Debug)]
pub struct TcpRow {
    state: TcpState,
    local: SocketAddr,
    remote: SocketAddr,
}

impl TryFrom<MIB_TCPROW_LH> for TcpRow {
    type Error = windows::core::Error;

    fn try_from(value: MIB_TCPROW_LH) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            state: TcpState::try_from(unsafe { value.Anonymous.dwState })?,
            local: SocketAddr::new(
                IpAddr::from(u32::from_be(value.dwLocalAddr).to_be_bytes()),
                u16::from_be(value.dwLocalPort as u16),
            ),
            remote: SocketAddr::new(
                IpAddr::from(u32::from_be(value.dwRemoteAddr).to_be_bytes()),
                u16::from_be(value.dwRemotePort as u16),
            ),
        })
    }
}

impl TryFrom<MIB_TCP6ROW> for TcpRow {
    type Error = windows::core::Error;

    fn try_from(value: MIB_TCP6ROW) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            state: TcpState::try_from(value.State.0 as u32)?,
            local: SocketAddr::new(
                IpAddr::from(unsafe { value.LocalAddr.u.Byte }),
                u16::from_be(value.dwLocalPort as u16),
            ),
            remote: SocketAddr::new(
                IpAddr::from(unsafe { value.RemoteAddr.u.Byte }),
                u16::from_be(value.dwRemotePort as u16),
            ),
        })
    }
}

#[derive(Debug)]
enum TcpState {
    Closed,
    Listen,
    SynSent,
    SynReceived,
    Established,
    FinWait1,
    FinWait2,
    CloseWait,
    Closing,
    LastAck,
    TimeWait,
    DeleteTcb,
}

impl TryFrom<u32> for TcpState {
    type Error = windows::core::Error;

    fn try_from(value: u32) -> std::result::Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Closed),
            2 => Ok(Self::Listen),
            3 => Ok(Self::SynSent),
            4 => Ok(Self::SynReceived),
            5 => Ok(Self::Established),
            6 => Ok(Self::FinWait1),
            7 => Ok(Self::FinWait2),
            8 => Ok(Self::CloseWait),
            9 => Ok(Self::Closing),
            10 => Ok(Self::LastAck),
            11 => Ok(Self::TimeWait),
            12 => Ok(Self::DeleteTcb),
            _ => Err(Error::from_hresult(E_NOTIMPL)),
        }
    }
}

fn main() -> windows::core::Result<()> {
    // Example IPv4 TCP row conversion
    let ipv4_row = MIB_TCPROW_LH {
        Anonymous: MIB_TCPROW_LH_0 { dwState: 5 }, // MIB_TCP_STATE_ESTAB
        dwLocalAddr: 0x0100007f,                   // 127.0.0.1 in network byte order
        dwLocalPort: 0x5000,                       // Port 80 in network byte order
        dwRemoteAddr: 0x00000000,                  // 0.0.0.0
        dwRemotePort: 0x0000,
    };

    let tcp_row = TcpRow::try_from(ipv4_row)?;
    println!("IPv4 TCP Row: {:?}", tcp_row);

    // Example IPv6 TCP row conversion
    let ipv6_row = MIB_TCP6ROW {
        State: MIB_TCP_STATE(5), // ESTABLISHED
        dwLocalPort: 0x5000,     // Port 80 in network byte order
        LocalAddr: IN6_ADDR {
            u: IN6_ADDR_0 {
                Byte: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1], // ::1
            },
        },
        dwLocalScopeId: 0,
        dwRemotePort: 0x0000,
        RemoteAddr: IN6_ADDR {
            u: IN6_ADDR_0 {
                Byte: [0; 16], // ::
            },
        },
        dwRemoteScopeId: 0,
    };

    let tcp_row_v6 = TcpRow::try_from(ipv6_row)?;
    println!("IPv6 TCP Row: {:?}", tcp_row_v6);

    Ok(())
}
