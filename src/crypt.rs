use std::{
    sync::{Arc, Mutex},
    task::Poll,
};

use smol::io::{AsyncRead, AsyncWrite};

#[derive(Clone)]
pub struct XorCrypt<T> {
    key: u8,
    inner: Arc<Mutex<T>>,
}

pub trait CryptSplit<T> {
    fn split(self, key: u8) -> (XorCrypt<T>, XorCrypt<T>);
}

impl<T: AsyncRead + AsyncWrite + Send + Sync + 'static> XorCrypt<T> {
    pub fn new(io: T, key: u8) -> Self {
        Self {
            key,
            inner: Arc::new(Mutex::new(io)),
        }
    }
}

impl<T: AsyncRead + Unpin + Send + Sync + 'static> AsyncRead for XorCrypt<T> {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        let mut io = self.inner.lock().unwrap();
        match std::pin::Pin::new(&mut *io).poll_read(cx, buf) {
            std::task::Poll::Pending => std::task::Poll::Pending,
            std::task::Poll::Ready(result) => {
                if let Ok(n) = result {
                    for idx in 0..n {
                        buf[idx] ^= self.key;
                    }
                }
                Poll::Ready(result)
            }
        }
    }
}

impl<T: AsyncWrite + Unpin + Send + Sync + 'static> AsyncWrite for XorCrypt<T> {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        let buf: Vec<u8> = if cfg!(windows) {
            if buf.ends_with("\r\n".as_bytes()) {
                let mut buf = buf.to_vec();
                buf.truncate(buf.len() - 2);
                buf.extend("\n".as_bytes());
                buf.to_vec()
            } else {
                buf.to_vec()
            }
        } else {
            buf.to_vec()
        }
        .iter()
        .map(|ele| ele ^ self.key)
        .collect();

        let mut io = self.inner.lock().unwrap();
        std::pin::Pin::new(&mut *io).poll_write(cx, &buf)
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let mut io = self.inner.lock().unwrap();
        std::pin::Pin::new(&mut *io).poll_flush(cx)
    }

    fn poll_close(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let mut io = self.inner.lock().unwrap();
        std::pin::Pin::new(&mut *io).poll_close(cx)
    }
}

impl<T: AsyncWrite + Clone + AsyncRead + Send + Sync + 'static> CryptSplit<T> for T {
    fn split(self, key: u8) -> (XorCrypt<T>, XorCrypt<T>) {
        let crypt = XorCrypt::new(self, key);
        (crypt.clone(), crypt)
    }
}
