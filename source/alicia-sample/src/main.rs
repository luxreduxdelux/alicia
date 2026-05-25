use alicia::machine::Enumerate;
use alicia::machine::Structure;
use alicia::{machine::Function, machine::Value, prelude::*};

//================================================================

use raylib::prelude::*;
use std::ffi::CString;

//================================================================

pub fn c_string(text: &str) -> CString {
    let convert = CString::new(text);

    if let Ok(convert) = convert {
        convert
    } else {
        panic!("Error converting Rust string to C string \"{text}\".")
    }
}

#[derive(Clone)]
enum Cell {
    Hidden,
    Open,
    Flag,
    Bomb,
}

impl From<Cell> for Value {
    fn from(value: Cell) -> Value {
        let (kind, kind_index) = match value {
            Cell::Hidden => ("Hidden".to_string(), 0),
            Cell::Open => ("Open".to_string(), 1),
            Cell::Flag => ("Flag".to_string(), 2),
            Cell::Bomb => ("Bomb".to_string(), 3),
        };

        Value::Enumerate(Enumerate::new("Cell".to_string(), kind, 0, kind_index))
    }
}

impl TryFrom<Value> for Cell {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Enumerate(value) = value {
            return match value.index_kind {
                0 => Ok(Self::Hidden),
                1 => Ok(Self::Open),
                2 => Ok(Self::Flag),
                _ => Ok(Self::Bomb),
            };
        }

        todo!()
    }
}

#[derive(Clone)]
struct GameState {
    size_x: i64,
    size_y: i64,
    mine_count: i64,
    finish: bool,
    board: Vec<Vec<i64>>,
    board_visible: Vec<Vec<Cell>>,
}

impl From<GameState> for Value {
    fn from(value: GameState) -> Value {
        let mut s = Structure::new("GameState".to_string());

        s.insert(value.size_x.into());
        s.insert(value.size_y.into());
        s.insert(value.mine_count.into());
        s.insert(value.finish.into());
        s.insert(value.board.into());
        s.insert(value.board_visible.into());

        Value::Structure(s)
    }
}

impl TryFrom<Value> for GameState {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Structure(value) = value {
            let size_x = value.data[0].borrow().clone().try_into()?;
            let size_y = value.data[1].borrow().clone().try_into()?;
            let mine_count = value.data[2].borrow().clone().try_into()?;
            let finish = value.data[3].borrow().clone().try_into()?;
            let board = value.data[4].borrow().clone().try_into()?;
            let board_visible = value.data[5].borrow().clone().try_into()?;

            return Ok(Self {
                size_x,
                size_y,
                mine_count,
                finish,
                board,
                board_visible,
            });
        }

        todo!()
    }
}

struct Script {
    texture: Texture2D,
    font: Font,
    instance: Instance,
    state: GameState,
    create_game_state: Function,
    mark_cell: Function,
    open_cell: Function,
    save_game: Function,
    load_game: Function,
}

impl Script {
    const TEXTURE: &[u8] = include_bytes!("../texture.png");
    const FONT: &[u8] = include_bytes!("../font.ttf");

    fn new(handle: &mut RaylibHandle, thread: &RaylibThread) -> Result<Self, Error> {
        let texture = Image::load_image_from_mem(".png", Self::TEXTURE).unwrap();
        let texture = handle.load_texture_from_image(thread, &texture).unwrap();
        let font = handle
            .load_font_from_memory(thread, ".ttf", Self::FONT, 32, None)
            .unwrap();

        let mut instance = Builder::default()
            // TO-DO adapt AliciaError to convert to std::error::Error
            .with_file("src/game.alicia".to_string())?
            .build()?;

        let create_game_state = Self::get_function(&instance, "create_game_state")?;
        let mark_cell = Self::get_function(&instance, "mark_cell")?;
        let open_cell = Self::get_function(&instance, "open_cell")?;
        let save_game = Self::get_function(&instance, "save_game")?;
        let load_game = Self::get_function(&instance, "load_game")?;

        let state = GameState::try_from(
            create_game_state
                .execute(
                    &mut instance.machine,
                    vec![Value::Integer(6), Value::Integer(6), Value::Integer(6)],
                )?
                .unwrap(),
        )
        .unwrap();

        Ok(Self {
            texture,
            font,
            instance,
            state,
            create_game_state,
            mark_cell,
            open_cell,
            save_game,
            load_game,
        })
    }

