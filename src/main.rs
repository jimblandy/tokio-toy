extern crate bytes;
extern crate futures;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_proto;
extern crate tokio_service;

use bytes::BytesMut;
use futures::future::{Future, ok};
use tokio_core::net::TcpStream;
use tokio_io::{AsyncRead};
use tokio_io::codec::{Decoder, Encoder, Framed};
use tokio_proto::TcpServer;
use tokio_proto::pipeline::ServerProto;
use tokio_service::Service;

use std::io;

struct Utf8Lines;

impl Decoder for Utf8Lines {
    type Item = String;
    type Error = io::Error;
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<String>, io::Error> {
        if let Some(i) = src.iter().position(|b| *b == b'\n') {
            let line = src.split_to(i + 1);
            std::str::from_utf8(&line)
                .map(|s| Some(s[..s.len()-1].to_string()))
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
        } else {
            Ok(None)
        }
    }

    fn decode_eof(&mut self, buf: &mut BytesMut) -> Result<Option<String>, io::Error> {
        if let Some(s) = self.decode(buf)? {
            return Ok(Some(s));
        }

        if buf.is_empty() {
            return Ok(None);
        }

        std::str::from_utf8(buf)
            .map(|s| Some(s.to_string()))
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

impl Encoder for Utf8Lines {
    type Item = String;
    type Error = io::Error;
    fn encode(&mut self, item: String, dst: &mut BytesMut) -> Result<(), io::Error> {
        dst.extend(item.as_bytes());
        dst.extend(b"\n");
        Ok(())
    }
}

struct Utf8LinesProto;

impl ServerProto<TcpStream> for Utf8LinesProto
{
    type Request = String;
    type Response = String;
    type Transport = Framed<TcpStream, Utf8Lines>;
    type BindTransport = Result<Self::Transport, io::Error>;
    fn bind_transport(&self, io: TcpStream) -> Result<Self::Transport, io::Error> {
        Ok(io.framed(Utf8Lines))
    }
}

#[derive(Default)]
struct Reverser;

impl Service for Reverser {
    type Request = String;
    type Response = String;
    type Error = io::Error;
    type Future = Box<Future<Item=Self::Response, Error=io::Error>>;
    fn call(&self, text: String) -> Self::Future {
        Box::new(ok(text.chars().rev().collect()))
    }
}

fn main() {
    let addr = "0.0.0.0:12345".parse()
        .expect("parsing server IP address");
    let server = TcpServer::new(Utf8LinesProto, addr);

    server.serve(move || { Ok(Reverser) });
}
