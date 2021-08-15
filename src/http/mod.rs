use crate::sftp::{SftpConnection, SftpElement, AsyncSftp};
use rocket::http::{Status, Header, ContentType};
use rocket::response::{content, status, Responder, Redirect};
use std::path::{PathBuf, Path};
use rocket::fs::{NamedFile, FileServer, Options};
use rocket::{Request, response, Response, Rocket, Build, Orbit, Data, Route, figment};
use rocket::tokio::io::{AsyncRead, AsyncReadExt};
use rocket::tokio::fs::File;
use serde::Serialize;

#[get("/list")]
pub fn list_root() -> status::Custom<content::Json<String>> {
    let response = SftpConnection::from_env_config();
    let x = response.file_list_root();
    let result = serde_json::to_string(&x).expect("Cannot parse response");
    let resultJson = content::Json(result);
    status::Custom(Status::Ok, resultJson)
}

#[get("/list/<path..>")]
pub fn list_sub_dir(path: PathBuf) -> status::Custom<content::Json<String>> {
    let response = SftpConnection::from_env_config();
    let x = response.file_list(path.to_str().unwrap_or("/"));
    let result = serde_json::to_string(&x).expect("Cannot parse response");
    let resultJson = content::Json(result);
    status::Custom(Status::Ok, resultJson)
}

#[get("/download/<path..>")]
pub async fn download_file(path: PathBuf) -> FileResponse {
    let response = SftpConnection::from_env_config();
    let (response_stream, file_name, file_stat) = response.download_file(path.to_str().expect("Invalid file"));
    FileResponse {
     read_source: response_stream,
     file_name: file_name,
     file_size: file_stat.size.unwrap_or(0)
    }
}

#[get("/instance/info")]
pub async fn instance_info() -> status::Custom<content::Json<String>> {
    let connection = SftpConnection::from_env_config();
    let info = InfoResponse {
        host: connection.connected_to()
    };
    let result = serde_json::to_string(&info).expect("Cannot parse response");
    status::Custom(Status::Ok, content::Json(result))

}

struct FileResponse {
    read_source: AsyncSftp,
    file_name: String,
    file_size: u64,
}

#[derive(Serialize)]
struct InfoResponse {
    host: String
}

impl<'r> Responder<'r, 'static> for FileResponse {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        Response::build().streamed_body(self.read_source)
            .header(Header::new("Content-Disposition", format!("attachment; filename=\"{}\"", self.file_name)))
            .header(Header::new("Content-Length", format!("{}", self.file_size)))
            .ok()
    }
}

use rocket::http::Method;
use rocket_cors::{AllowedOrigins, CorsOptions};
use rocket::fairing::{Fairing, Info, AdHoc, Kind};
use rocket::route::{Handler, Outcome};
use rocket::http::uri::Segments;
use rocket::http::ext::IntoOwned;
use std::io::Cursor;
use std::rc::Rc;
use std::sync::Arc;
use std::ops::Deref;

struct Cors {

}

impl Cors {
    fn new() -> Cors {
        Cors {

        }
    }
}

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "Cors",
            kind: Kind::Response
        }
    }

    async fn on_response<'r>(&self, _req: &'r Request<'_>, _res: &mut Response<'r>) {
        _res.set_raw_header("Access-Control-Allow-Methods", "POST, GET, OPTIONS");
        _res.set_raw_header("Access-Control-Allow-Origin", "*");
        _res.set_raw_header("Access-Control-Allow-Credentials", "true");
        _res.set_raw_header("Access-Control-Allow-Headers", "Content-Type");
    }
}

#[launch]
pub fn rocket() -> _ {
    let http_prefix = std::env::var("HTTP_PREFIX").unwrap_or(String::from(""));
    rocket::build()
        .attach(Cors::new())
        .mount(format!("/{}", http_prefix), routes![list_root, list_sub_dir, download_file, instance_info])
        .mount(format!("/{}/static", http_prefix), StaticServerWithEnvInjection::from(PathBuf::from("static")))

}

