use std::io::BufReader;

mod cli;
mod data;
mod fs;
mod math;
mod parser;
mod utils;

fn main() -> std::io::Result<()> {
    let matches = crate::cli::build_cli().get_matches();

    let mut config;

    if let Some(config_file) = matches.get_one::<String>("config") {
        config = crate::data::Config::load(config_file);
    } else {
        config = crate::data::Config::load(data::CONFIG);
    }

    if let Some(init) = matches.get_one::<u8>("init") {
        if init > &0 {
            let _ = config.gen_example_config();
            return Ok(());
        }
    }

    if let Some(dump_dir) = matches.get_one::<String>("dump") {
        config.dump_dir = Some(dump_dir.to_string());
    }

    if let Some(depth) = matches.get_one::<usize>("depth") {
        config.depth = Some(*depth);
    }

    let parse: Vec<(Option<String>, Option<String>)>;

    let mut set = vec![];

    // TODO: This is such a mess, hopefully I find the spoons to fix it some
    // day... as if...
    //
    // This decides what binary to use:
    // 1. If user provided `run` flag, we use what they provide
    // 2. Else if the config has a `gen_binaries` record, we use that
    // 3. Else, we use stdin
    if let Some(run) = matches.get_one::<String>("run") {
        parse = match crate::utils::get_help(run, &[]) {
            Ok(parse) => crate::parser::parse(BufReader::new(parse.as_slice())),
            Err(e) => panic!("{:?}", e),
        };
    // NOTE: for some reason this is always true for u8, but I don't trust it
    } else if let Some(stdin) = matches.get_one::<u8>("stdin") {
        // If --stdin set
        #[allow(clippy::suspicious_else_formatting)]
        if stdin > &0 {
            parse = crate::parser::parse(std::io::stdin().lock());
        } else {
            // HACK: We fake being a parsed help... obviously bad.
            let mut res = vec![];
            for (k, _) in config.commands.as_ref().unwrap().iter() {
                res.push(match k {
                    (Some(short), Some(long)) => (
                        Some(short.as_str().to_string()),
                        Some(long.as_str().to_string()),
                    ),
                    (Some(short), None) => (Some(short.as_str().to_string()), None),
                    (None, Some(long)) => (None, Some(long.as_str().to_string())),
                    (None, None) => (None, None),
                })
            }
            parse = res;
        }
    } else {
        unreachable!();
    }

    crate::parser::populate_set(&config, &parse, &mut set);

    let powerset = math::generate_powerset(&set, config.depth.unwrap());

    let output_strings: Vec<String> = powerset
        .iter()
        .map(|subset| {
            format!(
                "bin.name = \"{}\"\nargs = \"{} {}\"\n",
                &config.binary.as_ref().unwrap(),
                &config.args.as_ref().unwrap(),
                subset.join(" ")
            )
        })
        .collect();

    let dump_dir = config.dump_dir.unwrap();

    crate::fs::dump_dir(&dump_dir, output_strings);

    Ok(())
}
