extern crate socket2;

use std::net::SocketAddr;
use socket2::*;
use std::io::{Result, ErrorKind};
use std::env;
use std::thread;
use std::time::Duration;

const MAX_UDP_PAYLOAD_SIZE : usize = 65507;

fn start_server_udp() -> Result<()> {
    let socket = Socket::new(Domain::ipv4(), Type::dgram(), Some(Protocol::udp()))?;
    let mut buffer = [0; MAX_UDP_PAYLOAD_SIZE];
    let listen_addr = "127.0.0.1:12345".parse::<SocketAddr>().unwrap().into();
    socket.bind(&listen_addr).unwrap();

    let (bytes_read, peer_addr) = socket.recv_from(&mut buffer)?;
    let response = match String::from_utf8(buffer.to_vec()) {
        Ok(msg) => {
            println!("[SERVER]: {}", &msg);
            println!("[SERVER] {} bytes read from {:?}", bytes_read, peer_addr);
            buffer.reverse();
            String::from_utf8(buffer.to_vec()).unwrap()
        }
        Err(e) => { 
            println!("[SERVER]: Error decoding message {}", e);
            String::from("[SERVER ERROR]")
        }
    };

    match socket.send_to(response.as_bytes(), &peer_addr) {
        Ok(bytes_sent) => {
            println!("[SERVER] {} bytes sent.", bytes_sent);
        },
        Err(e) => println!("[SERVER] Error writing to socket: {}", e),
    }   

    Ok(())
}

fn start_client_udp( msg : String) -> Result<()> { 
    let socket = Socket::new(Domain::ipv4(), Type::dgram(), Some(Protocol::udp()))?;
    let mut buffer = [0; MAX_UDP_PAYLOAD_SIZE+1];
    let remote_addr = "127.0.0.1:12345".parse::<SocketAddr>().unwrap().into();

    let _res = socket.connect(&remote_addr)?;

    println!("[CLIENT] Local address: {:?}", socket.local_addr());
    match socket.send(msg.as_bytes()) {
        Ok(bytes_sent) => {
            println!("[CLIENT] {} bytes sent.", bytes_sent);
         },
        Err(e) => println!("[CLIENT] Error writing to socket: {}", e),
    }

    let (bytes_read, peer_addr) = socket.recv_from(&mut buffer)?;
    match String::from_utf8(buffer.to_vec()) {
        Ok(msg) => {
            println!("[CLIENT]: {}", msg);
            println!("[CLIENT] {} bytes read.", bytes_read);
        }
        Err(e) => { 
            println!("[CLIENT]: Error decoding message {}", e);
        }
    }

    Ok(())
}

fn start_server_unix() -> Result<()> {
    let socket = Socket::new(Domain::unix(), Type::dgram(), None)?;
    let timeout = Duration::from_millis(10);
    let mut buffer = [0; MAX_UDP_PAYLOAD_SIZE];
    let listen_addr = SockAddr::unix("/tmp/socket").unwrap();
    socket.set_read_timeout(Some(timeout)).expect("Could not set timeout for socket");
    socket.bind(&listen_addr).unwrap();
    
    println!("[SERVER] Local address: {:?}", socket.local_addr());
    loop {
        let (bytes_read, peer_addr) = match socket.recv_from(&mut buffer) {
            Ok((br, pa)) => { (br, pa)},
            Err(e) => {
                if e.kind() == ErrorKind::WouldBlock {
                    /* Not an error, continue */
                    continue;
                } else {
                    return Err(e)
                }

            }
        };

        match String::from_utf8(buffer.to_vec()) {
            Ok(msg) => {
                println!("[SERVER]: {}", &msg);
                println!("[SERVER] {} bytes read from {:?}", bytes_read, peer_addr);
                buffer.reverse();
                let response = String::from_utf8(buffer[(buffer.len() - bytes_read)..].to_vec()).unwrap();

                thread::sleep(Duration::from_millis(100));
    
                match socket.send_to(response.as_bytes(), &peer_addr) {
                    Ok(bytes_sent) => {
                        println!("[SERVER] {} bytes sent.", bytes_sent);
                    },
                    Err(e) => println!("[SERVER] Error writing to socket: {}", e),
                }
            }
            Err(e) => { 
                println!("[SERVER]: Error decoding message {}", e);
                // String::from("[SERVER ERROR]")
            }
        };
    
        if bytes_read > 0 {
            break;
        }
    }

    Ok(())
}

fn start_client_unix( msg : String) -> Result<()> { 
    let socket = Socket::new(Domain::unix(), Type::dgram(), None)?;
    let mut buffer = [0; MAX_UDP_PAYLOAD_SIZE+1];
    let remote_addr = SockAddr::unix("/tmp/socket").unwrap();
    let listen_addr = SockAddr::unix("/tmp/socket2").unwrap();
    socket.bind(&listen_addr).unwrap();

    //let _res = socket.connect(&remote_addr)?;

    println!("[CLIENT] Local address: {:?}", socket.local_addr());
    println!("[CLIENT] message size: {}", msg.len());
    match socket.send_to(msg.as_bytes(), &remote_addr) {
        Ok(bytes_sent) => {
            println!("[CLIENT] {} bytes sent.", bytes_sent);
         },
        Err(e) => println!("[CLIENT] Error writing to socket: {}", e),
    }

    let (bytes_read, peer_addr) = socket.recv_from(&mut buffer)?;
    match String::from_utf8(buffer.to_vec()) {
        Ok(msg) => {
            println!("[CLIENT]: {}", msg);
            println!("[CLIENT] {} bytes read.", bytes_read);
        }
        Err(e) => { 
            println!("[CLIENT]: Error decoding message {}", e);
        }
    }

    Ok(())
}

