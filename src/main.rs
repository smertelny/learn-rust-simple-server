use log::{error, info};
use std::io::Error as IoError;
use std::io::prelude::*;
use std::net::{ TcpListener, TcpStream };
use std::error::Error;
use std::path::Path;
use std::fmt::{ Formatter, Display };
use std::fs;

struct Request<'a> {
    method: &'a str,
    uri: &'a Path,
    http_version: &'a str,
}

impl Display for Request<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}\r\n", self.method, self.uri.display(), self.http_version)
    }
}


fn parse_request_line(request: &str) -> Result<Request, Box<dyn Error>> {
    let mut parts = request.split_whitespace();
    let method = parts.next().ok_or("Method not specified")?;

    if method != "GET" {
        Err("Unsupported method")?;
    }

    let uri = Path::new(parts.next().ok_or("URI not specified")?);
    let normalized_uri = uri.to_str().expect("Invalid Unicode");

    const ROOT: &str = "/home/sergey/projects/simple-server/staticfiles";

    if !Path::new(&format!("{}{}", ROOT, normalized_uri)).exists() {
        Err("Requested resourse does not exist")?;
    }

    let http_version = parts.next().ok_or("HTTP Version is not specified")?;
    if http_version != "HTTP/1.1" {
        Err("Unsupported version of HTTP, use HTTP 1.1 instead")?;
    }

    Ok(Request{ method, uri, http_version })
}

fn handle_connection(mut stream: TcpStream) -> Result<(), IoError> {
    let mut buffer = [0; 512];

    stream.read(&mut buffer).unwrap();

    let request = String::from_utf8_lossy(&buffer[..]);
    let request_line = request.lines().next().unwrap();

    match parse_request_line(&request_line) {
        Ok(request) => {
            info!("{}", &request);

            let contents = fs::read_to_string("index.html").unwrap();
            let response = format!("{}{}", "HTTP/1.1 200 OK \r\n\r\n", contents);

            info!("{}", &response);
            stream.write(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        },
        Err(err) => error!("Bad request: {}", err),
    }
    Ok(())
}


fn main() {
   simple_logger::init().unwrap();
    info!("Starting server...");

    let ip = "0.0.0.0:3000";

    let listener = TcpListener::bind(ip).expect(format!("Failed to bind to {}", ip).as_str());
    info!("Server started on http://{}", ip);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => match handle_connection(stream) {
                Ok(_) => (),
                Err(err) => error!("Error occured while handling connection: {}", err),
            },
            Err(err) => error!("Error while establishing connection: {}", err),
        }
    }
}
