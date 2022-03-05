use std::io;

use etherparse::{Ipv4HeaderSlice, TcpHeaderSlice, TcpHeader, IpHeader, Ipv4Header};

enum State {
    Close,
    Listen,
    SynRcvd,
}



///                   State of Send Sequence Space
/// ```
///
///             1         2          3          4      
///        ----------|----------|----------|---------- 
///               SND.UNA    SND.NXT    SND.UNA        
///                                    +SND.WND        
///
///  1 - old sequence numbers which have been acknowledged  
///  2 - sequence numbers of unacknowledged data            
///  3 - sequence numbers allowed for new data transmission 
///  4 - future sequence numbers which are not yet allowed  
///```

struct SendSequenceSpace {
    /// send unacknowledged
    una: usize,
    /// send next
    nxt: usize,
    /// send window
    wnd: usize,
    /// urgent pointer
    up: bool,
    /// segment sequence number used for last window update
    wl1: usize,
    /// segment acknowledgment number used for last window update
    wl2: usize,
    /// initial squence number 
    iss: usize
}




///              State of Receive Sequence Space
/// ```
///
///                 1          2          3      
///             ----------|----------|---------- 
///                    RCV.NXT    RCV.NXT        
///                              +RCV.WND        
///
///  1 - old sequence numbers which have been acknowledged  
///  2 - sequence numbers allowed for new reception         
///  3 - future sequence numbers which are not yet allowed  
///```

struct RecvSquenceSpace {
    /// receive next
    nxt: u32,
    /// receive window
    wnd: u16, 
    /// receive urgent pointer
    up: bool,
    /// initial receive sequence number
    irs: u32,
}


pub struct Connection {
    state: State,
    send: SendSequenceSpace,
    recv: RecvSquenceSpace
}


impl Connection {
    pub fn accept<'a> (
        nic: &mut tun_tap::Iface,
        iph: Ipv4HeaderSlice<'a>,
        tcph: TcpHeaderSlice<'a>,
        data: &'a [u8]
    ) -> io::Result<Option<Self>> {
        let mut buf = [0u8; 1500];
        if !tcph.syn() {
            // expected only a SYN packet
            return Ok(None);
        }
        let iss = 0;
        let c = Connection {
            state: State::SynRcvd,
            send: SendSequenceSpace {
                una: iss,
                nxt: iss + 1,
                wnd: 10,
                up: false,
                wl1: 0,
                wl2:0,
                iss,
            },
            recv: RecvSquenceSpace {
                nxt: tcph.sequence_number() + 1,
                wnd: tcph.window_size(),
                up: false,
                irs: tcph.sequence_number(),
            }
        };

        // need to start establishing a connection
        let mut syn_ack = TcpHeader::new(tcph.destination_port(), tcph.source_port(), 0, 10); 
        syn_ack.syn = true;
        syn_ack.ack = true;

        let mut ip= Ipv4Header::new(
            syn_ack.header_len(),
            64,
            etherparse::IpTrafficClass::Tcp,
            [
                iph.destination()[0],                            
                iph.destination()[1],                            
                iph.destination()[2],                            
                iph.destination()[3],                            
            ],
            [
                iph.source()[0],                            
                iph.source()[1],                            
                iph.source()[2],                            
                iph.source()[3],                            
            ]
        ); 
        let unwritten = {
            let mut unwritten = &mut buf[..];
            ip.write(&mut unwritten);
            syn_ack.write(&mut unwritten); 
            unwritten.len()
        };
        nic.send(&buf[..buf.len() - unwritten])?;
        eprintln!("=======================================================================");
        Ok(Some(c))
        //        eprintln!("{}:{} → {}:{} {}b of tcp",
        //            iph.source_addr(),
//            tcph.source_port(),
//            iph.destination_addr(),
//            tcph.destination_port(),
//            data.len()
//        );
    }
    pub fn on_packet<'a> (
        &mut self,
        nic: &mut tun_tap::Iface,
        iph: Ipv4HeaderSlice<'a>,
        tcph: TcpHeaderSlice<'a>,
        data: &'a [u8]
    ) -> io::Result<()> {
        Ok(())
    } 
}
