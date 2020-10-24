use rlua::Lua;
use std::fs::read_to_string;

const UTILS: &'static str = include_str!("utils.lua");

pub struct Game {
    name: String,
    version: String,
    lua: Lua,
}

impl Game {
    pub fn load<P: AsRef<std::path::Path>>(file: P) -> Self {
		let mut name = String::new();
		let mut version = String::new();
        let lua = rlua::Lua::new();
        lua.context(|ctx| {
            ctx.load(UTILS)
                .exec()
                .unwrap();
            let shuffle = ctx
                .create_function(|ctx, table: rlua::Table| {
                    use rand::seq::SliceRandom;
                    use rand::thread_rng;
                    let mut rng = thread_rng();
                    let res = ctx.create_table()?;
                    let mut seq: Vec<rlua::Value> =
                        table.sequence_values().map(|x| x.unwrap()).collect();
                    // println!("Unshuffled: [{}]", seq.iter().enumerate().map(|(i, x)| format!("{}: {:?}", i, x)).fold("".into(), |f, x| format!("{}, {}", f, x)));
                    seq.shuffle(&mut rng);
                    // println!("Unshuffled: [{}]", seq.iter().enumerate().map(|(i, x)| format!("{}: {:?}", i, x)).fold("".into(), |f, x| format!("{}, {}", f, x)));
                    for (i, x) in seq.into_iter().enumerate() {
                        res.set(i + 1, x)?;
                    }
                    Ok(res)
                })
                .unwrap();
            ctx.globals().set("shuffle", shuffle).unwrap();
        }); // Load utils
        lua.context(|ctx| {
            // println!("FOUND game.lua in {}", folder.path().display());
            ctx.load(&read_to_string(file).unwrap())
                .exec()
                .map_err(|e| {
                    println!("[LUA] {}", e);
                    panic!()
                })
                .unwrap();
            let globals = ctx.globals().clone();
            name = globals.get("name").unwrap();
            version = globals.get("version").unwrap();
			
			
            // let (piles, player_piles): (rlua::Table, rlua::Table) = setup
            //     .call(2)
            //     .map_err(|e| {
            //         println!("[LUA] {}", e);
            //         panic!()
            //     })
            //     .unwrap();
            // let return_vals = ctx.create_table().unwrap();
            // return_vals.set("piles", piles).unwrap();

            // return_vals.set("player_piles", player_piles).unwrap();
            // println!("Name: {}", name);
            // for i in 1..=return_vals
            //     .get::<_, rlua::Table>("piles")
            //     .unwrap()
            //     .len()
            //     .unwrap()
            // {
            //     let pile: rlua::Table = return_vals
            //         .get::<_, rlua::Table>("piles")
            //         .unwrap()
            //         .get(i)
            //         .unwrap();
            //     println!("DECK {}", i);
            //     print_table(" - ".into(), pile);
            // }

            // return_vals
            //     .get::<_, rlua::Table>("piles")
            //     .unwrap()
            //     .get::<_, rlua::Table>(1)
            //     .unwrap()
            //     .get::<_, rlua::Function>("on_click")
            //     .unwrap()
            //     .call::<_, ()>((
            //         return_vals
            //             .get::<_, rlua::Table>("piles")
            //             .unwrap()
            //             .get::<_, rlua::Table>(1)
            //             .unwrap(),
            //         return_vals.get::<_, rlua::Table>("player_piles").unwrap(),
            //     ))
            //     .unwrap();
            // return_vals
            //     .get::<_, rlua::Table>("piles")
            //     .unwrap()
            //     .get::<_, rlua::Table>(1)
            //     .unwrap()
            //     .get::<_, rlua::Function>("on_click")
            //     .unwrap()
            //     .call::<_, ()>((
            //         return_vals
            //             .get::<_, rlua::Table>("piles")
            //             .unwrap()
            //             .get::<_, rlua::Table>(1)
            //             .unwrap(),
            //         return_vals.get::<_, rlua::Table>("player_piles").unwrap(),
            //     ))
            //     .unwrap();
            // for i in 1..=return_vals
            //     .get::<_, rlua::Table>("player_piles")
            //     .unwrap()
            //     .len()
            //     .unwrap()
            // {
            //     let pile: rlua::Table = return_vals
            //         .get::<_, rlua::Table>("player_piles")
            //         .unwrap()
            //         .get(i)
            //         .unwrap();
            //     println!("DECK [PLAYER] {}", i);
            //     print_table(" - ".into(), pile);
            // }
		});
		Self {name, version, lua}
    }
    
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn version(&self) -> &String {
        &self.version
    }

    pub fn thread_safe(&self) -> ThreadSafeGame {
        ThreadSafeGame {
            name: self.name().clone(),
            version: self.version().clone()
        }
    }
}

impl std::fmt::Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} [{}]", self.name, self.version)
    }
}

impl std::fmt::Debug for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} [{}]", self.name, self.version)
    }
}

#[derive(Debug, Clone)]
pub struct ThreadSafeGame {
    name: String,
    version: String
}

impl ThreadSafeGame {    
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn version(&self) -> &String {
        &self.version
    }
}

impl std::fmt::Display for ThreadSafeGame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} [{}]", self.name, self.version)
    }
}

// fn print_table(indent: String, table: rlua::Table) {
//     for pair in table.pairs::<String, rlua::Value>() {
//         if let Ok((k, v)) = pair {
//             match v {
//                 rlua::Value::Table(t) => {
//                     println!("{}{}: TABLE", indent, k);
//                     print_table(format!("   {}", indent), t);
//                 }
//                 rlua::Value::String(s) => println!("{}{}: \"{}\"", indent, k, s.to_str().unwrap()),
//                 x => println!("{}{}: {:?}", indent, k, x),
//             }
//         }
//     }
// }
