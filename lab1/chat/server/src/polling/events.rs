use std::{
    collections::HashMap,
    io::stdin,
    net::{TcpListener, UdpSocket},
    sync::{Arc, RwLock},
};

use super::handlers::{accept_connection, close_clients_sockets, handle_udp_message};
use super::register::{SOCKET_CONN, UDP_MESSAGE, USER_INPUT};
use mio::{Events, Poll};

pub fn poll(mut poll: Poll, mut events: Events, listener: TcpListener, udp_socket: UdpSocket) {
    let (stdin, mut buffer) = (stdin(), String::new());
    let (sockets_by_ports, mut join_handles) = (Arc::new(RwLock::new(HashMap::new())), Vec::new());
    let mut udp_buffer = [0; 2048];

    'outer: loop {
        poll.poll(&mut events, None).unwrap();

        for event in events.iter() {
            if !event.is_readable() {
                continue;
            }

            match event.token() {
                SOCKET_CONN => accept_connection(&listener, &sockets_by_ports, &mut join_handles),
                UDP_MESSAGE => handle_udp_message(&udp_socket, &mut udp_buffer, &sockets_by_ports),
                USER_INPUT => {
                    stdin.read_line(&mut buffer).unwrap();
                    if buffer == "exit\n" {
                        close_clients_sockets(&sockets_by_ports);
                        break 'outer;
                    }
                    buffer.clear();
                }
                _ => {}
            }
        }
    }

    for handle in join_handles {
        handle.join().unwrap();
    }
}
