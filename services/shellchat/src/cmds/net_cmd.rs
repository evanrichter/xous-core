use crate::{ShellCmdApi, CommonEnv};
use xous_ipc::String;
use net::Duration;
use xous::MessageEnvelope;
use num_traits::*;
use std::net::{SocketAddr, IpAddr};
use std::thread;

pub struct NetCmd {
    udp: Option<net::UdpSocket>,
    udp_clone: Option<net::UdpSocket>,
    callback_id: Option<u32>,
    callback_conn: u32,
    udp_count: u32,
    dns: dns::Dns,
}
impl NetCmd {
    pub fn new(xns: &xous_names::XousNames) -> Self {
        NetCmd {
            udp: None,
            udp_clone: None,
            callback_id: None,
            callback_conn: xns.request_connection_blocking(crate::SERVER_NAME_SHELLCHAT).unwrap(),
            udp_count: 0,
            dns: dns::Dns::new(&xns).unwrap(),
        }
    }
}

#[derive(Debug, num_derive::FromPrimitive, num_derive::ToPrimitive)]
pub(crate) enum NetCmdDispatch {
    UdpTest1,
    UdpTest2,
    Ping,
}

pub const UDP_TEST_SIZE: usize = 64;
impl<'a> ShellCmdApi<'a> for NetCmd {
    cmd_api!(net); // inserts boilerplate for command API

    fn process(&mut self, args: String::<1024>, env: &mut CommonEnv) -> Result<Option<String::<1024>>, xous::Error> {
        if self.callback_id.is_none() {
            let cb_id = env.register_handler(String::<256>::from_str(self.verb()));
            log::trace!("hooking net callback with ID {}", cb_id);
            self.callback_id = Some(cb_id);
        }

        use core::fmt::Write;
        let mut ret = String::<1024>::new();
        let helpstring = "net [udp [port]] [udpclose] [udpclone] [udpcloneclose]";

        let mut tokens = args.as_str().unwrap().split(' ');

        if let Some(sub_cmd) = tokens.next() {
            match sub_cmd {
                // Testing of udp is done with netcat:
                // to send packets run `netcat -u <precursor ip address> 6502` on a remote host, and then type some data
                // to receive packets, use `netcat -u -l 6502`, on the same remote host, and it should show a packet of counts received
                "udp" => {
                    if let Some(udp_socket) = &self.udp {
                        write!(ret, "Socket listener already installed on {:?}.", udp_socket.socket_addr().unwrap()).unwrap();
                    } else {
                        let port = if let Some(tok_str) = tokens.next() {
                            if let Ok(n) = tok_str.parse::<u16>() { n } else { 6502 }
                        } else {
                            6502
                        };
                        let mut udp = net::UdpSocket::bind_xous(
                            format!("127.0.0.1:{}", port),
                            Some(UDP_TEST_SIZE as u16)
                        ).unwrap();
                        udp.set_read_timeout(Some(Duration::from_millis(1000))).unwrap();
                        udp.set_scalar_notification(
                            self.callback_conn,
                            self.callback_id.unwrap() as usize, // this is guaranteed in the prelude
                            [Some(NetCmdDispatch::UdpTest1.to_usize().unwrap()), None, None, None]
                        );
                        self.udp = Some(udp);
                        write!(ret, "Created UDP socket listener on port {}", port).unwrap();
                    }
                }
                "udpclose" => {
                    self.udp = None;
                    write!(ret, "Closed primary UDP socket").unwrap();
                }
                "udpclone" => {
                    if let Some(udp_socket) = &self.udp {
                        let mut udp_clone = udp_socket.duplicate().unwrap();
                        udp_clone.set_scalar_notification(
                            self.callback_conn,
                            self.callback_id.unwrap() as usize, // this is guaranteed in the prelude
                            [Some(NetCmdDispatch::UdpTest2.to_usize().unwrap()), None, None, None]
                        );
                        let sa = udp_clone.socket_addr().unwrap();
                        self.udp_clone = Some(udp_clone);
                        write!(ret, "Cloned UDP socket on {:?}", sa).unwrap();
                    } else {
                        write!(ret, "Run `net udp` before cloning.").unwrap();
                    }
                }
                "udpcloneclose" => {
                    self.udp_clone = None;
                    write!(ret, "Closed cloned UDP socket").unwrap();
                }
                "dns" => {
                    if let Some(name) = tokens.next() {
                        match self.dns.lookup(name) {
                            Ok(ipaddr) => {
                                write!(ret, "DNS resolved {}->{:?}", name, ipaddr).unwrap();
                            }
                            Err(e) => {
                                write!(ret, "DNS lookup error: {:?}", e).unwrap();
                            }
                        }
                    }
                }
                "ping" => {
                    if let Some(name) = tokens.next() {
                        match self.dns.lookup(name) {
                            Ok(ipaddr) => {
                                log::debug!("sending ping to {:?}", ipaddr);
                                let count = if let Some(count_str) = tokens.next() {
                                    count_str.parse::<u32>().unwrap()
                                } else {
                                    1
                                };
                                thread::spawn({
                                    let count = count.clone();
                                    let cb_conn = self.callback_conn.clone();
                                    let cb_id = self.callback_id.unwrap() as usize;
                                    move || {
                                        let tt = ticktimer_server::Ticktimer::new().unwrap();
                                        let mut pinger = net::Ping::new();
                                        pinger.set_scalar_notification(
                                            cb_conn,
                                            cb_id,
                                            NetCmdDispatch::Ping.to_u32().unwrap()
                                        );
                                        for i in 0..count {
                                            log::info!("ping {} of {}", i + 1, count);
                                            pinger.ping(IpAddr::from(ipaddr));
                                            tt.sleep_ms(1000).unwrap();
                                        }
                                        // pinger.wait_pong();
                                    }
                                });
                                write!(ret, "Sending {} pings to {} ({:?})", count, name, ipaddr).unwrap();
                            }
                            Err(e) => {
                                write!(ret, "Can't ping, DNS lookup error: {:?}", e).unwrap();
                            }
                        }
                    } else {
                        write!(ret, "Missing url: net ping [url]").unwrap();
                    }
                }
                _ => {
                    write!(ret, "{}", helpstring).unwrap();
                }
            }

        } else {
            write!(ret, "{}", helpstring).unwrap();
        }
        Ok(Some(ret))
    }


