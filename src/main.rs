use lazy_static::lazy_static;
use regex::Regex;
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, Barrier};
use threadpool::ThreadPool;

lazy_static! {
    static ref RE: Regex = Regex::new("(?P<url>\\(https.*?\\))").unwrap();
}

fn main() {
    let urls = Arc::new(
        std::fs::read_dir("./arl")
            .unwrap()
            .map(|file| {
                std::fs::read_to_string(file.unwrap().path().display().to_string()).unwrap()
            })
            .map(|contents| {
                RE.captures_iter(&contents)
                    .map(|u| {
                        let l = &u["url"][1..];
                        let l = &l[..l.len() - 1];
                        return l.to_string();
                    })
                    .collect::<Vec<String>>()
            })
            .flatten()
            .collect::<Vec<String>>(),
    );

    let jobs = urls.len();
    let workers = 8;
    let pool = ThreadPool::new(workers);
    let barrier = Arc::new(Barrier::new(jobs + 1));
    std::env::set_current_dir(Path::new("./repos")).unwrap();

    for i in 0..jobs {
        let barrier = barrier.clone();
        let urls = Arc::clone(&urls);
        pool.execute(move || {
            match Command::new("git").args(&["clone", &urls[i]]).spawn() {
                Ok(_) => println!("Downloaded: {}", &urls[i]),
                Err(e) => println!("Failed to download: {} for reason: {}", &urls[i], e),
            };
            barrier.wait();
        })
    }

    barrier.wait();
}
