use std::error::Error;
use std::path::PathBuf;
use std::process::Command;

pub(crate) fn install_package(
    python: &String,
    pip_args: &String,
    path: &PathBuf,
) -> Result<(), Box<dyn Error>> {
    let path = path.to_str().unwrap();
    let extra: Vec<&str> = pip_args
        .split_whitespace()
        .fold(Vec::new(), |mut result, x| {
            match x.trim() {
                arg if !arg.is_empty() => result.push(arg),
                _ => (),
            }
            result
        });

    println!("Installing {} (Interpreter={})", path, python);

    let mut child = Command::new(python)
        .args(
            ["-m", "pip", "install", "--upgrade"]
                .iter()
                .chain(extra.iter())
                .chain([path].iter()),
        )
        .spawn()?;

    child.wait()?;

    Result::Ok(())
}
