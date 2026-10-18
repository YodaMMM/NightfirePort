//! UPnP IGD port mapping (docs/systems/net-p2p.md §1 step 1): SSDP discovery, the device
//! description, and the WANIPConnection / WANPPPConnection SOAP actions — std only, blocking with
//! short timeouts (the app runs it on a background thread).

use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream, ToSocketAddrs, UdpSocket};
use std::time::{Duration, Instant};

pub const SSDP_ADDR: &str = "239.255.255.250:1900";
pub const DESCRIPTION: &str = "NightfirePort";

/// The search targets, most specific first.
const TARGETS: &[&str] = &[
    "urn:schemas-upnp-org:device:InternetGatewayDevice:1",
    "urn:schemas-upnp-org:device:InternetGatewayDevice:2",
    "urn:schemas-upnp-org:service:WANIPConnection:1",
    "urn:schemas-upnp-org:service:WANIPConnection:2",
    "urn:schemas-upnp-org:service:WANPPPConnection:1",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpnpError {
    /// No gateway answered the SSDP search.
    NoGateway,
    /// The description had no WAN*Connection service.
    NoService,
    /// Network/HTTP failure (text for the report/UI).
    Io(String),
    /// The router answered a SOAP fault (UPnPError code + description).
    Soap(u32, String),
}

/// A discovered IGD WAN connection service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gateway {
    /// The description URL (SSDP LOCATION).
    pub location: String,
    /// host:port of the HTTP server (the router).
    pub http: SocketAddr,
    /// The absolute control path ("/ctl/IPConn").
    pub control_path: String,
    /// The full service type URN (used in the SOAPAction header).
    pub service_type: String,
    /// This PC's address as the router sees it (the TCP connection's local address).
    pub local_ip: Option<Ipv4Addr>,
}

/// One M-SEARCH round: send every target, collect LOCATION headers for `wait`.
/// `ssdp` is the multicast address (tests pass a loopback fake).
pub fn discover_locations(ssdp: &str, wait: Duration) -> Result<Vec<String>, UpnpError> {
    let to: SocketAddr = ssdp.parse().map_err(|_| UpnpError::Io(format!("bad ssdp addr {ssdp}")))?;
    // Bind to the LAN interface for the real multicast search: on a PC with virtual adapters
    // (Hyper-V / WSL's vEthernet) a 0.0.0.0 socket's multicast leaves on the wrong interface and
    // no router answers [L 2026-10-18 on the user's PC: 0 answers unbound, the OpenWRT IGD bound].
    let bind = match (to.ip().is_multicast(), super::local_ipv4()) {
        (true, Some(ip)) => SocketAddr::new(ip.into(), 0),
        _ => "0.0.0.0:0".parse().expect("static"),
    };
    let sock = UdpSocket::bind(bind).map_err(|e| UpnpError::Io(e.to_string()))?;
    sock.set_read_timeout(Some(Duration::from_millis(100))).map_err(|e| UpnpError::Io(e.to_string()))?;
    for st in TARGETS {
        let msg = format!("M-SEARCH * HTTP/1.1\r\nHOST: 239.255.255.250:1900\r\nST: {st}\r\nMAN: \"ssdp:discover\"\r\nMX: 2\r\n\r\n");
        let _ = sock.send_to(msg.as_bytes(), to);
    }
    let mut out: Vec<String> = Vec::new();
    let t0 = Instant::now();
    let mut buf = [0u8; 2048];
    while t0.elapsed() < wait {
        if let Ok((n, _)) = sock.recv_from(&mut buf) {
            let text = String::from_utf8_lossy(&buf[..n]);
            if let Some(loc) = header(&text, "location") {
                if !out.contains(&loc) {
                    out.push(loc);
                }
            }
        }
    }
    // The default gateway's own device first (x.y.z.1 of the LAN address [S heuristic]); link-local
    // 169.254 locations (a range extender advertising a WAN it does not route) last.
    let gw = super::local_ipv4().map(|ip| { let o = ip.octets(); format!("http://{}.{}.{}.1:", o[0], o[1], o[2]) });
    out.sort_by_key(|l| (!gw.as_deref().is_some_and(|g| l.starts_with(g)), l.starts_with("http://169.254.")));
    if out.is_empty() { Err(UpnpError::NoGateway) } else { Ok(out) }
}

