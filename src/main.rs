#[macro_use] extern crate rocket;
use std::net::ToSocketAddrs;
use sftp_http::http;

#[launch]
fn start() -> _ {
    http::rocket()
}