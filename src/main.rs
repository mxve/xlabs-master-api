#![feature(proc_macro_hygiene, decl_macro)]

use clokwerk::{Scheduler, TimeUnits};
use std::{fs::File, io::BufWriter, time::Duration};

mod api;
mod master;

#[macro_use]
extern crate rocket;

fn cache_servers(game: master::Game) {
    let game_name = &game.to_string();
    let servers = master::get_servers_full(game);
    // write servers to json file
    let file = File::create(format!("servers_{}.json", game_name)).unwrap();
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, &servers).unwrap();
}

fn cache_servers_all() {
    cache_servers(master::Game::IW4);
    cache_servers(master::Game::IW6);
    cache_servers(master::Game::S1);
}

fn main() {
    let mut scheduler = Scheduler::new();
    scheduler.every(2.minutes()).run(|| {
        cache_servers_all();
    });
    //cache_servers_all();
    let thread_handle = scheduler.watch_thread(Duration::from_millis(500));
    api::launch();
    thread_handle.stop();
}
