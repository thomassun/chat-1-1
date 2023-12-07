use chat_1_1::{master, worker, Event};
use std::sync::Arc;
use std::thread;
use std::{net::TcpListener, sync::mpsc::channel};

fn main() -> std::io::Result<()> {
    let (sender, receiver) = channel::<Event>();
    // let mut conns = Connection::new();
    let listener = TcpListener::bind("127.0.0.1:1234")?;
    eprintln!("Info: listening on port 1234");
    thread::Builder::new()
        .name("master".into())
        .spawn(move || master(receiver))
        .unwrap();
    for connect in listener.incoming() {
        match connect {
            Ok(incoming) => {
                if let Ok(client_addr) = incoming.peer_addr() {
                    let stream = Arc::new(incoming);
                    // conns.insert(client_addr, stream.clone());
                    let sender = sender.clone();
                    let _ = sender.send(Event::NewConnection(client_addr, stream.clone()));
                    thread::Builder::new()
                        .name("worker".into())
                        .spawn(move || worker(stream.clone(), client_addr, sender))
                        .unwrap();
                }
            }
            Err(e) => {
                eprintln!("Something went wrong:{:?}", e);
            }
        }
    }
    Ok(())
}
