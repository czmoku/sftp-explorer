use rocket::fairing::{Fairing, Info, Kind};
use rocket::{Request, Response};

pub struct Cors {}

impl Cors {
    pub fn new() -> Cors {
        Cors {}
    }
}

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "Cors",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _req: &'r Request<'_>, _res: &mut Response<'r>) {
        _res.set_raw_header("Access-Control-Allow-Methods", "POST, GET, OPTIONS");
        _res.set_raw_header("Access-Control-Allow-Origin", "*");
        _res.set_raw_header("Access-Control-Allow-Credentials", "true");
        _res.set_raw_header("Access-Control-Allow-Headers", "Content-Type");
    }
}
