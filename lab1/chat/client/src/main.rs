use mio::{Events, Poll};
use socket2::{Domain, Protocol, Socket, Type};
use std::net::{Ipv4Addr, SocketAddr, TcpStream, UdpSocket};

mod polling;

use polling::events;
use polling::handlers::MULTICAST_GROUP;
use polling::register;

const SERVER_PORT: u16 = 9090;

fn main() {
    let stream = TcpStream::connect(("127.0.0.1", SERVER_PORT))
        .expect("\x1B[31;1mThe server is not running on provided address!\x1B[0m");

    let local_port = stream.local_addr().unwrap().port();
    let udp_socket = UdpSocket::bind(("0.0.0.0", local_port))
        .expect("\x1B[31;1mCould not bind UDP socket!\x1B[0m");

    let multicast_socket: UdpSocket = create_multicast_listener();

    println!("\n\x1B[32;1mYou've been connected to the chat!\x1B[0m\n");

    let (poll, events) = (Poll::new().unwrap(), Events::with_capacity(1024));
    let poll_registry = poll.registry();

    register::tcp_stream(&stream, poll_registry);
    register::udp_sockets(&udp_socket, &multicast_socket, poll_registry);
    register::stdin(poll_registry);

    events::poll(poll, events, stream, udp_socket, multicast_socket);
}

fn create_multicast_listener() -> UdpSocket {
    let multicast_socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).unwrap();
    multicast_socket.set_reuse_port(true).unwrap();

    let address: SocketAddr = MULTICAST_GROUP.into();
    multicast_socket.bind(&address.into()).unwrap();
    multicast_socket
        .join_multicast_v4(&MULTICAST_GROUP.0, &Ipv4Addr::UNSPECIFIED)
        .unwrap();

    multicast_socket.into()
}
