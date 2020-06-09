use regex::Regex;
use serde::Deserialize;
use serde_regex;
use std::net::Ipv4Addr;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    #[serde(default = "default_ip")]
    pub addr: Ipv4Addr,

    #[serde(default = "default_port")]
    pub port: u16,

    pub routes: Vec<Routes>,

    #[serde(rename = "static")]
    pub static_rounts: Vec<StaticRoutes>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            addr: default_ip(),
            port: default_port(),
            routes: Vec::new(),
            static_rounts: Vec::new(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub enum StaticRoutes {
    #[serde(rename = "dir")]
    Dir {
        route: String,
        path: String,
        listings: Option<bool>,
    },
}

#[derive(Debug, Deserialize)]
pub struct StaticRouteDir {
    pub folder: String,
    pub show_listings: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub enum Routes {
    #[serde(rename = "respond")]
    Respond(RespondRoute),

    #[serde(rename = "proxy")]
    Proxy(ProxyRoute),
}

impl Routes {
    pub fn matches<T: AsRef<str>>(&self, r: T) -> bool {
        match self {
            Routes::Respond(rsp) => &rsp.common,
            Routes::Proxy(px) => &px.common,
        }
        .path
        .is_match(r.as_ref())
    }
}

fn default_ip() -> Ipv4Addr {
    Ipv4Addr::UNSPECIFIED
}

fn default_port() -> u16 {
    8080
}

fn default_content_type() -> String {
    String::from("text/plain")
}

fn default_status_code() -> u16 {
    200
}

#[derive(Debug, Deserialize, Clone)]
pub struct Common {
    #[serde(with = "serde_regex")]
    pub path: Regex,
}

#[derive(Debug, Deserialize, Clone)]
pub enum ResponseBody {
    #[serde(rename = "body-string")]
    String(String),
}
#[derive(Debug, Deserialize, Clone)]
pub struct RespondRoute {
    #[serde(flatten)]
    pub common: Common,

    #[serde(rename = "content-type", default = "default_content_type")]
    pub content_type: String,

    #[serde(rename = "status-code", default = "default_status_code")]
    pub status_code: u16,

    #[serde(flatten)]
    pub body: ResponseBody,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ProxyRoute {
    #[serde(flatten)]
    pub common: Common,

    pub target: String,
}
