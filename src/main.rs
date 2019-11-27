use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::net::{UnixStream,UnixListener};
use std::thread;
use std::ops::Add;

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize, Debug, Clone)]
enum MsgTypes {
    register,
    ok,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Message {
    header: MsgTypes,
    value: String,
}

fn handle_client(stream: UnixStream, clients: Arc<Mutex<HashMap<String, MsgTypes>>>) {
    let read_server = BufReader::new(&stream);

    let srs = read_server;
    for res in srs.lines() {
        match res {
            Ok(mut line) => {
                if line.len() == 0 {
                    break;
                }
                println!("sent {}", line);

                let msg = serde_json::from_str(&line.as_str());
                let msg: Message = match msg {
                    Ok(msg) => {
                        msg
                    },
                    _ => {
                        println!("{}", "Bad Message!");
                        return;
                    }
                };

                match msg.header {
                    MsgTypes::register => {
                        let tmp = msg.clone();
                        clients.lock().unwrap().insert(tmp.value, tmp.header);

                        let client = UnixStream::connect(&msg.value).unwrap();
                        let mut write_client = BufWriter::new(&client);

                        write_client.write(serde_json::to_string(&msg).unwrap().as_bytes());
                        write_client.write(b"\n");
                        write_client.flush();
                    },
                    _ => {

                    }
                }

                let read_client = BufReader::new(&client);
                let mut write_server = BufWriter::new(&stream);
                let trs = read_client;
                for client_res in trs.lines() {
                    match client_res {
                        Ok(mut line) => {
                            if line.len() == 0 {
                                break;
                            }
                            println!("recv {}", line);
                            line = line.add("\n\n");
                            let bytes_wrote: usize = write_server.write(line.as_bytes()).unwrap();
                            println!("{}", bytes_wrote);
                            write_server.flush();
                        },
                        _ => {}
                    }
                }
            },
            _ => {}
        }
    }
}

fn remove_socket(path: &String) {
    std::fs::remove_file(path);
}

fn main() {
    let socket = String::from("/tmp/rust-uds.sock");

    remove_socket(&socket);
    let server = UnixListener::bind(socket).unwrap();

    let clients = Arc::new(Mutex::new(HashMap::new()));

    for stream in server.incoming() {
        match stream {
            Ok(stream) => {
                let w = clients.clone();
                thread::spawn(move || handle_client(stream, w));
            }
            Err(err) => {
                println!("Error: {}", err);
                break;
            }
        }
    }
}
