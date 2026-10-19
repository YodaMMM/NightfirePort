//! Live check of the host's port mapping against the real router (docs/systems/net-p2p.md §1): UPnP IGD first
//! (SSDP discovery → device description → AddPortMapping UDP → GetExternalIPAddress), then NAT-PMP; the mapping is
//! deleted at the end. `cargo run -p nf-net --example upnp_probe [port]` (default 27420). It changes the router's
//! port mappings for a few seconds — run it only with the router owner's OK.

use std::time::Duration;

use nf_net::p2p::{natpmp, upnp};

fn main() {
    let port: u16 = std::env::args().nth(1).and_then(|a| a.parse().ok()).unwrap_or(27420);
    println!("SSDP discovery ({})...", upnp::SSDP_ADDR);
    match upnp::discover_locations(upnp::SSDP_ADDR, Duration::from_secs(3)) {
        Ok(locs) if !locs.is_empty() => for l in &locs { println!("  device: {l}"); },
        Ok(_) => println!("  no UPnP device answered"),
        Err(e) => println!("  discovery failed: {e:?}"),
    }
    println!("UPnP: map UDP {port} (lease 60 s)...");
    match upnp::map_port(upnp::SSDP_ADDR, port, 60, Duration::from_secs(3)) {
        Ok((gw, public, lease)) => {
            println!("  MAPPED: public {public}, lease {lease} s");
            match gw.external_ip() { Ok(ip) => println!("  router's external IP: {ip}"), Err(e) => println!("  external IP query failed: {e:?}") }
            match gw.delete_mapping(public.port()) { Ok(()) => println!("  mapping deleted"), Err(e) => println!("  DELETE FAILED: {e:?} — remove UDP {} 'NightfirePort' in the router", public.port()) }
            return;
        }
        Err(e) => println!("  UPnP mapping failed: {e:?}"),
    }
    let local = std::net::UdpSocket::bind("0.0.0.0:0").and_then(|s| { s.connect("8.8.8.8:53")?; s.local_addr() });
    let gw = match local { Ok(std::net::SocketAddr::V4(a)) => natpmp::guess_gateway(*a.ip()), _ => { println!("no local IPv4 address"); return; } };
    println!("NAT-PMP: map UDP {port} via {gw}...");
    match natpmp::map_port(gw, port, 60) {
        Ok((public, life)) => {
            println!("  MAPPED: public {public}, lifetime {life} s");
            match natpmp::delete(gw, port) { Ok(()) => println!("  mapping deleted"), Err(e) => println!("  DELETE FAILED: {e:?}") }
        }
        Err(e) => println!("  NAT-PMP failed: {e:?}"),
    }
}
