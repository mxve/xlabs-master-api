#![feature(proc_macro_hygiene, decl_macro)]

use clokwerk::{Scheduler, TimeUnits};
use std::{fs::File, io::BufWriter, time::Duration};

mod master;

fn cache_servers(game: master::Game) {
    let game_name = &game.to_string();
    let servers = master::get_servers_full(game);
    let cache_file_path = format!("servers_{}.json", game_name);
    // write servers to json file
    let file = File::create(&cache_file_path).unwrap();
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, &servers).unwrap();

    println!(
        "Cached {1} servers for {0} -> {2}",
        game_name,
        servers.len(),
        cache_file_path
    );
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
    cache_servers_all();
    let thread_handle = scheduler.watch_thread(Duration::from_millis(500));
    thread_handle.stop();
}
