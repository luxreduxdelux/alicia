mod server;

//================================================================

use crate::server::server_main;
use alicia::prelude::*;
use thiserror::Error;

//================================================================

enum Command {
    Help,
    Server,
    Run(Run),
}

impl Command {
    fn parse() -> Result<Self, CommandError> {
        let mut path = None;
        let mut main = None;
        let mut list = std::env::args();
        list.next();

        while let Some(argument) = list.next() {
            match argument.as_str() {
                "--help" => return Ok(Self::Help),
                "--server" => return Ok(Self::Server),
                "--path" => {
                    if let Some(value) = list.next() {
                        path = Some(value);
                    } else {
                        return Err(CommandError::MissingArgument(
                            "{path}".to_string(),
                            "--path".to_string(),
                        ));
                    }
                }
                "--main" => {
                    if let Some(value) = list.next() {
                        main = Some(value);
                    } else {
                        return Err(CommandError::MissingArgument(
                            "{path}".to_string(),
                            "--path".to_string(),
                        ));
                    }
                }
                x => {
                    if x.starts_with("--") {
                        return Err(CommandError::UnknownArgument(x.to_string()));
                    }

                    path = Some(x.to_string());
                }
            }
        }

        Ok(Self::Server)
        //Ok(Self::Run(Run { path, main }))
    }
}

struct Run {
    path: Option<String>,
    main: Option<String>,
}

#[derive(Error, Debug)]
enum CommandError {
    #[error("error: unknown argument \"{0}\".")]
    UnknownArgument(String),
    #[error("error: missing argument \"{0}\" for command \"{1}\".")]
    MissingArgument(String, String),
    #[error("error: missing main function \"{0}\" in source file \"{1}\"")]
    MissingFunction(String, String),
    #[error("error: invalid main function \"{0}\" in source file \"{1}\"")]
    InvalidFunction(String, String),
    #[error("error: {0}")]
    AliciaError(Error),
}

impl From<Error> for CommandError {
    fn from(value: Error) -> Self {
        Self::AliciaError(value)
    }
}

//================================================================

fn alicia_run(path: &str, main: &str) -> Result<(), CommandError> {
    let instance = Builder::default().with_file(path.to_string())?;
    let mut instance = instance.build()?;

    if let Some(function) = instance.machine.function.get(main).cloned() {
        if let FunctionKind::Function(function) = function {
            function.execute(&mut instance.machine, vec![]);
            Ok(())
        } else {
            Err(CommandError::InvalidFunction(
                main.to_string(),
                path.to_string(),
            ))
        }
    } else {
        Err(CommandError::MissingFunction(
            main.to_string(),
            path.to_string(),
        ))
    }
}

#[tokio::main]
async fn main() {
    let command = Command::parse();

    match command {
        Ok(command) => match command {
            Command::Help => {
                println!("Alicia 1.0.0");
                println!("--help: Show this help message.");
                println!("--path {{path}}: Load a given source file.");
                println!("--main {{name}}: Load a given \"main\" function name.");
            }
            Command::Server => {
                server_main().await;
            }
            Command::Run(run) => {
                let path = run.path.unwrap_or("src/test.alicia".to_string());
                let main = run.main.unwrap_or("main".to_string());

                unsafe {
                    std::env::set_var("RUST_BACKTRACE", "1");
                }

                if let Err(error) = alicia_run(&path, &main) {
                    println!("{error}");
                }
            }
        },
        Err(error) => {
            eprintln!("{error}");
        }
    }
}
