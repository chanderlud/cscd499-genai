use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::ptr;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr}; // 移除未使用的导入 Ipv4Addr 和 Ipv6Addr
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs;
use std::path::Path;
use std::str::FromStr;
use serde::{Serialize, Deserialize};
use windows::{
    Win32::Foundation::*, Win32::NetworkManagement::WindowsFilteringPlatform::*,
    Win32::System::Rpc::*, core::*,
};

// CIDR网段结构体
#[derive(Debug, Clone)]
pub struct IpNetwork {
    pub ip: IpAddr,
    pub prefix_len: u8,
}

impl IpNetwork {
    pub fn new(ip: IpAddr, prefix_len: u8) -> Self {
        Self { ip, prefix_len }
    }
    pub fn from_cidr(cidr: &str) -> std::result::Result<Self, String> {
        let parts: Vec<&str> = cidr.split('/').collect();
        if parts.len() != 2 {
            return Err("Invalid CIDR format".to_string());
        }

        let ip: IpAddr = parts[0].parse().map_err(|_| "Invalid IP address")?;
        let prefix_len: u8 = parts[1].parse().map_err(|_| "Invalid prefix length")?;

        // 验证前缀长度
        let max_prefix = match ip {
            IpAddr::V4(_) => 32,
            IpAddr::V6(_) => 128,
        };

        if prefix_len > max_prefix {
            return Err(format!("Prefix length {} exceeds maximum {}", prefix_len, max_prefix));
        }

        // 将IP地址转换为正确的网络地址
        let network_ip = match ip {
            IpAddr::V4(ipv4) => {
                let ip_bytes = ipv4.octets();
                let ip_u32 = u32::from_be_bytes(ip_bytes);
                let mask = if prefix_len == 0 {
                    0u32
                } else if prefix_len == 32 {
                    u32::MAX
                } else {
                    !((1u32 << (32 - prefix_len)) - 1)
                };
                let network_u32 = ip_u32 & mask;
                let network_bytes = network_u32.to_be_bytes();
                IpAddr::V4(std::net::Ipv4Addr::from(network_bytes))
            },
            IpAddr::V6(_) => ip, // IPv6 处理复杂，暂时保持原样
        };

        Ok(Self::new(network_ip, prefix_len))
    }

    pub fn contains(&self, ip: &IpAddr) -> bool {
        match (self.ip, ip) {
            (IpAddr::V4(network_ip), IpAddr::V4(test_ip)) => {
                let mask = !((1u32 << (32 - self.prefix_len)) - 1);
                let network_addr = u32::from(network_ip) & mask;
                let test_addr = u32::from(*test_ip) & mask;
                network_addr == test_addr
            }
            (IpAddr::V6(network_ip), IpAddr::V6(test_ip)) => {
                let network_bytes = network_ip.octets();
                let test_bytes = test_ip.octets();
                let prefix_bytes = self.prefix_len / 8;
                let prefix_bits = self.prefix_len % 8;

                // 比较完整字节
                for i in 0..prefix_bytes as usize {
                    if network_bytes[i] != test_bytes[i] {
                        return false;
                    }
                }

                // 比较部分字节
                if prefix_bits > 0 {
                    let mask = 0xFF << (8 - prefix_bits);
                    let network_byte = network_bytes[prefix_bytes as usize] & mask;
                    let test_byte = test_bytes[prefix_bytes as usize] & mask;
                    if network_byte != test_byte {
                        return false;
                    }
                }

                true
            }
            _ => false, // IPv4 vs IPv6 不匹配
        }
    }
}

// WFP 常量定义
const FWP_ACTION_BLOCK: u32 = 0x00000001 | 0x00001000;
const FWP_ACTION_PERMIT: u32 = 0x00000002 | 0x00001000;
static mut WEIGHT_VALUE: u64 = 1000;
static mut EFFECTIVE_WEIGHT_VALUE: u64 = 0;

// 缓存结构体，用于提高性能
#[derive(Debug, Clone)]
pub struct FilterCache {
    pub app_path_cache: std::collections::HashMap<String, String>, // 原始路径 -> NT路径
    pub layer_cache: std::collections::HashMap<String, Vec<GUID>>, // 规则签名 -> 层列表
}

impl FilterCache {
    pub fn new() -> Self {
        Self {
            app_path_cache: std::collections::HashMap::new(),
            layer_cache: std::collections::HashMap::new(),
        }
    }

    pub fn get_nt_path(&mut self, original_path: &str) -> Option<String> {
        if let Some(cached) = self.app_path_cache.get(original_path) {
            return Some(cached.clone());
        }

        if let Some(nt_path) = crate::nt::get_nt_path(original_path) {
            self.app_path_cache.insert(original_path.to_string(), nt_path.clone());
            Some(nt_path)
        } else {
            None
        }
    }
}

// 过滤规则结构体
#[derive(Debug, Clone)]
// 过滤规则结构体
pub struct FilterRule {
    pub name: String,                        // 规则名称
    pub app_path: Option<String>,            // 应用程序路径（可选）
    pub local: Option<String>,    // 本地IP地址/网段，格式如: "192.168.1.1" 或 "192.168.1.0/24"（可选）
    pub remote: Option<String>,   // 远程IP地址/网段，格式如: "8.8.8.8" 或 "8.8.0.0/16"（可选）
    pub local_port: Option<u16>,             // 本地端口（可选）
    pub remote_port: Option<u16>,            // 远程端口（可选）
    pub local_port_range: Option<(u16, u16)>, // 本地端口范围（可选）
    pub remote_port_range: Option<(u16, u16)>, // 远程端口范围（可选）
    pub protocol: Option<Protocol>,          // 协议类型（可选）
    pub direction: Direction,                // 流量方向
    pub action: FilterAction,                // 过滤动作（允许/阻止）
    pub priority: u32,                       // 规则优先级（数字越大优先级越高）
    pub group: Option<String>,               // 规则分组
    pub enabled: bool,                       // 规则是否启用
    pub time_control: Option<TimeControl>,   // 时间控制
    pub description: Option<String>,         // 规则描述
}

