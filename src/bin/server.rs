extern crate tokio_example as example;

use example::Server;

fn main() {
    let addr = "0.0.0.0:12345".parse().unwrap();
    Server::serve(addr);
}