fn start_server_stream() -> Result<()> {
    let socket = Socket::new(Domain::ipv4(), Type::stream(), Some(Protocol::tcp()))?;
    let mut buffer = [0; MAX_UDP_PAYLOAD_SIZE];
    // let listen_addr = SockAddr::unix("/tmp/socket").unwrap();
    let listen_addr = "127.0.0.1:12345".parse::<SocketAddr>().unwrap().into();
    socket.bind(&listen_addr).unwrap();
    socket.listen(128)?;
    // socket.set_read_timeout(Some(Duration::from_millis(1000))).expect("Could not set socket read timeout");
    socket.set_nonblocking(true).expect("Could not set the socket in nonblocking mode");
    let mut finish = false;
    println!("[SERVER] Local address: {:?}", socket.local_addr());


    while !finish {
        let (client, peer_addr) = match socket.accept() {
            Ok(res) => res,
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    /* Not an issue */
                    continue;
                } else {
                    println!("Error accepting");
                    return Err(e);
                }
            }
        };
        // finish = true;
        // client.set_read_timeout(Some(Duration::from_millis(1000))).expect("Could not set client read timeout");
        client.set_nonblocking(false).expect("Could not set the client in nonblocking mode");
        let bytes_read = client.recv(&mut buffer)?;
        let response = match String::from_utf8(buffer.to_vec()) {
            Ok(msg) => {
                println!("[SERVER]: {}", &msg);
                println!("[SERVER] {} bytes read from {:?}", bytes_read, peer_addr);
                buffer.reverse();
                String::from_utf8(buffer[(buffer.len() - bytes_read)..].to_vec()).unwrap()
            }
            Err(e) => { 
                println!("[SERVER]: Error decoding message {}", e);
                String::from("[SERVER ERROR]")
            }
        };
    
        thread::sleep(Duration::from_millis(100));
    
        match client.send(response.as_bytes()) {
            Ok(bytes_sent) => {
                println!("[SERVER] {} bytes sent.", bytes_sent);
            },
            Err(e) => println!("[SERVER] Error writing to socket: {}", e),
        }   
    }


    Ok(())
}

fn start_client_stream( msg : String) -> Result<()> {
    let socket = Socket::new(Domain::ipv4(), Type::stream(), Some(Protocol::tcp()))?;
    // socket.set_nonblocking(true).expect("Could not set the socket in nonblocking mode");
    println!("Socket created and configured successfully");

    let mut buffer = [0; MAX_UDP_PAYLOAD_SIZE+1];
    // let remote_addr = SockAddr::unix("/tmp/socket").unwrap();
    let remote_addr = "127.0.0.1:12345".parse::<SocketAddr>().unwrap().into();

    let _res = socket.connect_timeout(&remote_addr, Duration::from_millis(100))?;

    println!("[CLIENT] Local address: {:?}", socket.local_addr());
    match socket.send(msg.as_bytes()) {
        Ok(bytes_sent) => {
            println!("[CLIENT] {} bytes sent.", bytes_sent);
         },
        Err(e) => println!("[CLIENT] Error writing to socket: {}", e),
    }

    let bytes_read = socket.recv(&mut buffer)?;
    match String::from_utf8(buffer.to_vec()) {
        Ok(msg) => {
            println!("[CLIENT]: {}", msg);
            println!("[CLIENT] {} bytes read.", bytes_read);
        }
        Err(e) => { 
            println!("[CLIENT]: Error decoding message {}", e);
        }
    }

    Ok(())
}

fn main() {
    let args : Vec<String> = env::args().collect();
    if args.len() > 1 {
        if args[1] == "udp" {
            if args[2] == "server" {
                match start_server_udp() {
                    Ok(_) => { },
                    Err(e) => {
                        println!("[SERVER] Error: {}", e);
                    },
                }
            } else {
                match start_client_udp(args[3].clone()) {
                    Ok(_) => { },
                    Err(e) => { 
                        println!("[CLIENT] Error: {}", e);
                    }
                }
            }
        } else if args[1] == "unix" {
            if args[2] == "server" {
                match start_server_unix() {
                    Ok(_) => { },
                    Err(e) => {
                        println!("[SERVER] Error: {}", e);
                    },
                }
            } else {
                match start_client_unix(args[3].clone()) {
                    Ok(_) => { },
                    Err(e) => { 
                        println!("[CLIENT] Error: {}", e);
                    }
                }
            }
        } else {
            if args[2] == "server" {
                match start_server_stream() {
                    Ok(_) => { },
                    Err(e) => {
                        println!("[SERVER] Error type: {:?}, message: {}", e.kind(), &e);
                    },
                }
            } else {
                match start_client_stream(args[3].clone()) {
                    Ok(_) => { },
                    Err(e) => { 
                        println!("[CLIENT] Error type: {:?}, message: {}", e.kind(), &e);
                    }
                }
            }            
        }

    } 
}
