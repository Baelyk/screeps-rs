use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::collections::VecDeque;
use std::iter::FromIterator;

const ROOM_SIZE: usize = 2500;
/// The Node is the index of a tile, which is x + y * 50
pub type Node = u16;
type FlowCapacityValue = i8;
type Capacity = [[FlowCapacityValue; ROOM_SIZE]; ROOM_SIZE];
type Flow = [[FlowCapacityValue; ROOM_SIZE]; ROOM_SIZE];
type Excess = [FlowCapacityValue; ROOM_SIZE];
type HeightValue = u16;
type Height = [HeightValue; ROOM_SIZE];
pub type DistanceTransform = [u8; ROOM_SIZE];

#[derive(Serialize, Deserialize)]
pub struct ScreepsGraph {
    /// Tiles making up the source
    sources: Vec<Node>,
    /// Tiles making up the sink
    sinks: Vec<Node>,
    /// Tiles *not* part of the graph, i.e. walls
    walls: Vec<Node>,
}
js_serializable!(ScreepsGraph);
js_deserializable!(ScreepsGraph);

impl ScreepsGraph {
    pub fn new(sources: Vec<Node>, sinks: Vec<Node>, walls: Vec<Node>) -> ScreepsGraph {
        return ScreepsGraph {
            sources,
            sinks,
            walls,
        };
    }

    /// Get the node at the provided index.
    /// Returns the first source or sink if the node is part of the source or sink.
    /// Returns None if the node is a wall.
    fn get_node(&self, node: Node) -> Option<Node> {
        if self.sources.contains(&node) {
            return Some(self.sources[0]);
        }
        if self.sinks.contains(&node) {
            return Some(self.sinks[0]);
        }
        if self.walls.contains(&node) {
            return None;
        }
        return Some(node);
    }

    fn get_surrounding_indices(node: Node) -> Vec<Node> {
        let mut neighbors = vec![];
        let node_y = node.div_euclid(50);
        let node_x = node.rem_euclid(50);
        let mut y_minus = 1;
        let mut y_plus = 1;
        let mut x_minus = 1;
        let mut x_plus = 1;
        if node_y == 0 {
            y_minus = 0;
        } else if node_y == 49 {
            y_plus = 0;
        }
        if node_x == 0 {
            x_minus = 0;
        } else if node_x == 49 {
            x_plus = 0;
        }
        for y in (node_y - y_minus)..=(node_y + y_plus) {
            for x in (node_x - x_minus)..=(node_x + x_plus) {
                // Node isn't surrounded by itself
                if y == node_y && x == node_x {
                    continue;
                }
                neighbors.push(x + y * 50);
            }
        }
        return neighbors;
    }

    fn get_all_surrounding_indices(nodes: &Vec<Node>) -> Vec<Node> {
        let mut surrounding = vec![];
        nodes
            .iter()
            .for_each(|node| surrounding.append(&mut Self::get_surrounding_indices(*node)));
        surrounding
    }

    fn get_neighbors(&self, node: Node) -> Vec<Node> {
        // If the node is part of the source or sink, or not in the graph, return no neighbors
        if self.walls.contains(&node) {
            return vec![];
        }

        let surrounding;
        if self.sources.contains(&node) {
            surrounding = Self::get_all_surrounding_indices(&self.sources);
        } else if self.sinks.contains(&node) {
            surrounding = Self::get_all_surrounding_indices(&self.sinks);
        } else {
            surrounding = Self::get_surrounding_indices(node);
        }

        let mut neighbors = vec![];
        for adjacent in surrounding.iter() {
            if let Some(neighbor) = self.get_node(*adjacent) {
                neighbors.push(neighbor);
            }
        }

        neighbors
    }

    /// Create a residual graph through the Push-Relabel max-flow algorithm
    fn create_residual_graph(&self) -> Flow {
        // Initialize push-relable max-flow algorithm
        let source = self.sources[0];
        let mut capacity: Capacity = [[0; ROOM_SIZE]; ROOM_SIZE];
        let mut flow: Flow = [[0; ROOM_SIZE]; ROOM_SIZE];
        let mut excess: Excess = [0; ROOM_SIZE];
        let mut height: Height = [0; ROOM_SIZE];
        // Initialize the source (and its neighbors)
        height[source as usize] =
            (ROOM_SIZE - self.sources.len() - self.sinks.len() - self.walls.len()) as HeightValue;
        let neighbors = self.get_neighbors(source);
        for neighbor in neighbors {
            let amount = capacity[source as usize][neighbor as usize];
            flow[source as usize][neighbor as usize] = amount;
            flow[neighbor as usize][source as usize] = -amount;
            excess[neighbor as usize] = amount;
        }

        loop {
            if let Some(node) = self.max_flow_get_overflowing_node(&excess) {
                let neighbors = self.get_neighbors(node);
                let target = neighbors.iter().find(|target| {
                    let target_index = **target as usize;
                    height[node as usize] == height[target_index] + 1
                        && capacity[node as usize][target_index] > flow[node as usize][target_index]
                });
                match target {
                    Some(target) => {
                        self.max_flow_push(node, *target, &capacity, &mut excess, &mut flow)
                    }
                    None => self.max_flow_relabel(node, &mut height, &capacity, &flow),
                }
            } else {
                break;
            }
        }

        /* // Nodes with flow from > 0
        flow.iter()
            .enumerate()
            .filter(|(_, flow_from)| flow_from.iter().sum::<FlowCapacityValue>() > 0)
            .map(|(node, _)| node as Node)
            .collect()
            */
        flow
    }