/// A header value (case-insensitive name) from an HTTP-style message.
pub fn header(msg: &str, name: &str) -> Option<String> {
    msg.lines().find_map(|l| {
        let (k, v) = l.split_once(':')?;
        k.trim().eq_ignore_ascii_case(name).then(|| v.trim().to_string())
    })
}

/// `http://host:port/path` → (host:port, /path).
pub fn split_url(url: &str) -> Option<(String, String)> {
    let rest = url.strip_prefix("http://")?;
    let (hp, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, "/"),
    };
    let hp = if hp.contains(':') { hp.to_string() } else { format!("{hp}:80") };
    Some((hp, path.to_string()))
}

/// Minimal HTTP/1.1 request (Connection: close); returns (status, body, local addr).
pub fn http(hostport: &str, method: &str, path: &str, extra: &[(&str, &str)], body: &str) -> Result<(u32, String, SocketAddr), UpnpError> {
    let addr = hostport
        .to_socket_addrs()
        .map_err(|e| UpnpError::Io(e.to_string()))?
        .next()
        .ok_or_else(|| UpnpError::Io(format!("no address for {hostport}")))?;
    let mut s = TcpStream::connect_timeout(&addr, Duration::from_secs(2)).map_err(|e| UpnpError::Io(e.to_string()))?;
    s.set_read_timeout(Some(Duration::from_secs(3))).ok();
    s.set_write_timeout(Some(Duration::from_secs(3))).ok();
    let local = s.local_addr().map_err(|e| UpnpError::Io(e.to_string()))?;
    let mut req = format!("{method} {path} HTTP/1.1\r\nHost: {hostport}\r\nConnection: close\r\nContent-Length: {}\r\n", body.len());
    for (k, v) in extra {
        req.push_str(&format!("{k}: {v}\r\n"));
    }
    req.push_str("\r\n");
    req.push_str(body);
    s.write_all(req.as_bytes()).map_err(|e| UpnpError::Io(e.to_string()))?;
    let mut raw = Vec::new();
    let _ = s.read_to_end(&mut raw); // a timeout after a complete close-delimited body is fine
    let text = String::from_utf8_lossy(&raw).to_string();
    let (head, body) = text.split_once("\r\n\r\n").ok_or_else(|| UpnpError::Io("no HTTP header end".into()))?;
    let status = head.split_whitespace().nth(1).and_then(|c| c.parse().ok()).ok_or_else(|| UpnpError::Io("no HTTP status".into()))?;
    let body = if header(head, "transfer-encoding").is_some_and(|v| v.eq_ignore_ascii_case("chunked")) { dechunk(body) } else { body.to_string() };
    Ok((status, body, local))
}

/// Undo `Transfer-Encoding: chunked`.
pub fn dechunk(mut s: &str) -> String {
    let mut out = String::new();
    while let Some((size, rest)) = s.split_once("\r\n") {
        let n = usize::from_str_radix(size.split(';').next().unwrap_or("").trim(), 16).unwrap_or(0);
        if n == 0 || rest.len() < n {
            break;
        }
        out.push_str(&rest[..n]);
        s = rest[n..].strip_prefix("\r\n").unwrap_or(&rest[n..]);
    }
    out
}

/// The text of the first `<tag>…</tag>` (namespace prefixes on the tag are accepted).
pub fn xml_tag<'a>(xml: &'a str, tag: &str) -> Option<&'a str> {
    let mut from = 0;
    while let Some(i) = xml[from..].find('<') {
        let start = from + i + 1;
        let end = start + xml[start..].find('>')?;
        let name = xml[start..end].split_whitespace().next().unwrap_or("");
        let local = name.rsplit(':').next().unwrap_or(name);
        if local == tag && !name.starts_with('/') {
            let body_start = end + 1;
            let close = xml[body_start..].find("</")?;
            return Some(xml[body_start..body_start + close].trim());
        }
        from = end;
    }
    None
}

