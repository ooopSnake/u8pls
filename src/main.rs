use std::path::PathBuf;

use u8pls::{cmd, file_util, utf8s};

async fn use_func(p: PathBuf) -> anyhow::Result<()> {
    let ent_path = p.as_path();
    let f = file_util::read(ent_path).await?;
    let u8encoded = utf8s::Coding::new(&f).parse().await?;
    file_util::write(ent_path, u8encoded).await?;
    Ok(())
}

/// way1: use closure
/// ```rust
/// use closure
/// utf8s::scan(
///     &args.dir,
///     utf8s::ScannerExec::new_with_closure(
///         args.recursive,
///         args.max_depth,
///         args.matcher.clone(),
///         args.max_concurrency,
///         |ent_path| { // ent_path is &Path not PathBuf
///             async move {
///                  let ent_path = p.as_path();
///                  let f = file_util::read(ent_path).await?;
///                  let u8encoded = utf8s::Coding::new(&f).parse().await?;
///                  file_util::write(ent_path, u8encoded).await?;
///             }.into()
///         }),
/// ).await
/// ```
/// way2: use async function
/// ```
/// utf8s::scan(
///         &args.dir,
///         utf8s::ScannerExec::new_with_fn(
///             args.recursive,
///             args.max_depth,
///             args.matcher.clone(),
///             args.max_concurrency,
///             use_func),
///     ).await
/// ```
///
/// ```rust
/// utf8s::scan(
///     &args.dir,
///     utf8s::ScannerExec::new_with_fn(
///         args.recursive,
///         args.max_depth,
///         args.matcher.clone(),
///         args.max_concurrency,
///         |p: PathBuf| {
///             Box::pin(async move {
///                 let ent_path = p.as_path();
///                 let f = file_util::read(ent_path).await?;
///                 let u8encoded = utf8s::Coding::new(&f).parse().await?;
///                 file_util::write(ent_path, u8encoded).await?;
///                 Ok(())
///             })
///         }),
/// ).await
/// ```
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = cmd::parse();
    utf8s::scan(
        &args.dir,
        utf8s::ScannerExec::new_with_fn(
            args.recursive,
            args.max_depth,
            args.matcher.clone(),
            args.max_concurrency,
            use_func),
    ).await
}
