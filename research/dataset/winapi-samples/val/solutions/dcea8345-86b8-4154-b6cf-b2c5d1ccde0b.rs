use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::{ERROR_BUFFER_OVERFLOW, NO_ERROR, WIN32_ERROR};
use windows::Win32::NetworkManagement::IpHelper::{
    GetAdaptersAddresses, GET_ADAPTERS_ADDRESSES_FLAGS, IP_ADAPTER_ADDRESSES_LH,
};
use windows::Win32::Networking::WinSock::{ADDRESS_FAMILY, AF_UNSPEC, SOCKADDR_IN, SOCKADDR_IN6};

fn is_unicast_site_local(ipv6: &std::net::Ipv6Addr) -> bool {
    // Check if IPv6 address is site-local (fec0::/10)
    (ipv6.segments()[0] & 0xffc0) == 0xfec0
}

fn main() -> Result<()> {
    println!("Enumerating network adapters...\n");

    let flags = GET_ADAPTERS_ADDRESSES_FLAGS::default();

    let mut buf_size = 0u32;
    let error = unsafe {
        GetAdaptersAddresses(
            AF_UNSPEC.0.into(),
            flags,
            Some(std::ptr::null_mut()),
            None,
            &mut buf_size,
        )
    };

    match WIN32_ERROR(error) {
        ERROR_BUFFER_OVERFLOW => {
            let mut buf = vec![0u8; buf_size as usize];

            let error = unsafe {
                GetAdaptersAddresses(
                    AF_UNSPEC.0.into(),
                    flags,
                    Some(std::ptr::null_mut()),
                    Some(buf.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH),
                    &mut buf_size,
                )
            };

            match WIN32_ERROR(error) {
                NO_ERROR => {
                    let p_adapter = buf.as_ptr() as *const IP_ADAPTER_ADDRESSES_LH;

                    while !p_adapter.is_null() {
                        let adapter = unsafe { &*p_adapter };
                        println!("Adapter: {:?}", adapter.Description);

                        let mut p_dns = adapter.FirstDnsServerAddress;

                        while !p_dns.is_null() {
                            let sock_addr = unsafe { (*p_dns).Address.lpSockaddr };
                            if !sock_addr.is_null() {
                                let sa_family = unsafe { (*sock_addr).sa_family };
                                match sa_family {
                                    AF_UNSPEC => {}
                                    ADDRESS_FAMILY(2) => {
                                        let p_sockaddr_in = sock_addr.cast::<SOCKADDR_IN>();
                                        let ipv4 = u32::from_be(unsafe {
                                            (*p_sockaddr_in).sin_addr.S_un.S_addr
                                        });
                                        let ip = std::net::Ipv4Addr::from(ipv4);
                                        println!("    DNS Server (IPv4): {}", ip);
                                    }
                                    ADDRESS_FAMILY(23) => {
                                        let p_sockaddr_in6 = sock_addr.cast::<SOCKADDR_IN6>();
                                        let ipv6 = std::net::Ipv6Addr::from(unsafe {
                                            (*p_sockaddr_in6).sin6_addr.u.Byte
                                        });
                                        if !ipv6.is_unspecified() && !is_unicast_site_local(&ipv6) {
                                            println!("    DNS Server (IPv6): {}", ipv6);
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            p_dns = unsafe { (*p_dns).Next };
                        }
                    }
                }
                e => {
                    return Err(Error::from_hresult(HRESULT::from_win32(e.0)));
                }
            }
        }
        e => {
            return Err(Error::from_hresult(HRESULT::from_win32(e.0)));
        }
    }

    Ok(())
}