/// Find the WANIPConnection (preferred) or WANPPPConnection service in a device description.
/// Returns (service type, control URL as written).
pub fn find_service(desc: &str) -> Option<(String, String)> {
    let mut best: Option<(String, String)> = None;
    for chunk in desc.split("<service>").skip(1) {
        let chunk = chunk.split("</service>").next().unwrap_or(chunk);
        let (Some(st), Some(ctl)) = (xml_tag(chunk, "serviceType"), xml_tag(chunk, "controlURL")) else { continue };
        if st.contains(":WANIPConnection:") {
            return Some((st.to_string(), ctl.to_string()));
        }
        if st.contains(":WANPPPConnection:") && best.is_none() {
            best = Some((st.to_string(), ctl.to_string()));
        }
    }
    best
}

/// Fetch the description at `location` and resolve the control URL.
pub fn gateway_from_location(location: &str) -> Result<Gateway, UpnpError> {
    let (hp, path) = split_url(location).ok_or_else(|| UpnpError::Io(format!("bad location {location}")))?;
    let (status, desc, local) = http(&hp, "GET", &path, &[], "")?;
    if status != 200 {
        return Err(UpnpError::Io(format!("description HTTP {status}")));
    }
    let (service_type, ctl) = find_service(&desc).ok_or(UpnpError::NoService)?;
    // Absolute controlURL, else relative to URLBase (UDA 1.0) or the location's host.
    let (http_hp, control_path) = if let Some((h, p)) = split_url(&ctl) {
        (h, p)
    } else {
        let base_hp = xml_tag(&desc, "URLBase").and_then(split_url).map(|(h, _)| h).unwrap_or(hp.clone());
        (base_hp, if ctl.starts_with('/') { ctl } else { format!("/{ctl}") })
    };
    let http = http_hp.to_socket_addrs().map_err(|e| UpnpError::Io(e.to_string()))?.next().ok_or(UpnpError::NoService)?;
    let local_ip = match local {
        SocketAddr::V4(a) => Some(*a.ip()),
        _ => None,
    };
    Ok(Gateway { location: location.to_string(), http, control_path, service_type, local_ip })
}

impl Gateway {
    /// One SOAP action; returns the response body or the UPnPError.
    pub fn soap(&self, action: &str, args: &[(&str, String)]) -> Result<String, UpnpError> {
        let mut a = String::new();
        for (k, v) in args {
            a.push_str(&format!("<{k}>{v}</{k}>"));
        }
        let body = format!(
            "<?xml version=\"1.0\"?>\r\n<s:Envelope xmlns:s=\"http://schemas.xmlsoap.org/soap/envelope/\" s:encodingStyle=\"http://schemas.xmlsoap.org/soap/encoding/\"><s:Body><u:{action} xmlns:u=\"{st}\">{a}</u:{action}></s:Body></s:Envelope>",
            st = self.service_type
        );
        let soap_action = format!("\"{}#{action}\"", self.service_type);
        let (status, resp, _) = http(
            &self.http.to_string(),
            "POST",
            &self.control_path,
            &[("Content-Type", "text/xml; charset=\"utf-8\""), ("SOAPAction", &soap_action)],
            &body,
        )?;
        if status == 200 {
            return Ok(resp);
        }
        let code = xml_tag(&resp, "errorCode").and_then(|c| c.parse().ok()).unwrap_or(status);
        let desc = xml_tag(&resp, "errorDescription").unwrap_or("").to_string();
        Err(UpnpError::Soap(code, desc))
    }

    pub fn external_ip(&self) -> Result<Ipv4Addr, UpnpError> {
        let r = self.soap("GetExternalIPAddress", &[])?;
        xml_tag(&r, "NewExternalIPAddress").and_then(|s| s.parse().ok()).ok_or_else(|| UpnpError::Io("no NewExternalIPAddress".into()))
    }

