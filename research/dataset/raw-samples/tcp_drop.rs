use bytemuck::try_cast;
use color_eyre::eyre::eyre;
use color_eyre::Report;
use serde::Serialize;
use std::net::{IpAddr, SocketAddr};
use windows::Win32::NetworkManagement::IpHelper::{MIB_TCP6ROW, MIB_TCPROW_LH};

#[derive(Serialize)]
pub struct TcpRow {
    state: TcpState,
    local: SocketAddr,
    remote: SocketAddr,
}

impl TryFrom<MIB_TCPROW_LH> for TcpRow {
    type Error = Report;

    fn try_from(value: MIB_TCPROW_LH) -> Result<Self, Self::Error> {
        Ok(Self {
            state: TcpState::try_from(unsafe { value.Anonymous.dwState })?,
            local: SocketAddr::new(
                IpAddr::from(try_cast::<u32, [u8; 4]>(value.dwLocalAddr)?),
                value.dwLocalPort as u16,
            ),
            remote: SocketAddr::new(
                IpAddr::from(try_cast::<u32, [u8; 4]>(value.dwRemoteAddr)?),
                value.dwRemotePort as u16,
            ),
        })
    }
}

impl TryFrom<MIB_TCP6ROW> for TcpRow {
    type Error = Report;

    fn try_from(value: MIB_TCP6ROW) -> Result<Self, Self::Error> {
        Ok(Self {
            state: TcpState::try_from(value.State.0 as u32)?,
            local: SocketAddr::new(
                IpAddr::from(unsafe { value.LocalAddr.u.Byte }),
                value.dwLocalPort as u16,
            ),
            remote: SocketAddr::new(
                IpAddr::from(unsafe { value.RemoteAddr.u.Byte }),
                value.dwRemotePort as u16,
            ),
        })
    }
}

#[derive(Serialize)]
enum TcpState {
    Closed,
    Listen,
    #[serde(rename = "SYN-Sent")]
    SynSent,
    #[serde(rename = "SYN-Received")]
    SynReceived,
    Established,
    #[serde(rename = "FIN-Wait-1")]
    FinWait1,
    #[serde(rename = "FIN-Wait-2")]
    FinWait2,
    #[serde(rename = "Close-Wait")]
    CloseWait,
    Closing,
    #[serde(rename = "Last-ACK")]
    LastAck,
    #[serde(rename = "Time-Wait")]
    TimeWait,
    #[serde(rename = "Delete-TCB")]
    DeleteTcb,
}

impl TryFrom<u32> for TcpState {
    type Error = Report;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
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
            _ => Err(eyre!("invalid tcp state")),
        }
    }
}