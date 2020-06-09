mod config;
mod vortex;

use std::fs;
use std::net::SocketAddrV4;

use clap::Clap;
use serde_yaml;

use chrono;
use fern;
use log::{self, info};

use config::{Config, StaticRoutes};

#[derive(Clap, Clone)]
#[clap(version = "1.0", author = "Sam M.")]
pub struct CommonArgs {
    /// Load config from yaml file
    #[clap(short = "c", long = "config", global = true)]
    pub cfg_file: Option<String>,

    /// Run a static file server in the current directory
    #[clap(short = "s", long = "file-server", global = true)]
    pub file_server: bool,

    /// Verbose level. e.g. -vvv
    #[clap(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: i32,
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let args = CommonArgs::parse();

    let log_level = match args.verbose {
        1 => log::LevelFilter::Debug,
        d if d >= 2 => log::LevelFilter::Trace,
        _ => log::LevelFilter::Info,
    };

    let fern_logger = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{}[{}] {}",
                chrono::Local::now().format("[%H:%M:%S]"),
                record.level(),
                message
            ))
        })
        .level(log_level)
        .level_for("actix-connect", log::LevelFilter::Debug)
        .level_for("actix-http", log::LevelFilter::Info)
        .chain(std::io::stdout());

    fern_logger.apply().expect("Failed to initilize logging!");

    let mut cfg = if let Some(cfg_file) = args.cfg_file {
        let str = fs::read_to_string(&cfg_file)?;
        serde_yaml::from_str(&str).unwrap()
    } else {
        Config::default()
    };

    if args.file_server {
        cfg.static_rounts.push(StaticRoutes::Dir {
            route: "/".to_string(),
            path: ".".to_string(),
            listings: Some(true),
        });
    }

    let addr = SocketAddrV4::new(cfg.addr, cfg.port);

    let vx = vortex::Vortex::from_config(cfg);

    info!("Starting server on port {:?}", addr);

    vx.run_server(addr).await
}
