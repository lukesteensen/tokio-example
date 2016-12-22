extern crate futures;
extern crate tokio_core as core;
extern crate tokio_proto as proto;
extern crate tokio_service as service;

use std::{io, str};
use std::net::SocketAddr;
use futures::BoxFuture;
use core::io::{Codec, EasyBuf, Framed, Io};
use proto::TcpServer;
use proto::pipeline::{ClientProto, ServerProto};
use service::Service;

pub struct LineCodec;

impl Codec for LineCodec {
    type In = String;
    type Out = String;

    fn decode(&mut self, buf: &mut EasyBuf) -> Result<Option<Self::In>, io::Error> {
        // If our buffer contains a newline...
        if let Some(n) = buf.as_ref().iter().position(|b| *b == b'\n') {
            // remove this line and the newline from the buffer.
            let line = buf.drain_to(n);
            buf.drain_to(1); // Also remove the '\n'.

            // Turn this data into a UTF string and return it in a Frame.
            return match str::from_utf8(line.as_ref()) {
                Ok(s) => Ok(Some(s.to_string())),
                Err(_) => Err(io::Error::new(io::ErrorKind::Other, "invalid string")),
            }
        }

        Ok(None)
    }

    fn encode(&mut self, msg: Self::Out, buf: &mut Vec<u8>) -> io::Result<()> {
        for byte in msg.as_bytes() {
            buf.push(*byte);
        }

        buf.push(b'\n');
        Ok(())
    }
}

struct LineServerProto;

impl<T: Io + 'static> ServerProto<T> for LineServerProto {
    type Request = String;
    type Response = String;
    type Error = io::Error;
    type Transport = Framed<T, LineCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(LineCodec))
    }
}

pub struct LineClientProto;

impl<T: Io + 'static> ClientProto<T> for LineClientProto {
    type Request = String;
    type Response = String;
    type Error = io::Error;
    type Transport = Framed<T, LineCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(LineCodec))
    }
}

struct WorldService;

impl Service for WorldService {
    type Request = String;
    type Response = String;
    type Error = io::Error;
    type Future = BoxFuture<String, io::Error>;

    fn call(&self, req: String) -> Self::Future {
        if req.chars().find(|&c| c == '\n').is_some() {
            Box::new(futures::failed(io::Error::new(io::ErrorKind::InvalidInput, "message contained new line")))
        } else {
            let resp = match req.as_str() {
                "hello" => "world".into(),
                _ => "idk".into(),
            };
            Box::new(futures::finished(resp))
        }
    }
}

pub struct Server;

impl Server {
    pub fn serve(addr: SocketAddr) {
        TcpServer::new(LineServerProto, addr)
            .serve(|| Ok(WorldService));
    }
}
