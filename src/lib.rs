use std::error::Error;
use std::fmt::Display;
use std::str;
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    sync::{
        mpsc::{Receiver, Sender},
        Arc,
    },
    time::Duration,
};
#[derive(Debug)]
pub enum Event {
    NewConnection(SocketAddr, Arc<TcpStream>),
    MewMessage { from: SocketAddr, text: String },
    Disconnect(SocketAddr),
}
#[derive(Debug)]
struct MyError(String);
impl Display for MyError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
impl Error for MyError {}
pub type Connection = HashMap<SocketAddr, Arc<TcpStream>>;
pub fn master(receiver: Receiver<Event>) {
    let mut clients = Connection::new();
    loop {
        match receiver.recv_timeout(Duration::from_millis(100)) {
            Ok(event) => {
                match event {
                    Event::NewConnection(addr, stream) => {
                        // stream.as_ref().write_fmt("fmt");
                        clients.insert(addr, stream.clone());
                        let _ = writeln!(stream.as_ref(), "Welcome, welcome");
                    }
                    Event::MewMessage { from, text } => {
                        println!("{}", text);
                        clients
                            .iter()
                            .filter(|kv| kv.0 != &from)
                            .for_each(|(to, stream)| writeln!(stream.as_ref(), "{text}").unwrap_or_else(|e|eprintln!("Master ERROR: something went wrong for sending message to {to}:{:?}",e)));
                    }
                    Event::Disconnect(from) => {
                        eprintln!("Master INFO: {from} left.");
                        clients.remove(&from);
                    }
                }
            }
            Err(_) => {
                // eprintln!("ERROR: something not correct:{:?}", e)
            }
        }
    }
}
pub fn worker(stream: Arc<TcpStream>, peer: SocketAddr, sender: Sender<Event>) {
    eprintln!("INFO: new worker created for {:?}", peer);
    let mut buffer = [0; 64];
    let mut pos = 0;
    /* fn take_str_from_stream(
        mut stream: TcpStream,
        buffer: &mut [u8],
    ) -> Result<&[u8], Box<dyn Error>> {
        let n = stream.read(buffer)?;
        if n == 0 {
            stream.shutdown(std::net::Shutdown::Both)?;
            return Err(Box::new(MyError("CUOWU".to_owned())));
        } else {
            let text = str::from_utf8(&buffer)?;

        }

        Ok(buffer)
    } */
    loop {
        match stream.as_ref().read(&mut buffer[pos..]) {
            Ok(n) => {
                if n == 0 {
                    let _ = stream.as_ref().shutdown(std::net::Shutdown::Both);
                    let _ = sender.send(Event::Disconnect(peer));
                    break;
                }
                eprintln!("Worker INFO: {n} of string read");
                let text = match str::from_utf8(&buffer[..pos + n]) {
                    Ok(text) => {
                        pos = 0;
                        text
                    }
                    Err(e) => {
                        let end_up = e.valid_up_to();
                        match str::from_utf8(&buffer[..end_up]) {
                            Ok(text) => {
                                pos = end_up;
                                text
                            }
                            Err(e) => {
                                buffer.rotate_left(pos);
                                pos = 64 - pos;
                                eprintln!("Worker UNRECOVERABLE ERROR: data is not a valid utf8 stream:{:?}", e);
                                ""
                            }
                        }
                    }
                };
                let text = text.to_owned();
                if text.is_empty() {
                    continue;
                }
                match sender.send(Event::MewMessage { from: peer, text }) {
                    Ok(_) => {
                        eprintln!("Worker Info: message event published!")
                    }
                    Err(e) => {
                        eprintln!(
                            "Worker ERROR: bad things happen during publishing event:{:?}!",
                            e
                        )
                    }
                }
            }
            Err(e) => {
                eprintln!(
                    "Worker ERROR: something went wrong when read from the wire:{:?}",
                    e
                )
            }
        }
    }
}
