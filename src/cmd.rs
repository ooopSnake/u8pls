use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version)]
pub struct ScanArgs {
    /// target dir
    pub dir: String,
    /// recursive on sub dir
    #[arg(short, default_value_t = true)]
    pub recursive: bool,
    /// max sub dir depth
    #[arg(short = 'd')]
    pub max_depth: Option<usize>,
    /// max concurrency, increase if you change system `max_open_file` property
    #[arg(short = 'j', default_value_t = 32)]
    pub max_concurrency: usize,
    #[command(subcommand)]
    pub matcher: Expr,
}

#[derive(Subcommand, Clone, Debug)]
pub enum Expr {
    /// match file suffix like: ".txt"
    Suffix {
        suffix: String
    },
    /// match file prefix eg:
    /// "data_" will match: data_1.docx,data_2.bin
    Prefix {
        prefix: String
    },
    /// advance match pattern eg: `202\d_[a-z]{3}\.txt` match `2023_abc.txt`
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

pub fn parse() -> ScanArgs {
    let out = ScanArgs::parse();
    out.matcher.can_match(""); // make sure regexp init
    out
}