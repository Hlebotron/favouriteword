const ROOT_DIR: &str = "";

use std::process::exit;
use std::env::args;
use tiny_http::{Server, Response, Method, Request, HeaderField, ReadWrite};
use std::fs::{File, OpenOptions, write};
use std::io::{Read, Write};
use local_ip_address::local_ip;
use std::net::{Ipv4Addr, IpAddr};
use colored::Colorize;
use std::thread::{self, sleep, Thread, ScopedJoinHandle, spawn};
use std::time::Duration;
use futures::{stream::Stream, executor::block_on};
use ws::{Sender, Factory, Handler, Handshake, WebSocket, listen, Message::{self, Text}, CloseCode};
use spmc;
use std::sync::mpsc::channel;
use bounded_static::ToBoundedStatic;
use rand::{thread_rng, Rng};

fn main() {
    if ROOT_DIR == "" {
        println!("Please specify the root directory in 'main.rs'");
        exit(1) }
    let args: Vec<String> = args().collect();
    if args.len() != 2{
        usage();
        exit(1);
    } else {
        let ip_addr = &args[1];
        let success = start_server(ip_addr);
        match success {
            Ok(()) => exit(0),
            Err(()) => exit(1),
        } 
    }
}
fn usage() {
    println!(r#"
USAGE:
    cargo run <ip address>
"#);
}

struct EventHandler {
    ws: Sender,
    id: u16,
}
impl Handler for EventHandler {
    fn on_open(&mut self, shake: Handshake) -> Result<(), ws::Error> {
        println!("{info}: Connection has been made, ID: {}", self.id, info = "INFO".green().bold());
        self.ws.send(format!("Welcome, Client {}", self.id));
        Ok(())
    } 
    fn on_message(&mut self, msg: Message) -> Result<(), ws::Error> {
        println!("{info}: Message received: {msg}", info = "INFO".bold().green());
        self.ws.send("pog");
        Ok(())
    }
    fn on_close(&mut self, code: CloseCode, reason: &str) {
        println!("{warn}: Connection closed: CODE {code:?} - {reason}", warn = "WARN".yellow().bold());
    }
}
/*impl EventHandler {
    fn broadcast(&mut self) {
        self.ws.send("broadcast");
    }
}*/
struct HandlerFactory {
    id: u16,
}
impl Factory for HandlerFactory {
    type Handler = EventHandler;
    fn connection_made(&mut self, ws: Sender) -> EventHandler {
        let handler = EventHandler {
            ws: ws,
            id: self.id
        };
        self.id += 1;
        handler
    }
}
struct Thread_List<'a> {
    list: Vec<ScopedJoinHandle<'a, ()>>,
}

