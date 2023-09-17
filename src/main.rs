use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::Path;

use clap::{command, crate_authors, Arg};

mod data;
mod math;
mod parser;

const CONFIG: &'static str = ".ptest.yaml";

pub mod utils {
    use std::io::{self, BufReader};
    use std::process::{Command, Output};

    pub fn get_help(
        command: &str,
        _args: &[&str],
        //) -> Result<BufReader<&'a [u8]>, std::io::Error> {
    ) -> Result<Vec<u8>, std::io::Error> {
        // TODO: parse args
        // let output: Output = Command::new(command).args(args).output()?;
        let output: Output = Command::new(command).args(["--help"]).output()?;

        if !output.status.success() {
            eprintln!("Command failed with status: {:?}", output.status);
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Command execution failed",
            ));
        }

        //Ok(BufReader::new(output.stdout.as_slice()))
        Ok(output.stdout)
    }
}

fn main() -> std::io::Result<()> {
    let config = crate::data::Config::load(CONFIG);

    let matches = command!()
        .author(crate_authors!("\n"))
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("config")
                .help("Specify config file"),
        )
        .arg(
            Arg::new("dump")
                .short('d')
                .long("dump")
                .value_name("dump")
                .help("Specify dump dir"),
        )
        .arg(
            Arg::new("run")
                .short('r')
                .long("run")
                .value_name("run")
                .help("Specify help command"),
        )
        //.arg(arg!(-s --short ... "Shows a short aporism."))
        .get_matches();

    if let Some(_config) = matches.get_one::<String>("config") {
        todo!()
    } else if let Some(_dump) = matches.get_one::<String>("dump") {
        todo!()
    }

    let parse: Vec<(Option<String>, Option<String>)>;

    if let Some(run) = matches.get_one::<String>("run") {
        //println!("{:?}", crate::utils::get_help(run, &[]));
        parse = match crate::utils::get_help(run, &[]) {
            Ok(parse) => crate::parser::parse(BufReader::new(parse.as_slice())),
            Err(e) => panic!("{:?}", e),
        };
    } else {
        parse = crate::parser::parse(std::io::stdin().lock());
    }

    let mut set = vec![];

    for option in parse {
        if let Some((key, stuff)) = config.commands.as_ref().unwrap().get_key_value(&option) {
            match key {
                (Some(left), Some(right)) => {
                    for value in stuff.values.as_ref().unwrap() {
                        set.push(format!("{} {}", left, &value));
                        set.push(format!("{} {}", right, &value));
                    }
                }
                (Some(left), None) => {
                    for value in stuff.values.as_ref().unwrap() {
                        set.push(format!("{} {}", left, &value));
                    }
                }
                (None, Some(right)) => {
                    for value in stuff.values.as_ref().unwrap() {
                        set.push(format!("{} {}", right, &value));
                    }
                }
                (None, None) => todo!(),
            }
        } else {
            match &option {
                (Some(left), Some(right)) => {
                    set.push(format!("{}", left));
                    set.push(format!("{}", right));
                }
                (Some(left), None) => {
                    set.push(format!("{}", left));
                }
                (None, Some(right)) => {
                    set.push(format!("{}", right));
                }
                (None, None) => todo!(),
            }
        }
    }

    // println!("{set:?}");

    // println!("{set:#?}");

    let powerset = math::generate_powerset(&set, config.depth.unwrap());

    // println!("{powerset:#?}");

    let output_strings: Vec<String> = powerset
        .iter()
        .map(|subset| {
            format!(
                "bin.name = \"{}\"\nargs = \"{} {}\"",
                &config.binary.as_ref().unwrap(),
                &config.args.as_ref().unwrap(),
                subset.join(" ")
            )
        })
        .collect();

    println!("{output_strings:#?}");

    // println!("Output Strings: {:#?}", output_strings);

    // Create the dump directory if it doesn't exist
    let dump_path = Path::new("dump");
    if !dump_path.exists() {
        fs::create_dir(dump_path).expect("Failed to create dump directory");
    }

    // Write each string in output_strings to a new file in the dump directory
    for content in &output_strings {
        // Extract the arguments part from the content
        let args_line = content.lines().nth(1).unwrap_or("");
        let args = args_line
            .split("args = \"")
            .nth(1)
            .unwrap_or("")
            .trim_end_matches('\"')
            .replace(" ", "_")
            .replace("/", "_"); // This is to handle the "tests/itest" in ARGS

        let file_path = dump_path.join(format!("ptest_{}.toml", args));
        let mut file = File::create(file_path).expect("Failed to create file");
        file.write_all(content.as_bytes())
            .expect("Failed to write to file");
    }

    Ok(())
}
