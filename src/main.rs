//#![feature(mpmc_channel)]
#![allow(warnings)]
use std::{
    str::FromStr,
    net::{IpAddr, IpAddr::V4, Ipv4Addr},
    error::Error,
    sync::{
        mpsc::{self, *},
        Arc,
        Mutex,
    },
    marker::Send,
    thread::{ self, JoinHandle },
    io::{ Cursor, Read, Write },
    //ops::Drop,
    time::Duration,
    //collections::HashMap,
    fmt::{ Display, Formatter },
    fs::OpenOptions,
    path::Path,
    process::exit,
};
use tiny_http::{
    Server,
    Request,
    Response,
    Header,
    StatusCode,
    ReadWrite,
    Method::*,
};
//use crypto::sha1::Sha1;
//use crypto::digest::Digest;
//use base64::prelude::*;
//use base16;
use getch_rs::{ Getch, Key::* };
use local_ip_address::local_ip;
use postgres::{Config, NoTls, types::{Type, ToSql, FromSql}, Statement, Client};
use chrono::offset::Local;
///Function to execute for a `ThreadPool`
pub type Job = Box<dyn FnOnce() + Send + 'static>;
#[doc(hidden)]
type ServerError = Box<dyn Error + Send + Sync + 'static>;
///Just a special type of `Response`.
pub type Answer = Response<Cursor<Vec<u8>>>;
// ///The type given by calling `upgrade()` on a `Request`.
// ///
// ///This is used to communicate via WebSocket.
// type Socket = Box<dyn ReadWrite + Send>;

//use Opcode::*;
///Struct for managing `Worker` threads.

