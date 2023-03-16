use std::future::Future;
use std::path::Path;
use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;
use futures_core::future::BoxFuture;

use crate::cmd;

type ProxyFn =
Box<dyn for<'a> Fn(&'a Path) -> BoxFuture<'a, anyhow::Result<()>> + 'static + Send + Sync>;

pub struct ScannerExec {
    recursive: bool,
    max_depth: Option<u32>,
    matcher: cmd::Expr,
    proxy: ProxyFn,
}

pub struct AsyncClosure<'a>(BoxFuture<'a, anyhow::Result<()>>);

impl<'a, T> From<T> for AsyncClosure<'a>
    where T: Future<Output=anyhow::Result<()>> + 'a + Send {
    fn from(value: T) -> Self {
        Self(Box::pin(value))
    }
}

impl ScannerExec {
    pub fn new_with_closure<Closure>(
        recursive: bool,
        max_depth: Option<u32>,
        matcher: cmd::Expr,
        proxy: Closure,
    ) -> Self
        where Closure: for<'a> Fn(&'a Path) -> AsyncClosure<'a> + Sync + Send + 'static {
        Self {
            recursive,
            max_depth,
            matcher,
            proxy: Box::new(move |s| proxy(s).0),
        }
    }
}

#[async_trait]
pub trait Scanner: Sync + Send {
    fn should_recursive(&self, cur_depth: u32) -> bool;
    fn match_file(&self, file_name: &str) -> bool;
    async fn process_file(&self, file_path: &Path) -> anyhow::Result<()>;
}

#[async_trait]
impl Scanner for ScannerExec {
    fn should_recursive(&self, cur_depth: u32) -> bool {
        self.recursive && (self.max_depth.is_none() || self.max_depth <= Some(cur_depth))
    }

    fn match_file(&self, file_name: &str) -> bool {
        self.matcher.can_match(file_name)
    }

    async fn process_file(&self, ent_path: &Path) -> anyhow::Result<()> {
        (self.proxy)(ent_path).await
    }
}

fn scan_impl<T: Scanner + 'static>(
    p: &Path,
    cfg: Arc<T>,
    cur_depth: u32,
) -> BoxFuture<'static, anyhow::Result<()>> {
    let p = p.to_path_buf();
    Box::pin(async move {
        let mut child_tasks = tokio::task::JoinSet::new();
        let mut d = tokio::fs::read_dir(&p)
            .await
            .with_context(|| format!("read dir:{:?}", &p))?;
        while let Some(ent) = d.next_entry().await? {
            let ent_path_buf = ent.path();
            let ent_path = ent_path_buf.as_path();
            let ft = ent
                .file_type()
                .await
                .with_context(|| format!("get file type:{:?}", ent_path))?;
            let file_name = ent_path.file_name()
                .unwrap_or_default()
                .to_str().
                unwrap_or_default();
            if ft.is_file() && cfg.match_file(file_name) {
                // submit to parse
                println!("process file:{:?}", ent_path);
                // process
                cfg.process_file(ent_path)
                    .await
                    .with_context(|| format!("process:{:?}", ent_path))?;
            } else if ft.is_dir() && cfg.should_recursive(cur_depth) {
                println!("enter dir:{:?}", ent_path);
                child_tasks.spawn_local(scan_impl(ent_path,
                                                  cfg.clone(),
                                                  cur_depth + 1));
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


pub async fn scan<P: AsRef<Path>, T: Scanner + 'static>(path: P,
                                                        cfg: T) -> anyhow::Result<()> {
    scan_impl(path.as_ref(),
              Arc::new(cfg),
              0).await
}
