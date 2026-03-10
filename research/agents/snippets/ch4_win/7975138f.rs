use windows::core::{Error, Result};
use windows::Win32::Foundation::{ERROR_BUFFER_OVERFLOW, NO_ERROR, WIN32_ERROR};
use windows::Win32::NetworkManagement::IpHelper::{
    GetAdaptersAddresses, GAA_FLAG_INCLUDE_TUNNEL_BINDINGORDER, IP_ADAPTER_ADDRESSES_LH,
};
use windows::Win32::Networking::WinSock::{
    AF_INET, AF_INET6, AF_UNSPEC, SOCKADDR_IN, SOCKADDR_IN6,
};

fn main() -> Result<()> {
    println!("=== Parsing IPv4 addresses from SOCKADDR_IN structures ===\n");

    // Get all network adapters with their DNS server addresses
    let dns_servers = get_dns_servers()?;

    println!("Found {} unique IPv4 DNS servers:\n", dns_servers.len());
    for (i, ip) in dns_servers.iter().enumerate() {
        println!("{}. {}", i + 1, ip);
    }

    Ok(())
}

fn get_dns_servers() -> Result<Vec<std::net::IpAddr>> {
    let flags = GAA_FLAG_INCLUDE_TUNNEL_BINDINGORDER;

    // First call to get required buffer size
    let mut buf_size = 0u32;
    let error =
        unsafe { GetAdaptersAddresses(AF_UNSPEC.0.into(), flags, None, None, &mut buf_size) };

    match WIN32_ERROR(error) {
        ERROR_BUFFER_OVERFLOW => {}
        NO_ERROR => return Ok(Vec::new()),
        _ => return Err(Error::from_thread()),
    }

    // Allocate properly aligned buffer
    let mut buf = vec![0u8; buf_size as usize];
    let buf_ptr = buf.as_mut_ptr().cast::<IP_ADAPTER_ADDRESSES_LH>();

    // Second call with actual buffer
    let error = unsafe {
        GetAdaptersAddresses(
            AF_UNSPEC.0.into(),
            flags,
            None,
            Some(buf_ptr),
            &mut buf_size,
        )
    };

    match WIN32_ERROR(error) {
        NO_ERROR => {}
        _ => return Err(Error::from_thread()),
    }

    // Iterate through adapters and collect DNS servers
    let mut dns_servers = Vec::new();
    let mut p_adapter = buf_ptr;

    while !p_adapter.is_null() {
        let adapter_dns = unsafe { get_adapter_dns_servers(&*p_adapter) };
        for ip in adapter_dns {
            if !dns_servers.contains(&ip) {
                dns_servers.push(ip);
            }
        }
        p_adapter = unsafe { (*p_adapter).Next };
    }

    Ok(dns_servers)
}

unsafe fn get_adapter_dns_servers(a: &IP_ADAPTER_ADDRESSES_LH) -> Vec<std::net::IpAddr> {
    let mut p_address = a.FirstDnsServerAddress;
    let mut dns_servers = Vec::new();

    while !p_address.is_null() {
        let sock_addr = (*p_address).Address.lpSockaddr;
        if !sock_addr.is_null() {
            match (*sock_addr).sa_family {
                AF_INET => {
                    // Parse IPv4 address from SOCKADDR_IN
                    let p_sockaddr_in = sock_addr.cast::<SOCKADDR_IN>();
                    let ipv4 = std::net::Ipv4Addr::from(u32::from_be(
                        (*p_sockaddr_in).sin_addr.S_un.S_addr,
                    ));
                    if !ipv4.is_unspecified() {
                        dns_servers.push(ipv4.into());
                    }
                }
                AF_INET6 => {
                    // Parse IPv6 address from SOCKADDR_IN6
                    let p_sockaddr_in6 = sock_addr.cast::<SOCKADDR_IN6>();
                    let ipv6 = std::net::Ipv6Addr::from((*p_sockaddr_in6).sin6_addr.u.Byte);
                    if !ipv6.is_unspecified() && !is_unicast_site_local(&ipv6) {
                        dns_servers.push(ipv6.into());
                    }
                }
                _ => {}
            }
        }
        p_address = (*p_address).Next;
    }

    dns_servers
}

#[inline]
fn is_unicast_site_local(ipv6: &std::net::Ipv6Addr) -> bool {
    // Check if IPv6 address is site-local (fec0::/10)
    (ipv6.segments()[0] & 0xffc0) == 0xfec0
}