pub enum UserAction<'a> {
    Add(&'a str, u8, char),
    Rename(u16, &'a str)
}
//unsafe impl<'a> Send for UserAction<'a> {}

pub struct ThreadPool {
    workers: Vec<Worker>,
    limit: Option<usize>,
    sender: Sender<Job>
}
///Worker thread in a `ThreadPool`.
struct Worker {
    id: usize,
    handle: Option<JoinHandle<()>>,
}
impl Worker {
    pub fn new(id: usize, rx: Arc<Mutex<Receiver<Job>>>) -> Worker {
        let handle = thread::spawn(move || loop {
            let res = rx.lock().unwrap().recv();
            match res {
                Ok(job) => {
                    job(); 
                    i!("Served ({})", id);
                }
                Err(_) => {
                    e!("I: Worker {} shutting down", id); 
                    break;
                }
            }
        });
        Worker { 
            id: id,
            handle: Some(handle),
        }
    }
}
impl ThreadPool {
    pub fn new(amount: usize, limit: Option<usize>) -> Result<ThreadPool, String> {
        let mut workers: Vec<Worker> = Vec::with_capacity(amount);
        let (tx, rx) = channel::<Job>();
        let rx = Arc::new(Mutex::new(rx));
        for id in 0..amount {
            let worker = Worker::new(id, rx.clone());
            workers.push(worker);
        }     
        Ok(ThreadPool {
            workers: workers,
            limit: limit,
            sender: tx
        })
    }
    ///Execute a `Job` in a `ThreadPool`.
    pub fn execute(&self, job: Job) -> Result<(), ()> {
        let res = &self.sender.send(job);
        match res {
            Ok(_) => Ok(()),
            Err(_) => Err(())
        }
    }
    ///Create a disposable `Worker` thread that executes a `Job`.
    pub fn spawn(&self, job: Job) {
        let id = &self.workers.len();
        match &self.limit {
            Some(n) if id <= n => {
                let handle = thread::spawn(|| job());
                let worker = Worker {id: *id, handle: Some(handle)};
            } 
            None => {
                let handle = thread::spawn(|| job());
                let worker = Worker {id: *id, handle: Some(handle)};
            }
            _ => e!("Pool max size reached"),
        }
    }
}
/*impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(&self.sender);
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);
            if let Some(thread) = worker.handle.take() {
                thread.join().unwrap();
            }
        }
    }
}*/
// ///Check if a `Request` is a WebSocket request.
// pub fn check_ws(request: &Request) -> bool {
//     let res = request
//         .headers()
//         .iter()
//         .find(|&h| h.field.equiv("Connection"))
//         .map(|h| h.value.as_str());
//     match res {
//         Some(v) if v.to_ascii_lowercase().contains("upgrade") => true,
//         _ => false
//     }
// }
///Encode the content of the header `Sec-WebSocket-Key`.
///
// ///It does the following:
// /// - Concatenate `258EAFA5-E914-47DA-95CA-C5AB0DC85B11` to it
// /// - Hash it using SHA1
// /// - Encode it into Base64
// pub fn encode(key: String) -> String {
//     let mut str = key;
//     str.push_str("258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
//     let mut sha1 = Sha1::new();
//     sha1.input_str(&str);
//     let hash = sha1.result_str();
//     let mut buf: &mut [u8] = &mut [0u8; 20];
//     base16::decode_slice(hash.as_bytes(), &mut buf);
//     let res = BASE64_STANDARD.encode(buf);
//     res
// }
// ///Create a response (`Answer`) to send back to the client.
// pub fn get_upgrade_response(request: &Request) -> Answer {
//     let key = request
//         .headers()
//         .iter()
//         .find(|&h| h.field.equiv("Sec-WebSocket-Key"))
//         .map(|h| h.value.as_str())
//         .unwrap();
//     let accept = encode(key.to_string());
//     let headers: Vec<Header> = vec![
//         Header::from_bytes(&*b"Sec-WebSocket-Accept", accept.as_bytes()).unwrap(),
//         Header::from_bytes(&*b"Connection", &*b"Upgrade").unwrap(),
//         Header::from_bytes(&*b"Upgrade", &*b"websocket").unwrap(),
//     ];
//     let mut response = Response::from_string(accept);
//     for header in headers {
//         response.add_header(header);
//     }
//     response = response.with_status_code(StatusCode(101));
//     response
// }
// #[derive(Copy, Clone)]
// #[non_exhaustive]
// pub enum Opcode {
//     //TODO: Fragment
//     Fragment = 0,
//     Text = 1,
//     Binary = 2,
//     Close = 8,
//     Ping = 9,
//     Pong = 10,
//     Unknown = 16,
// }
// impl From<u8> for Opcode {
//     fn from(value: u8) -> Self {
//         match value {
//             0 => Self::Fragment,
//             1 => Self::Text,
//             2 => Self::Binary,
//             8 => Self::Close,
//             9 => Self::Ping,
//             10 => Self::Pong,
//             _ => Self::Unknown
//         }
//     }
// }
// impl Display for Opcode {
//     fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
//         let display = match self {
//             Self::Fragment => "Fragment",
//             Self::Text => "Text",
//             Self::Binary => "Binary", 
//             Self::Close => "Close", 
//             Self::Ping => "Ping", 
//             Self::Pong => "Pong", 
//             Self::Unknown => "Unknown", 
//         };
//         write!(f, "{display}")
//     }
// }
// ///Frame struct for WebSockets.
// ///
// ///Please see more about its fields at <https://wikipedia.com>.
// pub struct Frame {
//     fin: bool,
//     rsv1: bool,
//     rsv2: bool,
//     rsv3: bool,
//     opcode: Opcode,
//     mask: Option<Vec<u8>>,
//     content: Vec<u8>
// }
// impl Frame {
//     pub fn new(fin: bool, rsv1: bool, rsv2: bool, rsv3: bool, opcode: Opcode, mask: Option<Vec<u8>>, content: Vec<u8>) -> Frame {
//         Frame {
//             fin: fin,
//             rsv1: rsv1,
//             rsv2: rsv2,
//             rsv3: rsv3,
//             opcode: opcode,
//             mask: mask,
//             content: content
//         }
//     }
//     pub fn fin(&self) -> bool {
//         self.fin
//     }
//     pub fn rsv1(&self) -> bool {
//         self.rsv1
//     }
//     pub fn rsv2(&self) -> bool {
//         self.rsv2
//     }
//     pub fn rsv3(&self) -> bool {
//         self.rsv3
//     }
//     pub fn opcode(&self) -> Opcode {
//         self.opcode
//     }
//     pub fn mask(&self) -> Option<&Vec<u8>> {
//         self.mask.as_ref()
//     }
//     ///Get the content of the `Frame` in bytes.
//     pub fn bytes(&self) -> Vec<u8> {
//         self.content.to_vec()
//     }
//     ///Get the content of the `Frame` in the form of a `String`.
//     pub fn text(&self) -> Result<String, std::string::FromUtf8Error> {
//         String::from_utf8(self.bytes())
//     }
//     ///Get the length of the content of the `Frame`.
//     pub fn content_len(&self) -> u64 {
//         self
//             .bytes()
//             .len()
//             .try_into()
//             .expect("Math ain't mathing: III")
//     }
// }
// impl Display for Frame {
//     fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
//         write!(f, "Frame(fin: {}, rsv1: {}, rsv2: {}, rsv3: {}, opcode: {}, mask: {:?}, content: {:?})", self.fin, self.rsv1, self.rsv2, self.rsv3, self.opcode, self.mask, self.content)
//     }
// }
// ///Stream struct for WebSockets.
// ///
// ///This holds the `Socket` used in communication via WebSockets.
// pub struct Stream { 
//     inner: Socket,
//     timeout: Duration
// }
// impl Stream {
//     pub fn new(stream: Socket, timeout: Duration) -> Self {
//         Stream {
//             inner: stream, 
//             timeout: timeout
//         }
//     }
//     ///Get 1 `Frame` from the `Stream`.
//     ///
//     ///NOTE: This function is a blocking function. It will wait until it receives a frame.
//     pub fn get_frame(&mut self) -> Frame {
//         let control_bytes: Vec<u8> = self.read_n(2).expect("Could not get stream control content");
//         let control_bits1 = into_bits(control_bytes[0]);
//         let fin = control_bits1[0];
//         let rsv1 = control_bits1[1];
//         let rsv2 = control_bits1[2];
//         let rsv3 = control_bits1[3];
//         let opcode = Opcode::from(from_bits(&control_bits1[4..8]));
//         let control_bits2 = into_bits(control_bytes[1]);
//         let is_masked = control_bits2[0]; 
//         let len = from_bits(&control_bits2[1..8]);
//         let real_len: u64;
//         let len_len: u8 = match len {
//             0..126 => 0,
//             126 => 2,
//             127 => 8,
//             _ => unreachable!("len > 127")
//         };
//         if len_len == 0 {
//             real_len = len as u64; 
//         } else {
//             let len_bytes: Vec<u8> = self.read_n(len_len as u64).expect("Could not get stream control content");
//             let mut buf: u64 = 0;
//             for i in 0..len_len {
//                 let shift = (len_len - i - 1) * 8;
//                 buf += (len_bytes[i as usize] as u64) << shift as u64;
//             }
//             real_len = buf;
//         }
//         let mask: Option<Vec<u8>> = match is_masked {
//             false => None,
//             true => {
//                 let res = self.read_n(4).expect("Could not get stream control content");
//                 Some(res)
//             },
//         };
//         let mut content = self.read_n(real_len).expect("Could not get stream control content");
//         if let Some(key) = &mask { 
//             for i in 0..content.len() {
//                 content[i] = content[i] ^ key[i % 4];
//             }
//         }
//         Frame::new(fin, rsv1, rsv2, rsv3, opcode, mask, content)
//     }
//     pub fn try_get_frame(&mut self) -> Option<Frame> {
//         let control_bytes: Vec<u8> = self.try_read_n(2)?;
//         let control_bits1 = into_bits(control_bytes[0]);
//         let fin = control_bits1[0];
//         let rsv1 = control_bits1[1];
//         let rsv2 = control_bits1[2];
//         let rsv3 = control_bits1[3];
//         let opcode = Opcode::from(from_bits(&control_bits1[4..8]));
//         let control_bits2 = into_bits(control_bytes[1]);
//         let is_masked = control_bits2[0]; 
//         let len = from_bits(&control_bits2[1..8]);
//         let real_len: u64;
//         let len_len: u8 = match len {
//             0..126 => 0,
//             126 => 2,
//             127 => 8,
//             _ => unreachable!("len > 127")
//         };
//         if len_len == 0 {
//             real_len = len as u64; 
//         } else {
//             let len_bytes: Vec<u8> = self.read_n(len_len as u64).expect("Could not get stream control content");
//             let mut buf: u64 = 0;
//             for i in 0..len_len {
//                 let shift = (len_len - i - 1) * 8;
//                 buf += (len_bytes[i as usize] as u64) << shift as u64;
//             }
//             real_len = buf;
//         }
//         let mask: Option<Vec<u8>> = match is_masked {
//             false => None,
//             true => {
//                 let res = self.read_n(4).expect("Could not get stream control content");
//                 Some(res)
//             },
//         };
//         let mut content = self.read_n(real_len).expect("Could not get stream control content");
//         if let Some(key) = &mask { 
//             for i in 0..content.len() {
//                 content[i] = content[i] ^ key[i % 4];
//             }
//         }
//         Some(Frame::new(fin, rsv1, rsv2, rsv3, opcode, mask, content))
//     }
//     ///Write the `Frame` to the `Stream`.
//     pub fn write_frame(&mut self, frame: Frame) {
//         let mut byte1: u8 = 0;
//         byte1 |= from_bits(&[frame.fin(), frame.rsv1(), frame.rsv2(), frame.rsv3()]) << 4;
//         byte1 |= frame.opcode() as u8;
//         let mut byte2: u8 = 0;
//         let (len_len, len) = set_len(frame.content_len());
//         byte2 |= len_len; 
//         let mut content = frame.bytes();
//         self.write_all(&[byte1, byte2]);
//         self.write_all(&len);
//         if let Some(key) = frame.mask() { 
//             byte2 |= 1 << 7;
//             for i in 0..content.len() {
//                 content[i] = content[i] ^ key[i % 4];
//             }
//             self.write_all(&key);
//         }
//         self.write_all(&content);
//         self.flush();
//     }
//     ///Close the `Stream`.
//     pub fn close(&mut self, mut frame: Frame) {
//         frame.mask = None;
//         self.write_frame(frame);
//     }
//     ///Send a Pong frame via the `Stream`.
//     ///
//     ///In most cases this will be a reply to a Pong frame, but this function can also be used
//     ///outside of that use case.
//     pub fn pong(&mut self, content: &[u8], limit: u64) {
//         if content.len() <= limit as usize {
//             let mut byte2 = 0x00;
//             let (len_len, len) = set_len(content.len().try_into().expect("Math ain't mathing again"));
//             byte2 |= len_len;
//             self.write_all(&[0x8A, byte2]);
//             self.write_all(len.as_slice());
//             self.write_all(content);
//             self.flush();
//         }
//     }
//     ///Send a Ping frame via the `Stream`.
//     pub fn ping(&mut self, content: &[u8]) {
//         let (len_len, len) = set_len(content.len().try_into().expect("Math ain't mathing again"));
//         let byte1: u8 = 0b10001001; //fin: true, opcode: 9
//         let mut byte2: u8 = 0b00000000;
//         byte2 |= len_len;
//         self.write_all(&[byte1, byte2]);
//         self.write_all(len.as_slice());
//         self.write_all(content);
//         self.flush();
//         i!("Sent Ping: {:?}", content);
//         let frame = self.get_frame();
//         if let Pong = frame.opcode() {
//             i!("Received Pong: {:?}", frame.bytes());
//         } else {
//             i!("Received other frame: {}", frame);
//         }
//     }
//     ///Get a mutable reference of the inner `Socket`.
//     pub fn inner(&mut self) -> &mut Socket {
//         &mut self.inner
//     }
//     ///Consume the `Stream` and turn it into a `Stream`.
//     pub fn into_socket(self) -> Socket {
//         self.inner
//     }
//     ///Read an N amount of bytes from the `Stream`.
//     ///
//     ///NOTE: This function will block until it receives a frame.
//     pub fn read_n(&mut self, bytes: u64) -> Option<Vec<u8>> {
//         if bytes == 0 {
//             return Some(Vec::<u8>::new());
//         }
//         let mut buf: Vec<u8> = Vec::with_capacity(bytes as usize);
//         let read = self
//             .take(bytes as u64)
//             .read_to_end(&mut buf);
//         match read {
//             Ok(0) => None,
//             Ok(n) => Some(buf),
//             Err(err) => None
//         }
//     }
//     pub fn try_read_n(&mut self, bytes: u64) -> Option<Vec<u8>> {
//         if bytes == 0 {
//             return Some(Vec::<u8>::new());
//         }
//         let mut buf: Vec<u8> = Vec::with_capacity(bytes as usize);
//         let read = self
//             .take(bytes as u64)
//             .read_to_end(&mut buf);
//         match read {
//             Ok(0) => None,
//             Ok(n) => Some(buf),
//             Err(err) => None
//         }
//     }
// }
// impl Write for Stream {
//     fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
//         self.inner().write(buf)
//     }
//     fn flush(&mut self) -> std::io::Result<()> {
//         self.inner().flush()
//     }
// }
// impl Read for Stream {
//     fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
//         self.inner().read(buf)
//     }
// }
// ///Turns a `u8` byte into 8 `bool` bits.
// fn into_bits(byte: u8) -> [bool; 8] {
//     let mut bits: [bool; 8] = [false; 8];
//     for i in 0..8 {
//         let check: u8 = 0b10000000 >> i;
//         let bit: u8 = byte & check;
//         bits[i] = match bit {
//             0 => false,
//             _ => true
//         };
//     }
//     bits
// }
// ///Turns an slice of `bool` bits into a `u8` byte.
// fn from_bits(bits: &[bool]) -> u8 {
//     let mut byte: u8 = 0;
//     for i in 0..8 {
//         if let Some(true) = bits.get(i) {
//             byte |= 0b10000000 >> 8 - bits.len() + i;
//         }
//     }
//     byte
// }
// ///Set the length(s) in the `Frame`.
// fn set_len(size: u64) -> (u8, Vec<u8>) {
//     let (len_len, len): (u8, Vec<u8>) = match size {
//         0..126 => (size.try_into().expect("Math ain't mathing"), Vec::new()),
//         126..=65535 => (126, size.to_be_bytes().to_vec()),
//         _ => (127, size.to_be_bytes().to_vec())
//     };
//     (len_len, len)
// }

fn main() {
    //TODO: print options
    //TODO: display live count of clocked in people via stats endpoint and via the cli
    let (ip, port) = args();
    println!("Press CONTROL-C to exit\nLegend:\n  \x1b[32m|\x1b[0m - Info\n  \x1b[101m \x1b[0m - Error\n  \x1b[103m \x1b[0m - Warning");
    thread::sleep(Duration::from_secs(1));
    //TODO: Decide on type here
    //  Form data (Action)
    //  Actions:
    //      Add(Name, ClassNum, ClassLetter) -> Id
    //      Rename(Id, NewName) -> IsSuccess
    let (tx_http, rx_pg) = channel::<UserAction>();
    //let (tx_pg, rx_http) = channel::<>();
    let thread_pool = ThreadPool::new(10, Some(10)).unwrap();
    thread::scope(|s| {
        //HTTP server
        s.spawn(move || {
            let _ = run_server(ip, port, thread_pool).map_err(|err| {
                e!("Cannot start server: {}", err);
                exit(2);
            });
        });
        //Command line control
        s.spawn(move || {
            let getch = Getch::new();
            loop {
                let key = getch.getch();
                match key {
                    Ok(Ctrl('c')) => {
                        w!("Gracefully stopping"); 
                        break;
                    }
                    Ok(_) => (),
                    Err(err) => e!("E: Getch - {err}")
                }
            }
        });
        //Postgres
        s.spawn(move || {
            let mut config = Config::new();
            config
               .user("sasha")
               .password("sasha")
               .dbname("sasha")
               .hostaddr(V4(Ipv4Addr::new(127, 0, 0, 1)))
               .port(6970);
            let mut client = config.connect(NoTls).unwrap();
            i!("Connected to DB");
            let date = Local::now();
            let date_fmt: String = format!("{}", date.date_naive())
                .split('-')
                .map(|x| format!("_{}", x))
                .collect();
            let create_table_res = client.execute(&format!("CREATE TABLE {} (name VARCHAR(50) NOT NULL, classnum SMALLINT CHECK (classnum >= 7 AND classnum <= 12), classletter CHAR CHECK (classletter IN ('A', 'B', 'C', 'D')), id SMALLINT GENERATED ALWAYS AS IDENTITY);", &date_fmt), &[]);
            if let Ok(_) = create_table_res {
                i!("Created table {}", &date_fmt);
            }
            let new_person = client.prepare_typed(&format!("INSERT INTO {} VALUES ($1, $2, $3)", &date_fmt), &[Type::VARCHAR, Type::INT2, Type::CHAR]).unwrap();
            for query in rx_pg.iter() {
                println!("pog");
            }
            add_record(&mut client, new_person, "jogger", 12, 'C');
            //TODO: SQL injections
        });
    });
}
fn args() -> (IpAddr, u16) {
    let args: Vec<String> = std::env::args().collect();
    let ip: Option<IpAddr>;
    let port: Option<u16>;
    let res = match args.len() {
        2 => {
            port = args[1].parse().ok(); 
            ip = local_ip().ok();
            (ip, port)
        }
        3 => {
            port = args[2].parse().ok(); 
            ip = IpAddr::from_str(&args[1]).ok();
            (ip, port)
        }
        _ => panic!("USAGE:\n\tcargo run [IP Address] <Port>\n")
    };
    if let None = &res.0 {
        e!("Invalid port, please enter a u16 (0 to 65535)"); 
        exit(1);
    }
    if let None = &res.1 {
        e!("Invalid IP address, please following this format: {u8}.{u8}.{u8}.{u8}"); 
        exit(1);
    }
    (res.0.unwrap(), res.1.unwrap())
}

///Start a server with the specifications in the args.
///
///NOTE: This is a blocking function and is not meant to return.
pub fn run_server(ip: IpAddr, port: u16, pool: ThreadPool) -> Result<(), ServerError> {
    let address = format!("{}:{}", &ip, &port);
    let server = Server::http(address)?;
    //let (tx_response, rx_response) = channel::<(Request, Answer, bool)>();
    thread::scope(move |s| {
        s.spawn(move || {
            i!("Started server: {}:{}", &ip, &port);
            for request in server.incoming_requests() {
                i!("{}: {}", request.method(), request.url());
                let job = Box::new(move || process_request(request));
                pool.execute(job);
            }
        });
        /*s.spawn(move || {
            for (request, response, is_ws) in rx_response.into_iter() {
            } 
        });*/
    });
    Ok(())
}
// ///Listen for WebSocket frames, handle them and respond to them accordingly.
// pub fn handle_ws(stream: Stream, rec: mpmc::Receiver<GameAction>, sender: mpmc::Sender<PlayerAction>) {
//     //TODO: Message system
//     //When a write needs to be made, the write thread sends a message to the other thread to unlock
//     //the stream, then waits for it to be unlocked
//     thread::scope(|s| {
//         let stream1 = Arc::new(Mutex::new(stream));
//         let stream2 = stream1.clone();
//         s.spawn(move || {
//             loop {
//                 thread::sleep(Duration::from_secs(1));
//                 let msg = rec.recv();
//                 //i!("Message: {}", message);
//                 let mut stream_locked = stream1.lock().expect("Could not lock onto stream (write)");
//                 let res = stream_locked.write(&[0b10000001, 3, 97, 98, 99]);
//                 stream_locked.flush();
//                 if let Err(_) = res {
//                     e!("Disconnected (write)");
//                     break;
//                 }
//             }
//         });
//         s.spawn(move || {
//             loop {
//                 let mut stream_locked = stream2.lock().expect("Could not get access to stream (read)");
//                 let frame_res = stream_locked.try_get_frame(); 
//                 if let None = &frame_res {
//                     thread::sleep(Duration::from_millis(100));
//                     continue;
//                 }
//                 let frame = frame_res.unwrap();
//                 let content = frame.bytes();
//                 match frame.opcode() {
//                     Fragment => println!("fragment"),
//                     Text => {
//                         let text = frame.text().expect("Those bastards lied about UTF-8");
//                         i!("Text: {}", text);
//                     }
//                     Binary => println!("data"),
//                     Close => {
//                         i!("Closing");
//                         let mut stream_locked = stream2.lock().expect("Could not write to stream (write, close)");
//                         stream_locked.close(frame);
//                         break;
//                     },
//                     Ping => {
//                         i!("Received Ping; responding");
//                         let mut stream_locked = stream2.lock().expect("Could not write to stream (write, pong)");
//                         stream_locked.pong(&content, 1024);
//                     },
//                     Pong => i!("Received Pong: {:?}", content),
//                     Unknown => e!("Unknown")
//                 }
//                 let buf: Vec<u8> = Vec::new();
//                 thread::sleep(Duration::from_secs(1));
//             }
//         });
//     });
// }
///Handle the request based on the method and endpoint.
///
///The request itself, the response and if the request is a WebSocket request will be sent
///using the sender.
pub fn process_request(request: Request) {
    match (request.method(), request.url()) {
        (Get, "/") => {
            respond_with_file(request, "form.html")
        },
        (Get, "/htmx") => {
            respond_with_file(request, "htmx")
        },
        (Get, "/script.js") => {
            respond_with_file(request, "script.js")
        },
        (Get, "/style.css") => {
            respond_with_file(request, "style.css")
        },
        (Post, "/form") => {
            request.respond(Response::from_string("abcd"));
        },
        _ => {
            w!("Invalid request");
            let _ = request.respond(Response::empty(StatusCode(404)));
        }
    };
}
fn respond_with_file(request: Request, path: impl AsRef<Path> + Display) {
    let res = OpenOptions::new()
        .read(true)
        .open(&path);
    if let Ok(mut file) = res {
        request.respond(Response::from_file(file));
    } else {
        w!("File {} not found", path);
        request.respond(Response::empty(StatusCode(404)));
    }
}
fn add_record(client: &mut Client, statement: Statement, one: &str, two: u8, three: char) {
    client.execute(&statement, &[&one, &(two as i16), &(three as i8)]).unwrap();
}
///Print an error message
///
///It puts a red space before the content.
#[macro_export]
macro_rules! e {
    ($content:literal) => {
        eprintln!("\x1b[101m \x1b[0m {}", $content)
    };
    {$content:literal, $($args:tt)+} => {
        eprintln!("{}", format!("\x1b[101m \x1b[0m {}", format_args!($content, $($args)+)))
    };
}
///Print an info message
///
///It puts a green pipe before the content.
#[macro_export]
macro_rules! i {
    ($content:literal) => {
        println!("\x1b[32m|\x1b[0m {}", $content)
    };
    { $content:literal, $($args:tt)+} => {
        println!("{}", format!("\x1b[32m|\x1b[0m {}", format!($content, $($args)+)))
    };
}
///Print a warning message
///
///It puts a yellow space before the content.
#[macro_export]
macro_rules! w {
    ($content:literal) => {
        eprintln!("\x1b[103m \x1b[0m {}", $content)
    };
    { $content:literal, $($args:tt)+} => {
        eprintln!("{}", format!("\x1b[103m \x1b[0m {}", format!($content, $($args)+)))
    };
}
