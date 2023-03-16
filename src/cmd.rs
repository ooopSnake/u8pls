use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Arg {
    dir: String,
    #[arg(short, default_value_t = 1)]
    recursive: i32,
    #[command(subcommand)]
    expr: Expr,
}

#[derive(Subcommand, Debug)]
pub enum Expr {
    Suffix {
        suffix: String
    },
    Prefix {
        prefix: String
    },
    Regexp {
        regexp: String
    },
}


static mut _CACHED_REGEX: Option<regex::Regex> = None;

impl Expr {
    pub fn can_match(&self, target: &str) -> bool {
        match self {
            Expr::Suffix { suffix } => {
                target.ends_with(suffix)
            }
            Expr::Prefix { prefix } => {
                target.starts_with(prefix)
            }
            Expr::Regexp { regexp } => {
                let re = unsafe {
                    if _CACHED_REGEX.is_none() {
                        let v = regex::Regex::new(regexp).expect("bad regex!");
                        _CACHED_REGEX = Some(v)
                    }
                    _CACHED_REGEX.as_ref().unwrap()
                };
                re.is_match(target)
            }
        }
    }
}

pub fn parse() -> Arg {
    Arg::parse()
}