use mlua::Lua;

struct Player {
    name: String,
}

struct World {
    name: String,
    players: Vec<Player>,
}

fn tokio_lets_gooo() {
    println!("Tokio lets go!");
}

// The main thread is considered the game thread. Therefore main is not async
fn main() {
    let world = setup();

    // Create mlua lua state
    let lua = Lua::new();
    lua.set_app_data(world);

    // Create thread that spawns the tokio runtime and runs stuff
    std::thread::spawn(|| {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            // Run the game loop
            tokio_lets_gooo();
        });
    });

    loop {
        //println!("Game loop!");

        input(&lua);
        update(&lua);
        render(&lua);
    }
}

fn setup() -> World {
    let world = World {
        name: "World".to_string(),
        players: vec![Player {
            name: "Player".to_string(),
        }],
    };
    world
}

fn input(lua: &Lua) {}

fn update(lua: &Lua) {}

fn render(lua: &Lua) {}
