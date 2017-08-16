extern crate futures;
extern crate tokio_core as core;
extern crate tokio_proto as proto;
extern crate tokio_service as service;
extern crate tokio_io;
extern crate bytes;

use std::{io, str};
use std::net::SocketAddr;
use futures::future::FutureResult;
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_io::codec::{Framed, Decoder, Encoder};
use proto::TcpServer;
use proto::pipeline::{ClientProto, ServerProto};
use service::Service;
use bytes::{BytesMut, BufMut};

pub struct LineCodec;

impl Decoder for LineCodec {
    type Item = String;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, io::Error> {
        // If our buffer contains a newline...
        if let Some(n) = buf.as_ref().iter().position(|b| *b == b'\n') {
            // remove this line and the newline from the buffer.
            let line = buf.split_to(n);
            buf.split_to(1); // Also remove the '\n'.

            // Turn this data into a UTF-8 string and return it
            return match str::from_utf8(line.as_ref()) {
                Ok(s) => Ok(Some(s.to_string())),
                Err(_) => Err(io::Error::new(io::ErrorKind::Other, "invalid string")),
            }
        }

        Ok(None)
    }
}

impl Encoder for LineCodec {
    type Item = String;
    type Error = io::Error;

    fn encode(&mut self, msg: Self::Item, buf: &mut BytesMut) -> io::Result<()> {
        for byte in msg.as_bytes() {
            buf.put_u8(*byte);
        }

        buf.put_u8(b'\n');
        Ok(())
    }
}

struct LineServerProto;

impl<T: AsyncRead + AsyncWrite + 'static> ServerProto<T> for LineServerProto {
    type Request = String;
    type Response = String;
    type Transport = Framed<T, LineCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(LineCodec))
    }
}

pub struct LineClientProto;

impl<T: AsyncRead + AsyncWrite + 'static> ClientProto<T> for LineClientProto {
    type Request = String;
    type Response = String;
    type Transport = Framed<T, LineCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(LineCodec))
    }
}

struct HelloWorldService;

impl Service for HelloWorldService {
    type Request = String;
    type Response = String;
    type Error = io::Error;
    type Future = FutureResult<Self::Response, Self::Error>;

    fn call(&self, req: String) -> Self::Future {
        if req.contains('\n') {
            futures::failed(io::Error::new(io::ErrorKind::InvalidInput, "message contained new line"))
        } else {
            let resp = match req.as_str() {
                "hello" => "world".into(),
                _ => "idk".into(),
            };
            futures::finished(resp)
        }
    }
}

pub struct Server;

impl Server {
    pub fn serve(addr: SocketAddr) {
        TcpServer::new(LineServerProto, addr)
            .serve(|| Ok(HelloWorldService));
    }
}
