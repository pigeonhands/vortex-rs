use crate::config::{Config, ResponseBody, Routes, StaticRoutes};

use actix_files as fs;
use actix_http::http::{header::ContentType, StatusCode};
use actix_http::Response;
use actix_web::{client::{Client, ClientBuilder}, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use url::Url;

use log::info;

use std::net::ToSocketAddrs;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct Vortex {
    cfg: Config,
}

impl Vortex {
    pub fn from_config(cfg: Config) -> Self {
        Vortex { cfg }
    }

    pub async fn run_server<ADDR: ToSocketAddrs>(self, bind_addr: ADDR) -> std::io::Result<()> {
        for static_route in &self.cfg.static_rounts {
            match static_route {
                StaticRoutes::Dir {
                    route,
                    path,
                    listings: _,
                } => {
                    info!("+ STATIC DIR {} {}", &route, &path);
                }
            }
        }
        for r in self.cfg.routes.iter() {
            match r {
                Routes::Respond(resp) => {
                    info!("+ RESPOND TO {}", &resp.common.path);
                }
                Routes::Proxy(proxy) => {
                    info!("+ PROXY TO {} -> {}", &proxy.common.path, &proxy.target);
                }
            }
        }

        HttpServer::new(move || {
            let routes = Arc::new(self.cfg.routes.clone());

            let client = ClientBuilder::new()
                .timeout(Duration::from_secs(100))
                .finish();

            let mut app = App::new()
                .data(routes)
                .data(client)
                .default_service(web::to(Self::handle_err_req));
            // .wrap(Logger::new("%a %{User-Agent}i"));

            for static_route in &self.cfg.static_rounts {
                match static_route {
                    StaticRoutes::Dir {
                        route,
                        path,
                        listings,
                    } => {
                        let mut fs = fs::Files::new(route, path);
                        if listings.unwrap_or(true) {
                            fs = fs.show_files_listing();
                        }
                        app = app.service(fs);
                    }
                }
            }

            app
        })
        .bind(bind_addr)
        .expect("To bind HttpServer")
        .shutdown_timeout(5)
        .run()
        .await
    }

    async fn handle_err_req(
        routes: web::Data<Arc<Vec<Routes>>>,
        client: web::Data<Client>,
        req: HttpRequest,
        body: web::Bytes,
    ) -> Response {
        match Self::handle_request(&routes, &client, &req, body).await {
            Ok(r) => r,
            Err(e) => HttpResponse::InternalServerError().body(format!("{:?}", e)),
        }
    }

    async fn handle_request(
        routes: &Arc<Vec<Routes>>,
        client: &Client,
        req: &HttpRequest,
        body: web::Bytes,
    ) -> Result<HttpResponse, Box<dyn std::error::Error>> {
        for route in routes.iter() {
            if !route.matches(req.uri().path()) {
                continue;
            }

            let resp = match route {
                Routes::Respond(resp) => {
                    let mut builder = Response::build(StatusCode::from_u16(resp.status_code)?);
                    builder.set(ContentType(mime::Mime::from_str(&resp.content_type)?));
                    match &resp.body {
                        ResponseBody::String(s) => builder.body(s),
                    }
                }
                Routes::Proxy(proxy) => {
                    let mut new_url = Url::parse(&proxy.target)?;

                    new_url.set_path(req.uri().path());
                    new_url.set_query(req.uri().query());

                    let proxy_req =
                        client.request_from(new_url.as_str(), req.head());
                    let proxy_req = if let Some(addr) = req.head().peer_addr {
                        proxy_req.header("x-forwarded-for", format!("{}", addr.ip()))
                    } else {
                        proxy_req
                    };
                    let mut res = proxy_req.send_body(body).await.map_err(Error::from)?;

                    let mut client_resp = HttpResponse::build(res.status());

                    for (header_name, header_value) in
                        res.headers().iter().filter(|(h, _)| *h != "connection")
                    {
                        client_resp.header(header_name.clone(), header_value.clone());
                    }
                    client_resp.body(res.body().await.map_err(Error::from)?)
                }
                _ => HttpResponse::NotFound().body("not found"),
            };
            return Ok(resp);
        }

        Ok(HttpResponse::NotFound().body("not found"))
    }
}