    fn get_function(instance: &Instance, name: &str) -> Result<Function, Error> {
        Ok(instance.machine.get_function(name).unwrap().clone())
    }

    fn draw(&mut self, draw: &mut RaylibDrawHandle) {
        let s_h = Vector2::new(
            draw.get_screen_width() as f32 * 0.5,
            draw.get_screen_height() as f32 * 0.5,
        );
        let mut open = None;
        let mut mark = None;

        for (i_x, x) in self.state.board.iter().enumerate() {
            for (i_y, y) in x.iter().enumerate() {
                let point = Vector2::new(i_x as f32, i_y as f32) * 100.0;
                let point = point
                    - Vector2::new(
                        self.state.size_x as f32 * 50.0,
                        self.state.size_y as f32 * 50.0,
                    );
                let point = point + s_h;

                let cell = &self.state.board_visible[i_x][i_y];

                if self.state.finish {
                    match cell {
                        Cell::Bomb => {
                            self.draw_cell(draw, point, true);
                            self.draw_bomb(draw, point, 0.0);
                        }
                        _ => {
                            self.draw_cell(draw, point, true);
                        }
                    }
                } else {
                    match cell {
                        Cell::Hidden => {
                            let (d_open, d_mark) = self.draw_cell(draw, point, false);

                            if d_open {
                                open = Some((i_x, i_y));
                            }

                            if d_mark {
                                mark = Some((i_x, i_y));
                            }
                        }
                        Cell::Open => {
                            self.draw_cell(draw, point, true);

                            let number = self.state.board[i_x][i_y];

                            if number > 0 {
                                self.draw_number(draw, point, (number - 1) as f32);
                            }
                        }
                        Cell::Flag => {
                            let (d_open, d_mark) = self.draw_cell(draw, point, false);

                            if d_open {
                                open = Some((i_x, i_y));
                            }

                            if d_mark {
                                mark = Some((i_x, i_y));
                            }

                            self.draw_flag(draw, point, 0.0);
                        }
                        _ => {
                            self.draw_cell(draw, point, true);
                            self.draw_bomb(draw, point, 0.0);
                        }
                    }
                }
            }
        }

        if let Some((x, y)) = open {
            self.state = GameState::try_from(
                self.open_cell
                    .execute(
                        &mut self.instance.machine,
                        vec![self.state.clone().into(), x.into(), y.into()],
                    )
                    .unwrap()
                    .unwrap(),
            )
            .unwrap();
        }

        if let Some((x, y)) = mark {
            self.state = GameState::try_from(
                self.mark_cell
                    .execute(
                        &mut self.instance.machine,
                        vec![self.state.clone().into(), x.into(), y.into()],
                    )
                    .unwrap()
                    .unwrap(),
            )
            .unwrap();
        }

        if self.state.finish {
            if self.draw_button(draw, Vector2::new(150.0 + 128.0, 16.0), "Restart Game") {
                self.state = GameState::try_from(
                    self.create_game_state
                        .execute(
                            &mut self.instance.machine,
                            vec![Value::Integer(6), Value::Integer(6), Value::Integer(6)],
                        )
                        .unwrap()
                        .unwrap(),
                )
                .unwrap();
            }
        } else {
            if self.draw_button(
                draw,
                Vector2::new(24.0 + s_h.x - 300.0 * 1.0, 16.0),
                "Save Game",
            ) {
                self.save_game
                    .execute(
                        &mut self.instance.machine,
                        vec![self.state.clone().into(), "game.txt".into()],
                    )
                    .unwrap();
            }

            if self.draw_button(
                draw,
                Vector2::new(24.0 + s_h.x - 300.0 * 0.0, 16.0),
                "Load Game",
            ) {
                self.state = GameState::try_from(
                    self.load_game
                        .execute(&mut self.instance.machine, vec!["game.txt".into()])
                        .unwrap()
                        .unwrap(),
                )
                .unwrap();
            }
        }
    }

