use std::path::Path;

use async_trait::async_trait;

use u8pls::{cmd, file_util, utf8s};
use u8pls::cmd::ScanArgs;

struct ExampleTraitScanner {
    sa: ScanArgs,
}

#[async_trait]
impl utf8s::ScanBot for ExampleTraitScanner {
    fn should_recursive(&self, cur_depth: u32) -> bool {
        self.recursive && (self.max_depth.is_none() || self.max_depth <= Some(cur_depth))
    }

    fn match_file(&self, name: &str) -> bool {
        name.ends_with(&self.suffix)
    }

    async fn process_file(&self, ent_path: &Path) -> anyhow::Result<()> {
        let f = file_util::read(&ent_path).await?;
        let u8encoded = utf8s::Coding::new(&f).parse().await?;
        file_util::write(&ent_path, u8encoded).await?;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let scan_args = cmd::parse();
    // use closure
    utf8s::scan(
        ".",
        utf8s::SimpleScanner::new(
            scan_args,
            |ent_path| {
                async move {
                    let f = file_util::read(ent_path).await?;
                    let u8encoded = utf8s::Coding::new(&f).parse().await?;
                    file_util::write(ent_path, u8encoded).await?;
                    Ok(())
                }.into()
            }),
    ).await
}
