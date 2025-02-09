use tcod::{
    colors::*,
    console::*,
    input::{ Key, KeyCode::* },
    map::{FovAlgorithm, Map as FovMap},
};
use std::{
    cmp,
    ops::RangeInclusive,
};
use rand::Rng;

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const LIMIT_FPS: i32 = 60;

const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 45;

const COLOR_DARK_WALL: Color = Color { r: 0, g: 0, b: 100 };
const COLOR_LIGHT_WALL: Color = Color { r: 130, g: 110, b: 50 };
const COLOR_DARK_GROUND: Color = Color { r: 50, g: 50, b: 150 };
const COLOR_LIGHT_GROUND: Color = Color { r: 200, g: 180, b: 50 };

const ROOM_MAX_SIZE: i32 = 10;
const ROOM_MIN_SIZE: i32 = 6;
const MAX_ROOMS: i32 = 30;
const MAX_ROOM_MONSTERS: i32 = 5;

const FOV_ALGO: FovAlgorithm = FovAlgorithm::Basic;
const FOV_LIGHT_WALLS: bool = true;
const TORCH_RADIUS: i32 = 10;

const PLAYER: usize = 0;

type Map = Vec<Vec<Tile>>;


#[derive(Copy, Clone)]
struct RectRoom {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}

impl RectRoom {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> RectRoom {
        RectRoom {
            x1: x,
            y1: y,
            x2: x + w,
            y2: y + h,
        }
    }
    
    pub fn center(&self) -> (i32, i32) {
        let center_x = (self.x1 + self.x2) / 2;
        let center_y = (self.y1 + self.y2) / 2;
        (center_x, center_y)
    }
    
    pub fn intersects_with(&self, other: &RectRoom) -> bool {
        (self.x1 <= other.x2)
            && (self.x2 >= other.x1)
            && (self.y1 <= other.y2)
            && (self.y2 >= other.y1)
    }
}

fn create_room(room: RectRoom, map: &mut Map) {
    for x in (room.x1 + 1)..room.x2 {
        for y in (room.y1 + 1)..room.y2 {
            map[x as usize][y as usize] = Tile::empty();
        }
    }
}

fn create_horizontal_tunnel(x1: i32, x2: i32, y: i32, map: &mut Map) {
    for x in cmp::min(x1, x2)..(cmp::max(x1, x2) + 1) {
        map[x as usize][y as usize] = Tile::empty();
    }
}

fn create_vertical_tunnel(y1: i32, y2: i32, x: i32, map: &mut Map) {
    for y in cmp::min(y1, y2)..(cmp::max(y1, y2) + 1) {
        map[x as usize][y as usize] = Tile::empty();
    }
}

fn place_objects(room: RectRoom, map: &Map, objects: &mut Vec<Object>) {
    let num_monsters = rand::thread_rng().gen_range(RangeInclusive::new(0, MAX_ROOM_MONSTERS));
    for _ in 0..num_monsters {
        let x = rand::thread_rng().gen_range(RangeInclusive::new(room.x1 + 1, room.x2 - 1));
        let y = rand::thread_rng().gen_range(RangeInclusive::new(room.y1 + 1, room.y2 - 1));
        if is_blocked(x,y,map,objects) {continue;}
        let mut monster = if rand::random::<f32>() < 0.8 {
            //Orc
            Object::new(x, y, 'O', DESATURATED_GREEN, "Orc".to_string(), true)
        } else {
            //Troll
            Object::new(x, y, 'T', DARKER_GREEN, "Troll".to_string(), true)
        };

        monster.alive = true;
        objects.push(monster)
    }
}

fn is_blocked(x: i32, y: i32, map: &Map, objects: &[Object]) -> bool {
    if map[x as usize][y as usize].blocked {
        return true;
    }
    objects
        .iter()
        .any(|object| object.blocks && object.pos() == (x, y))
}

struct Game {
    map: Map,
}

impl Game {
    pub fn new(objects: &mut Vec<Object>) -> Game {
        Game {
            map: build_map(objects),
        }
    }
    
}

#[derive(Clone, Copy, Debug)]
struct Tile {
    blocked: bool,
    explored: bool,
    block_sight: bool,
}

impl Tile {
    pub fn empty() -> Tile {
        Tile {
            blocked: false,
            explored: false,
            block_sight:false,
        }
    }

    pub fn wall() -> Tile {
        Tile {
            blocked: true,
            explored: false,
            block_sight: true,
        }
    }
}

#[derive(Debug)]
struct Object {
    x: i32,
    y: i32,
    r#char: char,
    color: Color,
    name: String,
    blocks: bool,
    alive: bool,
}

impl Object {
    pub fn new(_x: i32, _y: i32, _char: char, _color: Color, _name: String, _blocks: bool) -> Object {
        Object {
            x: _x,
            y: _y,
            r#char: _char,
            color: _color,
            name: _name,
            blocks: _blocks,
            alive: false,
        }
    }


    pub fn draw(&self, con: &mut dyn Console) {
        con.set_default_foreground(self.color);
        con.put_char(self.x, self.y, self.char, BackgroundFlag::None);
    }

    pub fn pos(&self) -> (i32,i32) {
        (self.x,self.y)
    }

    pub fn set_pos(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }
}

fn move_by(id: usize, dx: i32, dy: i32, map: &Map, objects: &mut [Object]) {
    let (x,y) = objects[id].pos();
    if !is_blocked(x + dx, y + dy, map, objects) {
        objects[id].set_pos(x + dx, y + dy);
    }
}

