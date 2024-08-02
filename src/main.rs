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
    /*if args.len() == 4{
        let ip_addr = &args[1];
        let port1 = &args[1];
        let port2 = &args[2];
        let success = start_server(ip_addr, port1, port2);
        match success {
            Ok(()) => exit(0),
            Err(()) => exit(1),
        } 
    } else if args.len() == 3 {
        let ip_addr = 
    } else {
    }*/
    let success = match args.len() {
        3 => {
            let port2 = (&args[2].parse::<u16>().expect("Not a number") + 1).to_string();
            let port3 = (&args[2].parse::<u16>().expect("Not a number") + 2).to_string();
            start_server(&args[1], &args[2], &port2, &port3)
        },
        _ => {
            usage();
            exit(1);
        }
    };
    match success {
        Ok(()) => exit(0),
        Err(()) => exit(1),
    } 
}
fn usage() {
    println!(r#"
USAGE:
    cargo run <ip address>

NOTE: The WebSocket and Command server each take up 1 port (eg. if you set the port to 6969, 6970 and 6971 will also be taken up), so distance each instance of this server by at least 3 ports
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

pub fn start_server(address: &str, port1: &str, port2: &str, port3: &str) -> Result<(), ()> {
    let ip_addr = local_ip().unwrap_or_else(|err| {
        eprintln!("{error}: Could not get local IP address: {}", err, error = "ERROR".red().bold());
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
    }).to_string();
    run_server(address, port1, port2, port3, ROOT_DIR).or_else(|_|{
        run_server(&ip_addr, port1, port2, port3, ROOT_DIR)
    }).or_else(|err| {
        eprintln!("{fatal_error}: Could not start server", fatal_error = "FATAL ERROR".red().bold());
        Err(())
    });
    Ok(())
}
fn run_server(address: &str, port1: &str, port2: &str, port3: &str, directory: &str) -> Result<(), ()> {
    let server_address = format!("{address}:{port1}");
    let server = Server::http(&server_address).map_err(|err| {
        eprintln!("{error}: Could not start server at {}: {}", &address, err, error = "ERROR".red().bold());
    })?;
    println!("{info}: Server is up at {}", &server_address.bold().yellow(), info = "INFO".green().bold());
    
    let control_address = format!("{address}:{port3}");
    let control_server = Server::http(&control_address).map_err(|err| {
        eprintln!("{error}: Could not start server at {}: {}", &address, err, error = "ERROR".red().bold());
    })?;
    println!("{info}: Control server is up at {}", &control_address.bold().yellow(), info = "INFO".green().bold());
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
            /*let split_address: Vec<&str> = address.split(":").collect();
            let address: &str = split_address[0];
            let mut port = split_address[1].parse::<u16>().unwrap();
            port += 1;*/
            let socket_address: String = format!("{address}:{port2}");
            //let handle = thread::scope(|s| /*-> ScopedJoinHandle<_>*/ {
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
                sleep(Duration::from_millis(250));
                if let Ok(message) = rx.try_recv() {
                    broadcast_ws.send(message);
                }
            }
        });
        /*let tx_thread = s.spawn(move || {
            loop {
                let random: u64 = thread_rng().gen_range(1..=30);
                let random_string = random.to_string();
                sleep(Duration::from_secs(random)); 
                tx.send(&random_string);
            }
        });*/
        let control_thread = s.spawn(move || {
            loop {
                let mut request = control_server.recv().unwrap(); 
                println!("{comm}: Received request: {}", &request.url(), comm = "CTRL".blue().bold());
                match request.url() {
                    "/" | "/index.html" | "/command.html" => {
                        serve(&file("command.html"), request);
                    },
                    "/command.js" => {
                        serve(&file("command.js"), request);
                    },
                    "/favicon.ico" | "/apple-touch-icon.png" => {
                        serve(&file("favicon.ico"), request);
                    },
                    "/startAsking" => {
                        tx.send("cmd:startAsking");
                        request.respond(Response::from_string("ok"));
                    },
                    _ => { 
                        let mut content: String = "".to_string();
                        request.as_reader().read_to_string(&mut content).map_err(|err| {
                            eprintln!("{error}: Could not read request content to string: {err}", error = "ERROR".red().bold());
                            "Unknown".to_string()
                        });
                        eprintln!("{error}: Invalid command {}",  request.url(), error = "ERROR".red().bold());
                        request.respond(Response::from_string("404"));
                    },
                }
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
