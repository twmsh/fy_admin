#[cfg(unix)]
pub fn get_local_ips() -> Vec<String> {
    use pnet::datalink;
    use pnet::ipnetwork::IpNetwork;

    let mut ips = Vec::new();
    for ifc in datalink::interfaces() {
        for ipn in ifc.ips {
            if let IpNetwork::V4(v) = ipn {
                let ip = v.ip();
                if !ip.is_loopback() {
                    ips.push(ip.to_string());
                }
            }
        }
    }
    ips
}

#[cfg(windows)]
pub fn get_local_ips() -> Vec<String> {
    use ipconfig::IfType;
    use ipconfig::OperStatus;
    use std::net::IpAddr;

    let adater_list = match ipconfig::get_adapters() {
        Ok(v) => v,
        Err(e) => {
            println!("error, get_local_ips, err: {:?}", e);
            return vec![];
        }
    };

    let mut ips = Vec::new();

    for adapter in adater_list {
        if adapter.oper_status() == OperStatus::IfOperStatusUp
            && adapter.if_type() != IfType::SoftwareLoopback
        {
            let address_list = adapter.ip_addresses();
            for address in address_list {
                if let IpAddr::V4(addr) = address {
                    if !addr.is_loopback() {
                        ips.push(addr.to_string());
                    }
                }
            }
        }
    }

    ips
}
