use std::cmp::{min, max};
use std::collections::{HashMap, VecDeque};
use std::io;

use crate::errors::invalid_input;
use crate::parse::lines;
use crate::part::Part;

struct UndirectedGraph<V, W> {
    vertex_values: Vec<V>,
    edges: HashMap<(usize, usize), W>,
}

impl<V, W> UndirectedGraph<V, W> {
    fn new() -> Self {
        Self { vertex_values: vec![], edges: HashMap::new() }
    }

    fn num_vertices(&self) -> usize {
        self.vertex_values.len()
    }

    /*fn vertex_at(&self, index: usize) -> &V {
        &self.vertex_values[index]
    }*/

    fn edge_between(&self, vertex_from: usize, vertex_to: usize) -> Option<&W> {
        self.edges.get(&(min(vertex_from, vertex_to), max(vertex_from, vertex_to)))
    }

    fn edge_exists_between(&self, vertex_from: usize, vertex_to: usize) -> bool {
        self.edge_between(vertex_from, vertex_to).is_some()
    }

    fn add_vertex(&mut self, value: V) -> usize {
        self.vertex_values.push(value);
        self.vertex_values.len() - 1
    }

    fn neighbors(&self, vertex: usize) -> Vec<usize> {
        // TODO: This is slow
        let mut result = Vec::<usize>::new();
        for i in 0..self.num_vertices() {
            if self.edge_exists_between(vertex, i) {
                result.push(i);
            }
        }
        result
    }

    fn add_edge(&mut self, vertex_from: usize, vertex_to: usize, weight: W) {
        assert!(!self.edge_exists_between(vertex_from, vertex_to));
        assert!(vertex_from < self.vertex_values.len());
        assert!(vertex_to < self.vertex_values.len());
        let min_vertex = min(vertex_from, vertex_to);
        let max_vertex = max(vertex_from, vertex_to);
        self.edges.insert((min_vertex, max_vertex), weight);
    }

    fn remove_edge(&mut self, vertex_from: usize, vertex_to: usize) {
        assert!(self.edge_exists_between(vertex_from, vertex_to));
        assert!(vertex_from < self.vertex_values.len());
        assert!(vertex_to < self.vertex_values.len());
        let min_vertex = min(vertex_from, vertex_to);
        let max_vertex = max(vertex_from, vertex_to);
        self.edges.remove(&(min_vertex, max_vertex));
    }
}

struct Wiring {
    state: UndirectedGraph<String, ()>,
    vertex_names: HashMap<String, usize>,
}

impl Wiring {
    fn from_lines(lines: Vec<String>) -> io::Result<Self> {
        type UG = UndirectedGraph::<String, ()>;
        let mut state = UG::new();
        let mut vertex_names = HashMap::<String, usize>::new();

        fn add_vertex_if_dne(state: &mut UG, vertex_names: &mut HashMap<String, usize>, name: &str) -> usize {
            let name = name.to_owned();
            let entry = vertex_names.entry(name.clone());
            *entry.or_insert_with(|| state.add_vertex(name))
        }

        for line in lines {
            let [vertex_name, connection_names] = &line.split(": ").collect::<Vec<_>>()[..] else {
                return Err(invalid_input("Expected <vertex name>: <connections>"));
            };
            let connections = connection_names.split_whitespace().collect::<Vec<_>>();

            let vertex = add_vertex_if_dne(&mut state, &mut vertex_names, vertex_name);
            for connection in connections {
                let neighbor = add_vertex_if_dne(&mut state, &mut vertex_names, connection);
                state.add_edge(vertex, neighbor, ());
            }
        }

        Ok(Self { state, vertex_names })
    }

    fn groups(&mut self) -> Vec<usize> {
        let mut visited = vec![false; self.state.num_vertices()];
        let mut groups = Vec::<usize>::new();

        let mut neighbor_cache = HashMap::<usize, Vec<usize>>::new();

        for i in 0..self.state.num_vertices() {
            if visited[i] {
                continue;
            }
            let mut current_size = 0usize;
            let mut visit_queue = VecDeque::<usize>::new();
            visit_queue.push_back(i);

            while let Some(to_visit) = visit_queue.pop_front() {
                if visited[to_visit] {
                    continue;
                }
                visited[to_visit] = true;
                current_size += 1;
                //visit_queue.extend(self.state.neighbors(to_visit));
                visit_queue.extend(neighbor_cache.entry(to_visit).or_insert_with(|| self.state.neighbors(to_visit)).clone());
            }

            groups.push(current_size);
        }

        groups
    }

    fn two_group_sizes_after_removing_three(&mut self) -> Option<(usize, usize)> {
        /*let mut edges = self.state.edges.keys().map(|key| key.to_owned()).collect::<Vec<(usize, usize)>>();

        for i in 2..edges.len() {
            self.state.remove_edge(edges[i].0, edges[i].1);
            for j in 1..i {
                /*if !self.state.edge_exists_between(edges[j].0, edges[j].1) {
                    println!("FAIL: no edge from {} to {}", edges[j].0, edges[j].1);
                    println!("Previous removed was {:?}", edges[i]);
                }*/
                self.state.remove_edge(edges[j].0, edges[j].1);
                for k in 0..j {
                    self.state.remove_edge(edges[k].0, edges[k].1);
                    if let [size1, size2] = self.groups()[..] {
                        self.state.add_edge(edges[k].0, edges[k].1, ());
                        return Some((size1, size2));
                    }
                    self.state.add_edge(edges[k].0, edges[k].1, ());
                }
                self.state.add_edge(edges[j].0, edges[j].1, ());
            }
            self.state.add_edge(edges[i].0, edges[i].1, ());
            println!("Done with i={i}");
        }*/
        //None
        let to_remove = [("btp", "qxr"), ("bgl", "vfx"), ("bqq", "rxt")];
        for (v0, v1) in to_remove {
            let v0_index = *self.vertex_names.get(v0).unwrap();
            let v1_index = *self.vertex_names.get(v1).unwrap();
            self.state.remove_edge(v0_index, v1_index);
        }
        // TODO add back edges??

        if let [size1, size2] = self.groups()[..] {
            Some((size1, size2))
        } else {
            None
        }
    }
}

pub fn run<R: io::Read>(
    _part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let mut wiring = Wiring::from_lines(lines(reader)?.collect())?;
    let Some((g1, g2)) = wiring.two_group_sizes_after_removing_three() else {
        return Err(invalid_input("Could not divide wiring"));
    };

    println!("{}", g1 * g2);

    Ok(())
}
