use std::{
    process::exit,
    sync::{Arc, Mutex},
};

use rshell::crypt::CryptSplit;
use smol::{future::FutureExt, io::AsyncRead, net::TcpListener, Unblock};

struct Input<T> {
    inner: Arc<Mutex<T>>,
}

impl<T: AsyncRead + Sync + Send + 'static> Input<T> {
    pub fn new(io: T) -> Self {
        Self {
            inner: Arc::new(Mutex::new(io)),
        }
    }
}

impl<T: AsyncRead + Unpin + Sync + Send + 'static> AsyncRead for Input<T> {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        let mut io = self.inner.lock().unwrap();
        std::pin::Pin::new(&mut *io).poll_read(cx, buf)
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("用法: {} port", args.get(0).unwrap());
        exit(0)
    }

    smol::block_on(async move {
        loop {
            match TcpListener::bind(format!("0.0.0.0:{}", args[1])).await {
                Ok(server) => {
                    println!("Listen {}", server.local_addr().unwrap());

                    match server.accept().await {
                        Ok((client, addr)) => {
                            println!("accept connect {}", addr);

                            let (io_read, io_write) = client.split(10);
                            let input = Input::new(Unblock::new(std::io::stdin()));

                            if let Err(e) = smol::io::copy(input, io_write)
                                .race(smol::io::copy(io_read, Unblock::new(std::io::stdout())))
                                .await
                            {
                                println!("{}", e);
                            }
                            println!("disconnect {}", addr);
                        }
                        Err(_) => {}
                    };
                }
                Err(_) => {}
            }
        }
    })
}
