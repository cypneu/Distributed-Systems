use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    net::{Shutdown, SocketAddr, TcpStream},
    sync::{Arc, RwLock},
};

pub fn handle_client(
    stream: TcpStream,
    sockets_by_ports: Arc<RwLock<HashMap<SocketAddr, TcpStream>>>,
) {
    let client_addr = stream.peer_addr().unwrap();

    let client_port = client_addr.port();
    println!("\x1B[1m[{client_port}]\x1B[0m has joined the chat!");

    insert_with_lock(&sockets_by_ports, client_addr, &stream);

    let (mut buf_reader, mut message) = (BufReader::new(stream), String::new());
    loop {
        let bytes = buf_reader.read_line(&mut message).unwrap();
        if bytes == 0 {
            let sock = remove_with_lock(&sockets_by_ports, client_addr);
            if let Ok(_) = sock.shutdown(Shutdown::Write) {
                println!("\x1B[1m[{client_port}]\x1B[0m has left the chat!");
            }
            break;
        }

        send_to_all_tcp_clients(&sockets_by_ports, client_addr, &message);
        message.clear();
    }
}

pub fn send_to_all_tcp_clients(
    sockets_by_ports: &Arc<RwLock<HashMap<SocketAddr, TcpStream>>>,
    src_addr: SocketAddr,
    msg: &String,
) {
    let sender = format!("\x1B[37;1m[{}]:\x1B[0m \x1B[3m", src_addr.port());
    let sockets_by_ports = sockets_by_ports.read().unwrap();
    for (recv_addr, mut sock) in &*sockets_by_ports {
        if *recv_addr != src_addr {
            sock.write_all(format!("{sender}{msg}").as_bytes()).unwrap();
        }
    }
}

pub fn insert_with_lock(
    hashmap: &Arc<RwLock<HashMap<SocketAddr, TcpStream>>>,
    client_addr: SocketAddr,
    stream: &TcpStream,
) {
    let mut locked_hashmap = hashmap.write().unwrap();
    locked_hashmap.insert(client_addr, stream.try_clone().unwrap());
}

pub fn remove_with_lock(
    hashmap: &Arc<RwLock<HashMap<SocketAddr, TcpStream>>>,
    client_addr: SocketAddr,
) -> TcpStream {
    let mut locked_hashmap = hashmap.write().unwrap();
    locked_hashmap.remove(&client_addr).unwrap()
}
