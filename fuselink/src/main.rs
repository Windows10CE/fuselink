use anyhow::bail;
use base64::Engine;
use crossterm::{
    execute,
    style::{self, Print},
    terminal,
};
use std::{
    fs,
    io::{self, stdin, stdout, Read},
    path::Path,
    sync::atomic::AtomicBool,
    time::Duration,
};

use fuselink_common::*;

fn main() -> anyhow::Result<()> {
    execute!(
        stdout(),
        terminal::Clear(terminal::ClearType::All),
        style::Print("Please input a name: ")
    )?;

    let name = readline()?;

    execute!(stdout(), style::Print("Please input a shared password: "))?;

    let pass = readline()?;

    let thread_stop: &'static _ = Box::leak(Box::new(AtomicBool::new(false)));

    let thread_vars = (name.clone(), pass.clone());
    let thread = std::thread::spawn(move || {
        let (name, pass) = thread_vars;
        while !thread_stop.load(std::sync::atomic::Ordering::SeqCst) {
            match fs::File::options().read(true).open("to_export.dat") {
                Ok(mut f) => {
                    let mut data = vec![];
                    f.read_to_end(&mut data).unwrap();
                    if let Some(mut data) = data.strip_suffix(b"eod") {
                        let species_len = usize::from_ne_bytes(data[..8].try_into().unwrap());
                        data = &data[8..];
                        let species = std::str::from_utf8(&data[..species_len]).unwrap();
                        data = &data[species_len..];
                        let nick_len = usize::from_ne_bytes(data[..8].try_into().unwrap());
                        data = &data[8..];
                        let nick = std::str::from_utf8(&data[..nick_len]).unwrap();
                        data = &data[nick_len..];
                        let obtain_len = usize::from_ne_bytes(data[..8].try_into().unwrap());
                        data = &data[8..];
                        let obtain = std::str::from_utf8(&data[..obtain_len]).unwrap();
                        data = &data[obtain_len..];

                        ureq::post("https://fuselink.windows10ce.com/api/add")
                            .send_json(PokemonUploadData {
                                pokemon: PokemonData {
                                    owner_name: name.clone(),
                                    data: base64::engine::general_purpose::STANDARD_NO_PAD
                                        .encode(data),
                                    species_name: species.to_string(),
                                    nickname: nick.to_string(),
                                    obtained_location: obtain.to_string(),
                                },
                                pass: pass.clone(),
                            })
                            .unwrap();
                        drop(f);
                        fs::remove_file("to_export.dat").unwrap();
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::NotFound => (),
                Err(e) => panic!("{}", e),
            }
            std::thread::sleep(Duration::from_millis(500));
        }
    });

    loop {
        execute!(
            stdout(),
            terminal::Clear(terminal::ClearType::All),
            Print(
                r#"
1. Import a Pokemon
2. Exit

Enter a number: "#
            )
        )?;

        let Ok(num): Result<u32, _> = readline()?.parse() else { continue; };

        match num {
            1 => {
                if Path::new("to_import.dat").exists() {
                    execute!(stdout(), Print("Are you sure you want to import? A Pokemon is already in the process of being imported. (y/n): "))?;

                    match readline()?.as_str() {
                        "y" => {
                            fs::remove_file("to_import.dat")?;
                        }
                        _ => continue,
                    }
                }

                let pokemon: Vec<PokemonData> =
                    match ureq::get("https://fuselink.windows10ce.com/api/get")
                        .query("pass", &pass)
                        .call()
                    {
                        Ok(res) => res.into_json()?,
                        Err(ureq::Error::Status(404, _)) => {
                            execute!(stdout(), Print("No Pokemon found."))?;
                            std::thread::sleep(Duration::from_secs(2));
                            continue;
                        }
                        Err(e) => bail!(e),
                    };

                let mut out = String::new();
                for (i, p) in pokemon.iter().enumerate() {
                    out.push_str(&format!(
                        "{}. {} ({}) caught by {} and obtained in {}.\n",
                        i + 1,
                        p.nickname,
                        p.species_name,
                        name,
                        p.obtained_location
                    ));
                }

                execute!(
                    stdout(),
                    terminal::Clear(terminal::ClearType::All),
                    Print(format!(
                        "{out}\nEnter a number (anything else will cancel): "
                    ))
                )?;
                let Ok(num): Result<usize, _> = readline()?.parse() else { continue; };
                if num > pokemon.len() {
                    execute!(stdout(), Print("Number out of range."))?;
                    continue;
                }
                fs::write(
                    "to_import.dat",
                    base64::engine::general_purpose::STANDARD_NO_PAD
                        .decode(&pokemon[num - 1].data)?,
                )?;
            }
            2 => break,
            _ => {}
        }
    }

    thread_stop.store(true, std::sync::atomic::Ordering::SeqCst);
    thread.join().unwrap();
    Ok(())
}

fn readline() -> Result<String, std::io::Error> {
    let mut line = String::new();
    stdin().read_line(&mut line)?;
    line.pop();
    Ok(line)
}
