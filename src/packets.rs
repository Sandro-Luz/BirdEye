use std::net::SocketAddr;
use tokio::{io, net};
use pnet::datalink::{self, interfaces};
use pnet::datalink::Channel::Ethernet;
use pnet::packet::ethernet::EthernetPacket;
use pnet::packet::{Packet, FromPacket};


pub async fn cap_packet(){
    let all_interfaces = interfaces();

    let interface = all_interfaces.iter()
    .find(|e| e.is_up() && !e.is_loopback() && !e.ips.is_empty());

    let (_, mut rx) = match datalink::channel(&interface.unwrap(), Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unhandled channel type: {}", &interface.unwrap()),
        Err(e) => panic!("An error occurred when creating the datalink channel: {}", e),
    };
    println!("Start reading packets: ");
    loop {
        match rx.next() {
            Ok(packet) => {
                if let Some(ethernet_packet) = EthernetPacket::new(packet) {
                    let packet = ethernet_packet.packet();
                    let payload = ethernet_packet.payload();
                    let from_packet = ethernet_packet.from_packet();
                    //println!("---");
                    println!("packet: {:?}", packet);
                    // print the full packet as an array of u8
                    println!("payload: {:?}", payload);
                    // print the payload as an array of u8
                    println!("from_packet: {:?}", from_packet);
                    // print the hearder infos: mac address, ethertype, ...
                    // and the payload as an array of u8
                    println!("---");
                }
            },
            Err(e) => {
                panic!("An error occurred while reading: {}", e);
            }
        }
    }
}




pub async fn ip_resover(url: &str) -> io::Result<Vec<SocketAddr>>  {
    let full_url = format!("{}:433", url);
    let mut addresses = Vec::new();
    for addr in net::lookup_host(full_url).await? {
        println!("socket address is {}", addr.to_string().trim());
        addresses.push(addr);
    }
    Ok(addresses)
}

