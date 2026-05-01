use alicia::stage_4::machine::Function;
use alicia::{prelude::*, stage_4::machine::Value};

//================================================================

use raylib::prelude::*;
use std::ffi::CString;
use std::sync::OnceLock;

//================================================================

pub fn c_string(text: &str) -> CString {
    let convert = CString::new(text);

    if let Ok(convert) = convert {
        convert
    } else {
        panic!("Error converting Rust string to C string \"{text}\".")
    }
}

#[derive(Debug)]
struct Game {
    texture: ffi::Texture,
}

impl Game {
    fn new() -> Self {
        let texture = unsafe { ffi::LoadTexture(c_string("texture.png").as_ptr()) };

        Self { texture }
    }
}

static GAME: OnceLock<Game> = OnceLock::new();

fn window_should_close(_: Argument) -> Option<Value> {
    unsafe { Some(Value::Boolean(!ffi::WindowShouldClose())) }
}

fn draw_begin(_: Argument) -> Option<Value> {
    unsafe {
        ffi::BeginDrawing();
        ffi::ClearBackground(Color::WHITE.into());
    }

    None
}

fn draw_close(_: Argument) -> Option<Value> {
    unsafe {
        ffi::EndDrawing();
    }

    None
}

fn draw_text(mut argument: Argument) -> Option<Value> {
    let text = argument.next().unwrap().as_string();
    let text = c_string(&text);

    unsafe {
        ffi::DrawText(text.as_ptr(), 8, 8, 32, Color::BLACK.into());
    }

    None
}

fn draw_texture(mut argument: Argument) -> Option<Value> {
    let p_x = argument.next().unwrap().as_decimal() as f32;
    let p_y = argument.next().unwrap().as_decimal() as f32;
    let s_x = (argument.next().unwrap().as_decimal() as f32) * 64.0;
    let s_y = (argument.next().unwrap().as_decimal() as f32) * 64.0;

    let game = GAME.get().unwrap();

    unsafe {
        ffi::DrawTexturePro(
            game.texture,
            Rectangle::new(s_x, s_y, 64.0, 64.0).into(),
            Rectangle::new(p_x, p_y, 128.0, 128.0).into(),
            Vector2::default().into(),
            0.0,
            Color::WHITE.into(),
        );
    }

    None
}

fn to_integer(mut argument: Argument) -> Option<Value> {
    let number = argument.next().unwrap();

    match number {
        Value::Integer(value) => Some(Value::Integer(value)),
        Value::Decimal(value) => Some(Value::Integer(value as i64)),
        _ => todo!(),
    }
}

fn is_key_press(mut argument: Argument) -> Option<Value> {
    let key = argument.next().unwrap().as_integer() as i32;

    unsafe { Some(Value::Boolean(ffi::IsKeyPressed(key))) }
}

fn new_instance() -> Result<(Instance, Function), Error> {
    let instance = Builder::default()
        .add_function(FunctionNative::new(
            "window_should_close".to_string(),
            self::window_should_close,
            NativeArgument::Constant(vec![]),
            ExpressionKind::Boolean,
        ))?
        .add_function(FunctionNative::new(
            "draw_begin".to_string(),
            self::draw_begin,
            NativeArgument::Constant(vec![]),
            ExpressionKind::Null,
        ))?
        .add_function(FunctionNative::new(
            "draw_close".to_string(),
            self::draw_close,
            NativeArgument::Constant(vec![]),
            ExpressionKind::Null,
        ))?
        .add_function(FunctionNative::new(
            "draw_text".to_string(),
            self::draw_text,
            NativeArgument::Constant(vec![ExpressionKind::String]),
            ExpressionKind::Null,
        ))?
        .add_function(FunctionNative::new(
            "draw_texture".to_string(),
            self::draw_texture,
            NativeArgument::Constant(vec![
                ExpressionKind::Decimal,
                ExpressionKind::Decimal,
                ExpressionKind::Decimal,
                ExpressionKind::Decimal,
            ]),
            ExpressionKind::Null,
        ))?
        .add_function(FunctionNative::new(
            "is_key_press".to_string(),
            self::is_key_press,
            NativeArgument::Constant(vec![ExpressionKind::Integer]),
            ExpressionKind::Boolean,
        ))?
        .add_function(FunctionNative::new(
            "to_integer".to_string(),
            self::to_integer,
            NativeArgument::Constant(vec![ExpressionKind::Decimal]),
            ExpressionKind::Integer,
        ))?
        .with_file("src/game.alicia".to_string())?;

    let instance = instance.build()?;

    if let Some(function) = instance.machine.function.get("main").cloned()
        && let FunctionKind::Function(function) = function
    {
        Ok((instance, function))
    } else {
        panic!("no main function")
    }
}

fn run() -> Result<(), Error> {
    let (mut instance, mut function) = new_instance()?;

    let (mut handle, _thread) = raylib::init()
        .size(7 * 128, 5 * 128)
        .title("Alicia - Sokoban")
        .resizable()
        .log_level(TraceLogLevel::LOG_NONE)
        .build();

    handle.set_target_fps(30);

    GAME.set(Game::new()).unwrap();

    loop {
        let new = function.execute(&mut instance.machine, vec![]).unwrap();

        if let Value::Boolean(new) = new {
            if new {
                println!("restart");
                (instance, function) = new_instance()?;
            } else {
                break;
            }
        }
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
