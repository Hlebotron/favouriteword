const ROOT_DIR: &str = ".";

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
use std::sync::mpsc::channel;
use rand::{thread_rng, Rng};

fn main() {
    println!("TODO: Wrong page at asking (when reloading)");
    if ROOT_DIR == "" {
        println!("Please specify the root directory in 'main.rs' (ROOT_DIR)");
        exit(1) }
    let args: Vec<String> = args().collect();
    let success = match args.len() {
        3 => {
            let port2 = (&args[2].parse::<u16>().expect("Not a number") + 1).to_string();
            let port3 = (&args[2].parse::<u16>().expect("Not a number") + 2).to_string();
            let port4 = (&args[2].parse::<u16>().expect("Not a number") + 3).to_string();
            start_server(&args[1], &args[2], &port2, &port3, &port4)
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
    cargo run <ip address> <server port>

NOTE: The server, WebSocket and command server each take up 1 port (eg. if you set the server port to 6969, 6970 and 6971 will also be taken up), so distance each instance of this server by at least 3 ports
"#);
}

struct EventHandler {
    ws: Sender,
    id: u16,
}
impl Handler for EventHandler {
    fn on_open(&mut self, shake: Handshake) -> Result<(), ws::Error> {
        println!("{info}: Connection has been made, ID: {}", self.id, info = "INFO".green().bold());
        Ok(())
    } 
    fn on_message(&mut self, msg: Message) -> Result<(), ws::Error> {
        println!("{info}: Message received: {msg}", info = "INFO".bold().green());
        self.ws.send("response:pog");
        Ok(())
    }
    fn on_close(&mut self, code: CloseCode, reason: &str) {
        println!("{warn}: Connection closed: CODE {code:?} - {reason}", warn = "WARN".yellow().bold());
    }
}
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

fn start_server(address: &str, port1: &str, port2: &str, port3: &str, port4: &str) -> Result<(), ()> {
    delete_file_content(&file("data"));
    delete_file_content(&file("answerData"));
    delete_file_content(&file("events"));
    let ip_addr = local_ip().unwrap_or_else(|err| {
        eprintln!("{error}: Could not get local IP address: {}", err, error = "ERROR".red().bold());
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
    }).to_string();
    run_server(address, port1, port2, port3, port4, ROOT_DIR).or_else(|_|{
        run_server(&ip_addr, port1, port2, port3, port4, ROOT_DIR)
    }).or_else(|err| {
        eprintln!("{fatal_error}: Could not start server", fatal_error = "FATAL ERROR".red().bold());
        Err(())
    });
    Ok(())
}
fn run_server(address: &str, port1: &str, port2: &str, port3: &str, port4: &str, directory: &str) -> Result<(), ()> {
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
    let (tx, rx) = channel::<(String, bool)>();
    let (ctx, crx) = channel::<String>();
    thread::scope(|s| {
        s.spawn(move || { //Client Thread
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
                    (Method::Get, "/style.css") => {
                        serve(&file("/style.css"), request);
                    },
                    (Method::Get, "/favicon.ico") | (Method::Get, "/apple-touch-icon.png") => {
                        serve(&file("favicon.ico"), request);
                    },
                    (Method::Post, "/addData") => {
                        let content = read_request_content(&mut request);
                        post(&content, &file("data"));
                    },
                    (Method::Get, "/asking") => {
                        serve(&file("asking.html"), request);
                    },
                    (Method::Get, "/asking.js") => {
                        serve(&file("asking.js"), request);
                    },
                    (Method::Get, "/events") => {
                        serve(&file("events"), request);
                    },
                    (Method::Post, "/addAnswerData") => {
                        let content = read_request_content(&mut request);
                        request.respond(Response::from_string("ok"));
                        post(&content, &file("answerData"));
                    },
                    _ => { 
                        let mut content: String = "".to_string();
                        request.as_reader().read_to_string(&mut content).map_err(|err| {
                            eprintln!("{error}: Could not read request content to string: {err}", error = "ERROR".red().bold());
                            "Unknown".to_string()
                        });
                        eprintln!("{error}: Invalid request {}, content: {}",  request.url(), content, error = "ERROR".red().bold());
                        request.respond(Response::from_string("404"));
                        ctx.send("alert:error".to_string());
                    },
                }
            }
        });
        let ws = WebSocket::new(HandlerFactory {id: 0}).unwrap();
        let broadcast_ws = ws.broadcaster();
        let cws = WebSocket::new(HandlerFactory {id: 0}).unwrap();
        let cbroadcast_ws = cws.broadcaster();
        s.spawn(move || { //Listener Thread
            let socket_address: String = format!("{address}:{port2}");
            let listener = ws.listen(socket_address);
            println!("Not listening anymore");
        });
        s.spawn(move || { //Control Listener Thread
            let socket_address: String = format!("{address}:{port4}");
            let listener = cws.listen(socket_address);
            println!("Not listening anymore");
        });
        s.spawn(move || { //Broadcast Thread
            loop {
                sleep(Duration::from_millis(100));
                if let Ok((message, send)) = rx.try_recv() {
                    broadcast_ws.send(&*message);
                    if send {
                        post(&message, &file("events"));
                    }
                }
                if let Ok(message) = crx.try_recv() {
                    cbroadcast_ws.send(&*message);
                }
            }
            println!("Stopped broadcasting");
        });
        s.spawn(move || { //Control Thread
            let mut word_revealed = true;
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
                    "/proceed" => {
                        if word_revealed {
                            let line = cut_line_from_data().unwrap_or("".to_string());
                            if line != "" {
                                tx.send((format!("word:{line}"), true));
                                delete_file_content(&file("answerData"));
                                word_revealed = false;
                            }
                        } else {
                            tx.send(("cmd:reveal".to_string(), true));
                            word_revealed = true;
                        }
                        request.respond(Response::from_string(word_revealed.to_string()));
                    },
                    "/message" => {
                        let mut content: String = "".to_string();
                        request.as_reader().read_to_string(&mut content).map_err(|err| {
                            eprintln!("{error}: Could not read request content to string: {err}", error = "ERROR".red().bold());
                            "Unknown".to_string()
                        });
                        if &content != "Unknown" { 
                            tx.send((format!("msg:{content}"), false));
                        }
                        request.respond(Response::from_string("ok"));
                    },
                    "/getAnswerData" => {
                        serve(&file("answerData"), request);
                    },
                    "/reset" => {
                        tx.send(("cmd:reset".to_string(), true));
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
fn delete_file_content(file_path: &str){
    File::options().truncate(true).open(file_path).unwrap_or_else(|err| {
        //eprintln!("{error}: Could not open file {}: {}", file_path, err, error = "ERROR".red().bold());
        File::create(file_path).unwrap()
    });
}
fn file(file_path: &str) -> String {
    let path = format!("{ROOT_DIR}/src/{file_path}");
    path
}
fn cut_line_from_data() -> Result<String, ()>{
    let mut file_reader = File::options().read(true).open(file("data")).unwrap_or_else(|err| {
        eprintln!("{error}: Could not open file {}: {} => Creating file", "data", err, error = "ERROR".red().bold());
        File::create("data").unwrap()
    });
    let mut file_writer = File::options().write(true).open(file("data")).unwrap_or_else(|err| {
        eprintln!("{error}: Could not open file {}: {} => Creating file", "data", err, error = "ERROR".red().bold());
        File::create("data").unwrap()
    });
    let mut file_content: String = Default::default();
    file_reader.read_to_string(&mut file_content);
    let mut file_lines: Vec<_> = file_content.lines().collect();
    if file_lines.len() > 0 {
        let line_number = thread_rng().gen_range(0..file_lines.len()); 
        let line = file_lines.remove(line_number);
        let mut file_string = file_lines.join("\n");
        file_string.push_str("\n");
        File::options().write(true).truncate(true).open(file("data")).unwrap_or_else(|err| {
            eprintln!("{error}: Could not open file {}: {} => Creating file", "data", err, error = "ERROR".red().bold());
            File::create("data").unwrap()
        });
        file_writer.write(file_string.as_bytes());
        Ok(line.to_string())
    } else {
        Ok("".to_string())
    }
}
///Get the content of a request
fn read_request_content(request: &mut Request) -> String {
    let mut content: String = "".to_string();
    request.as_reader().read_to_string(&mut content).map_err(|err| {
        eprintln!("{error}: Could not read request content to string: {}", err, error = "ERROR".red().bold())
    });
    content
}
