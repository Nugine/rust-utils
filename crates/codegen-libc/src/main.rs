use anyhow::Result;
use camino::Utf8PathBuf;
use clap::Parser;
use codegen_libc::CfgItem;
use regex::RegexSet;
use std::io::Write as _;
use std::ops::Not;

#[derive(clap::Parser)]
struct Opt {
    #[clap(long)]
    libc: Utf8PathBuf,

    filters: Vec<String>,
}

fn main() -> Result<()> {
    env_logger::init();
    let opt = Opt::parse();

    anyhow::ensure!(opt.filters.is_empty().not(), "no filters specified");

    let re = RegexSet::new(&opt.filters)?;
    let ans = codegen_libc::search(&opt.libc, &re)?;

    let mut stdout = std::io::stdout().lock();
    for CfgItem { cfg, name } in ans {
        writeln!(stdout, "#[cfg({cfg})]\npub use libc::{name};\n")?;
    }

    Ok(())
}
