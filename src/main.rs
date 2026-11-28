//! Entry point of `satop`, ported from `bin/satop` and `lib/cli.js`.

use std::process::ExitCode;

use clap::Parser;
use satoshitop::app;
use satoshitop::config::{self, Cli, Config};

fn main() -> ExitCode {
    // Clap handles `--help` and invalid options itself (exit code 2).
    let cli = Cli::parse();

    if cli.version {
        println!("{}", config::cli::HEADER);
        return ExitCode::SUCCESS;
    }

    let config = match config::home_dir().and_then(|home| Config::load(&cli, &home)) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("satop: {e}");
            return ExitCode::from(2);
        }
    };

    if config.verbose {
        eprintln!("{}", config.verbose_json());
    }

    if cli.confgen {
        let (stdin, stdout, stderr) = (std::io::stdin(), std::io::stdout(), std::io::stderr());
        return match config::confgen(
            &config.conf,
            &mut stdin.lock(),
            &mut stdout.lock(),
            &mut stderr.lock(),
        ) {
            Ok(_) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("{e}");
                ExitCode::FAILURE
            }
        };
    }

    if let Err(e) = app::run(config) {
        eprintln!("satop: {e}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
