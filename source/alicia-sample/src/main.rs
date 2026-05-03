use alicia::machine::Machine;
use alicia::machine::ValueType;
use alicia::{machine::Value, prelude::*};
use alicia_macro::builder_function;
use alicia_macro::function;

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

fn window_should_close(machine: &mut Machine, _: Argument) -> Option<Value> {
    unsafe { Some(Value::Boolean(!ffi::WindowShouldClose())) }
}

fn draw_begin(machine: &mut Machine, _: Argument) -> Option<Value> {
    unsafe {
        ffi::BeginDrawing();
        ffi::ClearBackground(Color::WHITE.into());
    }

    None
}

fn draw_close(machine: &mut Machine, _: Argument) -> Option<Value> {
    unsafe {
        ffi::EndDrawing();
    }

    None
}

fn draw_text(machine: &mut Machine, mut argument: Argument) -> Option<Value> {
    let text = argument.next().unwrap().as_string();
    let text = c_string(&text);
    let p_x = argument.next().unwrap().as_integer() as i32;
    let p_y = argument.next().unwrap().as_integer() as i32;

    unsafe {
        ffi::DrawText(text.as_ptr(), p_x, p_y, 64, Color::WHITE.into());
    }

    None
}

fn draw_texture(machine: &mut Machine, mut argument: Argument) -> Option<Value> {
    let p_x = argument.next().unwrap().as_decimal() as f32;
    let p_y = argument.next().unwrap().as_decimal() as f32;
    let s_x = (argument.next().unwrap().as_decimal() as f32);
    let s_y = (argument.next().unwrap().as_decimal() as f32);
    let c_r = (argument.next().unwrap().as_integer());
    let c_g = (argument.next().unwrap().as_integer());
    let c_b = (argument.next().unwrap().as_integer());
    let c_a = (argument.next().unwrap().as_integer());

    let game = GAME.get().unwrap();

    unsafe {
        ffi::DrawTexturePro(
            game.texture,
            Rectangle::new(s_x * 64.0, s_y * 64.0, 64.0, 64.0).into(),
            Rectangle::new(p_x, p_y, 128.0, 128.0).into(),
            Vector2::default().into(),
            0.0,
            Color::new(c_r as u8, c_g as u8, c_b as u8, c_a as u8).into(),
        );
    }

    None
}

#[function]
fn draw_texture_macro(
    p_x: f32,
    p_y: f32,
    s_x: f32,
    s_y: f32,
    c_r: i64,
    c_g: i64,
    c_b: i64,
    c_a: i64,
) -> () {
    let game = GAME.get().unwrap();

    unsafe {
        ffi::DrawTexturePro(
            game.texture,
            Rectangle::new(s_x * 64.0, s_y * 64.0, 64.0, 64.0).into(),
            Rectangle::new(p_x, p_y, 128.0, 128.0).into(),
            Vector2::default().into(),
            0.0,
            Color::new(c_r as u8, c_g as u8, c_b as u8, c_a as u8).into(),
        );
    }

    None
}

fn to_integer(machine: &mut Machine, mut argument: Argument) -> Option<Value> {
    let number = argument.next().unwrap();

    match number {
        Value::Integer(value) => Some(Value::Integer(value)),
        Value::Decimal(value) => Some(Value::Integer(value as i64)),
        _ => todo!(),
    }
}

fn is_key_press(machine: &mut Machine, mut argument: Argument) -> Option<Value> {
    let key = argument.next().unwrap().as_integer() as i32;

    unsafe { Some(Value::Boolean(ffi::IsKeyPressed(key))) }
}

fn compile_aux(machine: &mut Machine) -> Result<(), Error> {
    let builder = new_builder()?;
    let scope = builder.build_scope()?;
    machine.compile(&scope)?;

    Ok(())
}

fn compile(machine: &mut Machine, _: Argument) -> Option<Value> {
    println!("compile");

    if let Err(error) = compile_aux(machine) {
        println!("recompilation error: {error}");
    }

    None
}

fn new_builder() -> Result<Builder, Error> {
    Builder::default()
        .add_function(FunctionNative::new(
            "window_should_close".to_string(),
            self::window_should_close,
            NativeArgument::Constant(&[]),
            ValueType::Boolean,
        ))?
        .add_function(FunctionNative::new(
            "draw_begin".to_string(),
            self::draw_begin,
            NativeArgument::Constant(&[]),
            ValueType::Null,
        ))?
        .add_function(FunctionNative::new(
            "draw_close".to_string(),
            self::draw_close,
            NativeArgument::Constant(&[]),
            ValueType::Null,
        ))?
        .add_function(FunctionNative::new(
            "draw_text".to_string(),
            self::draw_text,
            NativeArgument::Constant(&[ValueType::String, ValueType::Integer, ValueType::Integer]),
            ValueType::Null,
        ))?
        .add_function(FunctionNative::new(
            "draw_texture".to_string(),
            self::draw_texture,
            NativeArgument::Constant(&[
                ValueType::Decimal,
                ValueType::Decimal,
                ValueType::Decimal,
                ValueType::Decimal,
                ValueType::Integer,
                ValueType::Integer,
                ValueType::Integer,
                ValueType::Integer,
            ]),
            ValueType::Null,
        ))?
        .add_function(builder_function!(draw_texture_macro))?
        .add_function(FunctionNative::new(
            "is_key_press".to_string(),
            self::is_key_press,
            NativeArgument::Constant(&[ValueType::Integer]),
            ValueType::Boolean,
        ))?
        .add_function(FunctionNative::new(
            "to_integer".to_string(),
            self::to_integer,
            NativeArgument::Constant(&[ValueType::Decimal]),
            ValueType::Integer,
        ))?
        .add_function(FunctionNative::new(
            "compile".to_string(),
            self::compile,
            NativeArgument::Constant(&[]),
            ValueType::Null,
        ))?
        .with_file("src/game.alicia".to_string())
}

fn run() -> Result<(), Error> {
    let builder = new_builder()?;
    let mut instance = builder.build()?;

    let mut function = if let Some(function) = instance.machine.function.get("main").cloned()
        && let FunctionKind::Function(function) = function
    {
        function
    } else {
        panic!("no main function")
    };

    //================================================================

    let (mut handle, _thread) = raylib::init()
        .size(7 * 128, 5 * 128)
        .title("Alicia - Sokoban")
        .resizable()
        .log_level(TraceLogLevel::LOG_NONE)
        .build();

    handle.set_target_fps(30);

    GAME.set(Game::new()).unwrap();

    //================================================================

    loop {
        let new = function.execute(&mut instance.machine, vec![]).unwrap();

        if let Value::Boolean(new) = new {
            if new {
                println!("restart");
                let builder = new_builder()?;
                instance = builder.build()?;

                function = if let Some(function) = instance.machine.function.get("main").cloned()
                    && let FunctionKind::Function(function) = function
                {
                    function
                } else {
                    panic!("no main function")
                };
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
