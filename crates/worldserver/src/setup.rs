use crate::{Player, World};

pub fn setup() -> World {
    let world = World {
        name: "World".to_string(),
        players: vec![Player {
            name: "Player".to_string(),
        }],
        should_shutdown: false,
    };
    world
}
