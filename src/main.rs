use local_ip_address::local_ip;
use tiny_http::{Server, Request, Method, Response};
use std::process::exit;
use std::fs::File;
use std::net::{IpAddr::V4, Ipv4Addr};
use std::env::args;
use std::io::{Read, Write};
use std::thread::sleep;
use std::time::Duration;

///The constant SRC_DIR is for setting the path of the directory "src", where all the files are.
///
///Default: "."
///
///IMPORTANT: Make sure there is no leading slash.
///
const SRC_DIR: &str = ".";

fn main() -> Result<(), ()> {
    let args: Vec<String> = args().collect();
    match args.len() {
        2 => {
            let ip = local_ip().unwrap_or_else(|err| {
                eprintln!("ERROR: Could no get local IP: {err}");
                V4(Ipv4Addr::new(127, 0, 0, 1))
            });
            start_server(&ip.to_string(), &args[1]);
        }
        3 => {
            start_server(&args[1], &args[2]).unwrap_or_else(|_| {
                let ip = local_ip().unwrap_or_else(|err| {
                    eprintln!("ERROR: Could no get local IP: {err}");
                    V4(Ipv4Addr::new(127, 0, 0, 1))
                });
                start_server(&ip.to_string(), &args[2]);
            });
        
        }
        _ => {
            println!();            
            exit(1);
        }
    }
    exit(0);
}

fn start_server(address: &str, port: &str) -> Result<(), ()> {
    let full_addr = format!("{address}:{port}"); 
    let server = Server::http(&full_addr).map_err(|err| {
        eprintln!("ERROR: Could not start server at {full_addr}: {err}");
    })?; 
    println!("Server started at {full_addr}");
    loop {
        let mut request = server.recv().map_err(|err| {
            eprintln!("ERROR: Could not receive request: {err}");
        })?;
        println!("Request received for {}", request.url());
        match (request.method(), request.url()) {
            (Method::Get, "/" | "/index.html") => {
                serve("index.html", request);
            } 
            (Method::Get, "/script.js") => {
                serve("script.js", request);
            }
            (Method::Get, "/style.css") => {
                serve("style.css", request);
            }
            (Method::Get, "/favicon.ico") => {
                serve("favicon.ico", request);
            }
            (Method::Post, "/validate") => {
                post("data", "secretsauce"); 
                println!("{}", get_request_content(&mut request).unwrap());
                request.respond(Response::from_string("pass".to_string()));
            }
            _ => {
                eprintln!("ERROR: Invalid request: {}", request.url());
                request.respond(Response::from_string("404".to_string()));
            }
        }
    }
    Ok(())
}
fn serve(filename: &str, request: Request) {
    let path = format!("{SRC_DIR}/{filename}");
    let file = File::options().read(true).open(&path).unwrap_or_else(|err| {
        eprintln!("ERROR: Could not open file: {err}"); 
        File::create(filename).unwrap()
    });
    let _ = request.respond(Response::from_file(file)).map_err(|err| {
        eprintln!("ERROR: Could not respond: {err}");
    });
}
fn post(filename: &str, content: &str) {
    let path = format!("{SRC_DIR}/{filename}");
    let mut file = File::options().append(true).open(&path).unwrap_or_else(|err| {
        eprintln!("ERROR: Could not open file: {err}"); 
        File::create(filename).unwrap()
    });
    write!(file, "{}\n", content);
}
fn validate(filename: &str, name: &str) -> Result<bool, ()> {
    let path = format!("{SRC_DIR}/{filename}");
    let mut file = File::options().read(true).open(&path).unwrap_or_else(|err| {
        eprintln!("ERROR: Could not open file: {err}"); 
        File::create(filename).unwrap()
    });
    let mut file_content: String = Default::default();
    file.read_to_string(&mut file_content);
    Ok(true)
}
fn get_request_content(request: &mut Request) -> Result<String, ()> {
    let mut buffer: String = Default::default();
    let _ = request.as_reader().read_to_string(&mut buffer);
    Ok(buffer)
}
