use local_ip_address::local_ip;
use tiny_http::{Server, Request, Method, Response};
use std::process::exit;
use std::fs::{File, copy, remove_file};
use std::fs::read_to_string;
use std::net::{IpAddr::V4, Ipv4Addr};
use std::env::args;
use std::io::Write;
//use std::fs::write;
use std::thread::sleep;
use std::time::Duration;
use std::path::Path;

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
            //TODO
            (Method::Post, "/validate") => {
                let content = get_request_content(&mut request).unwrap();
                println!("Content: {}", &content);
                if !is_contained("nameList", &content) {
                    post("nameList", &content); 
                    println!("Success: {}", &content);
                    remove_file("nameList.swap");
                    request.respond(Response::from_string("success".to_string()));
                } else {
                    request.respond(Response::from_string("fail".to_string()));
                    println!("Already registered: {}", &content);
                }
            }
            //TODO
            (Method::Put, "/changeName") => {
                let content = get_request_content(&mut request).unwrap();
                let splitContent: Vec<&str> = content.split("&").collect();
                let name = splitContent[0];
                let newName = splitContent[1];
                println!("name: {name}, newname: {newName}");
                if is_contained("nameList", name) {
                    change_name("nameList", name, newName);
                } else {
                    post("nameList", newName); 
                }
                request.respond(Response::from_string("200".to_string()));
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
    let file = File::options()
        .read(true)
        .open(&path)
        .unwrap_or_else(|err| {
            eprintln!("ERROR: Could not open file: {err}"); 
            File::create(filename).unwrap()
        });
    let _ = request.respond(Response::from_file(file)).map_err(|err| {
        eprintln!("ERROR: Could not respond: {err}");
    });
}
fn post(filename: &str, content: &str) {
    let path = format!("{SRC_DIR}/{filename}");
    let swap_path = format!("{SRC_DIR}/{filename}.swap");
    copy(&path, &swap_path);
    let mut file = File::options()
        .append(true)
        .open(&swap_path)
        .unwrap_or_else(|err| {
            eprintln!("ERROR: Could not open file: {err}"); 
            File::create(filename).unwrap()
        });
    write!(file, "{}\n", content);
    copy(&swap_path, &path);
}
//TODO
fn is_contained(filename: &str, name: &str) -> bool {
    let swap_path = format!("{SRC_DIR}/{filename}.swap");
    let path = format!("{SRC_DIR}/{filename}");
    let file_content = read_to_string(path).unwrap_or_else(|err| {
        eprintln!("ERROR: Could not open file: {err}"); 
        File::create(filename.to_string() + ".swap").unwrap();
        "".to_string()
    });
    println!("{file_content}");
    //println!("File content: {lines:?}", lines = file_content.lines().collect::<Vec<_>>());
    println!("Name: {name}");
    let mut is_contained: bool = false;
    let is_contained_vec: Vec<bool> = file_content
        .lines()
        .map(|line| {
            if line.to_lowercase() == name.to_lowercase() {
                return true
            } else {
                return false
            }
        })
        .collect();
    for boolean in is_contained_vec {
        if boolean {
            is_contained = true;
        }
    }
    is_contained
}
fn get_request_content(request: &mut Request) -> Result<String, ()> {
    let mut buffer: String = Default::default();
    let _ = request.as_reader().read_to_string(&mut buffer);
    Ok(buffer)
}
//TODO
fn change_name(filename: &str, name: &str, newName: &str) {
    //let swap_path = format!("{SRC_DIR}/{filename}.swap");
    let path = format!("{SRC_DIR}/{filename}");
    //copy(&path, &swap_path);
    let file_content = read_to_string(&path).unwrap_or_else(|err| {
        eprintln!("ERROR: Could not open file: {err}"); 
        File::create(filename.to_string() + ".swap").unwrap();
        "".to_string()
    });
    /*let mut file = File::options()
        .write(true)
        .open(&swap_path)
        .unwrap_or_else(|err| {
            eprintln!("ERROR: Could not open file: {err}"); 
            File::create(filename).unwrap()
        });*/
    let mut content_lines: Vec<_> = file_content
        .lines()
        .collect();
    println!("Lines: {content_lines:?}");
    let mut changes: bool = false;
    let content_lines_processed: Vec<String> = content_lines
        .iter()
        .map(|line| {
            let mut line_processed = line;
            if line.to_lowercase() == name.to_lowercase() {
                line_processed = &newName;
                changes = true;
            }
            //*line.push_str("\n");
            line_processed.to_string() + "\n"
        })
        .collect();
    println!("Processed: {:?}", content_lines_processed);
    if changes {
        let content_string = content_lines_processed.join("");
        let content_bytes = content_string.as_bytes();
        //file.write_all(content_bytes);
        override_file(filename, content_string);
    } else {
        println!("Nothing changed");
    }
    /*copy(&swap_path, &path);
    remove_file(&swap_path);*/
    println!("Lines: {content_lines:?}");
    println!("{file_content}");
}
fn override_file(filename: &str, content: String) {
    let swap_path = format!("{SRC_DIR}/{filename}.swap");
    let path = format!("{SRC_DIR}/{filename}");
    copy(&path, &swap_path);
    let mut file = File::options()
        .write(true)
        .open(&swap_path)
        .unwrap_or_else(|err| {
            eprintln!("ERROR: Could not open file: {err}"); 
            File::create(filename).unwrap()
        });
    write!(file, "{content}");
    copy(&path, &swap_path);
    remove_file(&swap_path);
}
