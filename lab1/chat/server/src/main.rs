use std::net::{TcpListener, UdpSocket};

use mio::{Events, Poll};

mod polling;
use polling::{events, register};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:9090")
        .expect("\x1B[31;1mThe provided address for TcpListener is already in use!\x1B[0m");

    let udp_socket = UdpSocket::bind("127.0.0.1:9090")
        .expect("\x1B[31;1mThe provided address for UdpSocket is already in use!\x1B[0m");

    println!("\n\x1B[32;1mThe web chat server has started running on 127.0.0.1:9090!\x1B[0m\n");

    let (poll, events) = (Poll::new().unwrap(), Events::with_capacity(1024));
    let poll_registry = poll.registry();

    register::tcp_listener(&listener, poll_registry);
    register::udp_socket(&udp_socket, poll_registry);
    register::stdin(poll_registry);

    events::poll(poll, events, listener, udp_socket);
}
