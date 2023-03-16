use std::fs::FileType;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use futures_core::future::BoxFuture;

use crate::cmd;

type ProxyFn =
Box<dyn for<'a> Fn(&'a Path) -> BoxFuture<'a, anyhow::Result<()>> + 'static + Send + Sync>;

pub struct ScannerExec {
    recursive: bool,
    max_depth: Option<usize>,
    matcher: cmd::Expr,
    task_sema: tokio::sync::Semaphore,
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
        max_depth: Option<usize>,
        matcher: cmd::Expr,
        max_task: usize,
        proxy: Closure,
    ) -> Self
        where Closure: for<'a> Fn(&'a Path) -> AsyncClosure<'a> + Sync + Send + 'static {
        Self {
            recursive,
            max_depth,
            matcher,
            task_sema: tokio::sync::Semaphore::new(max_task),
            proxy: Box::new(move |s| proxy(s).0),
        }
    }
}

#[async_trait]
pub trait Scanner: Sync + Send {
    fn should_recursive(&self, cur_depth: usize) -> bool;
    fn match_file(&self, file_name: &str) -> bool;
    async fn process_file(&self, file_path: &Path) -> anyhow::Result<()>;
    async fn read_dir(&self, p: &Path) -> anyhow::Result<Vec<(PathBuf, FileType)>>;
}

#[async_trait]
impl Scanner for ScannerExec {
    fn should_recursive(&self, cur_depth: usize) -> bool {
        self.recursive && (self.max_depth.is_none() || self.max_depth <= Some(cur_depth))
    }

    fn match_file(&self, file_name: &str) -> bool {
        self.matcher.can_match(file_name)
    }

    async fn process_file(&self, ent_path: &Path) -> anyhow::Result<()> {
        let _guard = self.task_sema.acquire().await.expect("never fail");
        (self.proxy)(ent_path).await
    }

    async fn read_dir(&self, p: &Path) -> anyhow::Result<Vec<(PathBuf, FileType)>> {
        let _guard = self.task_sema.acquire().await.expect("never fail");
        let pb = p.to_path_buf();
        tokio::task::spawn_blocking(
            move || {
                let p = pb.as_path();
                let child: Vec<(PathBuf, FileType)> = std::fs::read_dir(p)?
                    .filter_map(|d| {
                        if let Ok(v) = d {
                            if let Ok(ft) = v.file_type() {
                                return Some((v.path(), ft));
                            }
                        }
                        None
                    })
                    .collect();
                Ok(child)
            }).await
            .expect("never fail")
    }
}

fn scan_impl<T: Scanner + 'static>(
    p: &Path,
    scanner: Arc<T>,
    cur_depth: usize,
) -> BoxFuture<'static, anyhow::Result<()>> {
    let p = p.to_path_buf();
    Box::pin(async move {
        let mut child_tasks = tokio::task::JoinSet::new();
        let d = scanner.read_dir(&p).await?;
        for (ent_path_buf, ft) in d {
            let ent_path = ent_path_buf.as_path();
            let file_name = ent_path.file_name()
                .unwrap_or_default()
                .to_str().
                unwrap_or_default();
            if ft.is_file() && scanner.match_file(file_name) {
                // process
                {
                    let scanner = scanner.clone();
                    let fp = ent_path_buf.clone();
                    // fuck
                    child_tasks.spawn(async move {
                        let ent_path = fp.as_path();
                        println!("process file:{:?}", ent_path);
                        scanner.process_file(ent_path).await
                    });
                }
            } else if ft.is_dir() && scanner.should_recursive(cur_depth) {
                child_tasks.spawn(scan_impl(ent_path,
                                            scanner.clone(),
                                            cur_depth + 1));
            }
        }
        while let Some(r) = child_tasks.join_next().await {
            if let Err(e) = r.unwrap() {
                println!("task failed: {:?}", e)
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
