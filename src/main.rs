#![feature(proc_macro_hygiene, decl_macro)]

use clokwerk::{Scheduler, TimeUnits};
use rocket::response::NamedFile;
use std::{fs::File, io::BufWriter, path::Path, time::Duration};

mod master;

fn cache_servers() {
    let servers = master::get_servers_full();
    // write servers to json file
    let file = File::create("servers.json").unwrap();
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, &servers).unwrap();
}

#[macro_use]
extern crate rocket;
#[get("/servers")]
fn index() -> Option<NamedFile> {
    NamedFile::open(Path::new("servers.json")).ok()
}

fn main() {
    let mut scheduler = Scheduler::new();
    scheduler.every(2.minutes()).run(|| {
        cache_servers();
    });
    cache_servers();
    let thread_handle = scheduler.watch_thread(Duration::from_millis(500));
    rocket::ignite().mount("/", routes![index]).launch();
    thread_handle.stop();
}
