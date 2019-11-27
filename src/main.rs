use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::net::{UnixStream,UnixListener};
use std::thread;
use std::ops::Add;

fn handle_client(stream: UnixStream) {
    let read_server = BufReader::new(&stream);

    let client = UnixStream::connect("/tmp/app-uds.sock").unwrap();

    let mut write_client = BufWriter::new(&client);

    let srs = read_server;
    for res in srs.lines() {
        match res {
            Ok(mut line) => {
                if line.len() == 0 {
                    break;
                }
                println!("{}", line);
                line = line.add("\n\n");
                let bytes_wrote: usize = write_client.write(line.as_bytes()).unwrap();
                println!("{}", bytes_wrote);
                write_client.flush();

                let read_client = BufReader::new(&client);
                let mut write_server = BufWriter::new(&stream);
                let trs = read_client;
                for client_res in trs.lines() {
                    match client_res {
                        Ok(mut line) => {
                            if line.len() == 0 {
                                break;
                            }
                            println!("{}", line);
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

fn main() {
    let server = UnixListener::bind("/tmp/rust-uds.sock").unwrap();

    for stream in server.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| handle_client(stream));
            }
            Err(err) => {
                println!("Error: {}", err);
                break;
            }
        }
    }
}
