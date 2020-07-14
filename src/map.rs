use rltk::{ RGB, Rltk, RandomNumberGenerator };
use super::{Rect};
use std::cmp::{max, min};


#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall, Floor
}

pub fn xy_idx(x: i32, y: i32) -> usize {
    (y as usize * 80) + x as usize
}

/// Makes a map with solid boundaries and 400 randomly placed walls. No guarantees that it won't
/// look awful.
pub fn new_map_test() -> Vec<TileType> {
    let mut map = vec![TileType::Floor; 80*50];

    // Make the boundaries walls
    for x in 0..80 {
        map[xy_idx(x, 0)] = TileType::Wall;
        map[xy_idx(x, 49)] = TileType::Wall;
    }
    for y in 0..50 {
        map[xy_idx(0, y)] = TileType::Wall;
        map[xy_idx(79, y)] = TileType::Wall;
    }

    // Now we'll randomly splat a bunch of walls. It won't be pretty, but it's a decent illustration.
    // First, obtain the thread-local RNG:
    let mut rng = rltk::RandomNumberGenerator::new();

    for _i in 0..400 {
        let x = rng.roll_dice(1, 79);
        let y = rng.roll_dice(1, 49);
        let idx = xy_idx(x, y);
        if idx != xy_idx(40, 25) {
            map[idx] = TileType::Wall;
        }
    }

    map
}

fn apply_room_to_map(room : &Rect, map: &mut [TileType]) {
    for y in room.y1 +1 ..= room.y2 {
        for x in room.x1 + 1 ..= room.x2 {
            map[xy_idx(x, y)] = TileType::Floor;
        }
    }
}

fn apply_horizontal_tunnel(map: &mut [TileType], x1:i32, x2:i32, y:i32) {
    for x in min(x1,x2) ..= max(x1,x2) {
        let idx = xy_idx(x, y);
        if idx > 0 && idx < 80*50 {
            map[idx as usize] = TileType::Floor;
        }
    }
}

fn apply_vertical_tunnel(map: &mut [TileType], y1:i32, y2:i32, x:i32) {
    for y in min(y1,y2) ..= max(y1,y2) {
        let idx = xy_idx(x, y);
        if idx > 0 && idx < 80*50 {
            map[idx as usize] = TileType::Floor;
        }
    }
}

pub fn new_map_clustered_rooms(dir: i32, room_count: i32) -> (Vec<Rect>, Vec<TileType>) {
    let mut map = vec![TileType::Wall; 80*50];

    let mut rooms : Vec<Rect> = Vec::new();
    const MIN_SIZE : i32 = 6;
    const MAX_SIZE : i32 = 10;

    let row_size : i32 = (room_count as f64).sqrt() as i32 + 1;
    let mut rng = RandomNumberGenerator::new();

    //apply_room_to_map(&base_room, &mut map);
    let mut base_room = Rect::new(1,1,1,1);
    for _i in 1..row_size { 
        
        let w = rng.range(MIN_SIZE, MAX_SIZE);
        let h = rng.range(MIN_SIZE, MAX_SIZE);
        
        if rooms.len() < 1 {
            //start the first room
            let x = 10-w/2;
            let y = 5;
            base_room = Rect::new(x, y, w, h);
        }
        else{
            //start a new row of rooms
            let (x, y) = side_switcher(2, base_room); 
            base_room = Rect::new(x, y+1, w, h);
            apply_vertical_tunnel(&mut map, rooms[0].center().1, 
                    base_room.center().1, base_room.center().0);
        }
        rooms.push(base_room);
        apply_room_to_map(&base_room, &mut map);
        let mut border_room = base_room;
        println!("{:?}", (border_room.x1, border_room.x2, "B"));
        //build a row of rooms
        for _j in 1..row_size{
            let (x, y) = side_switcher(3, border_room);
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            print!("{:?}", (x, y, w, h));
            let border_room = Rect::new(x, y, w, h);

            let mut ok = true;
            for other_room in rooms.iter() {
                if border_room.intersect(other_room) { ok = false }
                
            }
            println!("{:?}", (border_room.x1, border_room.x2, ok));
            if ok {
                apply_room_to_map(&border_room, &mut map);
                if !rooms.is_empty() {
                    let (new_x, new_y) = border_room.center();
                    let (prev_x, prev_y) = rooms[rooms.len()-1].center();
                    if rng.range(0,2) == 1 {
                        apply_horizontal_tunnel(&mut map, prev_x, new_x, prev_y);
                        apply_vertical_tunnel(&mut map, prev_y, new_y, new_x);
                    } else {
                        apply_vertical_tunnel(&mut map, prev_y, new_y, prev_x);
                        apply_horizontal_tunnel(&mut map, prev_x, new_x, new_y);
                    }
                }

                //rooms.push(new_room);
            }
        }
    }

    (rooms, map)
}

pub fn side_switcher(side: i32, room:Rect) -> (i32,i32) {
    let mut rng = RandomNumberGenerator::new();
    match side {
        0 => return (rng.range(room.x1-3,room.x2+3), room.y1-1),
        1 =>   return (room.x1-1, rng.range(room.y1-3,room.y2+3)),
        2 => return (rng.range(room.x1-3,room.x1+3), room.y2+1),
        3 =>    return (room.x2+1, rng.range(room.y1-3,room.y1+3)),
        _ => (1,1)
    }
}

pub fn draw_map(map: &[TileType], ctx : &mut Rltk) {
    let mut y = 0;
    let mut x = 0;
    for tile in map.iter() {
        // Render a tile depending upon the tile type
        match tile {
            TileType::Floor => {
                ctx.set(x, y, RGB::from_f32(0.5, 0.5, 0.5), RGB::from_f32(0., 0., 0.), rltk::to_cp437('.'));
            }
            TileType::Wall => {
                ctx.set(x, y, RGB::from_f32(0.0, 1.0, 0.0), RGB::from_f32(0., 0., 0.), rltk::to_cp437('#'));
            }
        }

        // Move the coordinates
        x += 1;
        if x > 79 {
            x = 0;
            y += 1;
        }
    }
}