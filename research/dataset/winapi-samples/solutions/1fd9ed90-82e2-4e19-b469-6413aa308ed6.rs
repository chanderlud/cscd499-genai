use windows::core::{Error, Result};
use windows::Win32::Foundation::{ERROR_BUFFER_OVERFLOW, NO_ERROR, WIN32_ERROR};
use windows::Win32::NetworkManagement::IpHelper::{
    GetAdaptersAddresses, GAA_FLAG_INCLUDE_TUNNEL_BINDINGORDER, IP_ADAPTER_ADDRESSES_LH,
};
use windows::Win32::Networking::WinSock::{
    AF_INET, AF_INET6, AF_UNSPEC, SOCKADDR_IN, SOCKADDR_IN6,
};

const AF_INET_VAL: u16 = AF_INET.0;
const AF_INET6_VAL: u16 = AF_INET6.0;

fn is_unicast_site_local(ipv6: &std::net::Ipv6Addr) -> bool {
    (ipv6.segments()[0] & 0xffc0) == 0xfec0
}

fn main() -> Result<()> {
    println!("=== Demonstrating ERROR_BUFFER_OVERFLOW handling with GetAdaptersAddresses ===\n");

    let flags = GAA_FLAG_INCLUDE_TUNNEL_BINDINGORDER;

    // First call to determine required buffer size
    let mut buf_size = 0u32;
    let error =
        unsafe { GetAdaptersAddresses(AF_UNSPEC.0.into(), flags, None, None, &mut buf_size) };

    match WIN32_ERROR(error) {
        ERROR_BUFFER_OVERFLOW => {
            println!(
                "✓ Got ERROR_BUFFER_OVERFLOW, required buffer size: {}",
                buf_size
            );

            // Allocate buffer with headroom and align to IP_ADAPTER_ADDRESSES_LH boundary
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
                NO_ERROR => {
                    println!("✓ GetAdaptersAddresses succeeded with allocated buffer\n");

                    // Iterate through adapter addresses
                    let mut p_adapter = body.as_ptr();
                    let mut adapter_count = 0u32;

                    while !p_adapter.is_null() {
                        adapter_count += 1;
                        let adapter = unsafe { &*p_adapter };

                        if let Some(name) = unsafe { adapter.Description.0.as_ref() } {
                            println!("Adapter {}: {}", adapter_count, name);
                        }

                        // Extract DNS server addresses
                        let mut p_dns = adapter.FirstDnsServerAddress;
                        let mut dns_count = 0;

                        while !p_dns.is_null() {
                            unsafe {
                                let sock_addr = (*p_dns).Address.lpSockaddr;
                                if !sock_addr.is_null() {
                                    match (*sock_addr).sa_family.0 {
                                        AF_INET_VAL => {
                                            let p_sockaddr_in = sock_addr.cast::<SOCKADDR_IN>();
                                            let ipv4 =
                                                u32::from_be((*p_sockaddr_in).sin_addr.S_un.S_addr);
                                            let ip = std::net::Ipv4Addr::from(ipv4);
                                            if !ip.is_unspecified() {
                                                println!("  - IPv4 DNS: {}", ip);
                                                dns_count += 1;
                                            }
                                        }
                                        AF_INET6_VAL => {
                                            let p_sockaddr_in6 = sock_addr.cast::<SOCKADDR_IN6>();
                                            let ipv6 = std::net::Ipv6Addr::from(
                                                (*p_sockaddr_in6).sin6_addr.u.Byte,
                                            );
                                            if !ipv6.is_unspecified()
                                                && !is_unicast_site_local(&ipv6)
                                            {
                                                println!("  - IPv6 DNS: {}", ipv6);
                                                dns_count += 1;
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                                p_dns = (*p_dns).Next;
                            }
                        }

                        if dns_count > 0 {
                            println!("  DNS servers: {}", dns_count);
                        }

                        p_adapter = adapter.Next;
                    }

                    println!("\n✓ Total adapters found: {}", adapter_count);
                }
                e => return Err(Error::from_hresult(windows::core::HRESULT::from_win32(e.0))),
            }
        }
        NO_ERROR => {
            println!("✓ GetAdaptersAddresses succeeded on first call (no overflow needed)\n");

            // Handle case where no overflow occurred
            let mut buf_size = 0u32;
            let error = unsafe {
                GetAdaptersAddresses(AF_UNSPEC.0.into(), flags, None, None, &mut buf_size)
            };

            if WIN32_ERROR(error) == ERROR_BUFFER_OVERFLOW {
                let block_size = std::mem::size_of::<IP_ADAPTER_ADDRESSES_LH>() as u32;
                let new_capacity = buf_size + block_size;
                let mut buf = Vec::<u8>::with_capacity(new_capacity as usize);
                let (prefix, body, _) = unsafe { buf.align_to_mut::<IP_ADAPTER_ADDRESSES_LH>() };

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
                    NO_ERROR => {
                        let mut p_adapter = body.as_ptr();
                        let mut adapter_count = 0u32;

                        while !p_adapter.is_null() {
                            adapter_count += 1;
                            let adapter = unsafe { &*p_adapter };

                            if let Some(name) = unsafe { adapter.Description.0.as_ref() } {
                                println!("Adapter {}: {}", adapter_count, name);
                            }

                            p_adapter = adapter.Next;
                        }

                        println!("\n✓ Total adapters found: {}", adapter_count);
                    }
                    e => return Err(Error::from_hresult(windows::core::HRESULT::from_win32(e.0))),
                }
            } else {
                return Err(Error::from_hresult(windows::core::HRESULT::from_win32(error)));
            }
        }
        e => return Err(Error::from_hresult(windows::core::HRESULT::from_win32(e.0))),
    }

    println!("\n=== Demonstration complete ===");
    Ok(())
}
