use std::io;

pub enum State {
    Closed,
    Listen,
    SynRcvd,
    // Estab,
}

pub struct Connection {
    state: State,
    send: SendSequenceSpace,
    recv: RecvSequenceSpace,
}

struct SendSequenceSpace {
    // send unacknowledged
    una: u32,
    // send next
    nxt: u32,
    // send window
    wnd: u16,
    // send urgent
    up: bool,
    // segment sequence number used for last window update
    wl1: usize,
    // segment acknowledgement number used for last window update
    wl2: usize,
    // initial send sequence number
    iss: u32,
}

struct RecvSequenceSpace {
    // receive next
    nxt: u32,
    // receive window
    wnd: u16,
    // receive urgent pointer
    up: bool,
    // initial send sequence number
    irs: u32,
}

impl Connection {
    pub fn accept<'a>(
        nic: &mut tun_tap::Iface,
        iph: etherparse::Ipv4HeaderSlice<'a>,
        tcph: etherparse::TcpHeaderSlice<'a>,
        data: &'a [u8],
    ) -> io::Result<Option<Self>> {
        let mut buf = [0u8; 1500];
        if !tcph.syn() {
            // only expected SYN packet
            return Ok(None);
        }
        let iss = 0;
        let mut c = Connection {
            state: State::SynRcvd,
            send: SendSequenceSpace{
                iss: 0,
                una: iss,
                nxt: iss + 1,
                wnd: 10,
                up: false,
                wl1: 0,
                wl2: 0,
            },
            recv: RecvSequenceSpace{
                irs: tcph.sequence_number(),
                nxt: tcph.sequence_number() + 1,
                wnd: tcph.window_size(),
                up: false,
            }
        };
        // need to start establishing a connection
        let mut syn_ack = etherparse::TcpHeader::new(
            tcph.destination_port(),
            tcph.source_port(),
            c.send.iss,
            c.send.wnd,
        );
        syn_ack.acknowledgment_number = c.recv.nxt;
        syn_ack.syn = true;
        syn_ack.ack = true;
        let mut ip = etherparse::Ipv4Header::new(
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
                iph.source()[3]
            ],
        );
        // write out the headers
        let mut unwritten = {
            let mut unwritten = &mut buf[..];
            ip.write(&mut unwritten);
            syn_ack.write(&mut unwritten);
            unwritten.len()
        };
        eprintln!("responding with {:02x?}", &buf[..unwritten]);
        nic.send(&buf[..unwritten]);
        Ok(Some(c))
    }

    pub fn on_packet<'a>(&mut self,
                         nic: &mut tun_tap::Iface,
                         iph: etherparse::Ipv4HeaderSlice<'a>,
                         tcph: etherparse::TcpHeaderSlice<'a>,
                         data: &'a [u8]) -> io::Result<()> {
        unimplemented!()
    }
}