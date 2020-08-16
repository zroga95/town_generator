use super::Rect;
use super::WorldInfo;
use petgraph::Graph;
use rltk::{RandomNumberGenerator, Rltk, RGB};

pub fn new_map_cellular(max_rooms: u32) {
    let worlds = WorldInfo::default();
    rects = generator_rooms_vector(max_rooms, &worlds);
    let max_distance = u32;
    pairs = check_closeness(max_distance, &rects);
}

pub fn generator_rooms_vector(max_rooms: u32, worlds: &WorldInfo) -> Vec<Rect> {
    let min_size: u32 = 6;
    let max_size: u32 = 10;
    let mut rng = RandomNumberGenerator::new();
    let mut rect_list: Vec<Rect> = Vec::new();

    for _ in 0..max_rooms {
        let w = rng.range(min_size, max_size);
        let h = rng.range(min_size, max_size);
        let x = rng.range(0, worlds.width - max_size as u32 - 1);
        let y = rng.range(0, worlds.height - max_size as u32 - 1);
        let new_room = Rect::new(x, y, w as u32, h as u32);
        rect_list.push(new_room);
    }
    rect_list
}

pub fn check_closeness(max_distance:u32, rect_a: Rect, rects: Vec<Rect>) -> Vec<Rect>{
    let mut rect_pairs: Vec<Rect> = Vec::new();
    for i in rects{
        for j in rects{
            if i!=j && ((rect[j],rect[i])) !in rect_pairs{//probably wrong syntax for "not in"
                if rects[i].distance(rect[j]){
                    rect_pairs.add((rect[i],rect[j]))
                }
            } 
        }
    }

}