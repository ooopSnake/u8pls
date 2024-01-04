use std::io::SeekFrom;
use std::path::Path;

use anyhow::Context;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

pub async fn read<T: AsRef<Path>>(file_path: T) -> anyhow::Result<Vec<u8>> {
    let file_path = file_path.as_ref();
    let mut f = tokio::fs::File::open(file_path).await.context("guess: open file")?;
    let f_len = f.seek(SeekFrom::End(0)).await.context("seek end")?;
    let mut buf: Vec<u8> = vec![0; f_len as usize];
    f.seek(SeekFrom::Start(0)).await.context(format!("seek start:{:?}", file_path))?;
    f.read_exact(&mut buf).await.context(format!("guess: read to end:{:?}", file_path))?;
    Ok(buf)
}

pub async fn write<T: AsRef<Path>, D: AsRef<[u8]>>(file_path: T, data: D) -> anyhow::Result<()> {
    let file_path = file_path.as_ref();
    let mut f = tokio::fs::OpenOptions::new()
        .truncate(true)
        .write(true)
        .open(file_path)
        .await
        .context(format!("write_file: open&truncate:{:?}", file_path))?;
    f.write_all(data.as_ref())
        .await
        .context(format!("write_file: write all:{:?}", file_path))
}
