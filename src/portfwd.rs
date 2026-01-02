use anyhow::Result;
use netstat2::{
    get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo, TcpState,
};

/// A discovered port forward
#[derive(Debug, Clone)]
pub struct PortForward {
    pub local_port: u16,
}

/// List all port forwards (listening TCP ports) owned by a given PID
pub fn list_forwards_by_pid(pid: u32) -> Result<Vec<PortForward>> {
    let af_flags = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
    let proto_flags = ProtocolFlags::TCP;

    let sockets = get_sockets_info(af_flags, proto_flags)?;

    let mut forwards: Vec<PortForward> = sockets
        .into_iter()
        .filter_map(|socket| {
            // Check if this socket belongs to our PID
            let pids: Vec<u32> = socket.associated_pids;
            if !pids.contains(&pid) {
                return None;
            }

            // Only interested in TCP listening sockets
            if let ProtocolSocketInfo::Tcp(tcp_info) = socket.protocol_socket_info {
                if tcp_info.state == TcpState::Listen {
                    return Some(PortForward {
                        local_port: tcp_info.local_port,
                    });
                }
            }

            None
        })
        .collect();

    // Deduplicate (IPv4 and IPv6 may both be listening on same port)
    forwards.sort_by_key(|f| f.local_port);
    forwards.dedup_by_key(|f| f.local_port);

    Ok(forwards)
}
