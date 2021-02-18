use crate::graph::Graph;
use log::*;
use screeps::objects::Room;

pub fn plan_walls(room: &Room) {
    let protected = rectangle_of_pos_indices(50 * 0 + 0, 50 * 23 + 34);
    let exits = room.find(screeps::constants::find::EXIT);
    let sinks: Vec<usize> = exits
        .iter()
        .map(|position| (position.y() * 50 + position.x()) as usize)
        .collect();
    let graph = terrain_to_graph(room, &protected, &sinks);
    let room_mem = room.memory();
    let wall_spots = match room_mem.get::<Vec<usize>>("wall_spots").unwrap() {
        None => {
            let protected = rectangle_of_pos_indices(50 * 0 + 0, 50 * 23 + 34);
            let exits = room.find(screeps::constants::find::EXIT);
            let sinks: Vec<usize> = exits
                .iter()
                .map(|position| (position.y() * 50 + position.x()) as usize)
                .collect();
            let (_, residual_graph) = graph.max_flow();
            let wall_spots = residual_graph.min_cut_boundary_nodes(&protected[0]);
            room_mem.set(
                "wall_spots",
                wall_spots
                    .iter()
                    .map(|spot| *spot as u32)
                    .collect::<Vec<u32>>(),
            );
            wall_spots
        }
        Some(wall_spots) => wall_spots,
    };
    /*
    let room_visual = room.visual();
    let mut style: screeps::objects::CircleStyle = Default::default();
    style = style.radius(0.5).fill("#000000");

    wall_spots.iter().for_each(|spot| {
        let x = spot % 50;
        let y = spot.div_euclid(50);
        room_visual.circle(x as f32, y as f32, Some(style.clone()));
    });
    */
    show_distance_transform(room, &graph);
}

fn terrain_to_graph(room: &Room, sources: &Vec<usize>, sinks: &Vec<usize>) -> Graph<usize> {
    let mut tiles = Vec::new();
    let terrain = room.get_terrain().get_raw_buffer();
    for (i, tile_type) in terrain.iter().enumerate() {
        // tile is a wall if tile_type & TERRAIN_MASK_WALL is 1u8
        if !(tile_type & screeps::constants::TERRAIN_MASK_WALL == 1) {
            tiles.push(i);
        }
    }
    let source = sources[0];
    let sink = sinks[0];
    let mut graph = Graph::new(source, sink);
    let mut added = 0;
    tiles.iter().for_each(|tile| {
        if !sources.contains(tile) && !sinks.contains(tile) {
            graph.add_node(tile);
            added += 1;
        }
    });
    info!("added {} tiles", added);
    let mut sink_count = 0;
    tiles.iter().for_each(|tile| {
        if !sources.contains(tile) && !sinks.contains(tile) {
            let (tile_x, tile_y) = (tile % 50, tile.div_euclid(50));
            let y_range = if tile_y == 0 {
                0..=1
            } else if tile_y == 49 {
                48..=49
            } else {
                (tile_y - 1)..=(tile_y + 1)
            };
            for y in y_range {
                let x_range = if tile_x == 0 {
                    0..=1
                } else if tile_x == 49 {
                    48..=49
                } else {
                    (tile_x - 1)..=(tile_x + 1)
                };
                for x in x_range {
                    let neighbor = y * 50 + x;
                    if !tiles.contains(&neighbor) {
                        // If neighbor is a wall tile, go to the next neighbor
                        continue;
                    }
                    // Note: duplicate edges won't be added by graph.add_edge
                    if sources.contains(&neighbor) {
                        // Edge from tile -> source with capacity 1 and 0 flow
                        graph.add_edge(tile, &source, 1, 0);
                        // Add reverse edge now since neighbor isn't in the graph
                        graph.add_edge(&source, tile, 1, 0);
                    } else if sinks.contains(&neighbor) {
                        sink_count += 1;
                        // Edge from tile -> sink with capacity 1 and 0 flow
                        graph.add_edge(tile, &sink, 1, 0);
                        // Add reverse edge now since neighbor isn't in the graph
                        graph.add_edge(&sink, tile, 1, 0);
                    } else if graph.contains_node(&neighbor) {
                        // Edge from tile -> neighbor with capacity 1 and 0 flow
                        graph.add_edge(tile, &neighbor, 1, 0);
                    }
                }
            }
        }
    });
    info!("Added {} sinks", sink_count);
    return graph;
}

fn get_graph_and_save(room: Room, protected: &Vec<usize>) -> Graph<usize> {
    // Get the exists in the room
    let exits = room.find(screeps::constants::find::EXIT);
    let sinks: Vec<usize> = exits
        .iter()
        .map(|position| (position.y() * 50 + position.x()) as usize)
        .collect();
    // Create the graph
    let graph = terrain_to_graph(room, protected, &sinks);

    // Save the graph to room memory
    let room_mem = room.memory();
    room_mem.set(
        "graph",
        wall_spots
            .iter()
            .map(|spot| *spot as u32)
            .collect::<Vec<u32>>(),
    );

    graph
}

fn rectangle_of_pos_indices(top_left: usize, bottom_right: usize) -> Vec<usize> {
    let mut positions = Vec::new();

    for y in top_left.div_euclid(50)..=bottom_right.div_euclid(50) {
        for x in (top_left % 50)..=(bottom_right % 50) {
            positions.push(50 * y + x);
        }
    }

    positions
}

fn surrounding_pos_indices(index: usize) -> Vec<usize> {
    // 51 = 50 * 1 + 1
    return rectangle_of_pos_indices(index - 51, index + 51);
}

fn get_distance_transform(graph: &Graph<usize>) -> Vec<usize> {
    let boundaries = graph.get_nodes_with_edges_less_than(8);
    let distance_map = graph.distance_transform(&boundaries);
    let mut distances = Vec::new();
    distances.resize(distance_map.len(), usize::MAX);
    distance_map.iter().for_each(|(name, distance)| {
        distances[*name] = *distance;
    });
    distances
}

fn save_distance_transform(room: &Room, distances: &Vec<usize>) {
    room.memory().set(
        "distance_transform",
        distances
            .iter()
            .map(|spot| *spot as u32)
            .collect::<Vec<u32>>(),
    );
}

fn load_distance_transform(room: &Room) -> Option<Vec<usize>> {
    room.memory()
        .get::<Vec<usize>>("distance_transform")
        .unwrap()
}

fn show_distance_transform(room: &Room, graph: &Graph<usize>) {
    let distance_transform = match load_distance_transform(room) {
        None => {
            let distance_transform = get_distance_transform(graph);
            save_distance_transform(room, &distance_transform);
            distance_transform
        }
        Some(distance_transform) => distance_transform,
    };
    let room_visual = room.visual();
    let mut style: screeps::objects::TextStyle = Default::default();

    distance_transform
        .iter()
        .enumerate()
        .for_each(|(i, distance)| {
            let x = i % 50;
            let y = i.div_euclid(50);
            room_visual.text(
                x as f32,
                y as f32,
                String::from(format!("{}", distance)),
                Some(style.clone()),
            );
        });
}
