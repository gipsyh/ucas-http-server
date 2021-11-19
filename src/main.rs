use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};

use crate::request::Request;
use crate::response::ResponseBuilder;
mod request;
mod response;

fn handle_connection(mut stream: TcpStream) {
    let mut buf = vec![0_u8; 4096];
    match stream.read(&mut buf) {
        Ok(_) => {
            let request = Request::parse(buf);
            let mut responsebuilder = ResponseBuilder::new();
            match request.method {
                request::Method::Get => {
                    let mut path = "./file".to_string();
                    path += &request.path;
                    let mut f = match File::open(&path) {
                        Ok(f) => f,
                        Err(_e) => return,
                    };
                    let metadata = fs::metadata(&path).expect("unable to read metadata");
                    let mut buffer = vec![0; metadata.len() as usize];
                    f.read_exact(&mut buffer).expect("buffer overflow");
                    responsebuilder.set_status("200 OK".to_string());
                    responsebuilder.set_content(buffer);
                    let response = responsebuilder.build();
                    let buf = response.into_vec();
                    stream.write_all(&buf).unwrap();
                }
                request::Method::Post => {
                    todo!()
                }
            }
        }
        Err(e) => println!("Unable to read stream: {}", e),
    }
}

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:9999")?;
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        std::thread::spawn(|| handle_connection(stream));
    }
    todo!()
}