struct Tcod {
    root: Root,
    con : Offscreen,
    fov : FovMap,
}

impl Tcod {
    pub fn new() -> Tcod {
        let _root = Root::initializer()
            .font("./assets/arial10x10.png", FontLayout::Tcod)
            .font_type(FontType::Greyscale)
            .size(SCREEN_WIDTH, SCREEN_HEIGHT)
            .title("ROGUE")
            .init();
        let _con = Offscreen::new(MAP_WIDTH, MAP_HEIGHT);
        let mut _fov = FovMap::new(MAP_WIDTH, MAP_HEIGHT);
        Tcod {
            root: _root,
            con : _con,
            fov : _fov,
        }
    }
}

fn render_all(tcod: &mut Tcod, game: &mut Game, objects: &[Object], fov_recompute: bool) {
        if fov_recompute {
            let player = &objects[PLAYER];
                tcod.fov
                .compute_fov(player.x, player.y, TORCH_RADIUS, FOV_LIGHT_WALLS, FOV_ALGO);
        }

        for y in 0..MAP_HEIGHT {
            for x in 0..MAP_WIDTH {
                let visible = tcod.fov.is_in_fov(x, y);
                let wall = game.map[x as usize][y as usize].block_sight;
                let color = match (visible, wall) {
                    (false, true) => COLOR_DARK_WALL,
                    (false, false) => COLOR_DARK_GROUND,
                    (true, true) => COLOR_LIGHT_WALL,
                    (true, false) => COLOR_LIGHT_GROUND,
                };
                let explored = &mut game.map[x as usize][y as usize].explored;
                if visible {
                    *explored = true;
                }
                if *explored {
                    tcod.con.set_char_background(x, y, color, BackgroundFlag::Set);
                }
            }
        }

        for object in objects {
            if tcod.fov.is_in_fov(object.x, object.y) {
                object.draw(&mut tcod.con);
            }
        }

        blit(&tcod.con, (0,0), (MAP_WIDTH,MAP_HEIGHT), &mut tcod.root, (0,0), 1., 1.);
}

fn main() {
    let mut player = Object::new(0, 0, '@', RED, "Player".to_string(), true);
    player.alive = true;

    let mut objects = vec![player];
    let mut tcod = Tcod::new();
    let mut game = Game::new(&mut objects);

    tcod::system::set_fps(LIMIT_FPS);
    for y in 0..MAP_HEIGHT {
       for x in 0..MAP_WIDTH {
            tcod.fov.set(
                x,
                y,
                !game.map[x as usize][y as usize].block_sight,
                !game.map[x as usize][y as usize].blocked,
            );
        }
    }
    let mut previous_player_position = (-1, -1);
    while !tcod.root.window_closed() {
        tcod.con.clear();
        let fov_recompute = previous_player_position != objects[PLAYER].pos();
        render_all(&mut tcod, &mut game, &objects, fov_recompute);
        tcod.root.flush();
        previous_player_position = objects[PLAYER].pos();
        if handle_keys(&mut tcod, &game, &mut objects,) {
            break;
        }
    }
}

fn build_map(objects: &mut Vec<Object>) -> Map {
    let mut map = vec![vec![Tile::wall(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];
    let mut rooms = vec![];

    for _ in 0..MAX_ROOMS {
        let w = rand::thread_rng().gen_range(RangeInclusive::new(ROOM_MIN_SIZE, ROOM_MAX_SIZE));
        let h = rand::thread_rng().gen_range(RangeInclusive::new(ROOM_MIN_SIZE, ROOM_MAX_SIZE));
        let x = rand::thread_rng().gen_range(RangeInclusive::new(0, MAP_WIDTH - w - 1));
        let y = rand::thread_rng().gen_range(RangeInclusive::new(0, MAP_HEIGHT - h - 1));
        let new_room = RectRoom::new(x, y, w, h);
        let failed = rooms
            .iter()
            .any(|other_room| new_room.intersects_with(other_room));
        if !failed {
            create_room(new_room, &mut map);
            let (new_x, new_y) = new_room.center();
            if rooms.is_empty() {
                objects[PLAYER].set_pos(new_x, new_y);
            } else {
                let (prev_x, prev_y) = rooms[rooms.len() - 1].center();
                if rand::random() {
                    create_horizontal_tunnel(prev_x, new_x, prev_y, &mut map);
                    create_vertical_tunnel(prev_y, new_y, new_x, &mut map);
                } else {
                    create_vertical_tunnel(prev_y, new_y, prev_x, &mut map);
                    create_horizontal_tunnel(prev_x, new_x, new_y, &mut map);
                }
            }
            place_objects(new_room, &map, objects);
            rooms.push(new_room);
        }
    }

    map
}

fn handle_keys(tcod: &mut Tcod, game: &Game, objects: &mut Vec<Object>) -> bool {
    let key = tcod.root.wait_for_keypress(true);
    match key {
        Key { code: Enter, alt: true, ..} => {
            let fullscreen = tcod.root.is_fullscreen();
            tcod.root.set_fullscreen(!fullscreen);
        },
        Key { code: Escape, ..}=> return true,
        Key { code: Up, .. }   => move_by(PLAYER, 0, -1, &game.map, objects),
        Key { code: Down, .. } => move_by(PLAYER, 0, 1, &game.map, objects),
        Key { code: Left, .. } => move_by(PLAYER, -1, 0, &game.map, objects),
        Key { code: Right, .. }=> move_by(PLAYER, 1, 0, &game.map, objects),

        _ => {},
    };
    false
}
