use std::{
    io::{BufRead, BufReader, Stdin, Write},
    net::{Ipv4Addr, SocketAddr, TcpStream, UdpSocket},
    str,
};

pub const MULTICAST_GROUP: (Ipv4Addr, u16) = (Ipv4Addr::new(239, 255, 0, 1), 9912);

const ASCII_ART_UNICAST: &[u8] = "
██╗░░░██╗██████╗░██████╗░  ██╗░██████╗  ███████╗██╗░░░██╗███╗░░██╗██╗
██║░░░██║██╔══██╗██╔══██╗  ██║██╔════╝  ██╔════╝██║░░░██║████╗░██║██║
██║░░░██║██║░░██║██████╔╝  ██║╚█████╗░  █████╗░░██║░░░██║██╔██╗██║██║
██║░░░██║██║░░██║██╔═══╝░  ██║░╚═══██╗  ██╔══╝░░██║░░░██║██║╚████║╚═╝
╚██████╔╝██████╔╝██║░░░░░  ██║██████╔╝  ██║░░░░░╚██████╔╝██║░╚███║██╗
░╚═════╝░╚═════╝░╚═╝░░░░░  ╚═╝╚═════╝░  ╚═╝░░░░░░╚═════╝░╚═╝░░╚══╝╚═╝"
    .as_bytes();

pub const ASCII_ART_MULTICAST: &[u8] = "
╭╮╱╭┳━━━┳━━━╮╭━━┳━━━╮╭━━━┳╮╱╭┳━╮╱╭┳╮
┃┃╱┃┣╮╭╮┃╭━╮┃╰┫┣┫╭━╮┃┃╭━━┫┃╱┃┃┃╰╮┃┃┃
┃┃╱┃┃┃┃┃┃╰━╯┃╱┃┃┃╰━━╮┃╰━━┫┃╱┃┃╭╮╰╯┃┃
┃┃╱┃┃┃┃┃┃╭━━╯╱┃┃╰━━╮┃┃╭━━┫┃╱┃┃┃╰╮┃┣╯
┃╰━╯┣╯╰╯┃┃╱╱╱╭┫┣┫╰━╯┃┃┃╱╱┃╰━╯┃┃╱┃┃┣╮
╰━━━┻━━━┻╯╱╱╱╰━━┻━━━╯╰╯╱╱╰━━━┻╯╱╰━┻╯"
    .as_bytes();

pub fn handle_tcp_message(buf_reader: &mut BufReader<TcpStream>, buffer: &mut String) -> bool {
    let bytes = buf_reader.read_line(buffer).unwrap();
    if bytes == 0 {
        println!(
            "\r\x1B[0m\x1B[31;1mThe web chat server is unreachable! Closing connection!\x1B[0m"
        );
        return false;
    }
    true
}

pub fn recv_udp_message(udp_socket: &UdpSocket, udp_buffer: &mut [u8]) {
    let (bytes, _) = udp_socket.recv_from(udp_buffer).unwrap();
    let message = str::from_utf8(&udp_buffer[..bytes]).unwrap();
    println!("\r\x1B[0m\x1B[31;1m<\x1B[0m {message}\x1B[0m");
}

pub fn recv_multicast_msg(multicast_socket: &UdpSocket, udp_buffer: &mut [u8], my_port: u16) {
    let (bytes, src_addr) = multicast_socket.recv_from(udp_buffer).unwrap();
    if src_addr.port() != my_port {
        let message = str::from_utf8(&udp_buffer[..bytes]).unwrap();
        println!("\r\x1B[0m\x1B[31;1m<\x1B[0m {message}\x1B[0m");
    }
}

pub fn handle_stdin(
    stdin: &Stdin,
    buffer: &mut String,
    mut stream: &TcpStream,
    udp_socket: &UdpSocket,
    msg: &Vec<u8>,
    server_addr: SocketAddr,
) -> bool {
    let bytes = stdin.read_line(buffer).unwrap();
    if buffer == "exit\n" {
        println!("\n\x1B[0m\x1B[32;1mYou've left the chat!");
        return false;
    } else if buffer == "U\n" {
        udp_socket.send_to(ASCII_ART_UNICAST, server_addr).unwrap();
    } else if buffer == "M\n" {
        udp_socket.send_to(&msg, MULTICAST_GROUP).unwrap();
    } else if bytes > 1 {
        stream.write_all(buffer.as_bytes()).unwrap();
    }
    true
}