#[derive(Debug, Clone, PartialEq)]
pub enum Protocol {
    Tcp,
    Udp,
    Icmp,
    IcmpV6,
    Igmp,
    Ah,      // Authentication Header
    Esp,     // Encapsulating Security Payload
    Gre,     // Generic Routing Encapsulation
    Ipsec,   // IP Security
    Any,     // 任意协议
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Protocol::Tcp => write!(f, "TCP"),
            Protocol::Udp => write!(f, "UDP"),
            Protocol::Icmp => write!(f, "ICMP"),
            Protocol::IcmpV6 => write!(f, "ICMPv6"),
            Protocol::Igmp => write!(f, "IGMP"),
            Protocol::Ah => write!(f, "AH"),
            Protocol::Esp => write!(f, "ESP"),
            Protocol::Gre => write!(f, "GRE"),
            Protocol::Ipsec => write!(f, "IPSEC"),
            Protocol::Any => write!(f, "任意协议"),
        }
    }
}

impl FromStr for Protocol {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "tcp" => Ok(Protocol::Tcp),
            "udp" => Ok(Protocol::Udp),
            "icmp" => Ok(Protocol::Icmp),
            "icmpv6" => Ok(Protocol::IcmpV6),
            "igmp" => Ok(Protocol::Igmp),
            "ah" => Ok(Protocol::Ah),
            "esp" => Ok(Protocol::Esp),
            "gre" => Ok(Protocol::Gre),
            "ipsec" => Ok(Protocol::Ipsec),
            "any" => Ok(Protocol::Any),
            _ => Err(format!("未知协议: {}", s)),
        }
    }
}

// 流量方向枚举
#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    Inbound,     // 入站流量
    Outbound,    // 出站流量
    Both,        // 双向流量
}
#[derive(Debug, Clone, PartialEq)]
pub enum FilterAction {
    Allow,
    Block,
}

impl FilterRule {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            app_path: None,
            local: None,
            remote: None,
            local_port: None,
            remote_port: None,
            local_port_range: None,
            remote_port_range: None,
            protocol: None,
            direction: Direction::Both,
            action: FilterAction::Block,
            priority: 0,
            group: None,
            enabled: true,
            time_control: None,
            description: None,
        }
    }

    pub fn app_path(mut self, path: &str) -> Self {
        self.app_path = Some(path.to_string());
        self
    }

    pub fn local_ip(mut self, ip: impl ToString) -> Self {
        self.local = Some(ip.to_string());
        self
    }

    pub fn remote_ip(mut self, ip: impl ToString) -> Self {
        self.remote = Some(ip.to_string());
        self
    }


    pub fn local_port(mut self, port: u16) -> Self {
        self.local_port = Some(port);
        self
    }

    pub fn remote_port(mut self, port: u16) -> Self {
        self.remote_port = Some(port);
        self
    }

    pub fn local_port_range(mut self, start: u16, end: u16) -> Self {
        self.local_port_range = Some((start, end));
        self
    }

    pub fn remote_port_range(mut self, start: u16, end: u16) -> Self {
        self.remote_port_range = Some((start, end));
        self
    }

    pub fn protocol(mut self, protocol: Protocol) -> Self {
        self.protocol = Some(protocol);
        self
    }

    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    pub fn action(mut self, action: FilterAction) -> Self {
        self.action = action;
        self
    }

    pub fn priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    pub fn group(mut self, group: &str) -> Self {
        self.group = Some(group.to_string());
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn time_control(mut self, time_control: TimeControl) -> Self {
        self.time_control = Some(time_control);
        self
    }

    pub fn description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    // 生成规则签名，用于缓存
    pub fn signature(&self) -> String {
        format!("{}_{:?}_{:?}_{:?}_{:?}_{:?}_{:?}_{:?}_{:?}",
                self.name,
                self.app_path,
                self.local,
                self.remote,
                self.local_port,
                self.remote_port,
                self.protocol,
                self.direction,
                self.action
        )
    }

    fn validate_ip(&self, ip: &IpAddr) -> bool {
        match ip {
            IpAddr::V4(ipv4) => {
                let octets = ipv4.octets();
                // 检查是否是有效的私有网络地址
                match octets[0] {
                    10 => true,  // 10.0.0.0/8
                    172 => (16..=31).contains(&octets[1]),  // 172.16.0.0/12
                    192 => octets[1] == 168,  // 192.168.0.0/16
                    // 对于公网 IP，这里可以添加其他验证规则
                    _ => true  // 暂时允许其他地址，可以根据需求修改
                }
            },
            IpAddr::V6(_) => true  // IPv6 地址验证逻辑
        }
    }

    pub fn validate(&self) -> std::result::Result<(), String> {
        // 验证远程 IP
        if let Some(remote) = &self.remote {
            // 尝试解析为单个IP地址
            if let Ok(ip) = remote.parse::<IpAddr>() {
                if !self.validate_ip(&ip) {
                    return Err(format!("无效的远程 IP 地址: {}", remote));
                }
            }
            // 尝试解析为CIDR网段
            else if let Ok(_network) = IpNetwork::from_cidr(remote) {
                // CIDR格式有效，通过验证
            }
            // 都不是，报错
            else {
                return Err(format!("无法解析的 IP 地址格式: {}", remote));
            }
        }

        // 验证本地 IP（如果存在）
        if let Some(local) = &self.local {
            // 尝试解析为单个IP地址
            if let Ok(ip) = local.parse::<IpAddr>() {
                if !self.validate_ip(&ip) {
                    return Err(format!("无效的本地 IP 地址: {}", local));
                }
            }
            // 尝试解析为CIDR网段
            else if let Ok(_network) = IpNetwork::from_cidr(local) {
                // CIDR格式有效，通过验证
            }
            // 都不是，报错
            else {
                return Err(format!("无法解析的本地 IP 地址格式: {}", local));
            }
        }

        Ok(())
    }
}

