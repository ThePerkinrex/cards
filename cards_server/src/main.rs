use std::fs::read_dir;

mod game;
mod server;
mod tracing_subscriber;



fn main() {
    tracing::subscriber::set_global_default(tracing_subscriber::CustomSubscriber::new(true, &["cards_server::server"])).unwrap();

    
    // Look for each game
    let mut games = Vec::new();
    for file in read_dir("games").unwrap() {
        if let Ok(folder) = file {
            if folder.path().is_dir() {
                for file in read_dir(folder.path()).unwrap() {
                    if let Ok(file) = file {
                        if file.file_name().to_str().unwrap() == "game.lua" {
                            games.push(game::Game::load(file.path()))
                        }
                    }
                }
            }
        }
    }
    // println!("[{}]", games.iter().map(|x|x.to_string()).rev().fold(String::new(), |s, x| format!("{}, {}", x, s)));
    server::run(games);
}
