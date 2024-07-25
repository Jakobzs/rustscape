use mlua::Lua;

struct Player {
    name: String,
}

struct World {
    name: String,
    players: Vec<Player>,
}

fn main() {
    let world = setup();

    // Create mlua lua state
    let lua = Lua::new();
    lua.set_app_data(world);

    loop {
        println!("Game loop!");

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
