use crate::config::{Config, Routes, ResponseBody, RespondRoute, ProxyRoute};


use actix_web::{get, web, App, HttpRequest, HttpServer, Responder, HttpResponse, client, Error};
use actix_http::{Response};
use actix_http::{
    http::{
        StatusCode,
        header::{ContentType}
    }
};
use url::Url;



use std::net::ToSocketAddrs;
use std::sync::Arc;
use std::str::FromStr;

#[derive(Clone)]
pub struct Vortex {
    cfg: Arc<Config>,
}

impl Vortex {
    pub fn from_config(cfg: Config) -> Self {
        Vortex {
            cfg: Arc::from(cfg),
        }
    }

    pub async fn run_server<ADDR: ToSocketAddrs>(self, bind_addr: ADDR) -> std::io::Result<()> {
        HttpServer::new(move || {
            let c = self.cfg.clone();
            App::new()
            .data(c)
            .default_service(web::to(move |c, r, b|  Self::handle_err_req(c, r, b)))
        })
        .bind(bind_addr)
        .expect("To bind HttpServer")
        .shutdown_timeout(5)
        .run()
        .await
    }

    async fn handle_err_req(cfg: web::Data<Arc<Config>>, req: HttpRequest, body: web::Bytes) -> Response {
        println!("Req");
        match Self::handle_request(&cfg, &req, body).await {
            Ok(r) => r,
            Err(e) => HttpResponse::InternalServerError().body(format!("{:?}", e)),
        }
    }

    async fn handle_request(cfg: &Arc<Config>, req: &HttpRequest, body: web::Bytes) -> Result<Response, Box<dyn std::error::Error>>  {
        
        for route in cfg.routes.iter() {
            if !route.matches(req.uri().path()){
                continue;
            }
           
            let resp = match route {
                Routes::Respond(resp) => {
                    let mut builder = Response::build(StatusCode::from_u16(resp.status_code)?);
                    builder.set(
                        ContentType(mime::Mime::from_str(&resp.content_type)?)
                    );
                    match &resp.body{
                        ResponseBody::String(s) => builder.body(s)
                    }
                    
                },
                Routes::Proxy(proxy) => {
                    println!("Proxy");
                    let mut new_url = Url::parse(&proxy.target)?;

                    new_url.set_path(req.uri().path());
                    new_url.set_query(req.uri().query());
                    
                    let proxy_req = client::Client::new().request_from(new_url.as_str(), req.head());
                    println!("{}", new_url.as_str());
                    let proxy_req = if let Some(addr) = req.head().peer_addr {
                        proxy_req.header("x-forwarded-for", format!("{}", addr.ip()))
                    } else {
                        proxy_req
                    };
                    let mut res = proxy_req.send_body(body).await.map_err(Error::from)?;
                    println!("body: {:?}", res.body().await);
                    
                    let mut client_resp = HttpResponse::build(res.status());
                    client_resp.body(res.body().await.map_err(Error::from)?)
                }
                _ =>  HttpResponse::NotFound().body("not found")
            };
            return Ok(resp)
        }

        Ok(
            HttpResponse::NotFound().body("not found")
        )
    }
}
