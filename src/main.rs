use std::collections::HashMap;
use std::collections::btree_map::Entry;
use std::io::Result;
use std::net::Ipv4Addr;

mod tcp;

#[derive(Debug, Hash, Copy, Clone, Eq, PartialEq)]
struct Quad {
    // (ip addr, port)
    src: (Ipv4Addr, u16),
    dst: (Ipv4Addr, u16)
}

fn main() -> Result<()> {
    let mut connections: HashMap<Quad, tcp::Connection> = Default::default();
    let mut nic = tun_tap::Iface::without_packet_info("tun0", tun_tap::Mode::Tun)?;
    let mut buf = [0u8; 1504];
    loop {
        let nbytes = nic.recv(&mut buf[..])?; 
        //let _eth_flags = u16::from_be_bytes([buf[0], buf[1]]);
        //let eth_proto = u16::from_be_bytes([buf[2], buf[3]]);
        //if eth_proto != 0x0800 {
        //    // not an IPv4
        //    continue;
        //}

        match etherparse::Ipv4HeaderSlice::from_slice(&buf[..nbytes]) {
            Ok(iph) => {
                let src = iph.source_addr();
                let dest = iph.destination_addr();

                if iph.protocol() != 0x06 {
                    // not a tcp
                    continue;
                }

                match etherparse::TcpHeaderSlice::from_slice( &buf[iph.slice().len()..nbytes]) {
                    Ok(tcph) => {
                        let datai = iph.slice().len() + tcph.slice().len();
                        use std::collections::hash_map::Entry;
                        match connections.entry(Quad {
                            src: (src, tcph.source_port()),
                            dst: (src, tcph.destination_port())
                        }) {
                            Entry::Occupied(mut c) => {
                                c.get_mut().on_packet(&mut nic, iph, tcph, &buf[datai..nbytes])?;
                            }

                            Entry::Vacant(e) => {
                                if let Ok(Some(c)) = tcp::Connection::accept(&mut nic, iph, tcph, &buf[datai..nbytes]) {
                                    e.insert(c); 
                                } 
                            }

                        }
                        
                    }

                    Err(error) => {
                        eprintln!("unexpected error {:?}", error);
                    }
                }
            }

            Err(e) => {
                eprintln!("Rejected a weird packet {:?}", e);
            }
        };
    };
}