// 创建宽字符字符串的辅助函数
pub fn to_wide_string(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

// WFP控制器结构体
pub struct WfpController {
    engine_handle: HANDLE,
    pub filter_ids: Vec<u64>,
}

impl WfpController {
    // 创建新的WFP控制器实例
    pub fn new() -> Result<Self> {
        Ok(Self {
            engine_handle: HANDLE::default(),
            filter_ids: Vec::new(),
        })
    }

    // 初始化WFP引擎
    pub fn initialize(&mut self) -> Result<()> {
        unsafe {
            println!("正在初始化 Windows Filtering Platform...");

            // 创建会话名称
            let session_name = to_wide_string("AstralWFP Manager");
            let session_desc = to_wide_string("AstralWFP网络流量管理会话");

            let session = FWPM_SESSION0 {
                sessionKey: GUID::zeroed(),
                displayData: FWPM_DISPLAY_DATA0 {
                    name: PWSTR(session_name.as_ptr() as *mut u16),
                    description: PWSTR(session_desc.as_ptr() as *mut u16),
                },
                flags: FWPM_SESSION_FLAG_DYNAMIC,
                txnWaitTimeoutInMSec: 0,
                processId: 0,
                sid: ptr::null_mut(),
                username: PWSTR::null(),
                kernelMode: FALSE,
            };

            // 打开WFP会话
            let result = FwpmEngineOpen0(
                None,
                RPC_C_AUTHN_DEFAULT as u32,
                None,
                Some(&session),
                &mut self.engine_handle,
            );

            if WIN32_ERROR(result) == ERROR_SUCCESS {
                println!("✓ WFP引擎打开成功！");
                Ok(())
            } else {
                println!("❌ 打开WFP引擎失败: {} (可能需要管理员权限)", result);
                Err(Error::from_win32())
            }
        }
    }


    // 添加高级过滤器（支持复杂规则）
    pub fn add_advanced_filters(&mut self, rules: &[FilterRule]) -> Result<Vec<u64>> {
        unsafe {
            let mut added_ids = Vec::new();
            let mut added_count = 0;

            for rule in rules {
                // 验证规则
                if let Err(e) = rule.validate() {
                    println!("❌ 规则验证失败: {}", e);
                    continue;
                }

                // 根据方向和IP版本确定需要的层
                let layers = self.get_layers_for_rule(rule);
                for layer in layers {
                    println!("🧪 尝试在层 {} 上添加过滤器...", self.get_layer_name(&layer));
                    match self.add_advanced_network_filter(rule, layer) {
                        Ok(filter_id) => {
                            self.filter_ids.push(filter_id);
                            added_ids.push(filter_id);
                            added_count += 1;
                            println!("✅ 过滤器在层 {} 上添加成功 (ID: {})", self.get_layer_name(&layer), filter_id);
                        },
                        Err(e) => {
                            println!("❌ 过滤器在层 {} 上添加失败: {:?}", self.get_layer_name(&layer), e);
                        }
                    }
                }
            }

            if added_count > 0 {
                println!(
                    "\n🔍 网络流量控制已启动，共添加了 {} 个过滤器",
                    added_count
                );
                Ok(added_ids)
            } else {
                println!("❌ 没有成功添加任何过滤器");
                Err(Error::from_win32())
            }
        }
    }

    // 根据规则获取对应的WFP层 - 测试所有可能的层组合
    pub fn get_layers_for_rule(&self, rule: &FilterRule) -> Vec<GUID> {
        let mut layers = Vec::new();

        // 根据IP地址类型确定IPv4还是IPv6
        let is_ipv6 = rule.local.as_ref().map_or(false, |ip| ip.contains(":")) ||
            rule.remote.as_ref().map_or(false, |ip| ip.contains(":"));

        println!("🔍 规则分析: {} - 方向: {:?}, IPv6: {}", rule.name, rule.direction, is_ipv6);
        println!("   APP路径: {:?}", rule.app_path.is_some());
        if let Some(remote) = &rule.remote {
            println!("   远程IP: {}", remote);
        }
        // 如果有APP_ID + 远程IP的组合，使用测试验证过的层
        if rule.app_path.is_some() && rule.remote.is_some() {
            println!("🎯 检测到APP_ID + 远程IP组合，使用测试验证的层...");

            if !is_ipv6 {
                // 根据测试结果，只使用成功的IPv4层
                match rule.direction {
                    Direction::Outbound => {
                        // 出站连接使用CONNECT层（测试成功）
                        layers.push(FWPM_LAYER_ALE_AUTH_CONNECT_V4);
                        layers.push(FWPM_LAYER_ALE_ENDPOINT_CLOSURE_V4); // 额外保护
                    },
                    Direction::Inbound => {
                        // 入站连接使用RECV_ACCEPT层（测试成功）
                        layers.push(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4);
                        layers.push(FWPM_LAYER_ALE_ENDPOINT_CLOSURE_V4); // 额外保护
                    },
                    Direction::Both => {
                        // 双向连接使用两个主要层（都测试成功）
                        layers.push(FWPM_LAYER_ALE_AUTH_CONNECT_V4);
                        layers.push(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4);
                        layers.push(FWPM_LAYER_ALE_ENDPOINT_CLOSURE_V4); // 额外保护
                        // 可选：如果需要连接重定向功能
                        // layers.push(FWPM_LAYER_ALE_CONNECT_REDIRECT_V4);
                    }
                }
            } else {
                // IPv6层（基于IPv4测试结果推断）
                match rule.direction {
                    Direction::Outbound => {
                        layers.push(FWPM_LAYER_ALE_AUTH_CONNECT_V6);
                        layers.push(FWPM_LAYER_ALE_ENDPOINT_CLOSURE_V6);
                    },
                    Direction::Inbound => {
                        layers.push(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6);
                        layers.push(FWPM_LAYER_ALE_ENDPOINT_CLOSURE_V6);
                    },
                    Direction::Both => {
                        layers.push(FWPM_LAYER_ALE_AUTH_CONNECT_V6);
                        layers.push(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6);
                        layers.push(FWPM_LAYER_ALE_ENDPOINT_CLOSURE_V6);
                    }
                }
            }
        } else {
            // 没有APP_ID + 远程IP组合的情况，使用标准层
            match rule.direction {
                Direction::Outbound => {
                    if is_ipv6 {
                        layers.push(FWPM_LAYER_ALE_AUTH_CONNECT_V6);
                    } else {
                        layers.push(FWPM_LAYER_ALE_AUTH_CONNECT_V4);
                    }
                },
                Direction::Inbound => {
                    if is_ipv6 {
                        layers.push(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6);
                    } else {
                        layers.push(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4);
                    }
                },
                Direction::Both => {
                    if is_ipv6 {
                        layers.push(FWPM_LAYER_ALE_AUTH_CONNECT_V6);
                        layers.push(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6);
                    } else {
                        layers.push(FWPM_LAYER_ALE_AUTH_CONNECT_V4);
                        layers.push(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4);
                    }
                }
            }
        }

        println!("   将测试 {} 个层", layers.len());
        layers
    }



    // 清理过滤器
    pub fn cleanup(&mut self) -> Result<()> {
        unsafe {
            println!("\n🛑 停止过滤器，正在清理...");

            // 清理过滤器
            for filter_id in &self.filter_ids {
                let delete_result = FwpmFilterDeleteById0(self.engine_handle, *filter_id);
                if WIN32_ERROR(delete_result) == ERROR_SUCCESS {
                    println!("✓ 过滤器 {} 已删除", filter_id);
                } else {
                    println!("⚠️  删除过滤器 {} 失败: {}", filter_id, delete_result);
                }
            }

            // 关闭引擎
            let result = FwpmEngineClose0(self.engine_handle);
            if WIN32_ERROR(result) != ERROR_SUCCESS {
                println!("❌ 关闭WFP引擎失败: {}", result);
                return Err(Error::from_win32());
            }
            println!("✓ WFP引擎已关闭");
            Ok(())
        }
    }

    // 添加高级网络过滤器的内部方法
    pub unsafe fn add_advanced_network_filter(
        &self,
        rule: &FilterRule,
        layer_key: GUID,
    ) -> Result<u64> {
        // 将过滤器名称转换为宽字符串
        let filter_name = to_wide_string(&rule.name);
        // 生成过滤器描述并转换为宽字符串
        let filter_desc = to_wide_string(&format!("控制 {} 的网络流量", rule.name));

        // 创建过滤条件向量
        let mut conditions = Vec::new();        // 添加应用程序路径条件
        let mut _app_id_data = None;
        let mut should_add_app_id = false;        if let Some(app_path) = &rule.app_path {
            // 基于测试结果，只在成功验证的层上添加APP_ID条件
            should_add_app_id = match layer_key {
                // 测试成功的层：支持APP_ID + 远程IP组合
                FWPM_LAYER_ALE_AUTH_CONNECT_V4 |
                FWPM_LAYER_ALE_AUTH_CONNECT_V6 |
                FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4 |
                FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6 |
                FWPM_LAYER_ALE_ENDPOINT_CLOSURE_V4 |
                FWPM_LAYER_ALE_ENDPOINT_CLOSURE_V6 |
                FWPM_LAYER_ALE_CONNECT_REDIRECT_V4 |
                FWPM_LAYER_ALE_CONNECT_REDIRECT_V6 => true,

                // 测试失败的层：不支持APP_ID + 远程IP组合（但单独APP_ID可能可以）
                FWPM_LAYER_ALE_AUTH_LISTEN_V4 |
                FWPM_LAYER_ALE_AUTH_LISTEN_V6 => {
                    // 只有在没有远程IP条件时才添加APP_ID
                    rule.remote.is_none()
                },

                // 其他层默认不添加APP_ID
                _ => false,
            };

            if should_add_app_id {
                let appid_utf16: Vec<u16> = app_path
                    .encode_utf16()
                    .chain(std::iter::once(0))
                    .collect();

                let app_id = FWP_BYTE_BLOB {
                    size: (appid_utf16.len() * 2) as u32,
                    data: appid_utf16.as_ptr() as *mut u8,
                };

                conditions.push(FWPM_FILTER_CONDITION0 {
                    fieldKey: FWPM_CONDITION_ALE_APP_ID,
                    matchType: FWP_MATCH_EQUAL,
                    conditionValue: FWP_CONDITION_VALUE0 {
                        r#type: FWP_BYTE_BLOB_TYPE,
                        Anonymous: FWP_CONDITION_VALUE0_0 {
                            byteBlob: &app_id as *const _ as *mut _,
                        },
                    },
                });

                _app_id_data = Some((appid_utf16, app_id));
                println!("✓ APP_ID条件已添加到过滤器: {}", app_path);
            } else {
                println!("⚠️ 跳过APP_ID条件（入站连接在此层不适用）");
            }
        }

        // 添加本地IP/网段条件
        if let Some(local) = &rule.local {
            if let Ok(ip) = local.parse::<IpAddr>() {
                match ip {
                    IpAddr::V4(ipv4) => {
                        let ip_bytes = ipv4.octets();
                        let ip_value = u32::from_be_bytes(ip_bytes);

                        conditions.push(FWPM_FILTER_CONDITION0 {
                            fieldKey: FWPM_CONDITION_IP_LOCAL_ADDRESS,
                            matchType: FWP_MATCH_EQUAL,
                            conditionValue: FWP_CONDITION_VALUE0 {
                                r#type: FWP_UINT32,
                                Anonymous: FWP_CONDITION_VALUE0_0 {
                                    uint32: ip_value,
                                },
                            },
                        });
                        println!("✓ 本地IPv4地址条件已添加: {}", ipv4);
                    },
                    IpAddr::V6(ipv6) => {
                        let ip_bytes = ipv6.octets();
                        let byte_array = FWP_BYTE_ARRAY16 {
                            byteArray16: ip_bytes,
                        };

                        conditions.push(FWPM_FILTER_CONDITION0 {
                            fieldKey: FWPM_CONDITION_IP_LOCAL_ADDRESS,
                            matchType: FWP_MATCH_EQUAL,
                            conditionValue: FWP_CONDITION_VALUE0 {
                                r#type: FWP_BYTE_ARRAY16_TYPE,
                                Anonymous: FWP_CONDITION_VALUE0_0 {
                                    byteArray16: &byte_array as *const _ as *mut _,
                                },
                            },
                        });
                        println!("✓ 本地IPv6地址条件已添加: {}", ipv6);
                    }
                }
            } else if let Ok(network) = IpNetwork::from_cidr(local) {
                match network.ip {
                    IpAddr::V4(network_ip) => {
                        let network_bytes = network_ip.octets();
                        // 使用安全的掩码计算方式
                        let mask = if network.prefix_len == 0 {
                            0u32 // 对于 0.0.0.0/0，掩码为全0
                        } else if network.prefix_len == 32 {
                            u32::MAX // 对于单个IP地址，掩码为全1
                        } else {
                            !((1u32 << (32 - network.prefix_len)) - 1)
                        };
                        let network_addr = u32::from_be_bytes(network_bytes) & mask;

                        let range = FWP_RANGE0 {
                            valueLow: FWP_VALUE0 {
                                r#type: FWP_UINT32,
                                Anonymous: FWP_VALUE0_0 {
                                    uint32: network_addr,
                                },
                            },
                            valueHigh: FWP_VALUE0 {
                                r#type: FWP_UINT32,
                                Anonymous: FWP_VALUE0_0 {
                                    uint32: network_addr | !mask,
                                },
                            },
                        };

                        conditions.push(FWPM_FILTER_CONDITION0 {
                            fieldKey: FWPM_CONDITION_IP_LOCAL_ADDRESS,
                            matchType: FWP_MATCH_RANGE,
                            conditionValue: FWP_CONDITION_VALUE0 {
                                r#type: FWP_RANGE_TYPE,
                                Anonymous: FWP_CONDITION_VALUE0_0 {
                                    rangeValue: &range as *const _ as *mut _,
                                },
                            },
                        });
                        println!("✓ 本地IPv4网段条件已添加: {}/{}", network_ip, network.prefix_len);
                    },
                    IpAddr::V6(_) => {
                        println!("⚠️ IPv6网段过滤暂不支持，将跳过此条件");
                    }
                }
            }
        }

        // 添加远程IP/网段条件
        if let Some(remote) = &rule.remote {
            if let Ok(ip) = remote.parse::<IpAddr>() {
                match ip {
                    IpAddr::V4(ipv4) => {
                        let ip_bytes = ipv4.octets();
                        let ip_value = u32::from_be_bytes(ip_bytes);

                        conditions.push(FWPM_FILTER_CONDITION0 {
                            fieldKey: FWPM_CONDITION_IP_REMOTE_ADDRESS,
                            matchType: FWP_MATCH_EQUAL,
                            conditionValue: FWP_CONDITION_VALUE0 {
                                r#type: FWP_UINT32,
                                Anonymous: FWP_CONDITION_VALUE0_0 {
                                    uint32: ip_value,
                                },
                            },
                        });
                        println!("✓ 远程IPv4地址条件已添加: {}", ipv4);
                    },
                    IpAddr::V6(ipv6) => {
                        let ip_bytes = ipv6.octets();
                        let byte_array = FWP_BYTE_ARRAY16 {
                            byteArray16: ip_bytes,
                        };

                        conditions.push(FWPM_FILTER_CONDITION0 {
                            fieldKey: FWPM_CONDITION_IP_REMOTE_ADDRESS,
                            matchType: FWP_MATCH_EQUAL,
                            conditionValue: FWP_CONDITION_VALUE0 {
                                r#type: FWP_BYTE_ARRAY16_TYPE,
                                Anonymous: FWP_CONDITION_VALUE0_0 {
                                    byteArray16: &byte_array as *const _ as *mut _,
                                },
                            },
                        });
                        println!("✓ 远程IPv6地址条件已添加: {}", ipv6);
                    }
                }
            } else if let Ok(network) = IpNetwork::from_cidr(remote) {
                match network.ip {
                    IpAddr::V4(network_ip) => {
                        let network_bytes = network_ip.octets();
                        // 使用安全的掩码计算方式
                        let mask = if network.prefix_len == 0 {
                            0u32 // 对于 0.0.0.0/0，掩码为全0
                        } else if network.prefix_len == 32 {
                            u32::MAX // 对于单个IP地址，掩码为全1
                        } else {
                            !((1u32 << (32 - network.prefix_len)) - 1)
                        };
                        let network_addr = u32::from_be_bytes(network_bytes) & mask;

                        let range = FWP_RANGE0 {
                            valueLow: FWP_VALUE0 {
                                r#type: FWP_UINT32,
                                Anonymous: FWP_VALUE0_0 {
                                    uint32: network_addr,
                                },
                            },
                            valueHigh: FWP_VALUE0 {
                                r#type: FWP_UINT32,
                                Anonymous: FWP_VALUE0_0 {
                                    uint32: network_addr | !mask,
                                },
                            },
                        };

                        conditions.push(FWPM_FILTER_CONDITION0 {
                            fieldKey: FWPM_CONDITION_IP_REMOTE_ADDRESS,
                            matchType: FWP_MATCH_RANGE,
                            conditionValue: FWP_CONDITION_VALUE0 {
                                r#type: FWP_RANGE_TYPE,
                                Anonymous: FWP_CONDITION_VALUE0_0 {
                                    rangeValue: &range as *const _ as *mut _,
                                },
                            },
                        });
                        println!("✓ 远程IPv4网段条件已添加: {}/{}", network_ip, network.prefix_len);
                    },
                    IpAddr::V6(_) => {
                        println!("⚠️ IPv6网段过滤暂不支持，将跳过此条件");
                    }
                }
            }
        }

        // 添加本地端口条件
        if let Some(local_port) = rule.local_port {
            conditions.push(FWPM_FILTER_CONDITION0 {
                fieldKey: FWPM_CONDITION_IP_LOCAL_PORT,
                matchType: FWP_MATCH_EQUAL,
                conditionValue: FWP_CONDITION_VALUE0 {
                    r#type: FWP_UINT16,
                    Anonymous: FWP_CONDITION_VALUE0_0 {
                        uint16: local_port,
                    },
                },
            });
            println!("✓ 本地端口条件已添加: {}", local_port);
        } else if let Some((start_port, end_port)) = rule.local_port_range {
            let range = FWP_RANGE0 {
                valueLow: FWP_VALUE0 {
                    r#type: FWP_UINT16,
                    Anonymous: FWP_VALUE0_0 {
                        uint16: start_port,
                    },
                },
                valueHigh: FWP_VALUE0 {
                    r#type: FWP_UINT16,
                    Anonymous: FWP_VALUE0_0 {
                        uint16: end_port,
                    },
                },
            };

            conditions.push(FWPM_FILTER_CONDITION0 {
                fieldKey: FWPM_CONDITION_IP_LOCAL_PORT,
                matchType: FWP_MATCH_RANGE,
                conditionValue: FWP_CONDITION_VALUE0 {
                    r#type: FWP_RANGE_TYPE,
                    Anonymous: FWP_CONDITION_VALUE0_0 {
                        rangeValue: &range as *const _ as *mut _,
                    },
                },
            });
            println!("✓ 本地端口范围条件已添加: {}-{}", start_port, end_port);
        }

        // 添加远程端口条件
        if let Some(remote_port) = rule.remote_port {
            conditions.push(FWPM_FILTER_CONDITION0 {
                fieldKey: FWPM_CONDITION_IP_REMOTE_PORT,
                matchType: FWP_MATCH_EQUAL,
                conditionValue: FWP_CONDITION_VALUE0 {
                    r#type: FWP_UINT16,
                    Anonymous: FWP_CONDITION_VALUE0_0 {
                        uint16: remote_port,
                    },
                },
            });
            println!("✓ 远程端口条件已添加: {}", remote_port);
        } else if let Some((start_port, end_port)) = rule.remote_port_range {
            let range = FWP_RANGE0 {
                valueLow: FWP_VALUE0 {
                    r#type: FWP_UINT16,
                    Anonymous: FWP_VALUE0_0 {
                        uint16: start_port,
                    },
                },
                valueHigh: FWP_VALUE0 {
                    r#type: FWP_UINT16,
                    Anonymous: FWP_VALUE0_0 {
                        uint16: end_port,
                    },
                },
            };

            conditions.push(FWPM_FILTER_CONDITION0 {
                fieldKey: FWPM_CONDITION_IP_REMOTE_PORT,
                matchType: FWP_MATCH_RANGE,
                conditionValue: FWP_CONDITION_VALUE0 {
                    r#type: FWP_RANGE_TYPE,
                    Anonymous: FWP_CONDITION_VALUE0_0 {
                        rangeValue: &range as *const _ as *mut _,
                    },
                },
            });
            println!("✓ 远程端口范围条件已添加: {}-{}", start_port, end_port);
        }

        // 添加协议条件
        if let Some(protocol) = &rule.protocol {
            let protocol_value = match protocol {
                Protocol::Tcp => 6u8,
                Protocol::Udp => 17u8,
                Protocol::Icmp => 1u8,
                Protocol::IcmpV6 => 58u8,
                Protocol::Igmp => 2u8,
                Protocol::Ah => 51u8,
                Protocol::Esp => 50u8,
                Protocol::Gre => 47u8,
                Protocol::Ipsec => 50u8,
                Protocol::Any => 0u8,
            };

            conditions.push(FWPM_FILTER_CONDITION0 {
                fieldKey: FWPM_CONDITION_IP_PROTOCOL,
                matchType: FWP_MATCH_EQUAL,
                conditionValue: FWP_CONDITION_VALUE0 {
                    r#type: FWP_UINT8,
                    Anonymous: FWP_CONDITION_VALUE0_0 {
                        uint8: protocol_value,
                    },
                },
            });
            println!("✓ 协议条件已添加: {:?}", protocol);
        }
        // 获取条件数量
        let num_conditions = conditions.len() as u32;

        // 确定过滤器动作
        let action_type = match rule.action {
            FilterAction::Allow => FWP_ACTION_PERMIT,
            FilterAction::Block => FWP_ACTION_BLOCK,
        };

        // 根据是否有远程IP条件调整权重
        let filter_weight = if rule.remote.is_some() {
            unsafe { WEIGHT_VALUE += 10; WEIGHT_VALUE } // 远程IP过滤器权重更高
        } else {
            unsafe { WEIGHT_VALUE += 1; WEIGHT_VALUE }
        };

        // 创建过滤器结构
        let filter = FWPM_FILTER0 {
            filterKey: GUID::zeroed(),
            displayData: FWPM_DISPLAY_DATA0 {
                name: PWSTR(filter_name.as_ptr() as *mut u16),
                description: PWSTR(filter_desc.as_ptr() as *mut u16),
            },
            flags: FWPM_FILTER_FLAGS(0),
            providerKey: ptr::null_mut(),
            providerData: FWP_BYTE_BLOB {
                size: 0,
                data: ptr::null_mut(),
            },
            layerKey: layer_key,
            subLayerKey: FWPM_SUBLAYER_UNIVERSAL,            weight: FWP_VALUE0 {
                r#type: FWP_UINT64,
                Anonymous: FWP_VALUE0_0 {
                    uint64: &filter_weight as *const u64 as *mut u64,
                },
            },
            numFilterConditions: num_conditions,
            filterCondition: if num_conditions > 0 {
                conditions.as_ptr() as *mut _
            } else {
                ptr::null_mut()
            },
            action: FWPM_ACTION0 {
                r#type: action_type,
                Anonymous: FWPM_ACTION0_0 {
                    calloutKey: GUID::zeroed(),
                },
            },
            Anonymous: FWPM_FILTER0_0 {
                rawContext: 0,
            },
            reserved: ptr::null_mut(),
            filterId: 0,
            effectiveWeight: FWP_VALUE0 {
                r#type: FWP_UINT64,
                Anonymous: FWP_VALUE0_0 {
                    uint64: &raw mut EFFECTIVE_WEIGHT_VALUE as *mut u64,
                },
            },
        };

        // 用于存储新添加的过滤器ID
        let mut filter_id = 0u64;
        // 添加过滤器到WFP引擎
        let add_result = unsafe { FwpmFilterAdd0(self.engine_handle, &filter, None, Some(&mut filter_id)) };

        // 检查添加结果
        if WIN32_ERROR(add_result) == ERROR_SUCCESS {
            Ok(filter_id)
        } else {
            let error_msg = match WIN32_ERROR(add_result) {
                ERROR_ACCESS_DENIED => "访问被拒绝 - 需要管理员权限",
                ERROR_INVALID_PARAMETER => "无效参数 - 检查过滤条件组合",
                ERROR_NOT_SUPPORTED => "不支持的操作 - 检查WFP层和条件兼容性",
                ERROR_ALREADY_EXISTS => "过滤器已存在",
                ERROR_NOT_FOUND => "找不到指定的层或条件",
                _ if add_result == 2150760450 => "FWP_E_INVALID_CONDITION - 条件组合无效，某些层不支持特定条件组合",
                _ => "未知错误",
            };
            println!("❌ 添加过滤器 '{}' 失败: {} (错误代码: {})", rule.name, error_msg, add_result);
            println!("   层: {:?}", layer_key);
            println!("   条件数量: {}", num_conditions);
            if rule.app_path.is_some() {
                println!("   包含APP_ID条件: {}", should_add_app_id);
            }
            if rule.remote.is_some() {
                println!("   包含远程IP条件: true");
            }
            Err(Error::from_win32())
        }
    }

    // 获取层的名称用于调试
    pub fn get_layer_name(&self, layer_key: &GUID) -> &'static str {
        match *layer_key {
            FWPM_LAYER_ALE_AUTH_CONNECT_V4 => "ALE_AUTH_CONNECT_V4",
            FWPM_LAYER_ALE_AUTH_CONNECT_V6 => "ALE_AUTH_CONNECT_V6",
            FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4 => "ALE_AUTH_RECV_ACCEPT_V4",
            FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6 => "ALE_AUTH_RECV_ACCEPT_V6",
            FWPM_LAYER_ALE_AUTH_LISTEN_V4 => "ALE_AUTH_LISTEN_V4",
            FWPM_LAYER_ALE_AUTH_LISTEN_V6 => "ALE_AUTH_LISTEN_V6",
            FWPM_LAYER_ALE_RESOURCE_ASSIGNMENT_V4 => "ALE_RESOURCE_ASSIGNMENT_V4",
            FWPM_LAYER_ALE_RESOURCE_ASSIGNMENT_V6 => "ALE_RESOURCE_ASSIGNMENT_V6",
            FWPM_LAYER_ALE_RESOURCE_RELEASE_V4 => "ALE_RESOURCE_RELEASE_V4",
            FWPM_LAYER_ALE_RESOURCE_RELEASE_V6 => "ALE_RESOURCE_RELEASE_V6",
            FWPM_LAYER_ALE_ENDPOINT_CLOSURE_V4 => "ALE_ENDPOINT_CLOSURE_V4",
            FWPM_LAYER_ALE_ENDPOINT_CLOSURE_V6 => "ALE_ENDPOINT_CLOSURE_V6",
            FWPM_LAYER_ALE_CONNECT_REDIRECT_V4 => "ALE_CONNECT_REDIRECT_V4",
            FWPM_LAYER_ALE_CONNECT_REDIRECT_V6 => "ALE_CONNECT_REDIRECT_V6",
            FWPM_LAYER_ALE_BIND_REDIRECT_V4 => "ALE_BIND_REDIRECT_V4",
            FWPM_LAYER_ALE_BIND_REDIRECT_V6 => "ALE_BIND_REDIRECT_V6",
            FWPM_LAYER_OUTBOUND_TRANSPORT_V4 => "OUTBOUND_TRANSPORT_V4",
            FWPM_LAYER_OUTBOUND_TRANSPORT_V6 => "OUTBOUND_TRANSPORT_V6",
            FWPM_LAYER_INBOUND_TRANSPORT_V4 => "INBOUND_TRANSPORT_V4",
            FWPM_LAYER_INBOUND_TRANSPORT_V6 => "INBOUND_TRANSPORT_V6",
            _ => "UNKNOWN_LAYER",
        }
    }

    // 删除指定的过滤器
    pub fn delete_filters(&mut self, filter_ids: &[u64]) -> Result<u32> {
        unsafe {
            let mut deleted_count = 0;

            for &filter_id in filter_ids {
                let delete_result = FwpmFilterDeleteById0(self.engine_handle, filter_id);
                if WIN32_ERROR(delete_result) == ERROR_SUCCESS {
                    // 从内部列表中移除
                    if let Some(pos) = self.filter_ids.iter().position(|&id| id == filter_id) {
                        self.filter_ids.remove(pos);
                    }
                    deleted_count += 1;
                    println!("✓ 过滤器 {} 已删除", filter_id);
                } else {
                    println!("⚠️ 删除过滤器 {} 失败: {}", filter_id, delete_result);
                }
            }

            if deleted_count > 0 {
                Ok(deleted_count)
            } else {
                Err(Error::from_win32())
            }
        }
    }

    // 删除单个过滤器
    pub fn remove_filter(&mut self, filter_id: u64) -> Result<()> {
        unsafe {
            let delete_result = FwpmFilterDeleteById0(self.engine_handle, filter_id);
            if WIN32_ERROR(delete_result) == ERROR_SUCCESS {
                // 从内部列表中移除
                if let Some(pos) = self.filter_ids.iter().position(|&id| id == filter_id) {
                    self.filter_ids.remove(pos);
                }
                println!("✓ 过滤器 {} 已删除", filter_id);
                Ok(())
            } else {
                println!("⚠️ 删除过滤器 {} 失败: {}", filter_id, delete_result);
                Err(Error::from_win32())
            }
        }
    }

    // 获取所有规则（简化版本，返回当前添加的规则）
    pub fn get_rules(&self) -> Result<Vec<FilterRule>> {
        // 这是一个简化实现，实际应该从WFP引擎查询
        // 由于WFP API复杂，这里返回一个空列表
        // 在实际应用中，需要实现完整的WFP枚举功能
        Ok(Vec::new())
    }

    // 获取规则对应的过滤器ID
    pub fn get_filter_ids(&self, _rule: &FilterRule) -> Result<Vec<u64>> {
        // 简化实现，返回当前存储的过滤器ID
        Ok(self.filter_ids.clone())
    }

    // 导出规则配置
    pub fn export_rules(&self, file_path: &Path) -> Result<()> {
        let config = RuleConfig {
            version: "1.0".to_string(),
            rules: self.get_rules()?.into_iter().map(|rule| {
                FilterRuleConfig {
                    name: rule.name,
                    app_path: rule.app_path,
                    local_ip: rule.local,
                    remote_ip: rule.remote,
                    local_port: rule.local_port,
                    remote_port: rule.remote_port,
                    local_port_range: rule.local_port_range,
                    remote_port_range: rule.remote_port_range,
                    protocol: rule.protocol.map(|p| p.to_string()),
                    direction: format!("{:?}", rule.direction),
                    action: format!("{:?}", rule.action),
                    priority: rule.priority,
                    group: rule.group,
                    enabled: rule.enabled,
                    description: rule.description,
                }
            }).collect(),
            groups: vec![], // TODO: 实现分组管理
            metadata: MetadataConfig {
                created_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    .to_string(),
                created_by: "AstralWFP".to_string(),
                description: Some("导出的WFP规则配置".to_string()),
                tags: vec!["wfp".to_string(), "firewall".to_string()],
            },
        };

        let json = serde_json::to_string_pretty(&config)
            .map_err(|e| Error::new(windows::core::HRESULT(0x80004005u32 as i32), (&e.to_string()).into()))?;

        fs::write(file_path, json)
            .map_err(|e| Error::new(windows::core::HRESULT(0x80004005u32 as i32), (&e.to_string()).into()))?;

        println!("✅ 规则配置已导出到: {:?}", file_path);
        Ok(())
    }

    // 导入规则配置
    pub fn import_rules(&mut self, file_path: &Path) -> Result<()> {
        let content = fs::read_to_string(file_path)
            .map_err(|e| Error::new(windows::core::HRESULT(0x80004005u32 as i32), (&e.to_string()).into()))?;

        let config: RuleConfig = serde_json::from_str(&content)
            .map_err(|e| Error::new(windows::core::HRESULT(0x80004005u32 as i32), (&e.to_string()).into()))?;

        let rules: Vec<FilterRule> = config.rules.into_iter().map(|rule_config| {
            let mut rule = FilterRule::new(&rule_config.name)
                .priority(rule_config.priority)
                .enabled(rule_config.enabled);

            if let Some(app_path) = rule_config.app_path {
                rule = rule.app_path(&app_path);
            }
            if let Some(local_ip) = rule_config.local_ip {
                rule = rule.local_ip(&local_ip);
            }
            if let Some(remote_ip) = rule_config.remote_ip {
                rule = rule.remote_ip(&remote_ip);
            }
            if let Some(local_port) = rule_config.local_port {
                rule = rule.local_port(local_port);
            }
            if let Some(remote_port) = rule_config.remote_port {
                rule = rule.remote_port(remote_port);
            }
            if let Some((start, end)) = rule_config.local_port_range {
                rule = rule.local_port_range(start, end);
            }
            if let Some((start, end)) = rule_config.remote_port_range {
                rule = rule.remote_port_range(start, end);
            }
            if let Some(protocol_str) = rule_config.protocol {
                if let Ok(protocol) = protocol_str.parse::<Protocol>() {
                    rule = rule.protocol(protocol);
                }
            }

            // 解析方向和动作
            match rule_config.direction.as_str() {
                "Inbound" => rule = rule.direction(Direction::Inbound),
                "Outbound" => rule = rule.direction(Direction::Outbound),
                "Both" => rule = rule.direction(Direction::Both),
                _ => rule = rule.direction(Direction::Both),
            }

            match rule_config.action.as_str() {
                "Allow" => rule = rule.action(FilterAction::Allow),
                "Block" => rule = rule.action(FilterAction::Block),
                _ => rule = rule.action(FilterAction::Block),
            }

            if let Some(group) = rule_config.group {
                rule = rule.group(&group);
            }
            if let Some(description) = rule_config.description {
                rule = rule.description(&description);
            }

            rule
        }).collect();

        // 应用导入的规则
        self.add_advanced_filters(&rules)?;

        println!("✅ 规则配置已从 {:?} 导入，共导入 {} 条规则", file_path, rules.len());
        Ok(())
    }
}

