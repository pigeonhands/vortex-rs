mod config;
mod vortex;

use std::fs;
use std::net::{Ipv4Addr, SocketAddrV4};

use actix_web::{get, web, App, HttpServer, Responder};
use serde::Deserialize;
use serde_yaml;

use config::Config;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let str = fs::read_to_string("vortex.yaml")?;
    let decoded: Config = serde_yaml::from_str(&str).unwrap();

    let addr = SocketAddrV4::new(decoded.addr, decoded.port);

    let vx = vortex::Vortex::from_config(decoded);

    println!("Starting server on port {:?}", addr);

    vx.run_server(addr).await
}
