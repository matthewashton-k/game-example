use rerun::components::GraphNode;
use rerun::{RecordingStream, RecordingStreamBuilder, GraphNodes, AsComponents, Component, Color, GraphEdges};

impl Tile {
    fn to_char(&self) -> char {
        match self {
            Tile::Player => 'P',
            Tile::Gold => 'G',
            Tile::Trap => 'T',
            Tile::Wall => '#',
            Tile::Floor => '.',
        }
    }
}

use std::io::{self, BufRead};
use std::collections::{VecDeque, HashSet};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Tile {
    Player,
    Gold,
    Trap,
    Wall,
    Floor,
}

impl Tile {
    fn from_char(c: char) -> Option<Self> {
        match c {
            'P' => Some(Tile::Player),
            'G' => Some(Tile::Gold),
            'T' => Some(Tile::Trap),
            '#' => Some(Tile::Wall),
            '.' => Some(Tile::Floor),
            _ => None,
        }
    }
}

fn find_player(map: &Vec<Vec<Tile>>) -> Option<(usize, usize)> {
    for (y, row) in map.iter().enumerate() {
        for (x, &tile) in row.iter().enumerate() {
            if tile == Tile::Player {
                return Some((x, y));
            }
        }
    }
    None
}

fn collect_gold(
    rec: &RecordingStream,
    map: &Vec<Vec<Tile>>,
    start: (usize, usize),
    node_ids: &[String],
    colors: &mut Vec<Color>,
    edges: &mut Vec<(String, String)>,
) -> usize {
    let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];
    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();
    let mut gold_count = 0;
    
    let width = map[0].len();
    let (start_x, start_y) = start;
    let start_index = start_y * width + start_x;
    
    queue.push_back(start);
    visited.insert(start);
    
    // Update start node color
    colors[start_index] = Color::from_rgb(255, 0, 0);
    std::thread::sleep(Duration::from_secs_f32(0.5));
    rec.log("board", &GraphNodes::update_fields()
        .with_colors(colors.clone())
    ).unwrap();
    rec.log("board", 
        &GraphEdges::new(edges.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect::<Vec<(&str,&str)>>())).unwrap();
    
    while let Some((x, y)) = queue.pop_front() {
        if map[y][x] == Tile::Gold {
            gold_count += 1;
        }
        
        for (dx, dy) in &directions {
            let nx = x as isize + dx;
            let ny = y as isize + dy;
            if nx >= 0 && ny >= 0 {
                let nx = nx as usize;
                let ny = ny as usize;
                
                if ny < map.len() && nx < map[ny].len() && !visited.contains(&(nx, ny)) {
                    match map[ny][nx] {
                        Tile::Floor | Tile::Gold => {
                            visited.insert((nx, ny));
                            queue.push_back((nx, ny));
                            
                            // Update neighbor's color
                            let neighbor_index = ny * width + nx;
                            colors[neighbor_index] = Color::from_rgb(255, 0, 0);
                            
                            // Add edge from current node to neighbor
                            let current_id = format!("{}_{}", y, x);
                            let neighbor_id = format!("{}_{}", ny, nx);
                            edges.push((current_id, neighbor_id));
                            
                            // Log the update
                            std::thread::sleep(Duration::from_secs_f32(0.5));
                            rec.log("board", &GraphNodes::update_fields()
                                .with_colors(colors.clone())
                            ).unwrap();
                            rec.log("board", 
                                &GraphEdges::new(edges.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect::<Vec<(&str,&str)>>())).unwrap();
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    
    gold_count
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    let rec = RecordingStreamBuilder::new("board").connect_tcp()?;
    // Read the first line containing W and H
    let first_line = lines.next().unwrap().unwrap();
    let mut parts = first_line.split_whitespace();
    let width: usize = parts.next().unwrap().parse().unwrap();
    let height: usize = parts.next().unwrap().parse().unwrap();
    
    let mut map = Vec::new();
    
    // Read the next H lines containing the map
    for _ in 0..height {
        if let Some(Ok(line)) = lines.next() {
            let row: Vec<Tile> = line.chars().filter_map(Tile::from_char).collect();
            map.push(row);
        }
    }
    
    let mut ids = vec![];
    let mut labels = vec![];
    let mut positions = vec![];
    // Print the parsed map
    println!("\nParsed Map:");
    for (i, row) in map.iter().enumerate() {
        for (j, tile) in row.iter().enumerate() {
            ids.push(format!("{i}_{j}"));
            labels.push(tile.to_char().to_string());
            positions.push((j as f32 * 100., i as f32 * 100.));
            print!("{:?} ", tile.to_char());
        }
        println!();
    }
    
    let default_color = Color::from_rgb(255, 255, 255);
    let mut colors = vec![default_color; ids.len()];
    let mut edges = vec![];
    
    rec.log_static(
        "board",
        &GraphNodes::new(ids.clone())
            .with_positions(positions)
            .with_labels(labels)
    )?;
    
    thread::sleep(Duration::from_secs(10));
    if let Some(start) = find_player(&map) {
        let gold_collected = collect_gold(&rec, &map, start, &ids, &mut colors, &mut edges);
        println!("Maximum gold collected: {}", gold_collected);
    } else {
        println!("No player found on the map.");
    }
    Ok(())
}
