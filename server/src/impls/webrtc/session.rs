use std::{
    net::{TcpListener, TcpStream},
    pin::Pin,
    task::{Context, Poll},
};

use async_dup::Arc;
use futures_core::Stream;
use http::{header, HeaderValue, Response};
use log::info;
use once_cell::sync::OnceCell;
use smol::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines},
    prelude::*,
    Async,
};
use webrtc_unreliable::SessionEndpoint;

use naia_socket_shared::SocketConfig;

use crate::{executor, server_addrs::ServerAddrs};

static RTC_URL_PATH: OnceCell<String> = OnceCell::new();

pub fn start_session_server(
    server_addrs: ServerAddrs,
    config: SocketConfig,
    session_endpoint: SessionEndpoint,
) {
    RTC_URL_PATH
        .set(format!("POST /{}", config.rtc_endpoint_path))
        .unwrap();
    executor::spawn(async move {
        listen(server_addrs, config, session_endpoint.clone()).await;
    })
    .detach();
}

/// Listens for incoming connections and serves them.
async fn listen(
    server_addrs: ServerAddrs,
    config: SocketConfig,
    session_endpoint: SessionEndpoint,
) {
    let socket_address = server_addrs.session_listen_addr;

    let listener = Async::<TcpListener>::bind(socket_address).unwrap();
    info!(
        "Session initiator available at POST http://{}/{}",
        listener.get_ref().local_addr().unwrap(),
        config.rtc_endpoint_path
    );

    loop {
        // Accept the next connection.
        let (response_stream, _) = listener.accept().await.unwrap();

        let session_endpoint_clone = session_endpoint.clone();

        // Spawn a background task serving this connection.
        executor::spawn(async move {
            serve(session_endpoint_clone, Arc::new(response_stream)).await;
        })
        .detach();
    }
}

/// Reads a request from the client and sends it a response.
async fn serve(mut session_endpoint: SessionEndpoint, mut stream: Arc<Async<TcpStream>>) {
    let remote_addr = stream.get_ref().local_addr().unwrap();
    let mut success: bool = false;

    {
        let buf_reader = BufReader::new(stream.clone());
        let mut lines = buf_reader.lines();
        {
            if let Some(line) = lines.next().await {
                let line = line.unwrap();
                if line.starts_with(RTC_URL_PATH.get().unwrap()) {
                    while let Some(line) = lines.next().await {
                        let line = line.unwrap();
                        if line.len() == 0 {
                            success = true;
                            break;
                        }
                    }
                }
            }
        }

        if success {
            success = false;

            let buf = RequestBuffer::new(&mut lines);

            match session_endpoint.http_session_request(buf).await {
                Ok(mut resp) => {
                    success = true;

                    resp.headers_mut().insert(
                        header::ACCESS_CONTROL_ALLOW_ORIGIN,
                        HeaderValue::from_static("*"),
                    );

                    let mut out = response_header_to_vec(&resp);
                    out.extend_from_slice(resp.body().as_bytes());

                    info!("WebRTC session request from {}", remote_addr);

                    stream.write_all(&out).await.unwrap();
                }
                Err(err) => {
                    info!("error: {}", err);
                }
            }
        }
    }

    if !success {
        stream.write_all(RESPONSE_BAD).await.unwrap();
    }

    stream.flush().await.unwrap();
    stream.close().await.unwrap();
}

const RESPONSE_BAD: &[u8] = br#"
HTTP/1.1 404 NOT FOUND
Content-Type: text/html
Content-Length: 0
Access-Control-Allow-Origin: *
"#;

struct RequestBuffer<'a, R: AsyncBufRead + Unpin> {
    buffer: &'a mut Lines<R>,
    add_newline: bool,
}

impl<'a, R: AsyncBufRead + Unpin> RequestBuffer<'a, R> {
    fn new(buf: &'a mut Lines<R>) -> Self {
        RequestBuffer {
            add_newline: false,
            buffer: buf,
        }
    }
}

type ReqError = std::io::Error; //Box<dyn error::Error + Send + Sync>;

const NEWLINE_STR: &str = "\n";

impl<'a, R: AsyncBufRead + Unpin> Stream for RequestBuffer<'a, R> {
    type Item = Result<String, ReqError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.add_newline {
            self.add_newline = false;
            return Poll::Ready(Some(Ok(String::from(NEWLINE_STR))));
        } else {
            unsafe {
                loop {
                    let mut_ref = Pin::new_unchecked(&mut self.buffer);
                    match Stream::poll_next(mut_ref, cx) {
                        Poll::Ready(Some(item)) => {
                            self.add_newline = true;
                            return Poll::Ready(Some(item));
                        }
                        Poll::Ready(None) => {
                            return Poll::Ready(None);
                        }
                        Poll::Pending => {
                            // TODO: This could be catastrophic.. I don't understand futures very
                            // well!
                            return Poll::Ready(None);
                        }
                    }
                }
            }
        }
    }
}

fn response_header_to_vec<T>(r: &Response<T>) -> Vec<u8> {
    let v = Vec::with_capacity(120);
    let mut c = std::io::Cursor::new(v);
    write_response_header(r, &mut c).unwrap();
    c.into_inner()
}

fn write_response_header<T>(
    r: &Response<T>,
    mut io: impl std::io::Write,
) -> std::io::Result<usize> {
    let mut len = 0;
    macro_rules! w {
        ($x:expr) => {
            io.write_all($x)?;
            len += $x.len();
        };
    }

    let status = r.status();
    let code = status.as_str();
    let reason = status.canonical_reason().unwrap_or("Unknown");
    let headers = r.headers();

    w!(b"HTTP/1.1 ");
    w!(code.as_bytes());
    w!(b" ");
    w!(reason.as_bytes());
    w!(b"\r\n");

    for (hn, hv) in headers {
        w!(hn.as_str().as_bytes());
        w!(b": ");
        w!(hv.as_bytes());
        w!(b"\r\n");
    }

    w!(b"\r\n");
    Ok(len)
}