pub fn start_server(ip_input: &str) -> Result<(), ()> {
    let ip_addr = local_ip().unwrap_or_else(|err| {
        eprintln!("{error}: Could not get local IP address: {}", err, error = "ERROR".red().bold());
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
    });
    let mut ip_address: String = ip_addr.to_string();
    ip_address.push_str(":6969");
    run_server(ip_input, ROOT_DIR).or_else(|_|{
        run_server(&ip_address, ROOT_DIR)
    }).or_else(|err| {
        eprintln!("{fatal_error}: Could not start server", fatal_error = "FATAL ERROR".red().bold());
        Err(())
    });
    Ok(())
}
fn run_server(address: &str, directory: &str) -> Result<(), ()> {
    let server = Server::http(address).map_err(|err| {
        eprintln!("{error}: Could not start server at {}: {}", &address, err, error = "ERROR".red().bold());
    })?;
    println!("{info}: Server is up at {}", &address.bold().yellow(), info = "INFO".green().bold());
    //let mut thread_list = Thread_List {list: Vec::new()};
    //let baddress: <&str as ToBoundedStatic>::Static = address.to_static();
    //println!("{}", baddress);
    let (tx, rx) = channel();
    thread::scope(|s| {
        let request_thread = s.spawn(move || {
            loop {
                let mut request = server.recv().unwrap(); 
                println!("{info}: Received request: {} {}", &request.method(), &request.url(), info = "INFO".green().bold());
                match (request.method(), request.url()) {
                    (Method::Get, "/") | (Method::Get, "/index.html") => {
                        serve(&file("index.html"), request);
                    },
                    (Method::Get, "/index.js") => {
                        serve(&file("index.js"), request);
                        start_stream(address/*, &mut thread_list*/);
                    },
                    (Method::Get, "/favicon.ico") | (Method::Get, "/apple-touch-icon.png") => {
                        serve(&file("favicon.ico"), request);
                    },
                    (Method::Post, "/adddata") => {
                        let mut content: String = "".to_string();
                        request.as_reader().read_to_string(&mut content).map_err(|err| {
                            eprintln!("{error}: Could not read request content to string: {}", err, error = "ERROR".red().bold())
                        });
                        post(&content, &format!("{ROOT_DIR}/src/data"));
                    },
                    (Method::Get, "/waiting") => {
                        serve(&file("waiting.html"), request);
                    },
                    (Method::Get, "/waiting.js") => {
                        serve(&file("waiting.js"), request);
                    },
                    _ => { 
                        let mut content: String = "".to_string();
                        request.as_reader().read_to_string(&mut content).map_err(|err| {
                            eprintln!("{error}: Could not read request content to string: {err}", error = "ERROR".red().bold());
                            "Unknown".to_string()
                        });
                        eprintln!("{error}: Invalid request {}, content: {}",  request.url(), content, error = "ERROR".red().bold());
                        request.respond(Response::from_string("404"));
                    },
                }
            }
        });
        let ws = WebSocket::new(HandlerFactory {id: 0}).unwrap();
        //println!("{}", listen_ws);
        let broadcast_ws = ws.broadcaster();
        let mut index = 0;
        let listen_thread = s.spawn(move || {
            let split_address: Vec<&str> = address.split(":").collect();
            let address: &str = split_address[0];
            let mut port = split_address[1].parse::<u16>().unwrap();
            port += 1;
            let socket_address: String = format!("{address}:{port}");
            //let handle = thread::scope(|s| /*-> ScopedJoinHandle<_>*/ {
            println!("{}", address);
            let listener = ws.listen(socket_address);
            //println!("{:?}", websocket.unwrap());
            /*listen(&socket_address, |socket| {
                println!("{info}: A connection has been made", info = "INFO".green().bold());
                move |msg| -> Result<(), ws::Error> {
                    if msg == Text("Connection closing".to_string()) {
                        println!("{info}: Connection closed", info = "INFO".green().bold());
                    } else {
                        println!("{info}: A message was received: {msg}", info = "INFO".green().bold());
                        socket.send("Hello from Server")?;
                    }
                    Ok(())
                }
            });*/
            //broadcaster.send("pog");
            println!("Not listening anymore");
            //});
        });
        let broadcast_thread = s.spawn(move || {
            //let address = address; 
            loop {
                sleep(Duration::from_secs(1));
                if let Ok(message) = rx.try_recv() {
                    broadcast_ws.send(format!("Interval: {message} seconds"));
                }
            }
        });
        let tx_thread = s.spawn(move || {
            loop {
                let random: u64 = thread_rng().gen_range(1..=30);
                sleep(Duration::from_secs(random)); 
                tx.send(random);
            }
        });
    });
    Ok(())
}

fn serve(path: &str, request: Request) -> Result<(), ()> {
    let file = File::open(path).map_err(|err| {
        eprintln!("{error}: Could not open file: {}", err, error = "ERROR".red().bold(),);
    });
    request.respond(Response::from_file(file?)).map_err(|err| {
        eprintln!("{error}: Could not respond to client: {}", err, error = "ERROR".red().bold());
    });
    println!("{info}: File served: {}", path, info = "INFO".green().bold());
    Ok(())
}

fn post(content: &str, file_path: &str) -> Result<(), ()> {
    let mut file = File::options().read(true).write(true).open(file_path).unwrap_or_else(|err| {
        eprintln!("{error}: Could not open file {}: {} => Creating file", file_path, err, error = "ERROR".red().bold());
            File::create("data").unwrap()
    });
    let mut file_content: String = Default::default();
    file.read_to_string(&mut file_content);
    file_content.push_str(&format!("{content}\n"));
    write(file_path, file_content.as_bytes()); 
    Ok(())
}
/*fn post_header(header: &[tiny_http::Header], file_path: &str) -> Result<(), ()> {
    let mut file = File::options().read(true).write(true).open(file_path).unwrap_or_else(|err| {
        eprintln!("{error}: Could not open file {}: {} => Creating file", file_path, err, error = "ERROR".red().bold());
        File::create("data").unwrap()
    });
        let mut file_content: String = Default::default();
        file.read_to_string(&mut file_content);
        file_content.push_str(&format!("\n{header:?}"));
        write(file_path, file_content.as_bytes()); 
        Ok(())
    }*/
fn delete_file_content(file_path: &str) -> Result<(), ()>{
    let mut file = File::options().write(true).open(file_path).unwrap_or_else(|err| {
        eprintln!("{error}: Could not open file {}: {}", file_path, err, error = "ERROR".red().bold());
        File::create("data").unwrap()
    });
    write(file_path, "");
    Ok(())
}
fn file(file_path: &str) -> String {
    let path = format!("{ROOT_DIR}/src/{file_path}");
    path
}
fn start_stream<'b>(address: &str/*, thread_list: &mut Thread_List*/) {
    //let (mut tx, rx) = spmc::channel::<String>();
    //tx.send(address.to_string()).unwrap();
    //thread_list.list.push(thread);
}
