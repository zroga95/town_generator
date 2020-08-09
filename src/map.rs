use super::Rect;
use rltk::{RandomNumberGenerator, Rltk, RGB};
use std::cmp::{max, min};

// TODO: Figure out how to handle out of bounds writes to the Tile vec
// TODO: Make world dims not magic numbers

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum TileType {
    Wall,
    Floor,
}

pub fn xy_idx(x: i32, y: i32) -> usize {
    (y as usize * 80) + x as usize
}

#[allow(unused)]
pub fn idx_to_xy(idx: usize) -> (i32, i32) {
    let x = idx % 80;
    let y = idx / 80;
    (x as i32, y as i32)
}

/// Makes a map with solid boundaries and 400 randomly placed walls. No
/// guarantees that it won't look awful.
#[allow(unused)]
pub fn new_map_test() -> Vec<TileType> {
    let mut map = vec![TileType::Floor; 80 * 50];

    // Make the boundaries walls
    for x in 0..80 {
        map[xy_idx(x, 0)] = TileType::Wall;
        map[xy_idx(x, 49)] = TileType::Wall;
    }
    for y in 0..50 {
        map[xy_idx(0, y)] = TileType::Wall;
        map[xy_idx(79, y)] = TileType::Wall;
    }

    // Now we'll randomly splat a bunch of walls. It won't be pretty, but it's a
    // decent illustration. First, obtain the thread-local RNG:
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

/// Builds floors inside of `room`
fn apply_room_to_map(room: &Rect, map: &mut [TileType]) {
    for y in room.y1 + 1..=room.y2 {
        for x in room.x1 + 1..=room.x2 {
            map[xy_idx(x, y)] = TileType::Floor;
        }
    }
}

/// Builds floors between `x1` and `x2` at `y`
fn apply_horizontal_tunnel(map: &mut [TileType], x1: i32, x2: i32, y: i32) {
    for x in min(x1, x2)..=max(x1, x2) {
        let idx = xy_idx(x, y);
        assert!(idx < 80 * 50);
        map[idx as usize] = TileType::Floor;
    }
}

/// Builds floors between `y1` and `y2` at `x`
fn apply_vertical_tunnel(map: &mut [TileType], y1: i32, y2: i32, x: i32) {
    for y in min(y1, y2)..=max(y1, y2) {
        let idx = xy_idx(x, y);
        assert!(idx < 80 * 50);
        map[idx as usize] = TileType::Floor;
    }
}

pub fn new_map_clustered_rooms(
    #[allow(unused)] dir: i32,
    room_count: i32,
) -> (Vec<Rect>, Vec<TileType>) {
    let mut map = vec![TileType::Wall; 80 * 50];

    let mut rooms: Vec<Rect> = Vec::new();
    const MIN_SIZE: i32 = 6;
    const MAX_SIZE: i32 = 10;

    let row_size: i32 = (room_count as f64).sqrt() as i32;
    let mut rng = RandomNumberGenerator::new();

    // apply_room_to_map(&base_room, &mut map);
    // temporary init
    let mut base_room = Rect::new(-1, -1, -1, -1);
    for _ in 0..row_size {
        let w = rng.range(MIN_SIZE, MAX_SIZE);

        let h = rng.range(MIN_SIZE, MAX_SIZE);

        // Set up the base room for reference later
        if rooms.is_empty() {
            // start the first room
            let x = 10 - w / 2;
            let y = 5;
            base_room = Rect::new(x, y, w, h);
        } else {
            // start a new row of rooms by finding coords for room below previous base
            // room
            let (x, y) = side_switcher(2, base_room);
            let new_room = Rect::new(x, y + 1, w, h);
            apply_vertical_tunnel(
                &mut map,
                // TODO: this causes all the vertical tunnels to be at the same x
                // position. This is fine for now, but lets fix later
                base_room.center().1,
                new_room.center().1,
                new_room.center().0,
            );
            base_room = new_room;
        }
        rooms.push(base_room);
        apply_room_to_map(&base_room, &mut map);

        // Supposed to keep track of the room to the right of the _jth room
        let mut border_room = base_room;
        println!("{:?}", (border_room.x1, border_room.x2, "B"));
        // build a row of rooms
        for _ in 0..row_size {
            let (x, y) = side_switcher(3, border_room);
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            println!("Room to right: {:?}", (x, y, w, h));
            let new_room = Rect::new(x, y, w, h);

            // Dont add a new room if there is overlap between the other rooms
            // TODO: Because `rooms` isn't actually filled with all the rooms, this wont
            // work properly rn
            // TODO: Do we actually want to prevent overlap? Maybe overlap is nice?
            let mut no_overlap = true;
            for other_room in rooms.iter() {
                if new_room.intersect(other_room) {
                    no_overlap = false
                }
            }
            println!("New room overlap? {}", !no_overlap);

            if no_overlap {
                apply_room_to_map(&new_room, &mut map);
                assert!(!rooms.is_empty());

                let (new_x, new_y) = new_room.center();
                // TODO: Fix pushing onto `rooms`! :P
                let (prev_x, prev_y) = rooms.last().unwrap().center();
                // 50-50 chance of what order the tunnels get built in
                if rng.range(0, 2) == 1 {
                    apply_horizontal_tunnel(&mut map, prev_x, new_x, prev_y);
                    apply_vertical_tunnel(&mut map, prev_y, new_y, new_x);
                } else {
                    apply_vertical_tunnel(&mut map, prev_y, new_y, prev_x);
                    apply_horizontal_tunnel(&mut map, prev_x, new_x, new_y);
                }

                rooms.push(new_room)
            }

            border_room = new_room;
        }
    }
    assert!(rooms.len() <= room_count as usize);

    (rooms, map)
}

/// Returns the coords for the next room, given a room and the side to build on
// TODO: Make `side` an enum
pub fn side_switcher(side: i32, room: Rect) -> (i32, i32) {
    let mut rng = RandomNumberGenerator::new();
    match side {
        0 => (rng.range(room.x1 - 3, room.x2 + 3), room.y1 - 1), // up
        1 => (room.x1 - 1, rng.range(room.y1 - 3, room.y2 + 3)), // left
        2 => (rng.range(room.x1 - 3, room.x1 + 3), room.y2 + 1), // below
        3 => (room.x2 + 1, rng.range(room.y1 - 3, room.y1 + 3)), // right
        _ => unreachable!(),
    }
}

pub fn draw_map(map: &[TileType], ctx: &mut Rltk) {
    let mut y = 0;
    let mut x = 0;
    for tile in map.iter() {
        // Render a tile depending upon the tile type
        match tile {
            TileType::Floor => {
                ctx.set(
                    x,
                    y,
                    RGB::from_f32(0.5, 0.5, 0.5),
                    RGB::from_f32(0., 0., 0.),
                    rltk::to_cp437('.'),
                );
            }
            TileType::Wall => {
                ctx.set(
                    x,
                    y,
                    RGB::from_f32(0.0, 1.0, 0.0),
                    RGB::from_f32(0., 0., 0.),
                    rltk::to_cp437('#'),
                );
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn make_map() -> Vec<TileType> {
        vec![TileType::Wall; 80 * 50]
    }

    #[test]
    fn test_xy_idx() {
        let inputs = vec![(0, 0), (10, 15), (79, 49)];
        let expected_outputs = vec![0, 1210, 3999];
        assert_eq!(inputs.len(), expected_outputs.len());

        for (input, expected_output) in inputs.iter().zip(expected_outputs) {
            let output = xy_idx(input.0, input.1);
            assert_eq!(output, expected_output)
        }
    }

    #[test]
    fn test_idx_to_xy() {
        let inputs = vec![0, 1210, 3999];
        let expected_outputs = vec![(0, 0), (10, 15), (79, 49)];
        assert_eq!(inputs.len(), expected_outputs.len());

        for (input, expected_output) in inputs.iter().zip(expected_outputs) {
            let output = idx_to_xy(*input as usize);
            assert_eq!(output, expected_output)
        }
    }

    /// Helper func that checks that all tiles are walls except for `floor_idxs`
    fn check_map(map: &[TileType], floor_idxs: HashSet<usize>) {
        assert_eq!(map.len(), 80 * 50);
        for (idx, tile) in map.iter().enumerate() {
            let coords = idx_to_xy(idx);
            if floor_idxs.contains(&idx) {
                assert_eq!(
                    tile,
                    &TileType::Floor,
                    "Expected tile at {:?} to be floor",
                    coords
                )
            } else {
                assert_eq!(
                    tile,
                    &TileType::Wall,
                    "Expected tile at {:?} to be wall",
                    coords
                )
            }
        }
    }

    #[test]
    fn test_apply_vertical_tunnel() {
        // Using a struct instead of a tuple to have explicit names
        #[derive(Debug)]
        struct Example {
            x: i32,
            y1: i32,
            y2: i32,
        }
        // All the differnet examples to test
        let examples = vec![
            Example {
                x: 5,
                y1: 10,
                y2: 15,
            },
            Example {
                x: 0,
                y1: 0,
                y2: 10,
            },
            Example {
                x: 79,
                y1: 40,
                y2: 49,
            },
        ];
        for example in examples {
            // Pattern matches the struct to easily get the variables!
            let Example { x, y1, y2 } = example;
            println!("Checking {:?}", example);
            let mut map = make_map();
            apply_vertical_tunnel(&mut map, y1, y2, x);

            // Build the set of idxs that should be floors
            let mut floor_idxs = HashSet::new();
            for y_offset in 0..y2 - y1 + 1 {
                let had_value = !floor_idxs.insert(xy_idx(x, y1 + y_offset));
                assert!(!had_value);
            }

            check_map(&map, floor_idxs)
        }
    }

    #[test]
    fn test_apply_horizontal_tunnel() {
        #[derive(Debug)]
        struct Example {
            x1: i32,
            x2: i32,
            y: i32,
        }
        // All the differnet examples to test
        let examples = vec![
            Example {
                x1: 10,
                x2: 15,
                y: 5,
            },
            Example {
                x1: 0,
                x2: 10,
                y: 0,
            },
            Example {
                x1: 70,
                x2: 79,
                y: 49,
            },
        ];
        for example in examples {
            // Pattern matches the struct to easily get the variables!
            let Example { x1, x2, y } = example;
            println!("Checking {:?}", example);
            let mut map = make_map();
            apply_horizontal_tunnel(&mut map, x1, x2, y);

            // Build the set of idxs that should be floors
            let mut floor_idxs = HashSet::new();
            for x_offset in 0..x2 - x1 + 1 {
                let had_value = !floor_idxs.insert(xy_idx(x1 + x_offset, y));
                assert!(!had_value);
            }

            check_map(&map, floor_idxs)
        }
    }
}
