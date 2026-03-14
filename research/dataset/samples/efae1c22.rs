use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use windows::core::{Error, Result};

use windows::Win32::{
    Foundation::{ERROR_BUFFER_OVERFLOW, NO_ERROR, WIN32_ERROR},
    NetworkManagement::IpHelper::{
        GetAdaptersAddresses, GAA_FLAG_INCLUDE_TUNNEL_BINDINGORDER, IP_ADAPTER_ADDRESSES_LH,
    },
    Networking::WinSock::{AF_INET, AF_INET6, AF_UNSPEC, SOCKADDR_IN, SOCKADDR_IN6},
};

fn main() -> Result<()> {
    let flags = GAA_FLAG_INCLUDE_TUNNEL_BINDINGORDER;

    // First call to get required buffer size
    let mut buf_size = 0u32;
    let error =
        unsafe { GetAdaptersAddresses(AF_UNSPEC.0.into(), flags, None, None, &mut buf_size) };

    match WIN32_ERROR(error) {
        ERROR_BUFFER_OVERFLOW => {
            // Allocate buffer with required size
            let mut buf = vec![0u8; buf_size as usize];

            let (prefix, body, _) = unsafe { buf.align_to_mut::<IP_ADAPTER_ADDRESSES_LH>() };

            let mut buf_size = buf_size - prefix.len() as u32;
            let error = unsafe {
                GetAdaptersAddresses(
                    AF_UNSPEC.0.into(),
                    flags,
                    None,
                    Some(body.as_mut_ptr()),
                    &mut buf_size,
                )
            };

            match WIN32_ERROR(error) {
                NO_ERROR => {
                    // Process adapters
                    let mut p_adapter = body.as_ptr();
                    while !p_adapter.is_null() {
                        let adapter = unsafe { &*p_adapter };
                        let dns_servers = unsafe { get_adapter_dns_servers(adapter) };
                        for ip in dns_servers {
                            println!("DNS Server: {}", ip);
                        }
                        p_adapter = adapter.Next;
                    }
                }
                e => {
                    return Err(Error::new(e.into(), "GetAdaptersAddresses failed"));
                }
            }
        }
        NO_ERROR => {
            // Buffer was already large enough
            let mut buf = vec![0u8; 16384];
            let mut buf_size = buf.len() as u32;
            let (_prefix, body, _) = unsafe { buf.align_to_mut::<IP_ADAPTER_ADDRESSES_LH>() };

            let error = unsafe {
                GetAdaptersAddresses(
                    AF_UNSPEC.0.into(),
                    flags,
                    None,
                    Some(body.as_mut_ptr()),
                    &mut buf_size,
                )
            };

            match WIN32_ERROR(error) {
                NO_ERROR => {
                    let mut p_adapter = body.as_ptr();
                    while !p_adapter.is_null() {
                        let adapter = unsafe { &*p_adapter };
                        let dns_servers = unsafe { get_adapter_dns_servers(adapter) };
                        for ip in dns_servers {
                            println!("DNS Server: {}", ip);
                        }
                        p_adapter = adapter.Next;
                    }
                }
                e => {
                    return Err(Error::new(e.into(), "GetAdaptersAddresses failed"));
                }
            }
        }
        e => {
            return Err(Error::new(e.into(), "GetAdaptersAddresses failed"));
        }
    }

    Ok(())
}

unsafe fn get_adapter_dns_servers(a: &IP_ADAPTER_ADDRESSES_LH) -> Vec<IpAddr> {
    let mut p_address = a.FirstDnsServerAddress;
    let mut dns_servers = Vec::new();

    while !p_address.is_null() {
        let address = unsafe { &*p_address };
        let sock_addr = address.Address.lpSockaddr;

        if !sock_addr.is_null() {
            let sa_family = unsafe { (*sock_addr).sa_family };

            match sa_family {
                AF_INET => {
                    let p_sockaddr_in = sock_addr.cast::<SOCKADDR_IN>();
                    let ipv4 = Ipv4Addr::from(u32::from_be((*p_sockaddr_in).sin_addr.S_un.S_addr));
                    if !ipv4.is_unspecified() {
                        dns_servers.push(ipv4.into());
                    }
                }
                AF_INET6 => {
                    let p_sockaddr_in6 = sock_addr.cast::<SOCKADDR_IN6>();
                    let ipv6 = Ipv6Addr::from((*p_sockaddr_in6).sin6_addr.u.Byte);
                    if !ipv6.is_unspecified() && !is_unicast_site_local(&ipv6) {
                        dns_servers.push(ipv6.into());
                    }
                }
                _ => {}
            }
        }

        p_address = address.Next;
    }

    dns_servers
}

fn is_unicast_site_local(ipv6: &Ipv6Addr) -> bool {
    // Check if IPv6 address is site-local (fec0::/10)
    (ipv6.segments()[0] & 0xffc0) == 0xfec0
}
