use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::iter::FromIterator;

pub fn main() {}

#[derive(Clone)]
struct Node<NodeName> {
    name: NodeName,
    height: i32,
    excess: i32,
}

impl<NodeName: std::marker::Copy> Node<NodeName> {
    fn new(name: NodeName) -> Node<NodeName> {
        Node {
            name,
            height: 0,
            excess: 0,
        }
    }

    fn name(&self) -> NodeName {
        self.name
    }

    fn height(&self) -> i32 {
        self.height
    }

    fn set_height(&mut self, height: i32) {
        self.height = height;
    }

    fn excess(&self) -> i32 {
        self.excess
    }

    fn set_excess(&mut self, excess: i32) {
        self.excess = excess;
    }
}

#[derive(Clone)]
struct Edge<NodeName> {
    to: NodeName,
    from: NodeName,
    flow: i32,
    capacity: i32,
}

impl<NodeName> Edge<NodeName> {
    fn to(&self) -> &NodeName {
        &self.to
    }

    fn from(&self) -> &NodeName {
        &self.from
    }

    fn flow(&self) -> i32 {
        self.flow
    }

    fn set_flow(&mut self, flow: i32) {
        self.flow = flow;
    }

    fn capacity(&self) -> i32 {
        self.capacity
    }

    fn set_capacity(&mut self, capacity: i32) {
        self.capacity = capacity;
    }
}

#[derive(Clone)]
pub struct Graph<NodeName> {
    // nodes[1] = Node 1
    nodes: HashMap<NodeName, Node<NodeName>>,
    // edges[from][to] = from -> to
    edges: HashMap<NodeName, HashMap<NodeName, Edge<NodeName>>>,
    sink: NodeName,
    source: NodeName,
}

impl<
        NodeName: std::hash::Hash + std::cmp::Eq + std::fmt::Display + std::marker::Copy + std::clone::Clone,
    > Graph<NodeName>
