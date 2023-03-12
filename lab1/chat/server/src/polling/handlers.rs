use std::{
    collections::HashMap,
    net::{Shutdown, SocketAddr, TcpListener, TcpStream, UdpSocket},
    sync::{Arc, RwLock},
    thread::{self, JoinHandle},
};

mod connection;
use connection::handle_client;

pub fn accept_connection(
    listener: &TcpListener,
    sockets_by_ports: &Arc<RwLock<HashMap<SocketAddr, TcpStream>>>,
    join_handles: &mut Vec<JoinHandle<()>>,
) {
    let (stream, _addr) = listener.accept().unwrap();
    let sockets_by_ports = Arc::clone(sockets_by_ports);
    join_handles.push(thread::spawn(|| handle_client(stream, sockets_by_ports)));
}

pub fn handle_udp_message(
    udp_socket: &UdpSocket,
    udp_buffer: &mut [u8],
    sockets_by_ports: &Arc<RwLock<HashMap<SocketAddr, TcpStream>>>,
) {
    let (bytes, src_addr) = udp_socket.recv_from(udp_buffer).unwrap();
    let sender_info = format!("\x1B[37;1m[{}]:\x1B[0m \x1B[3m", src_addr.port());

    let sockets_by_ports = sockets_by_ports.read().unwrap();
    for recv_addr in sockets_by_ports.keys() {
        if *recv_addr != src_addr {
            let msg = [&sender_info.as_bytes(), &udp_buffer[..bytes]].concat();
            udp_socket.send_to(&msg, recv_addr).unwrap();
        }
    }
}

pub fn close_clients_sockets(sockets_by_ports: &Arc<RwLock<HashMap<SocketAddr, TcpStream>>>) {
    let sockets = sockets_by_ports.write().unwrap();
    for socket in sockets.values() {
        socket.shutdown(Shutdown::Write).unwrap();
    }
    println!(
        "\n\x1B[32;1mThe server has been successfully shut down! Closed all connections!\x1B[0m"
    );
}
