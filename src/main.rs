use std::env;
use std::error::Error;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync;

use warp::{self, Filter, http::Response};

struct Server {}

impl Server {
    fn peek_sequence(&self, project: &str, tag: &str) -> Result<u64, Box<dyn Error>> {
        let proj = Path::new("sequences").join(project);

        let cur: u64 = match fs::read_to_string(proj.join(tag)) {
            Ok(v) => v.parse(),
            Err(_) => Ok(0),
        }?;

        Ok(cur)
    }

    fn consume_sequence(&self, project: &str, tag: &str) -> Result<u64, Box<dyn Error>> {
        let proj = Path::new("sequences").join(project);

        match fs::create_dir(&proj) {
            Err(ref e) if e.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
            v => v,
        }?;

        let cur: u64 = match fs::read_to_string(proj.join(tag)) {
            Ok(v) => v.parse(),
            Err(_) => Ok(0),
        }?;

        match fs::File::create(proj.join(tag)) {
            Ok(mut f) => f.write_all(format!("{}", cur + 1).as_bytes()),
            Err(v) => Err(v),
        }?;

        Ok(cur)
    }
}

#[tokio::main]
async fn main() {
    let port = match env::var("SQNZ_PORT").unwrap_or("8080".to_string()).parse() {
        Ok(v) => v,
        Err(e) => {
            println!("could not parse port: {}", e);
            std::process::exit(1);
        }
    };
    let s = sync::Arc::new(sync::Mutex::new(Server {}));

    match fs::create_dir("sequences") {
        Err(ref e) if e.kind() == std::io::ErrorKind::AlreadyExists => (),
        Err(e) => {
            println!("could not create sequence folder: {}", e);
            std::process::exit(1);
        }
        _ => (),
    };

    let s1 = s.clone();
    let consume = warp::post()
        .and(warp::path::param::<String>())
        .and(warp::path::param::<String>())
        .map(move |project: String, tag: String| {
            match s1
                .lock()
                .unwrap()
                .consume_sequence(project.as_str(), tag.as_str())
            {
                Ok(v) => Response::builder()
                            .body(format!("{}", v)),
                Err(e) => {
                    println!("could not consume sequence: {}", e);
                    Response::builder()
                        .status(500)
                        .body(format!("could not consume sequence: {}", e))
                }
            }
        });

    let s2 = s.clone();
    let peek = warp::get()
        .and(warp::path::param::<String>())
        .and(warp::path::param::<String>())
        .map(move |project: String, tag: String| {
            match s2
                .lock()
                .unwrap()
                .peek_sequence(project.as_str(), tag.as_str())
            {
                Ok(v) => Response::builder()
                            .body(format!("{}", v)),
                Err(e) => {
                    println!("could not peek at sequence: {}", e);
                    Response::builder()
                        .status(500)
                        .body(format!("could not peek at sequence: {}", e))
                }
            }
        });

    let help = warp::get().map(|| {
        format!(
            "
sqnz (https://github.com/kennylevinsen/sqnz)

To consume the current sequence number, POST to /PROJECT/TAG:

    curl -X POST http://sqnz/project/tag

To peek at the current sequence number without consuming it, GET /PROJECT/TAG:

    curl http://sqnz/project/tag

"
        )
    });

    warp::serve(consume.or(peek).or(help)).run(([0, 0, 0, 0], port)).await;
}