    /// Map UDP `external` → this PC's `internal` port. Returns the lease actually used.
    pub fn add_mapping(&self, external: u16, internal: u16, lease_s: u32) -> Result<u32, UpnpError> {
        let ip = self.local_ip.ok_or_else(|| UpnpError::Io("no local IPv4".into()))?;
        let args = |lease: u32| {
            vec![
                ("NewRemoteHost", String::new()),
                ("NewExternalPort", external.to_string()),
                ("NewProtocol", "UDP".to_string()),
                ("NewInternalPort", internal.to_string()),
                ("NewInternalClient", ip.to_string()),
                ("NewEnabled", "1".to_string()),
                ("NewPortMappingDescription", DESCRIPTION.to_string()),
                ("NewLeaseDuration", lease.to_string()),
            ]
        };
        match self.soap("AddPortMapping", &args(lease_s)) {
            Ok(_) => Ok(lease_s),
            // 725 OnlyPermanentLeasesSupported: retry permanent (deleted on close).
            Err(UpnpError::Soap(725, _)) if lease_s != 0 => self.soap("AddPortMapping", &args(0)).map(|_| 0),
            Err(e) => Err(e),
        }
    }

    pub fn delete_mapping(&self, external: u16) -> Result<(), UpnpError> {
        self.soap(
            "DeletePortMapping",
            &[("NewRemoteHost", String::new()), ("NewExternalPort", external.to_string()), ("NewProtocol", "UDP".to_string())],
        )
        .map(|_| ())
    }
}

/// The whole host-side attempt: discover, map `port` (trying port..port+9 on a 718 conflict),
/// read the external IP. Returns the gateway, the mapped external endpoint and the lease.
pub fn map_port(ssdp: &str, port: u16, lease_s: u32, wait: Duration) -> Result<(Gateway, SocketAddrV4, u32), UpnpError> {
    let mut last = UpnpError::NoGateway;
    for loc in discover_locations(ssdp, wait)? {
        let gw = match gateway_from_location(&loc) {
            Ok(g) => g,
            Err(e) => {
                last = e;
                continue;
            }
        };
        let mut ext = port;
        let lease = loop {
            match gw.add_mapping(ext, port, lease_s) {
                Ok(l) => break Ok(l),
                Err(UpnpError::Soap(718, _)) if ext < port.saturating_add(9) => ext += 1,
                Err(e) => break Err(e),
            }
        };
        match lease {
            Ok(l) => {
                let ip = gw.external_ip()?;
                return Ok((gw, SocketAddrV4::new(ip, ext), l));
            }
            Err(e) => last = e,
        }
    }
    Err(last)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_and_xml_helpers() {
        assert_eq!(split_url("http://192.168.1.1:5000/rootDesc.xml"), Some(("192.168.1.1:5000".into(), "/rootDesc.xml".into())));
        assert_eq!(split_url("http://router/x"), Some(("router:80".into(), "/x".into())));
        assert_eq!(split_url("https://x/"), None);
        assert_eq!(xml_tag("<a><s:b x=\"1\">hi</s:b></a>", "b"), Some("hi"));
        assert_eq!(dechunk("4\r\nWiki\r\n5\r\npedia\r\n0\r\n\r\n"), "Wikipedia");
        let desc = "<root><service><serviceType>urn:schemas-upnp-org:service:WANPPPConnection:1</serviceType><controlURL>/ppp</controlURL></service>\
                    <service><serviceType>urn:schemas-upnp-org:service:WANIPConnection:1</serviceType><controlURL>/ip</controlURL></service></root>";
        assert_eq!(find_service(desc), Some(("urn:schemas-upnp-org:service:WANIPConnection:1".into(), "/ip".into())));
        assert_eq!(header("HTTP/1.1 200 OK\r\nLOCATION: http://a/b\r\n", "location"), Some("http://a/b".into()));
    }
}
