extern crate futures;
extern crate tokio_core as core;
extern crate tokio_proto as proto;
extern crate tokio_service as service;
extern crate tokio_example as example;

use futures::Future;
use proto::TcpClient;
use service::Service;

use example::LineClientProto;

fn main() {
    let mut event_loop = core::reactor::Core::new().unwrap();
    let handle = event_loop.handle();

    let addr = "0.0.0.0:12345".parse().unwrap();
    let test = TcpClient::new(LineClientProto)
        .connect(&addr, &handle.clone())
        .and_then(|client| {
            let req = "hello".into();
            println!("req: {:?}", req);
            client.call(req)
        })
        .map(|res| println!("res: {:?}", res));

    event_loop.run(test).unwrap();
}
