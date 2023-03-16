use u8pls::{cmd, file_util, utf8s};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = cmd::parse();
    // use closure
    utf8s::scan(
        &args.dir,
        utf8s::ScannerExec::new_with_closure(
            args.recursive,
            args.max_depth,
            args.matcher.clone(),
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
