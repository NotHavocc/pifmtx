use colored::Colorize;
use std::env;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use walkdir::WalkDir;

fn main() {
    let mut needs_to_convert = false;

    println!(
        "{}",
        r#"     ____  ____________  __________  __
    / __ \/  _/ ____/  |/  /_  __/ |/ /
   / /_/ // // /_  / /|_/ / / /  |   / 
  / ____// // __/ / /  / / / /  /   |  
 /_/   /___/_/   /_/  /_/ /_/  /_/|_|  
                                      "#
        .yellow()
        .bold()
    );

    println!(
        "{}",
        "Welcome to PIFMTX v1.0.0 (nothavoc, 2025-2025)".blue()
    );
    print!("Enter the sound file you want to transmit: ");
    io::Write::flush(&mut io::stdout()).expect("flush error");
    let mut filename: String = String::new();
    io::stdin()
        .read_line(&mut filename)
        .expect("Error reading filename.");

    let filename = filename.trim();

    let mut fullpathfile: Option<PathBuf> = None;
    let mut found = false;
    for entry in WalkDir::new(".") {
        let entry = entry.expect("cannot access entry");
        if entry.file_type().is_file() {
            if let Some(name) = entry.path().file_name() {
                if name.to_string_lossy() == filename {
                    println!("{}", format!("Found: {}", entry.path().display()).green());
                    fullpathfile = Some(entry.path().to_path_buf());
                    found = true;
                }
            }
        }
    }

    if !found {
        println!(
            "{}",
            format!("ERROR: Cant find the file labeled {}.", filename)
                .red()
                .bold()
        );
    }

    if let Some(ref pathbuf) = fullpathfile {
        if let Some(path_str) = pathbuf.to_str() {
            match ask_is_wav(path_str) {
                Ok(true) => {
                    println!("{}", "WAV file detected.".green());
                    needs_to_convert = false;
                }
                Ok(false) => {
                    println!("{}", "WARNING: Not a WAV file (possibly a MP3?)".yellow());
                    needs_to_convert = true;
                }
                Err(e) => println!("{}", format!("ERROR: Cant read file: {}", e).red().bold()),
            }
        } else {
            println!("{}", "ERROR: File path is not valid.".red().bold());
        }
    } else {
        println!("{}", "ERROR: File not found.".red().bold());
    }
    if needs_to_convert == true {
        print!("Would you like to convert the file to WAV? (Y/N): ");
        io::Write::flush(&mut io::stdout()).expect("flush error");
        let mut convert_yn: String = String::new();
        io::stdin()
            .read_line(&mut convert_yn)
            .expect("Error reading.");
        let convert_yn = convert_yn.trim();

        if convert_yn.to_uppercase() == "Y" {
            let status = Command::new("sox")
                .arg(fullpathfile.as_ref().unwrap())
                .arg("-r")
                .arg("22050")
                .arg("-c")
                .arg("1")
                .arg("-b")
                .arg("16")
                .arg("-t")
                .arg("wav")
                .arg("temp.wav")
                .status()
                .expect("Failed to execute sox");
            if status.success() {
                println!("{}", "Conversion succeeded!".green());
            } else {
                println!("{}", "ERROR: Conversion failed!".red().bold());
            }
        } else if convert_yn.to_uppercase() == "N" {
            println!(
                "{}",
                "ERROR: Cant use this program with MP3s, please convert to WAV manually or here."
                    .red()
                    .bold()
            );
            return;
        } else {
            println!("Invalid option.");
            return;
        }
        let _ = transmit_file(true, fullpathfile.as_ref().unwrap());
    } else if needs_to_convert == false {
        let _ = transmit_file(false, fullpathfile.as_ref().unwrap());
    }
}

fn ask_is_wav(filename: &str) -> io::Result<bool> {
    let mut file = File::open(filename)?;
    let mut header = [0u8; 12];
    file.read_exact(&mut header)?;

    Ok(&header[0..4] == b"RIFF" && &header[8..12] == b"WAVE")
}

fn transmit_file(
    needs_to_convert: bool,
    original_file: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    print!("On what frequency should the file be transferred on? (76-108 MHz): ");
    io::Write::flush(&mut io::stdout())?;

    let mut freq: String = String::new();
    io::stdin().read_line(&mut freq)?;
    let freq = freq.trim();

    let mut path: PathBuf = env::current_dir()?;
    if needs_to_convert {
        path.push("temp.wav");
    } else {
        path = original_file.clone();
    }

    let mut child: Child = Command::new("sudo")
        .arg("pi_fm_rds")
        .arg("-freq")
        .arg(freq)
        .arg("-audio")
        .arg(&path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .spawn()?;

    println!("{}", "Transmitting... Press Enter to stop.".green());

    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;

    child.kill()?;
    child.wait()?;

    println!("Transmission stopped.");
    Ok(())
}
