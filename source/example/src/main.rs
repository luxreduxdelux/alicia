use alicia::helper::error::*;
use alicia::stage_1::helper::Point;
use alicia::stage_1::{
    buffer::TokenBuffer,
    helper::{Identifier, Source},
};
use alicia::stage_2::construct::{Function, FunctionNative};
use alicia::stage_2::construct::{FunctionSignature, Value};
use alicia::stage_2::scope::*;
use alicia::stage_3::analysis::*;
use alicia::stage_4::buffer::ArgumentBuffer;

//================================================================

use raylib::prelude::*;
use std::ffi::CString;

//================================================================

pub fn c_string(text: &str) -> CString {
    CString::new(text).unwrap()
}

fn draw_text(mut argument: ArgumentBuffer) -> Result<Option<Value>, Error> {
    let text = argument.next().unwrap().as_string()?;
    let point = argument.next().unwrap().as_structure()?;
    let p_x = point.data.get("x").unwrap();
    let p_y = point.data.get("y").unwrap();
    let color = argument.next().unwrap().as_structure()?;
    let c_r = color.data.get("r").unwrap();
    let c_g = color.data.get("g").unwrap();
    let c_b = color.data.get("b").unwrap();
    let c_a = color.data.get("a").unwrap();

    unsafe {
        ffi::DrawText(
            c_string(&text).as_ptr(),
            p_x.as_decimal()? as i32,
            p_y.as_decimal()? as i32,
            32,
            Color {
                r: c_r.as_integer()? as u8,
                g: c_g.as_integer()? as u8,
                b: c_b.as_integer()? as u8,
                a: c_a.as_integer()? as u8,
            }
            .into(),
        );
    }

    Ok(None)
}

fn is_key_down(mut argument: ArgumentBuffer) -> Result<Option<Value>, Error> {
    let key = argument.next().unwrap().as_integer()?;

    unsafe { Ok(Some(Value::Boolean(ffi::IsKeyDown(key as i32)))) }
}

fn alicia_function(scope: &mut Scope, name: &str, function: FunctionSignature) {
    scope.set_declaration(
        Identifier::from_string(name.to_string(), Point::new(0, 0)).unwrap(),
        Declaration::FunctionNative(FunctionNative::new(function, Vec::default(), None)),
    );
}

fn alicia_create() -> Result<(Scope, Function, Function), Error> {
    let mut scope = Scope::new(None);
    scope.parse_buffer(TokenBuffer::new(Source::new_file("src/test.alicia")?)?)?;

    //================================================================

    //alicia_function(&mut scope, "load_text", self::load_text);
    alicia_function(&mut scope, "draw_text", self::draw_text);
    //alicia_function(&mut scope, "load_texture", self::load_texture);
    //alicia_function(&mut scope, "draw_texture", self::draw_texture);
    //alicia_function(&mut scope, "load_sound", self::load_sound);
    //alicia_function(&mut scope, "play_sound", self::play_sound);
    alicia_function(&mut scope, "is_key_down", self::is_key_down);

    Analysis::analyze_tree(&mut scope)?;

    //================================================================

    let load = scope
        .get_declaration(Identifier::from_string("load".to_string(), Point::default()).unwrap())
        .cloned();

    let load = match load {
        Some(ref declaration) => match declaration {
            Declaration::Function(function) => function,
            _ => todo!(),
        },
        None => todo!(),
    };

    //================================================================

    let draw = scope
        .get_declaration(Identifier::from_string("draw".to_string(), Point::default()).unwrap())
        .cloned();

    let draw = match draw {
        Some(ref declaration) => match declaration {
            Declaration::Function(function) => function,
            _ => todo!(),
        },
        None => todo!(),
    };

    Ok((scope, load.clone(), draw.clone()))
}

fn main() {
    match alicia_create() {
        Ok((mut alicia, load, draw)) => {
            let (mut handle, thread) = raylib::init()
                .size(512, 768)
                .title("Alicia Example")
                .build();

            handle.set_target_fps(60);

            if let Err(error) = load.execute(&mut alicia) {
                panic!("{error}");
            }

            while !handle.window_should_close() {
                let mut handle = handle.begin_drawing(&thread);

                handle.clear_background(Color::WHITE);

                match draw.execute(&mut alicia) {
                    Ok(value) => {
                        if let Some(value) = value {
                            println!("{value}");
                        }
                    }
                    Err(error) => panic!("{error}"),
                }
            }
        }
        Err(error) => eprintln!("{error}"),
    }
}