#[derive(Debug, Clone)]
struct StaticServerWithEnvInjection {
    root_path: PathBuf
}

impl StaticServerWithEnvInjection {
    pub fn from(path: PathBuf) -> Self {
        StaticServerWithEnvInjection {
            root_path: path.into()
        }
    }
}

impl Into<Vec<Route>> for StaticServerWithEnvInjection {
    fn into(self) -> Vec<Route> {
        let source = figment::Source::File(self.root_path.clone());
        let mut route = Route::ranked(10, Method::Get, "/<path..>", self);
        route.name = Some(format!("FileServer: {}/", source).into());
        vec![route]
    }
}

struct ModifiedPage {
  content: String,
  content_type: String
}

impl<'r> Responder<'r, 'static> for ModifiedPage {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        Response::build().sized_body(self.content.len(),Cursor::new(self.content))
            .header(Header::new("content-type", self.content_type))
            .ok()
    }
}

#[async_trait]
impl Handler for StaticServerWithEnvInjection {
    //XXX copied from original FileServer
    async fn handle<'r>(&self, req: &'r Request<'_>, data: Data<'r>) -> Outcome<'r> {
        use rocket::http::uri::fmt::Path;

        // Get the segments as a `PathBuf`, allowing dotfiles requested.
        let root_path = &self.root_path;
        let path = req.segments::<Segments<'_, Path>>(0..).ok()
            .and_then(|segments| segments.to_path_buf(true).ok())
            .map(|path| root_path.join(path));

        match path {
            Some(p) if p.is_dir() => {
                // Normalize '/a/b/foo' to '/a/b/foo/'.
                if !req.uri().path().ends_with('/') {
                    let normal = req.uri().map_path(|p| format!("{}/", p))
                        .expect("adding a trailing slash to a known good path => valid path")
                        .into_owned();

                    return Outcome::from_or_forward(req, data, Redirect::permanent(normal));
                }

                let filename = NamedFile::open(p.join("index.html")).await.ok();
                self.process_and_response_with_file(req, data, filename).await
            },
            Some(p) => {
                let filename = NamedFile::open(&p).await.ok();
                if(filename.is_none()) {
                    let index_file = NamedFile::open(self.root_path.join("index.html")).await.ok();
                    return self.process_and_response_with_file(req, data, index_file).await
                }
                self.process_and_response_with_file(req, data, filename).await
            },
            None => Outcome::forward(data),
        }
    }
}


impl StaticServerWithEnvInjection {



}

impl StaticServerWithEnvInjection {
    fn get_content_type(&self, extension: &str) -> String {
        match extension {
            "js" => String::from("text/javascript"),
            "css" => String::from("text/css"),
            "html" => String::from("text/html"),
            _ => String::from("text/plain")
        }
    }

    async fn process_and_response_with_file<'r>(&self, req: &'r Request <'_>, data: Data <'r>, filename: Option<NamedFile>) -> Outcome<'r> {
        let mut file_content = String::new();
        rocket::tokio::fs::File::open(filename.as_ref().expect("File with resources cannot be found").path())
            .await.unwrap().read_to_string(&mut file_content).await;
        let http_prefix = std::env::var("HTTP_PREFIX").unwrap_or(String::from(""));
        let static_prefix = format!("/{}/static", http_prefix);
        file_content = file_content.replace("${PATH_PREFIX}", static_prefix.as_str());
        let api_prefix = format!("/{}", http_prefix);
        file_content = file_content.replace("${API_URL}", api_prefix.as_str());
        let extension = filename.as_ref().and_then(|f| f.path().extension())
            .and_then(|e| e.to_str())
            .and_then(|s| Some(s))
            .unwrap_or("");
        return Outcome::from_or_forward(req, data, ModifiedPage {
            content: file_content,
            content_type: self.get_content_type(extension)
        })
    }
}
