use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version)]
pub struct ScanArgs {
    pub dir: String,
    #[arg(short, default_value_t = true)]
    pub recursive: bool,
    pub max_depth: Option<u32>,
    #[command(subcommand)]
    pub matcher: Expr,
}

#[derive(Subcommand, Clone)]
pub enum Expr {
    Suffix {
        /// match file suffix like: ".txt"
        suffix: String
    },
    Prefix {
        /// match file prefix eg:
        /// "data_" will match: data_1.docx,data_2.bin
        prefix: String
    },
    Regexp {
        /// regexp
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

pub fn parse() -> ScanArgs {
    let out = ScanArgs::parse();
    out.matcher.can_match(""); // make sure regexp init
    out
}