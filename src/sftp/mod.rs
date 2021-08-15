use ssh2::{Sftp, Session, FileStat};
use std::net::{TcpStream, SocketAddr, ToSocketAddrs};
use std::path::{Path, PathBuf};
use rocket::http::ext::IntoCollection;
use rocket::serde::{Serialize, Deserialize};
use std::io::{Read, Write, IoSliceMut};
use std::fs::File;
use rocket::tokio::macros::support::thread_rng_n;
use tempfile::TempDir;
use std::ffi::OsStr;
use rocket::futures::AsyncRead;
use std::pin::Pin;
use std::task::{Context, Poll};
use rocket::tokio::io::ReadBuf;

pub struct SftpConnection {
    hostname: SocketAddr,
    username: String,
    password: String,
}

impl SftpConnection {
    pub fn connected_to(self) -> String {
        let hostname = std::env::var("SFTP_HOSTNAME").expect("SFTP_HOSTNAME not set");
        let port = std::env::var("SFTP_PORT").unwrap_or(String::from("22"));
        format!("{}:{}", hostname, port)
    }

    pub fn from_env_config() -> Self {
        let hostname = std::env::var("SFTP_HOSTNAME").expect("SFTP_HOSTNAME not set");
        let port = std::env::var("SFTP_PORT").unwrap_or(String::from("22"));
        let username = std::env::var("SFTP_USERNAME").expect("SFTP_USERNAME not set");
        let password = std::env::var("SFTP_PASSWORD").expect("SFTP_PASSWORD not set");

        SftpConnection::new(format!("{}:{}", hostname, port).as_str(), username.as_str(), password.as_str())
    }

    pub fn new(hostname: &str, username: &str, password: &str) -> Self {
        let hostname_as_addr = hostname.to_socket_addrs().expect("Cannot parse hostname to address").next().expect("Hostname not resolved");
        SftpConnection {
            hostname: hostname_as_addr,
            username: String::from(username),
            password: String::from(password),
        }
    }

    pub fn file_list_root(&self) -> Vec<SftpElement> {
        self.file_list("/")
    }

    pub fn file_list(&self, path: &str) -> Vec<SftpElement> {
        let sftp = self.connect_to();
        let result = sftp.readdir(Path::new(path)).expect("Cannot ls directories");
        result.into_iter().map(|(path, fileStat)| {
            let x = fileStat.is_dir();
            let mut filename = String::from(path.to_str().unwrap_or(""));
            if (!filename.starts_with("/")) {
                filename = format!("/{}", filename);
            }
            return SftpElement { path: filename, is_directory: x };
        }).collect()
    }


    pub fn download_file(&self, path: &str) -> (AsyncSftp, String, FileStat) {
        let sftp = self.connect_to();
        let mut file = sftp.open(Path::new(path)).expect("Cannot read file");
        let file_stat = sftp.lstat(Path::new(path)).expect("Cannot stat file");
        let file_name = Path::new(path).file_name().unwrap_or(OsStr::new("installer.exe")).to_str().unwrap_or("installer.exe");
        (AsyncSftp {
            file: file
        }, String::from(file_name),file_stat)
    }

    fn connect_to(&self) -> Sftp {
        let stream = TcpStream::connect(self.hostname).expect("Cannot connect to host");
        let mut session = Session::new().expect("Cannot create session");
        session.set_tcp_stream(stream);
        session.handshake();
        session.userauth_password(self.username.as_str(), self.password.as_str());
        let sftp = session.sftp().expect("Cannot create sftp");
        sftp
    }
}

pub struct AsyncSftp {
    file: ssh2::File,
}

impl rocket::tokio::io::AsyncRead for AsyncSftp {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<std::io::Result<()>> {
        let mut file = &mut self.get_mut().file;
        let mut x: [u8; 4096] = [0; 4096];
        print!("{}", x.len());
        file.read(&mut x).unwrap_or(0);
        print!("{:?}", x);
        buf.put_slice(&x);
        return Poll::Ready(Ok(()))
    }
}


#[derive(FromForm, Serialize)]
pub struct SftpElement {
    path: String,
    is_directory: bool,
}
