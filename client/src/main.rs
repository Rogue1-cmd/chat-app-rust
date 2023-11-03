use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;

const LOCAL: &str = "127.0.0.1:6000";
const MSG_SIZE: usize = 32;

fn main() {
    // Connect to the local TCP server
    let mut client = TcpStream::connect(LOCAL).expect("Stream failed to connect");
    client.set_nonblocking(true).expect("failed to initiate non-blocking");

    // Create a channel for communication between threads
    let (tx, rx) = mpsc::channel::<String>();

    // Spawn a new thread to handle sending and receiving messages
    thread::spawn(move || loop {
        // Initialize a buffer to read incoming messages
        let mut buff = vec![0; MSG_SIZE];
        match client.read_exact(&mut buff) {
            Ok(_) => {
                // Extract the received message and print it
                let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                println!("message recv {:?}", msg);
            },
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("connection with server was severed");
                break;
            }
        }
        // Check for messages to send and send them to the server
        match rx.try_recv() {
            Ok(msg) => {
                // Convert the message into bytes and send it to the server
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);
                client.write_all(&buff).expect("writing to socket failed");
                println!("message sent {:?}", msg);
            }, 
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break
        }
        // Sleep for a short duration to avoid busy-waiting
        thread::sleep(Duration::from_millis(100));
    });
    // Main loop for reading user input and sending messages
    println!("Write a Message:");
    loop {
        let mut buff = String::new();
        io::stdin().read_line(&mut buff).expect("reading from stdin failed");
        let msg = buff.trim().to_string();
        // Check for a command to exit or send the message via the channel
        if msg == ":quit" || tx.send(msg).is_err() {break}
    }
    println!("bye bye!");

}