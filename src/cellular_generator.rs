use super::Rect;
use super::WorldInfo;
use petgraph::Graph;
use rltk::{RandomNumberGenerator, Rltk, RGB};
use std::cmp::{max, min};

pub fn new_map_cellular(max_rooms: u32) {
    let worlds = WorldInfo::default();
    generator_rooms_vector(max_rooms, &worlds);
}

pub fn generator_rooms_vector(max_rooms: u32, worlds: &WorldInfo) -> Vec<Rect> {
    let min_size: u16 = 6;
    let max_size: u16 = 10;
    let mut rng = RandomNumberGenerator::new();
    let mut rect_list: Vec<Rect> = Vec::new();

    for _ in 0..max_rooms {
        let w = rng.range(min_size, max_size);
        let h = rng.range(min_size, max_size);
        let x = rng.range(0, worlds.width);
        let y = rng.range(0, worlds.height);
        let new_room = Rect::new(x, y, w, h);
        rect_list.append(new_room);

    }
    rect_list
    
}
