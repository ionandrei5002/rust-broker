use std::io::{BufReader, BufWriter, Write, Read};
use std::os::unix::net::{UnixStream,UnixListener};
use std::thread;

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use byteorder::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
enum MsgTypes {
    Register,
    Ok,
    Command,
    Close,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Message {
    header: MsgTypes,
    value: String,
}

fn read_message(stream: &UnixStream) -> String {
    let mut read = BufReader::new(stream);
    let mut msg_size = read.read_u64::<BigEndian>().unwrap() as usize;
    let mut line = String::from("");
    line.reserve(msg_size);

    while msg_size > 0 {
        let mut buffer = [0u8; 1024];
        let size = read.read(&mut buffer).unwrap();
        for ch in 0..size {
            line.push(char::from(buffer[ch]));
        }
        msg_size = msg_size - size;
    }
    println!("reading");
    return line;
}

fn write_message(stream: &UnixStream, message: &String) -> usize {
    let mut write = BufWriter::new(stream);
    let msg_size = message.as_bytes().len();
    if write.write_u64::<BigEndian>(msg_size as u64).is_err() {
        println!("Can't write to broker");
        return 0;
    }
    if write.write_all(message.as_bytes()).is_err() {
        println!("Can't write to broker");
        return 0;
    }
    if write.flush().is_err() {
        println!("Can't flush");
        return 0;
    }
    println!("writing");
    return msg_size;
}

fn register_client(stream: &UnixStream, msg: &Message, clients: &Arc<Mutex<HashMap<String, MsgTypes>>>) {
    let tmp = msg.clone();
    clients.lock().unwrap().insert(tmp.value, tmp.header);

    let close = Message { header: MsgTypes::Close, value: String::from(&msg.value) };
    {
        let msg = serde_json::to_string(&close).unwrap();
        write_message(stream, &msg);
        println!("{}", msg);
    }
}

fn broadcast_command(msg: &Message, clients: &Arc<Mutex<HashMap<String, MsgTypes>>>, stream: &UnixStream) {
    let w = clients.lock().unwrap();
    for c in w.iter() {
        let client = UnixStream::connect(c.0).unwrap();
        let msg = serde_json::to_string(&msg).unwrap();
        write_message(&client, &msg);

        let line = read_message(&client);

        write_message(&stream, &line);
    }
}

fn handle_client(stream: UnixStream, clients: Arc<Mutex<HashMap<String, MsgTypes>>>) {
    let line = read_message(&stream);

    let msg = serde_json::from_str(&line.as_str());
    let msg: Message = match msg {
        Ok(msg) => {
            msg
        },
        _ => {
            println!("{} {}", line, "Bad Message!");
            return;
        }
    };

    match msg.header {
        MsgTypes::Register => {
            register_client(&stream, &msg, &clients);
        },
        _ => {
            broadcast_command(&msg, &clients, &stream);
        }
    }
}

fn remove_socket(path: &String) {
    if std::fs::remove_file(path).is_err() {
        panic!("Can't remove socket {}", path);
    }
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
