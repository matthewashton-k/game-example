
use rerun::components::GraphNode;
use rerun::{RecordingStream, RecordingStreamBuilder, GraphNodes, AsComponents, Component, Color};

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

/// need to pass the graph ids and graph colors to keep track of the graphs state, and everything that has been visited so far
/// change update_fields.with stuff to take in the whole list of node ids and also node edges, node colors
fn collect_gold(rec: &RecordingStream, map: &Vec<Vec<Tile>>, start: (usize, usize)) -> usize {
    // also somehow use GraphEdges to show the traversal?
    
    let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];
    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();
    let mut gold_count = 0;
    
    let start_id = format!("{}_{}",start.1, start.0);
    
    std::thread::sleep(Duration::from_secs_f32(0.5));
    rec.log("board", 
        &GraphNodes::update_fields().with_node_ids(vec![start_id]).with_colors(vec![Color::from_rgb(255,0,0)])
    ).unwrap();

    queue.push_back(start);
    visited.insert(start);
    
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
                    std::thread::sleep(Duration::from_secs_f32(0.5));
                    match map[ny][nx] {
                        Tile::Floor | Tile::Gold => {
                            let id = format!("{}_{}",ny, nx);
                            rec.log("board", 
                                &GraphNodes::update_fields().with_node_ids(vec![id]).with_colors(vec![Color::from_rgb(255,0,0)])
                            ).unwrap();
                            queue.push_back((nx, ny));
                            visited.insert((nx, ny));
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
    for (i,row) in map.iter().enumerate() {
        for (j,tile) in row.iter().enumerate() {
            ids.push(format!("{i}_{j}"));
            labels.push(tile.to_char().to_string());
            positions.push((j as f32 * 100.,i as f32*100.));
            print!("{:?} ", tile.to_char());
        }
        println!();
    }
    
    let default_color = Color::from_rgb(255, 255, 255);
    let colors = vec![default_color; ids.len()];
    
    rec.log_static(
        "board",
        &GraphNodes::new(ids)
            .with_positions(positions)
            .with_labels(labels)
            .with_colors(colors),
    )?;
    
    if let Some(start) = find_player(&map) {
        let gold_collected = collect_gold(&rec, &map, start);
        println!("Maximum gold collected: {}", gold_collected);
    } else {
        println!("No player found on the map.");
    }
    Ok(())
}