    fn draw_cell(&self, draw: &mut RaylibDrawHandle, point: Vector2, reveal: bool) -> (bool, bool) {
        let mouse = draw.get_mouse_position();
        let hover = Rectangle::new(point.x, point.y, 100.0, 100.0).check_collision_point_rec(mouse);
        let color = if hover {
            Color::WHITE
        } else {
            Color::WHITE.brightness(-0.25)
        };
        let open = hover && draw.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);
        let mark = hover && draw.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_RIGHT);
        let which = if reveal { 890.0 } else { 790.0 };

        draw.draw_texture_rec(
            &self.texture,
            Rectangle::new(which, 0.0, 100.0, 100.0),
            point,
            color,
        );

        (open, mark)
    }

    fn draw_text(&self, draw: &mut RaylibDrawHandle, point: Vector2, text: &str) {
        draw.draw_text_pro(
            &self.font,
            text,
            point,
            Vector2::zero(),
            0.0,
            32.0,
            0.0,
            Color::WHITE,
        );
    }

    fn draw_button(&self, draw: &mut RaylibDrawHandle, point: Vector2, text: &str) -> bool {
        let mouse = draw.get_mouse_position();
        let shape = Rectangle::new(point.x, point.y, 256.0, 64.0);
        let hover = shape.check_collision_point_rec(mouse);
        let color = if hover {
            Color::WHITE.brightness(-0.25)
        } else {
            Color::WHITE.brightness(-0.50)
        };
        let click = hover && draw.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);

        draw.draw_rectangle_rounded(shape, 0.25, 8, color);
        draw.draw_rectangle_rounded_lines_ex(shape, 0.25, 8, 4.0, Color::BLACK);
        self.draw_text(draw, point + Vector2::new(8.0, 8.0), text);

        click
    }

    fn draw_number(&self, draw: &mut RaylibDrawHandle, point: Vector2, index: f32) {
        draw.draw_texture_rec(
            &self.texture,
            Rectangle::new(100.0 * index, 0.0, 100.0, 100.0),
            point + Vector2::new(10.0, 0.0),
            Color::WHITE,
        );
    }

    fn draw_flag(&self, draw: &mut RaylibDrawHandle, point: Vector2, index: f32) {
        draw.draw_texture_rec(
            &self.texture,
            Rectangle::new(0.0, 100.0, 100.0, 100.0),
            point + Vector2::new(10.0, 0.0),
            Color::WHITE,
        );
    }

    fn draw_bomb(&self, draw: &mut RaylibDrawHandle, point: Vector2, index: f32) {
        draw.draw_texture_rec(
            &self.texture,
            Rectangle::new(100.0, 100.0, 100.0, 100.0),
            point + Vector2::new(10.0, 0.0),
            Color::WHITE,
        );
    }
}

fn run() -> Result<(), Error> {
    let (mut handle, thread) = raylib::init()
        .size(800, 800)
        .title("Alicia - Minesweeper")
        .resizable()
        .log_level(TraceLogLevel::LOG_NONE)
        .build();

    handle.set_target_fps(30);

    //================================================================

    let mut script = Script::new(&mut handle, &thread)?;

    while !handle.window_should_close() {
        let mut draw = handle.begin_drawing(&thread);

        draw.clear_background(Color::WHITE);

        script.draw(&mut draw);
    }

    Ok(())
}

fn main() {
    unsafe {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    if let Err(error) = run() {
        eprintln!("{error}");
    }
}
