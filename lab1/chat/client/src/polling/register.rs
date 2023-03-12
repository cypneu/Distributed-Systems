use mio::Token;
use mio::{unix::SourceFd, Interest, Registry};
use std::io;
use std::net::{TcpStream, UdpSocket};
use std::os::fd::AsRawFd;

pub const TCP_SOCKET_MESSAGE: Token = Token(0);
pub const UDP_SOCKET_MESSAGE: Token = Token(1);
pub const UDP_MULTICAST_MESSAGE: Token = Token(2);
pub const USER_INPUT: Token = Token(3);

pub fn tcp_stream(stream: &TcpStream, poll_registry: &Registry) {
    let stream_fd = stream.as_raw_fd();
    let mut source_stream_fd = SourceFd(&stream_fd);
    poll_registry
        .register(
            &mut source_stream_fd,
            TCP_SOCKET_MESSAGE,
            Interest::READABLE,
        )
        .unwrap();
}

pub fn udp_sockets(udp_socket: &UdpSocket, multicast_socket: &UdpSocket, poll_registry: &Registry) {
    let udp_socket_fd = udp_socket.as_raw_fd();
    let mut source_udp_socket_fd = SourceFd(&udp_socket_fd);
    poll_registry
        .register(
            &mut source_udp_socket_fd,
            UDP_SOCKET_MESSAGE,
            Interest::READABLE,
        )
        .unwrap();

    let multicast_socket_fd = multicast_socket.as_raw_fd();
    let mut source_multicast_socket_fd = SourceFd(&multicast_socket_fd);
    poll_registry
        .register(
            &mut source_multicast_socket_fd,
            UDP_MULTICAST_MESSAGE,
            Interest::READABLE,
        )
        .unwrap();
}

pub fn stdin(poll_registry: &Registry) {
    let stdin_fd = io::stdin().as_raw_fd();
    let mut source_stdin_fd = SourceFd(&stdin_fd);
    poll_registry
        .register(&mut source_stdin_fd, USER_INPUT, Interest::READABLE)
        .unwrap();
}