    fn max_flow_get_overflowing_node(&self, excess: &Excess) -> Option<Node> {
        if let Some((node, _)) = excess.iter().enumerate().find(|(_, excess)| **excess > 0) {
            Some(node as Node)
        } else {
            None
        }
    }

    fn max_flow_relabel(&self, node: Node, height: &mut Height, capacity: &Capacity, flow: &Flow) {
        // Get the height of the lowest neighbor with residual capacity
        let neighbors = self.get_neighbors(node);
        let base_height = neighbors
            .iter()
            .filter(|neighbor| {
                capacity[node as usize][**neighbor as usize]
                    > flow[node as usize][**neighbor as usize]
            })
            .map(|neighbor| height[*neighbor as usize])
            .min()
            .expect("No neighbors while relabling");
        // Set the height of node to 1 + its lowest neighbor's height
        height[node as usize] = base_height + 1;
    }

    fn max_flow_push(
        &self,
        from: Node,
        to: Node,
        capacity: &Capacity,
        excess: &mut Excess,
        flow: &mut Flow,
    ) {
        let change = std::cmp::min(
            excess[from as usize],
            capacity[from as usize][to as usize] - flow[from as usize][to as usize],
        );
        flow[from as usize][to as usize] += change;
        flow[to as usize][from as usize] -= change;
        excess[from as usize] -= change;
        excess[to as usize] += change;
    }

    fn min_cut_connected_nodes(&self) -> Vec<Node> {
        let flow = self.create_residual_graph();
        // First-in, First-out queue
        let mut queue = VecDeque::new();
        let mut discovered = HashSet::new();

        // Start at the source
        discovered.insert(self.sources[0]);
        queue.push_back(self.sources[0]);

        while !queue.is_empty() {
            let node = queue.pop_front().expect("Queue unexpectedly empty");
            let neighbors = self.get_neighbors(node);
            neighbors.iter().for_each(|neighbor| {
                if !discovered.contains(neighbor) && flow[node as usize][*neighbor as usize] == 0 {
                    discovered.insert(*neighbor);
                    queue.push_back(*neighbor);
                }
            });
        }

        return Vec::from_iter(discovered);
    }

    pub fn min_cut_nodes(&self) -> Vec<Node> {
        let mut boundary = HashSet::new();
        let connected = self.min_cut_connected_nodes();

        connected.iter().for_each(|node| {
            let neighbors = self.get_neighbors(*node);
            neighbors.iter().for_each(|neighbor| {
                if !connected.contains(neighbor) {
                    boundary.insert(*neighbor);
                }
            });
        });

        return Vec::from_iter(boundary);
    }

    pub fn get_outter_nodes(&self) -> Vec<Node> {
        // A node is an outter node if it has < 8 edges or it connects to the sink (or is the sink)
        let mut outters = vec![self.sinks[0]];
        for index in 0..ROOM_SIZE {
            if let Some(node) = self.get_node(index as Node) {
                // Sinks (returned from get_node as sinks[0]) are always outter nodes, but the
                // sink is included by default
                if node != self.sinks[0] {
                    let neighbors = self.get_neighbors(node);
                    if neighbors.contains(&self.sinks[0]) || neighbors.len() < 8 {
                        outters.push(node);
                    }
                }
            }
        }
        return outters;
    }

    pub fn distance_transform(&self, boundaries: &Vec<Node>) -> [u8; ROOM_SIZE] {
        // First-in, first-out queue
        let mut queue = VecDeque::new();
        let mut discovered = HashSet::new();

        boundaries.iter().for_each(|node| {
            queue.push_back(*node);
            discovered.insert(*node);
        });

        let mut distance = 0;
        let mut distances = [u8::MAX; ROOM_SIZE];

        let mut previous_class_size = queue.len();
        let mut current_class_size = 0;

        while !queue.is_empty() {
            if current_class_size == previous_class_size {
                previous_class_size = queue.len();
                current_class_size = 0;
                distance += 1;
            }
            let node = queue.pop_front().expect("Queue unexpectedly empty");
            if distances[node as usize] == u8::MAX {
                distances[node as usize] = distance;
                current_class_size += 1;
            }
            let neighbors = self.get_neighbors(node);
            neighbors.iter().for_each(|neighbor| {
                if !discovered.contains(neighbor) {
                    discovered.insert(*neighbor);
                    queue.push_back(*neighbor);
                }
            });
        }

        return distances;
    }
}
