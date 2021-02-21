use crate::graph::Node;
use crate::graph::ScreepsGraph as Graph;
use log::*;
use screeps::objects::Room;

type DistanceTransform = Vec<u8>;

pub fn plan_walls(room: &Room) {
    info!("Planning walls");
    let graph = load_or_get_graph(room);
    show_distance_transform(room, &graph);
    show_boundary_walls(room, &graph);
}

fn terrain_to_graph(room: &Room, sources: &Vec<Node>, sinks: &Vec<Node>) -> Graph {
    let mut walls = Vec::new();
    let terrain = room.get_terrain().get_raw_buffer();
    for (i, tile_type) in terrain.iter().enumerate() {
        // tile is a wall if tile_type & TERRAIN_MASK_WALL is 1u8
        if tile_type & screeps::constants::TERRAIN_MASK_WALL == 1 {
            walls.push(i as Node);
        }
    }
    let sinks_without_walls = sinks
        .iter()
        .filter(|sink| !walls.contains(sink))
        .map(|sink| *sink)
        .collect();
    let sources_without_walls = sources
        .iter()
        .filter(|source| !walls.contains(source))
        .map(|source| *source)
        .collect();
    Graph::new(sources_without_walls, sinks_without_walls, walls)
}

fn load_or_get_graph(room: &Room) -> Graph {
    let protected = vec![18 + 12 * 50];
    match load_graph(room) {
        None => get_graph_and_save(room, &protected),
        Some(graph) => graph,
    }
}

fn get_graph_and_save(room: &Room, protected: &Vec<Node>) -> Graph {
    info!("Creating graph");
    // Get the exists in the room
    let mut sinks = Vec::new();
    room.find(screeps::constants::find::EXIT)
        .iter()
        .map(|position| (position.y() * 50 + position.x()) as Node)
        .for_each(|exit| {
            surrounding_pos_indices(exit)
                .iter()
                .for_each(|position| sinks.push(*position))
        });
    sinks.sort();
    sinks.dedup();
    // Create the graph
    let graph = terrain_to_graph(room, protected, &sinks);

    // Save the graph to room memory
    save_graph(room, &graph);

    graph
}

fn save_graph(room: &Room, graph: &Graph) {
    let room_mem = room.memory();
    room_mem.set("graph", graph);
}

fn load_graph(room: &Room) -> Option<Graph> {
    info!("Loading graph...");
    let room_mem = room.memory();
    let get_graph = room_mem.get::<Graph>("graph");
    match get_graph {
        Ok(graph_option) => match graph_option {
            None => return None,
            Some(graph) => Some(graph),
        },
        Err(error) => None,
    }
}

fn rectangle_of_pos_indices(top_left: Node, bottom_right: Node) -> Vec<Node> {
    let mut positions = Vec::new();

    for y in top_left.div_euclid(50)..=bottom_right.div_euclid(50) {
        if y > 49 {
            break;
        }
        for x in (top_left % 50)..=(bottom_right % 50) {
            positions.push(50 * y + x);
        }
    }

    positions
}

fn surrounding_pos_indices(index: Node) -> Vec<Node> {
    // 51 = 50 * 1 + 1
    let mut x_minus = 1;
    let mut x_plus = 1;
    if index % 50 == 0 {
        x_minus = 0;
    } else if index % 50 == 49 {
        x_plus = 0;
    }
    return rectangle_of_pos_indices(index - 50 - x_minus, index + 50 + x_plus);
}

fn get_distance_transform(graph: &Graph) -> DistanceTransform {
    let boundaries = graph.get_outter_nodes();
    info!("boundaries: {}", boundaries.len());
    info!(
        "bound contains (13,6) {}",
        boundaries.contains(&(13 + 6 * 50))
    );
    info!(
        "bound contains (30,33) {}",
        boundaries.contains(&(30 + 33 * 50))
    );
    info!(
        "bound contains (31,33) {}",
        boundaries.contains(&(31 + 33 * 50))
    );
    info!(
        "bound contains (47,23) {}",
        boundaries.contains(&(47 + 23 * 50))
    );
    graph.distance_transform(&boundaries).to_vec()
}

fn save_distance_transform(room: &Room, distances: &DistanceTransform) {
    room.memory().set("distance_transform", distances);
}

fn load_distance_transform(room: &Room) -> Option<DistanceTransform> {
    room.memory()
        .get::<DistanceTransform>("distance_transform")
        .unwrap()
}

fn show_distance_transform(room: &Room, graph: &Graph) {
    let distance_transform = match load_distance_transform(room) {
        None => {
            info!("Creating distance transform");
            let distance_transform = get_distance_transform(graph);
            save_distance_transform(room, &distance_transform);
            distance_transform
        }
        Some(distance_transform) => distance_transform,
    };
    let room_visual = room.visual();
    let mut style: screeps::objects::TextStyle = Default::default();
    let colors = [
        "#ff0000", "#00ff00", "#a0a0ff", "#ffff00", "#00ffff", "#ff00ff",
    ];

    distance_transform
        .iter()
        .enumerate()
        .for_each(|(i, distance)| {
            if *distance < 50 {
                let mut style = style.clone();
                if (*distance as usize) < colors.len() {
                    style = style.color(colors[*distance as usize]);
                }
                let x = i % 50;
                let y = i.div_euclid(50);
                room_visual.text(
                    x as f32,
                    y as f32,
                    String::from(format!("{}", distance)),
                    Some(style),
                );
            }
        });
}

fn get_boundary_walls(room: &Room, graph: &Graph) -> Vec<Node> {
    info!("Creating boundary walls");
    let boundaries = graph.min_cut_nodes();
    save_boundary_walls(room, &boundaries);
    boundaries
}

fn save_boundary_walls(room: &Room, boundaries: &Vec<Node>) {
    info!("Saving boundaries");
    let room_mem = room.memory();
    room_mem.set("boundary_walls", boundaries);
}

fn get_or_load_boundary_walls(room: &Room, graph: &Graph) -> Vec<Node> {
    info!("Loading boundaries");
    let room_mem = room.memory();
    match room_mem.get::<Vec<Node>>("boundary_walls").unwrap() {
        None => get_boundary_walls(room, graph),
        Some(boundaries) => boundaries,
    }
}

fn show_boundary_walls(room: &Room, graph: &Graph) {
    let boundaries = get_or_load_boundary_walls(room, graph);
    let room_visual = room.visual();

    boundaries.iter().for_each(|wall| {
        let x = wall % 50;
        let y = wall.div_euclid(50);
        room_visual.circle(x as f32, y as f32, None);
    });
}