    fn callback(&mut self, msg: &MessageEnvelope, _env: &mut CommonEnv) -> Result<Option<String::<1024>>, xous::Error> {
        use core::fmt::Write;

        log::debug!("net callback");
        let mut ret = String::<1024>::new();
        xous::msg_scalar_unpack!(msg, dispatch, response_time, remote, _, {
            match FromPrimitive::from_usize(dispatch) {
                Some(NetCmdDispatch::UdpTest1) => {
                    if let Some(udp_socket) = &mut self.udp {
                        let mut pkt: [u8; UDP_TEST_SIZE] = [0; UDP_TEST_SIZE];
                        match udp_socket.recv_from(&mut pkt) {
                            Ok((len, addr)) => {
                                write!(ret, "UDP rx {} bytes: {:?}: {}\n", len, addr, std::str::from_utf8(&pkt[..len]).unwrap()).unwrap();
                                log::info!("UDP rx {} bytes: {:?}: {:?}", len, addr, &pkt[..len]);
                                self.udp_count += 1;

                                let response_addr = SocketAddr::new(
                                    addr.ip(),
                                    udp_socket.socket_addr().unwrap().port()
                                );
                                match udp_socket.send_to(
                                    format!("Received {} packets\n\r", self.udp_count).as_bytes(),
                                    &response_addr
                                ) {
                                    Ok(len) => write!(ret, "UDP tx {} bytes", len).unwrap(),
                                    Err(_) => write!(ret, "UDP tx err").unwrap(),
                                }
                            },
                            Err(e) => {
                                log::error!("Net UDP error: {:?}", e);
                                write!(ret, "UDP receive error: {:?}", e).unwrap();
                            }
                        }
                    } else {
                        log::error!("Got NetCmd callback from uninitialized socket");
                        write!(ret, "Got NetCmd callback from uninitialized socket").unwrap();
                    }
                },
                Some(NetCmdDispatch::UdpTest2) => {
                    if let Some(udp_socket) = &mut self.udp_clone {
                        let mut pkt: [u8; UDP_TEST_SIZE] = [0; UDP_TEST_SIZE];
                        match udp_socket.recv_from(&mut pkt) {
                            Ok((len, addr)) => {
                                write!(ret, "Clone UDP rx {} bytes: {:?}: {}\n", len, addr, std::str::from_utf8(&pkt[..len]).unwrap()).unwrap();
                                log::info!("Clone UDP rx {} bytes: {:?}: {:?}", len, addr, &pkt[..len]);
                                self.udp_count += 1;

                                let response_addr = SocketAddr::new(
                                    addr.ip(),
                                    udp_socket.socket_addr().unwrap().port()
                                );
                                match udp_socket.send_to(
                                    format!("Clone received {} packets\n\r", self.udp_count).as_bytes(),
                                    &response_addr
                                ) {
                                    Ok(len) => write!(ret, "UDP tx {} bytes", len).unwrap(),
                                    Err(e) => write!(ret, "UDP tx err: {:?}", e).unwrap(),
                                }
                            },
                            Err(e) => {
                                log::error!("Net UDP error: {:?}", e);
                                write!(ret, "UDP receive error: {:?}", e).unwrap();
                            }
                        }
                    } else {
                        log::error!("Got NetCmd callback from uninitialized socket");
                        write!(ret, "Got NetCmd callback from uninitialized socket").unwrap();
                    }
                },
                Some(NetCmdDispatch::Ping) => {
                    log::info!("pong {:?} t={}ms", IpAddr::from((remote as u32).to_be_bytes()), response_time);
                    // this only works for ipv4, but ping is ipv6 compatible and could send you an ipv6 response. oh well.
                    if response_time > 0 {
                        write!(ret, "Pong from {:?} time={}ms",
                            IpAddr::from((remote as u32).to_be_bytes()),
                            response_time
                        ).unwrap();
                    } else {
                        write!(ret, "Ping timeout {:?}", IpAddr::from((remote as u32).to_be_bytes())).unwrap();
                    }
                }
                None => {
                    log::error!("NetCmd callback with unrecognized dispatch ID");
                    write!(ret, "NetCmd callback with unrecognized dispatch ID").unwrap();
                }
            }
        });
        Ok(Some(ret))
    }
}