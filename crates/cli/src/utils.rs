use std::{
    fmt::Display,
    fs::File,
    io::{stdin, stdout, BufRead, Read, Result as IOResult, Write},
    path::PathBuf,
};

pub fn input<T: Display>(query: T) -> IOResult<String> {
    print!("{query}");
    stdout().flush()?;

    let mut buffer = String::new();
    stdin().lock().read_line(&mut buffer)?;

    Ok(buffer.trim().to_string())
}

pub fn read_file(file_path: &PathBuf) -> IOResult<String> {
    let mut file = File::open(file_path)?;
    let mut buffer = String::new();

    file.read_to_string(&mut buffer)?;
    Ok(buffer)
}