// 时间控制结构体
#[derive(Debug, Clone)]
pub struct TimeControl {
    pub start_time: Option<u64>,    // 开始时间戳（Unix时间戳）
    pub end_time: Option<u64>,      // 结束时间戳（Unix时间戳）
    pub days_of_week: Option<Vec<u8>>, // 星期几（0=周日，1=周一，...）
    pub hours: Option<(u8, u8)>,    // 小时范围 (start_hour, end_hour)
}

impl TimeControl {
    pub fn new() -> Self {
        Self {
            start_time: None,
            end_time: None,
            days_of_week: None,
            hours: None,
        }
    }

    pub fn start_time(mut self, timestamp: u64) -> Self {
        self.start_time = Some(timestamp);
        self
    }

    pub fn end_time(mut self, timestamp: u64) -> Self {
        self.end_time = Some(timestamp);
        self
    }

    pub fn days_of_week(mut self, days: Vec<u8>) -> Self {
        self.days_of_week = Some(days);
        self
    }

    pub fn hours(mut self, start: u8, end: u8) -> Self {
        self.hours = Some((start, end));
        self
    }

    pub fn is_active(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // 检查时间范围
        if let Some(start) = self.start_time {
            if now < start {
                return false;
            }
        }

        if let Some(end) = self.end_time {
            if now > end {
                return false;
            }
        }

        // 检查星期几
        if let Some(days) = &self.days_of_week {
            let weekday = (now / 86400 + 4) % 7; // 计算星期几（0=周日）
            if !days.contains(&(weekday as u8)) {
                return false;
            }
        }

        // 检查小时范围
        if let Some((start_hour, end_hour)) = self.hours {
            let hour = (now % 86400) / 3600;
            if hour < start_hour as u64 || hour > end_hour as u64 {
                return false;
            }
        }

        true
    }
}

