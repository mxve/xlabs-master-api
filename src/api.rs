use rocket::{
    http::ContentType,
    response::{content::Content, NamedFile},
};
use std::{ffi::OsStr, fs, path::PathBuf};

#[get("/servers")]
fn servers() -> Content<String> {
    let mut result: String = "[\n".to_owned();

    for path in fs::read_dir("./").unwrap() {
        let path = path.unwrap().path();
        if path.extension() == Some(OsStr::new("json"))
            && path
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("servers_")
        {
            let content = fs::read_to_string(&path).unwrap();
            result.push_str(&content[1..content.len() - 1]);
            if path != fs::read_dir("./").unwrap().last().unwrap().unwrap().path() {
                result.push_str(",\n");
            }
        }
    }

    result.push_str("\n]");

    Content(ContentType::JSON, result)
}

#[get("/servers/<game..>")]
fn servers_game(game: PathBuf) -> Option<NamedFile> {
    let path = format!("servers_{}.json", game.to_str().unwrap());
    println!("{:?}", path);
    NamedFile::open(path).ok()
}

pub fn launch() {
    rocket::ignite()
        .mount("/", routes![servers, servers_game])
        .launch();
}
