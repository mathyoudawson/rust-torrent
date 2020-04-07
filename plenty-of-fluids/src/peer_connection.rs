use std::net::{TcpStream};
use std::time::Duration;
use std::net::{SocketAddrV4, SocketAddr};
use super::tracker;

pub fn initiate_handshake(peer: &tracker::Peer) -> std::io::Result<()> {
    let socket = SocketAddr::from(SocketAddrV4::new(peer.ip, peer.port));

    let _tcp_stream = match TcpStream::connect_timeout(&socket, Duration::from_secs(3)) {
        Ok(stream) => stream,
        Err(e) => {
            println!("Couldn't connect to the peer");
            return Err(e)
        },
    };

    Ok(())
}



#[cfg(test)]
mod test {
    // use super::*;
    //
    // #[test]
    // fn correctly_build_bindable_ip()  {
    //     assert_eq!(build_bindable_ip("192.168.1.1", &8080u16), "192.168.1.1:8080")
    // }
}
