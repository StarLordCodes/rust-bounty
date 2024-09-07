use std::{
    io::{Read, Result, Write},
    net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream},
};

use http_bounty::http::request::HttpRequest;

fn create_socket() -> SocketAddr {
    SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::LOCALHOST), 5500)
}

fn handle_client(stream: &mut TcpStream) -> std::io::Result<()> {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer)?;
    let buf_str = String::from_utf8_lossy(&buffer);
    let request = HttpRequest::new(&buf_str)?;
    let response = request.response()?;

    println!("{:?}", &response);
    println!("{:?}", &response.response_body);
    let mut body = response.response_body.clone();
    stream.write(&mut body)?;
    stream.flush()?;
    Ok(())
}
fn serve(socket: SocketAddr) -> Result<()> {
    let listener = TcpListener::bind(socket)?;
    let mut counter = 0;
    for stream in listener.incoming() {
        match std::thread::spawn(|| handle_client(&mut stream?)).join() {
            Ok(_) => {
                counter += 1;
                println!("Connected stream: {}", counter);
            }
            Err(_) => continue,
        }
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    let socket = create_socket();
    serve(socket)?;
    Ok(())
}
