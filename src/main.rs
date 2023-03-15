use async_trait::async_trait;

mod file_util;
mod utf8s;

struct ExampleTraitScanner {
    recursive: bool,
    max_depth: Option<u32>,
    suffix: String,
}

#[async_trait]
impl utf8s::ScanBot for ExampleTraitScanner {
    fn should_recursive(&self, cur_depth: u32) -> bool {
        self.recursive && (self.max_depth.is_none() || self.max_depth <= Some(cur_depth))
    }

    fn match_file(&self, name: &str) -> bool {
        name.ends_with(&self.suffix)
    }

    async fn process_file(&self, ent_path: &str) -> anyhow::Result<()> {
        let ent_path = ent_path.to_string();
        let f = file_util::read(&ent_path).await?;
        let u8encoded = utf8s::Coding::new(&f).parse().await?;
        file_util::write(&ent_path, u8encoded).await?;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // use closure
    utf8s::walk(
        ".",
        utf8s::SimpleScanner::new(true, None, ".h".into(), |ent_path| {
            async move {
                let f = file_util::read(ent_path).await?;
                let u8encoded = utf8s::Coding::new(&f).parse().await?;
                file_util::write(ent_path, u8encoded).await?;
                Ok(())
            }
                .into()
        }),
    )
        .await?;

    // use trait
    utf8s::walk(
        ".",
        ExampleTraitScanner {
            recursive: true,
            max_depth: None,
            suffix: ".c".into(),
        },
    )
        .await
}
