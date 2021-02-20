use crate::graph::Graph;
use log::*;
use screeps::objects::Room;

pub fn plan_walls(room: &Room) {
    info!("Planning walls");
    let graph = load_or_get_graph(room);
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
    // show_distance_transform(room, &graph);
}

fn terrain_to_graph(room: &Room, sources: &Vec<u32>, sinks: &Vec<u32>) -> Graph<u32> {
    let mut tiles = Vec::new();
    let terrain = room.get_terrain().get_raw_buffer();
    for (i, tile_type) in terrain.iter().enumerate() {
        // tile is a wall if tile_type & TERRAIN_MASK_WALL is 1u8
        if !(tile_type & screeps::constants::TERRAIN_MASK_WALL == 1) {
            tiles.push(i as u32);
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

fn load_or_get_graph(room: &Room) -> Graph<u32> {
    let protected = vec![18 + 12 * 50];
    match load_graph(room) {
        None => get_graph_and_save(room, &protected),
        Some(graph) => graph,
    }
}

fn get_graph_and_save(room: &Room, protected: &Vec<u32>) -> Graph<u32> {
    info!("Creating graph");
    // Get the exists in the room
    let mut sinks = Vec::new();
    room.find(screeps::constants::find::EXIT)
        .iter()
        .map(|position| (position.y() * 50 + position.x()) as u32)
        .for_each(|exit| {
            surrounding_pos_indices(exit)
                .iter()
                .for_each(|position| sinks.push(*position))
        });
    // Create the graph
    let graph = terrain_to_graph(room, protected, &sinks);

    // Save the graph to room memory
    save_graph(room, &graph);

    graph
}

fn save_graph(room: &Room, graph: &Graph<u32>) {
    let room_mem = room.memory(); /*
                                  let source_sink = vec![(graph.source(), graph.sink())];
                                  let graph_vec = source_sink
                                      .iter()
                                      .chain(graph.get_all_edges().iter())
                                      .map(|(from, to)| vec![*from as u32, *to as u32])
                                      .collect::<Vec<Vec<u32>>>();*/
    room_mem.set("graph", graph);
}

fn load_graph(room: &Room) -> Option<Graph<u32>> {
    info!("Loading graph...");
    let room_mem = room.memory();
    info!("room_mem");
    let get_graph = room_mem.get::<Graph<u32>>("graph");
    info!("got get_graph");
    match get_graph {
        Ok(graph_option) => {
            info!("Okay");
            match graph_option {
                None => return None,
                Some(graph) => {
                    info!("Somewhat loaded graph");
                    /*
                    let mut edges: Vec<(usize, usize)> =
                        edges_memory.iter().map(|edge| (edge[0], edge[1])).collect();
                    let (source, sink) = edges.remove(0);
                    let mut graph = Graph::new(source, sink);
                    for (from, to) in edges {
                        // add_node won't add duplicate nodes
                        graph.add_node(&from);
                        graph.add_node(&to);
                        // Add edge from -> to with capacity 1 and flow 0
                        graph.add_edge(&from, &to, 1, 0);
                    }*/
                    Some(graph)
                }
            }
        }
        Err(error) => {
            info!("Err");
            info!("{:?}", error);
            None
        }
    }
}

fn rectangle_of_pos_indices(top_left: u32, bottom_right: u32) -> Vec<u32> {
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

fn surrounding_pos_indices(index: u32) -> Vec<u32> {
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

fn get_distance_transform(graph: &Graph<u32>) -> Vec<u32> {
    let boundaries = graph.get_nodes_with_edges_less_than(8);
    let distance_map = graph.distance_transform(&boundaries);
    let mut distances = Vec::new();
    // 2500 = 50*50 which is the room size
    distances.resize(2500, u32::MAX);
    distance_map.iter().for_each(|(name, distance)| {
        distances[*name as usize] = *distance as u32;
    });
    distances
}

fn save_distance_transform(room: &Room, distances: &Vec<u32>) {
    room.memory().set(
        "distance_transform",
        distances
            .iter()
            .map(|spot| *spot as u32)
            .collect::<Vec<u32>>(),
    );
}

fn load_distance_transform(room: &Room) -> Option<Vec<u32>> {
    room.memory().get::<Vec<u32>>("distance_transform").unwrap()
}

fn show_distance_transform(room: &Room, graph: &Graph<u32>) {
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

    /*
    distance_transform
        .iter()
        .enumerate()
        .for_each(|(i, distance)| {
            if *distance < 50 {
                let mut style = style.clone();
                if *distance < colors.len() {
                    style = style.color(colors[*distance]);
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
        });*/
}
