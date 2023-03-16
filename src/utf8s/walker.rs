use std::future::Future;
use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;
use futures_core::future::BoxFuture;

#[async_trait]
pub trait SimpleWalkerProcessDelegate: 'static + Sync + Send {
    async fn process_file(&self, ent_path: &str) -> anyhow::Result<()>;
}

type ProxyFn =
Box<dyn for<'a> Fn(&'a str) -> BoxFuture<'a, anyhow::Result<()>> + 'static + Send + Sync>;

pub struct SimpleScanner {
    recursive: bool,
    max_depth: Option<u32>,
    suffix: String,
    proxy: ProxyFn,
}

pub struct AsyncClosure<'a>(BoxFuture<'a, anyhow::Result<()>>);

impl<'a, T> From<T> for AsyncClosure<'a>
    where T: Future<Output=anyhow::Result<()>> + 'a + Send {
    fn from(value: T) -> Self {
        Self(Box::pin(value))
    }
}

impl SimpleScanner {
    pub fn new<Closure>(
        recursive: bool,
        max_depth: Option<u32>,
        suffix: String,
        proxy: Closure,
    ) -> Self
        where Closure: for<'a> Fn(&'a str) -> AsyncClosure<'a> + Sync + Send + 'static {
        Self {
            recursive,
            max_depth,
            suffix,
            proxy: Box::new(move |s| proxy(s).0),
        }
    }
}

#[async_trait]
impl ScanBot for SimpleScanner {
    fn should_recursive(&self, cur_depth: u32) -> bool {
        self.recursive && (self.max_depth.is_none() || self.max_depth <= Some(cur_depth))
    }

    fn match_file(&self, name: &str) -> bool {
        name.ends_with(&self.suffix)
    }

    async fn process_file(&self, ent_path: &str) -> anyhow::Result<()> {
        (self.proxy)(ent_path).await
    }
}

#[async_trait]
pub trait ScanBot: Sync + Send {
    fn should_recursive(&self, cur_depth: u32) -> bool;
    fn match_file(&self, name: &str) -> bool;
    async fn process_file(&self, ent_path: &str) -> anyhow::Result<()>;
}

fn scan_impl<T: ScanBot + 'static>(
    p: &str,
    cfg: Arc<T>,
    cur_depth: u32,
) -> BoxFuture<'static, anyhow::Result<()>> {
    let p: String = p.into();
    Box::pin(async move {
        let mut child_tasks = tokio::task::JoinSet::new();
        let mut d = tokio::fs::read_dir(&p)
            .await
            .with_context(|| format!("read dir:{}", &p))?;
        while let Some(ent) = d.next_entry().await? {
            let ent_path = ent.path();
            let ent_path = ent_path.to_str().unwrap();
            let ft = ent
                .file_type()
                .await
                .with_context(|| format!("get file type:{}", ent_path))?;
            if ft.is_file() && cfg.match_file(ent_path) {
                // submit to parse
                println!("process file:{}", ent_path);
                // process
                cfg.process_file(ent_path)
                    .await
                    .with_context(|| format!("process:{}", ent_path))?;
            } else if ft.is_dir() && cfg.should_recursive(cur_depth) {
                println!("enter dir:{}", ent_path);
                child_tasks.spawn(scan_impl(ent_path, cfg.clone(), cur_depth + 1));
            }
        }
        while let Some(r) = child_tasks.join_next().await {
            if let Err(e) = r.unwrap() {
                println!("task failed:{}", e)
            }
        }
        Ok(())
    })
}

pub async fn scan<T: ScanBot + 'static>(path: &str, cfg: T) -> anyhow::Result<()> {
    scan_impl(path, Arc::new(cfg), 0).await
}
