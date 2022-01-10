use std::path::PathBuf;
use structopt::*;
use strum::{EnumString, EnumVariantNames, IntoStaticStr, VariantNames};

#[derive(StructOpt, Debug)]
#[structopt(name = "dmt")]
pub struct Cmd {
    #[structopt(
        short,
        long,
        possible_values = Format::VARIANTS,
        case_insensitive = true,
        default_value = Format::Auto.into(),
    )]
    pub from: Format,

    #[structopt(
        short,
        long,
        possible_values = Format::VARIANTS,
        case_insensitive = true,
        default_value = Format::Auto.into(),
    )]
    pub to: Format,

    #[structopt(short, long, parse(from_os_str))]
    pub input: Option<PathBuf>,

    #[structopt(short, long, parse(from_os_str))]
    pub output: Option<PathBuf>,

    #[structopt(short, long)]
    pub expr: Option<String>,
}

#[derive(EnumString, EnumVariantNames, IntoStaticStr, Debug)]
#[strum(serialize_all = "kebab_case")]
pub enum Format {
    Auto,
    Json,
    Yaml,
    Toml,
}
