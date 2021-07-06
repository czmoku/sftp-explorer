use crate::sftp::{SftpConnection, SftpElement};
use rocket::http::{Status, Header};
use rocket::response::{content, status, Responder};
use std::path::{PathBuf, Path};
use rocket::fs::NamedFile;
use rocket::{Request, response, Response};
use rocket::tokio::io::AsyncRead;
use rocket::tokio::fs::File;

#[get("/list")]
pub fn list_root() -> status::Custom<content::Json<String>> {
    let response = SftpConnection::from_env_config();
    let x = response.file_list_root();
    let result = serde_json::to_string(&x).expect("Cannot parse response");
    let a = content::Json(result);
    status::Custom(Status::Ok, a)
}

#[get("/list/<path..>")]
pub fn list_sub_dir(path: PathBuf) -> status::Custom<content::Json<String>> {
    let response = SftpConnection::from_env_config();
    let x = response.file_list(path.to_str().unwrap_or("/"));
    let result = serde_json::to_string(&x).expect("Cannot parse response");
    let a = content::Json(result);
    status::Custom(Status::Ok, a)
}

#[get("/download/<path..>")]
pub async fn download_file(path: PathBuf) -> Ress {
    let response = SftpConnection::from_env_config();
    let (temp_dir, temp_file, file_name) = response.download_file(path.to_str().expect("Invalid file"));
    Ress {
     file: rocket::tokio::fs::File::from_std(std::fs::File::open(temp_file).unwrap()),
     file_name: file_name
    }

}

struct Ress {
    file: rocket::tokio::fs::File,
    file_name: String
}

impl<'r> Responder<'r, 'static> for Ress {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        Response::build().sized_body(None, self.file)
            .header(Header::new("Content-Disposition", format!("attachment; filename=\"{}\"", self.file_name)))
            .ok()
    }
}

#[launch]
pub fn rocket() -> _ {
    rocket::build().mount("/", routes![list_root, list_sub_dir, download_file])
}