// 流量统计结构体
#[derive(Debug, Clone, Default)]
pub struct TrafficStats {
    pub packets_allowed: u64,
    pub packets_blocked: u64,
    pub bytes_allowed: u64,
    pub bytes_blocked: u64,
    pub connections_allowed: u64,
    pub connections_blocked: u64,
    pub last_activity: Option<u64>,
}

impl TrafficStats {
    pub fn new() -> Self {
        Self {
            packets_allowed: 0,
            packets_blocked: 0,
            bytes_allowed: 0,
            bytes_blocked: 0,
            connections_allowed: 0,
            connections_blocked: 0,
            last_activity: None,
        }
    }

    pub fn increment_allowed(&mut self, packets: u64, bytes: u64) {
        self.packets_allowed += packets;
        self.bytes_allowed += bytes;
        self.connections_allowed += 1;
        self.last_activity = Some(SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs());
    }

    pub fn increment_blocked(&mut self, packets: u64, bytes: u64) {
        self.packets_blocked += packets;
        self.bytes_blocked += bytes;
        self.connections_blocked += 1;
        self.last_activity = Some(SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs());
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

// 规则统计结构体
#[derive(Debug, Clone)]
pub struct RuleStats {
    pub rule_id: String,
    pub rule_name: String,
    pub traffic_stats: TrafficStats,
    pub hit_count: u64,
    pub miss_count: u64,
    pub average_response_time: f64,
}

// 规则配置结构体（用于导入/导出）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleConfig {
    pub version: String,
    pub rules: Vec<FilterRuleConfig>,
    pub groups: Vec<GroupConfig>,
    pub metadata: MetadataConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterRuleConfig {
    pub name: String,
    pub app_path: Option<String>,
    pub local_ip: Option<String>,
    pub remote_ip: Option<String>,
    pub local_port: Option<u16>,
    pub remote_port: Option<u16>,
    pub local_port_range: Option<(u16, u16)>,
    pub remote_port_range: Option<(u16, u16)>,
    pub protocol: Option<String>,
    pub direction: String,
    pub action: String,
    pub priority: u32,
    pub group: Option<String>,
    pub enabled: bool,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupConfig {
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataConfig {
    pub created_at: String,
    pub created_by: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
}