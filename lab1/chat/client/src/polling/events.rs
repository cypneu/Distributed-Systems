use mio::{Events, Poll};
use register::{TCP_SOCKET_MESSAGE, UDP_MULTICAST_MESSAGE, UDP_SOCKET_MESSAGE, USER_INPUT};
use std::{
    io::{stdin, stdout, BufReader, Write},
    net::{TcpStream, UdpSocket},
};

use super::handlers::{
    handle_stdin, handle_tcp_message, recv_multicast_msg, recv_udp_message, ASCII_ART_MULTICAST,
};
use super::register;

pub fn poll(
    mut poll: Poll,
    mut events: Events,
    stream: TcpStream,
    udp_socket: UdpSocket,
    multicast_sock: UdpSocket,
) {
    let (mut buf_reader, mut buffer) = (BufReader::new(stream.try_clone().unwrap()), String::new());
    let (stdin, mut stdout) = (stdin(), stdout());
    let (server_addr, mut udp_buff) = (stream.peer_addr().unwrap(), [0; 2048]);

    let port = stream.local_addr().unwrap().port();
    let sender_info = format!("\x1B[1m[{}]:\x1B[0m", port);
    let msg = [&sender_info.as_bytes(), ASCII_ART_MULTICAST].concat();

    'outer: loop {
        print!("\r\x1B[0m\x1B[32;1m>\x1B[0m \x1B[3m");
        stdout.flush().unwrap();

        poll.poll(&mut events, None).unwrap();

        for event in events.iter() {
            if !event.is_readable() {
                continue;
            }

            match event.token() {
                TCP_SOCKET_MESSAGE => match handle_tcp_message(&mut buf_reader, &mut buffer) {
                    true => print!("\r\x1B[0m\x1B[31;1m<\x1B[0m {buffer}\x1B[0m"),
                    false => break 'outer,
                },
                UDP_SOCKET_MESSAGE => recv_udp_message(&udp_socket, &mut udp_buff),
                UDP_MULTICAST_MESSAGE => recv_multicast_msg(&multicast_sock, &mut udp_buff, port),
                USER_INPUT => {
                    if !handle_stdin(&stdin, &mut buffer, &stream, &udp_socket, &msg, server_addr) {
                        break 'outer;
                    }
                }
                _ => {}
            }
            buffer.clear();
        }
    }
}
