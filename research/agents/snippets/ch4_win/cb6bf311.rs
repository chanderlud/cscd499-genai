use std::collections::HashSet;
use windows::core::{Error, Result};
use windows::Win32::Foundation::{ERROR_BUFFER_OVERFLOW, NO_ERROR, WIN32_ERROR};
use windows::Win32::NetworkManagement::IpHelper::{
    GetAdaptersAddresses, GAA_FLAG_INCLUDE_TUNNEL_BINDINGORDER, IP_ADAPTER_ADDRESSES_LH,
};
use windows::Win32::Networking::WinSock::{AF_INET, AF_INET6, AF_UNSPEC, SOCKADDR_IN6};

fn main() -> Result<()> {
    println!("=== IPv6 Address Parser Demo ===\n");

    let flags = GAA_FLAG_INCLUDE_TUNNEL_BINDINGORDER;

    // First call to get required buffer size
    let mut buf_size = 0u32;
    let error =
        unsafe { GetAdaptersAddresses(AF_UNSPEC.0.into(), flags, None, None, &mut buf_size) };

    match WIN32_ERROR(error) {
        ERROR_BUFFER_OVERFLOW => {
            println!(
                "Buffer overflow detected, required size: {} bytes",
                buf_size
            );
        }
        NO_ERROR => {
            println!("Initial call succeeded with size: {} bytes", buf_size);
        }
        e => return Err(Error::new(e.into(), "GetAdaptersAddresses failed")),
    }

    // Allocate buffer with headroom and align for IP_ADAPTER_ADDRESSES_LH
    let block_size = std::mem::size_of::<IP_ADAPTER_ADDRESSES_LH>() as u32;
    let new_capacity = buf_size + block_size;
    let mut buf = Vec::<u8>::with_capacity(new_capacity as usize);
    let (prefix, body, _) = unsafe { buf.align_to_mut::<IP_ADAPTER_ADDRESSES_LH>() };

    // Second call with allocated buffer
    let mut buf_size = new_capacity - prefix.len() as u32;
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
        NO_ERROR => println!("GetAdaptersAddresses succeeded\n"),
        e => return Err(Error::new(e.into(), "GetAdaptersAddresses failed")),
    }

    // Iterate through adapters and extract IPv6 addresses
    let mut ipv6_addresses = Vec::new();
    let mut p_adapter = body.as_mut_ptr();

    while !p_adapter.is_null() {
        let adapter = unsafe { &*p_adapter };
        let adapter_name =
            unsafe { adapter.Description.to_string() }.unwrap_or_else(|_| "<unknown>".to_string());
        println!("Adapter: {}", adapter_name);

        let mut p_address = adapter.FirstDnsServerAddress;
        let mut adapter_ipv6_count = 0;

        while !p_address.is_null() {
            unsafe {
                let sock_addr = (*p_address).Address.lpSockaddr;
                if !sock_addr.is_null() {
                    match (*sock_addr).sa_family {
                        AF_INET6 => {
                            let p_sockaddr_in6 = sock_addr.cast::<SOCKADDR_IN6>();
                            let ipv6_bytes = (*p_sockaddr_in6).sin6_addr.u.Byte;
                            let ipv6 = std::net::Ipv6Addr::from(ipv6_bytes);
                            if !ipv6.is_unspecified() && !is_unicast_site_local(&ipv6) {
                                ipv6_addresses.push(ipv6);
                                adapter_ipv6_count += 1;
                            }
                        }
                        AF_INET => println!("  - IPv4 address (skipped in IPv6 demo)"),
                        _ => println!("  - Unknown address family (skipped)"),
                    }
                }
                p_address = (*p_address).Next;
            }
        }

        if adapter_ipv6_count > 0 {
            println!("  Found {} IPv6 address(es)\n", adapter_ipv6_count);
        } else {
            println!("  No IPv6 addresses found\n");
        }

        p_adapter = adapter.Next;
    }

    // Print unique IPv6 addresses
    println!("=== Unique IPv6 Addresses Found ===");
    let mut unique_addresses = HashSet::new();
    for addr in ipv6_addresses {
        unique_addresses.insert(addr);
    }

    if unique_addresses.is_empty() {
        println!("No valid IPv6 addresses found on this system.");
    } else {
        for (i, addr) in unique_addresses.iter().enumerate() {
            println!("{}. {}", i + 1, addr);
        }
    }

    Ok(())
}

#[inline]
fn is_unicast_site_local(ipv6: &std::net::Ipv6Addr) -> bool {
    (ipv6.segments()[0] & 0xffc0) == 0xfec0
}