where
    std::vec::Vec<NodeName>: std::iter::FromIterator<NodeName>,
{
    pub fn source(&self) -> NodeName {
        self.source
    }

    pub fn sink(&self) -> NodeName {
        self.sink
    }

    pub fn new(source: NodeName, sink: NodeName) -> Self {
        let mut nodes = HashMap::new();
        nodes.insert(source, Node::new(source));
        nodes.insert(sink, Node::new(sink));
        Graph {
            nodes,
            edges: HashMap::new(),
            sink,
            source,
        }
    }

    /*
    fn duplicate(original: Graph) -> Graph {
        let copy = Graph::new(original.source, original.sink);
        for (_, node) in original.nodes.iter() {
            copy.add_node(*node.clone());
        }
        for (from, edges_from) in original.edges.iter() {
            for (to, edge) in edges_from.iter() {
                copy.add_edge(*from, *to, edge.capacity, edge.flow);
            }
        }

        copy
    }*/

    pub fn contains_node(&self, node: &NodeName) -> bool {
        self.nodes.contains_key(node)
    }

    pub fn contains_edge(&self, from: &NodeName, to: &NodeName) -> bool {
        match self.edges.get(from) {
            None => false,
            Some(edges_from) => match edges_from.get(to) {
                None => false,
                Some(_) => true,
            },
        }
    }

    pub fn add_node(&mut self, name: &NodeName) {
        if !self.nodes.contains_key(name) {
            let node = Node {
                name: *name,
                height: 0,
                excess: 0,
            };
            self.nodes.insert(*name, node);
        }
    }

    pub fn add_edge(&mut self, from: &NodeName, to: &NodeName, capacity: i32, flow: i32) {
        if self.contains_edge(&from, &to) {
            return;
        }

        let edge = Edge {
            to: *to,
            from: *from,
            capacity,
            flow,
        };

        match self.edges.get_mut(&from) {
            None => {
                let mut edges_from = HashMap::new();
                edges_from.insert(*to, edge);
                self.edges.insert(*from, edges_from);
            }
            Some(edges_from) => {
                edges_from.insert(*to, edge);
            }
        }
    }

    fn get_node(&self, name: &NodeName) -> Option<&Node<NodeName>> {
        self.nodes.get(name)
    }

    fn get_mut_node(&mut self, name: &NodeName) -> Option<&mut Node<NodeName>> {
        self.nodes.get_mut(name)
    }

    fn get_edge(&self, from: &NodeName, to: &NodeName) -> Option<&Edge<NodeName>> {
        match self.edges.get(from) {
            None => None,
            Some(edges_from) => edges_from.get(to),
        }
    }

    fn get_mut_edge(&mut self, from: &NodeName, to: &NodeName) -> Option<&mut Edge<NodeName>> {
        match self.edges.get_mut(from) {
            None => None,
            Some(edges_from) => edges_from.get_mut(to),
        }
    }

    fn get_adjacent_nodes(&self, name: &NodeName) -> Vec<NodeName> {
        match self.edges.get(name) {
            None => vec![],
            Some(edges_from) => edges_from.values().map(|edge| edge.to().clone()).collect(),
        }
    }

    fn capacity(&self, from: &NodeName, to: &NodeName) -> i32 {
        match self.get_edge(from, to) {
            None => 0,
            Some(edge) => edge.capacity(),
        }
    }

    fn flow(&self, from: &NodeName, to: &NodeName) -> i32 {
        match self.get_edge(from, to) {
            None => 0,
            Some(edge) => edge.flow(),
        }
    }

    pub fn residual_capacity(&self, from: &NodeName, to: &NodeName) -> i32 {
        return self.capacity(from, to) - self.flow(from, to);
    }

    fn excess(&self, name: &NodeName) -> i32 {
        match self.get_node(name) {
            None => 0,
            Some(node) => node.excess(),
        }
    }

    fn set_edge_flow(&mut self, from: &NodeName, to: &NodeName, flow: i32) {
        println!("        flow {} -> {} now {}", from, to, flow);
        match self.get_mut_edge(from, to) {
            None => {}
            Some(edge) => edge.set_flow(flow),
        }
    }

    fn set_node_excess(&mut self, name: &NodeName, excess: i32) {
        if let Some(node) = self.get_mut_node(name) {
            node.set_excess(excess);
        }
    }

    fn get_overflowing_node(&self) -> Option<NodeName> {
        self.nodes
            .values()
            .find(|node| {
                let name = node.name();
                // The source and sink don't overflow
                if name == self.sink || name == self.source {
                    return false;
                }
                if node.excess() > 0 {
                    // Only return this node if it has a node to overflow to (ignoring height,
                    // though)
                    let adjacent_nodes = self.get_adjacent_nodes(&name);
                    let target = adjacent_nodes.iter().find(|target_name| {
                        return self.capacity(&name, target_name) > self.flow(&name, target_name);
                    });
                    return match target {
                        None => false,
                        Some(_) => true,
                    };
                }
                // Node isn't overflowing
                false
            })
            .map(|node| node.name)
    }

    pub fn max_flow(&self) -> (i32, Graph<NodeName>) {
        let residual_graph = &mut self.clone();
        // Add the reverse edges to residual graph if they don't already exist
        for (from, edges_from) in self.edges.iter() {
            for (to, _) in edges_from.iter() {
                // Reverse edges don't exist in the real graph so they have no capacity
                residual_graph.add_edge(to, from, 0, 0);
            }
        }
        residual_graph.max_flow_initialize();

        let mut count = 0;

        loop {
            // TODO: This is for debugging
            if count >= 100 {
                println!("!!!! TOO MUCH !!!!");
                break;
            }
            count += 1;
            if let Some(name) = residual_graph.get_overflowing_node() {
                let node = residual_graph
                    .get_node(&name)
                    .expect("Overflowing node doesn't exist");
                println!("{} overflowing with {}", name, node.excess());
                let adjacent_nodes = residual_graph.get_adjacent_nodes(&name);
                let push_target = adjacent_nodes.iter().find(|target_name| {
                    let target = residual_graph
                        .get_node(target_name)
                        .expect("Adjacent node doesn't exist");
                    if node.height() == 1 + target.height() {
                        return residual_graph.capacity(&name, target_name)
                            > residual_graph.flow(&name, target_name);
                    }
                    false
                });
                match push_target {
                    None => {
                        println!("    relable {}", name);
                        residual_graph.max_flow_relabel(&name)
                    }
                    Some(target) => residual_graph.max_flow_push(&name, target),
                }
            } else {
                break;
            }
        }

        println!("done in {}", count);

        let max_flow = residual_graph
            .get_adjacent_nodes(&residual_graph.source)
            .iter()
            .map(|name| residual_graph.flow(&residual_graph.source, name))
            .sum();

        return (max_flow, residual_graph.clone());
    }

    fn max_flow_initialize(&mut self) {
        // Set the height and excess of every node to 0
        for (_, node) in self.nodes.iter_mut() {
            node.set_height(0);
            node.set_excess(0);
        }

        // Set the flow along each edge to 0
        for (_, edges_from) in self.edges.iter_mut() {
            for (_, edge) in edges_from.iter_mut() {
                edge.set_flow(0);
            }
        }

        // Set the source height to the graph order
        let source_name = self.source;
        let order = self.nodes.len() as i32;
        let source = self
            .get_mut_node(&source_name)
            .expect("Graph doesn't contain source");
        source.set_height(order);

        let adjacent_nodes = self.get_adjacent_nodes(&source_name);
        for name in adjacent_nodes.iter() {
            // Add the reverse edge (e.g. edge = u -> v, reverse is v -> u) if it doesn't exist
            self.add_edge(name, &source_name, 0, 0);
            let edge_capacity = self.capacity(&source_name, name);
            self.set_edge_flow(&source_name, name, edge_capacity);
            self.set_edge_flow(name, &source_name, -edge_capacity);
            self.set_node_excess(name, self.flow(&source_name, name));
        }
    }

    fn max_flow_relabel(&mut self, name: &NodeName) {
        let adjacent_nodes = self.get_adjacent_nodes(name);
        let base_height = adjacent_nodes
            .iter()
            .filter(|target_name| self.capacity(name, target_name) > self.flow(name, target_name))
            .map(|node| {
                self.get_node(node)
                    .expect("Adjacent node doesn't exist")
                    .height()
            })
            .min()
            .expect("No adjacent node while relabling");
        let node = self
            .get_mut_node(&name)
            .expect("Node to relabel doesn't exist");
        node.set_height(base_height + 1);
    }

    fn max_flow_push(&mut self, from: &NodeName, to: &NodeName) {
        println!(
            "    push {} {}/{}-> {}",
            from,
            self.flow(from, to),
            self.capacity(from, to),
            to
        );
        assert!(self.excess(from) > 0);
        assert!(self.capacity(from, to) > self.flow(from, to));
        // The change in flow should be whichever is less: the from node's excess or the remaining
        // capacity of the edge from -> to
        let excess = self.excess(from);
        let remaining_capacity = self.capacity(from, to) - self.flow(from, to);
        let change = std::cmp::min(excess, remaining_capacity);
        println!("        pushing {}", change);

        self.set_edge_flow(from, to, self.flow(from, to) + change);
        self.set_edge_flow(to, from, self.flow(to, from) - change);

        self.set_node_excess(from, self.excess(from) - change);
        self.set_node_excess(to, self.excess(to) + change);
    }

    pub fn min_cut_connected_nodes(&self, start: &NodeName) -> Vec<NodeName> {
        // First In First Out queue
        let mut queue = VecDeque::new();
        // Set of discovered nodes
        let mut discovered = HashSet::new();
        discovered.insert(*start);
        queue.push_back(*start);
        while !queue.is_empty() {
            let node = queue.pop_front().expect("Queue unexpectedly empty");
            let adjacent_nodes = self.get_adjacent_nodes(&node);
            adjacent_nodes.iter().for_each(|adjacent| {
                if !discovered.contains(adjacent) && self.residual_capacity(&node, adjacent) > 0 {
                    discovered.insert(*adjacent);
                    queue.push_back(*adjacent)
                }
            });
        }

        // TODO: This might be unnecessary based on how the output is used
        return Vec::from_iter(discovered);
    }
    pub fn min_cut_boundary_nodes(&self, start: &NodeName) -> Vec<NodeName> {
        let mut boundary = HashSet::new();
        let connected = self.min_cut_connected_nodes(start);

        connected.iter().for_each(|node| {
            let adjacent_nodes = self.get_adjacent_nodes(node);
            adjacent_nodes.iter().for_each(|adjacent| {
                if !connected.contains(adjacent) {
                    boundary.insert(*adjacent);
                }
            });
        });

        return Vec::from_iter(boundary);
    }

    pub fn get_nodes_with_edges_less_than(&self, count: usize) -> Vec<&NodeName> {
        self.edges
            .iter()
            .filter(|(_, edges_from)| edges_from.len() < count)
            .map(|(name, _)| name)
            .collect()
    }

    pub fn distance_transform(&self, boundaries: &Vec<&NodeName>) -> HashMap<NodeName, usize> {
        // First In First Out queue
        let mut queue = VecDeque::new();
        boundaries.iter().for_each(|node| queue.push_back(**node));
        // Set of discovered nodes
        let mut discovered = HashSet::new();

        let mut distance = 0;
        let mut distances: HashMap<NodeName, usize> = HashMap::new();

        // The number of nodes with the previous distance
        let mut previous_class_size = queue.len();
        // The number of nodes with this distance
        let mut current_class_size = 0;

        // BFS starting with boundary nodes
        while !queue.is_empty() {
            if current_class_size == previous_class_size {
                previous_class_size = current_class_size;
                current_class_size = 0;
                distance += 1;
            }
            let node = queue.pop_front().expect("Queue unexpectedly empty");
            let adjacent_nodes = self.get_adjacent_nodes(&node);
            adjacent_nodes.iter().for_each(|adjacent| {
                if !discovered.contains(adjacent) && self.residual_capacity(&node, adjacent) > 0 {
                    discovered.insert(*adjacent);
                    queue.push_back(*adjacent);
                    distances.insert(*adjacent, distance);
                    current_class_size += 1;
                }
            });
        }

        return distances;
    }

    pub fn get_all_edges(&self) -> Vec<(NodeName, NodeName)> {
        let mut edges = Vec::new();
        for (from, edges_from) in self.edges.iter() {
            for (to, _) in edges_from.iter() {
                edges.push((*from, *to));
            }
        }
        edges
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn parse_adrianhaarbach_graph(graph: &str) -> Graph<i32> {
        let mut vertices = -1;
        let mut edges = Vec::<[i32; 3]>::new();
        graph.lines().for_each(|line| {
            if line.starts_with("n") {
                vertices += 1;
            } else if line.starts_with("e") {
                let mut edge: [i32; 3] = [0, 0, 0];
                let parsed: Vec<i32> = line
                    .replace("e ", "")
                    .split_ascii_whitespace()
                    .map(|part| std::str::FromStr::from_str(part).unwrap())
                    .collect();
                edge[0] = parsed[0];
                edge[1] = parsed[1];
                edge[2] = parsed[2];
                edges.push(edge);
            }
        });
        let mut graph = Graph::new(0, vertices);
        for i in 1..vertices {
            graph.add_node(&i);
        }
        for edge in edges.iter() {
            // These are directed graphs
            graph.add_edge(&edge[0], &edge[1], edge[2], 0);
        }
        return graph;
    }

    #[test]
    fn test_graph_max_flow_empty() {
        let graph = Graph::new(0, 1);
        let (max, _) = graph.max_flow();
        assert_eq!(max, 0);
    }

    #[test]
    fn test_graph_max_flow_1() {
        let graph = parse_adrianhaarbach_graph(
            "n
n
n
n
e 0 1 2
e 0 2 4
e 1 2 3
e 2 3 5
e 1 3 1",
        );
        let (max, _) = graph.max_flow();
        assert_eq!(max, 6);
    }

    #[test]
    fn test_graph_max_flow_2() {
        let graph = parse_adrianhaarbach_graph(
            "n
n
n
n
n
n
e 0 1 16
e 0 2 13
e 1 3 12
e 2 1 4
e 2 4 14
e 3 2 9
e 3 5 20
e 4 3 7
e 4 5 4",
        );
        let (max, _) = graph.max_flow();
        assert_eq!(max, 23);
    }

    #[test]
    fn test_graph_max_flow_3() {
        let graph = parse_adrianhaarbach_graph(
            "n
n
n
n
n
n
n
n
e 3 5 8
e 3 4 20
e 4 5 1
e 2 3 26
e 1 4 13
e 1 3 10
e 0 2 1
e 0 1 38
e 1 2 8
e 4 7 7
e 5 7 7
e 3 7 1
e 6 7 27
e 3 6 24
e 0 6 2
e 4 2 2",
        );
        let (max, _) = graph.max_flow();
        assert_eq!(max, 31);
    }

    #[test]
    fn test_graph_get_connected_1() {
        let graph = parse_adrianhaarbach_graph(
            "n
n
n
n
n
n
e 0 1 16
e 0 2 13
e 1 3 12
e 2 1 4
e 2 4 14
e 3 2 9
e 3 5 20
e 4 3 7
e 4 5 4",
        );
        let (_, residual_graph) = graph.max_flow();
        // Node 0 is the source
        let mut connected = residual_graph.min_cut_connected_nodes(&0);
        // For testing, to verify
        connected.sort();
        assert_eq!(connected, vec![0, 1, 2, 4]);
    }

    #[test]
    fn test_graph_get_connected_2() {
        let graph = parse_adrianhaarbach_graph(
            "n
n
n
n
n
n
e 0 1 16
e 0 2 13
e 1 3 12
e 2 1 4
e 2 4 14
e 3 2 9
e 3 5 20
e 4 3 7
e 4 5 4",
        );
        let (_, residual_graph) = graph.max_flow();
        // Node 0 is the source
        let mut connected = residual_graph.min_cut_boundary_nodes(&0);
        // For testing, to verify
        connected.sort();
        assert_eq!(connected, vec![3, 5]);
    }
}
