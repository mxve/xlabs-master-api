use rocket::response::NamedFile;
use std::path::PathBuf;

#[get("/servers/<game..>")]
fn servers(game: PathBuf) -> Option<NamedFile> {
    let path = format!("servers_{}.json", game.to_str().unwrap());
    println!("{:?}", path);
    NamedFile::open(path).ok()
}

pub fn launch() {
    rocket::ignite().mount("/", routes![servers]).launch();
}
