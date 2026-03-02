use anyhow::Result;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::format::FmtSpan;

use wty::{
    cli::{Cli, Command, LangSpecs},
    dict::{
        DGlossary, DGlossaryExtended, DIpa, DIpaMerged, DMain, find_or_download_jsonl, make_dict,
        release::release,
    },
    lang::{Edition, Lang},
    path::PathManager,
};

fn init_logger(verbose: bool) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        if verbose {
            // Only we are set to debug. ureq and other libs stay the same.
            EnvFilter::new(format!("{}=debug", env!("CARGO_PKG_NAME")))
        } else {
            EnvFilter::new("warn")
        }
    });

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_span_events(FmtSpan::CLOSE)
        .with_timer(tracing_subscriber::fmt::time::ChronoLocal::new(
            "%H:%M:%S".to_string(),
        ))
        .init();
}

#[tracing::instrument(skip_all, level = "debug")]
fn run(cmd: Command) -> Result<()> {
    tracing::trace!("{:#?}", cmd);

    match cmd {
        Command::Main(args) => make_dict(DMain, args),
        Command::Glossary(args) => make_dict(DGlossary, args),
        Command::GlossaryExtended(args) => make_dict(DGlossaryExtended, args),
        Command::Ipa(args) => make_dict(DIpa, args),
        Command::IpaMerged(args) => make_dict(DIpaMerged, args),
        Command::Download(args) => {
            // NOTE: uses MainArgs, so it expects two language codes.
            let langs: LangSpecs = args.langs.clone().try_into()?;
            let source: Lang = langs.source.try_into().unwrap();
            let edition: Edition = langs.edition.try_into().unwrap();
            let pm = PathManager::try_from(args)?;

            let _ = std::fs::create_dir(pm.dir_kaik());

            let _ = find_or_download_jsonl(edition, Some(source), &pm)?;
            Ok(())
        }
        Command::Iso(args) => {
            if args.edition {
                println!("{}", Lang::help_editions());
            } else {
                println!("{}", Lang::help_isos_coloured());
            }
            Ok(())
        }
        Command::Release(args) => release(args),
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse_cli();
    init_logger(cli.verbose);
    run(cli.command)
}
