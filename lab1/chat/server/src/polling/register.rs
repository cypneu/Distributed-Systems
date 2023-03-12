use mio::Token;
use mio::{unix::SourceFd, Interest, Registry};
use std::io;
use std::net::{TcpListener, UdpSocket};
use std::os::fd::AsRawFd;

pub const SOCKET_CONN: Token = Token(0);
pub const UDP_MESSAGE: Token = Token(1);
pub const USER_INPUT: Token = Token(2);

pub fn tcp_listener(listener: &TcpListener, poll_registry: &Registry) {
    let listener_fd = listener.as_raw_fd();
    let mut source_listener_fd = SourceFd(&listener_fd);
    poll_registry
        .register(&mut source_listener_fd, SOCKET_CONN, Interest::READABLE)
        .unwrap();
}

pub fn udp_socket(udp_socket: &UdpSocket, poll_registry: &Registry) {
    let udp_socket_fd = udp_socket.as_raw_fd();
    let mut source_udp_socket_fd = SourceFd(&udp_socket_fd);
    poll_registry
        .register(&mut source_udp_socket_fd, UDP_MESSAGE, Interest::READABLE)
        .unwrap();
}

pub fn stdin(poll_registry: &Registry) {
    let stdin_fd = io::stdin().as_raw_fd();
    let mut source_stdin_fd = SourceFd(&stdin_fd);
    poll_registry
        .register(&mut source_stdin_fd, USER_INPUT, Interest::READABLE)
        .unwrap();
}
