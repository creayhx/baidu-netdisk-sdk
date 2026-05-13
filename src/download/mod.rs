//! File downloading functionality for Baidu NetDisk
//!
//! This module provides multiple download strategies:
//! - Auto download: selects optimal strategy based on file size
//! - Single-threaded: for small files (<10MB)
//! - Concurrent streaming: for large files, using futures buffer_unordered
//! - Parallel multi-threaded: using producer-consumer pattern
//!
//! All methods support both file path and fs_id for identifying files.
//!
//! # Quick Start
//!
//! ```
//! use baidu_netdisk_sdk::BaiduNetDiskClient;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = BaiduNetDiskClient::builder().build()?;
//! client.load_token_from_env()?;
//!
//! // Auto download (recommended)
//! client.download().auto_download("/myfile.txt", "./downloaded.txt").await?;
//!
//! // Or use single-threaded for small files
//! client.download().download_single("/small.txt", "./small.txt").await?;
//!
//! // Or use streaming for large files
//! client.download().download_streaming("/large.zip", "./large.zip", 4).await?;
//! # Ok(())
//! # }
//! ```
use futures::stream::{self, StreamExt, TryStreamExt};
use log::{debug, info};
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{mpsc, Semaphore};
use tokio::task;

use crate::client::TokenGetter;
use crate::errors::{NetDiskError, NetDiskResult};
use crate::file::FileClient;
use crate::file::FileMeta;

/// Download client for Baidu NetDisk
#[derive(Debug, Clone)]
pub struct DownloadClient {
    file_client: Arc<FileClient>,
    token_getter: Arc<dyn TokenGetter>,
}

impl DownloadClient {
    /// Create a new DownloadClient instance
    ///
    /// Usually you don't need to call this directly - use `BaiduNetDiskClient::download()` instead.
    pub fn new(file_client: Arc<FileClient>, token_getter: Arc<dyn TokenGetter>) -> Self {
        Self {
            file_client,
            token_getter,
        }
    }

    /// Get download link (dlink) from file path
    ///
    /// # Examples
    ///
    /// ```
    /// use baidu_netdisk_sdk::BaiduNetDiskClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = BaiduNetDiskClient::builder().build()?;
    /// client.load_token_from_env()?;
    ///
    /// let file_meta = client.download()
    ///     .get_dlink_from_path("/myfile.txt")
    ///     .await?;
    ///
    /// if let Some(dlink) = file_meta.dlink {
    ///     println!("Download link: {}", dlink);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_dlink_from_path(&self, path: &str) -> NetDiskResult<FileMeta> {
        let file_info = self.file_client.get_file_info(path).await?;
        let fs_id = file_info
            .fs_id
            .ok_or_else(|| NetDiskError::api_error(-1, "File has no fs_id"))?;
        self.file_client.get_file_meta(fs_id).await
    }

    /// Get download link (dlink) from file fs_id
    ///
    /// # Examples
    ///
    /// ```
    /// use baidu_netdisk_sdk::BaiduNetDiskClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = BaiduNetDiskClient::builder().build()?;
    /// client.load_token_from_env()?;
    ///
    /// let file_meta = client.download()
    ///     .get_dlink_from_fsid(123456)
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_dlink_from_fsid(&self, fs_id: u64) -> NetDiskResult<FileMeta> {
        self.file_client.get_file_meta(fs_id).await
    }

    /// Auto download based on file size with optimal strategy
    ///
    /// Uses file path to locate the file.
    /// Recommended for most use cases.
    ///
    /// - Small files (<10MB): single-threaded download
    /// - Large files (>=10MB): concurrent streaming with 4 workers
    ///
    /// # Examples
    ///
    /// ```
    /// use baidu_netdisk_sdk::BaiduNetDiskClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = BaiduNetDiskClient::builder().build()?;
    /// client.load_token_from_env()?;
    ///
    /// client.download()
    ///     .auto_download("/myfile.txt", "./downloaded.txt")
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn auto_download(
        &self,
        path: &str,
        save_path: impl AsRef<Path>,
    ) -> NetDiskResult<()> {
        let file_meta = self.get_dlink_from_path(path).await?;
        self.auto_download_with_meta(&file_meta, save_path).await
    }

    /// Auto download based on file size with optimal strategy
    ///
    /// Uses file fs_id to locate the file.
    pub async fn auto_download_by_fsid(
        &self,
        fs_id: u64,
        save_path: impl AsRef<Path>,
    ) -> NetDiskResult<()> {
        let file_meta = self.get_dlink_from_fsid(fs_id).await?;
        self.auto_download_with_meta(&file_meta, save_path).await
    }

    /// Auto download based on file size with optimal strategy (core implementation)
    async fn auto_download_with_meta(
        &self,
        file_meta: &FileMeta,
        save_path: impl AsRef<Path>,
    ) -> NetDiskResult<()> {
        let file_size = file_meta.size.unwrap_or(0);
        const PARALLEL_THRESHOLD: u64 = 10 * 1024 * 1024; // 10MB

        if file_size > PARALLEL_THRESHOLD {
            info!(
                "File size {} bytes exceeds {} bytes, using futures concurrent download",
                file_size, PARALLEL_THRESHOLD
            );
            self.download_streaming_with_meta(file_meta, save_path, 4)
                .await
        } else {
            info!(
                "File size {} bytes, using single-threaded download",
                file_size
            );
            self.download_single_with_meta(file_meta, save_path).await
        }
    }

    /// Single-threaded download using file path
    ///
    /// Best for small files.
    ///
    /// # Examples
    ///
    /// ```
    /// use baidu_netdisk_sdk::BaiduNetDiskClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = BaiduNetDiskClient::builder().build()?;
    /// client.load_token_from_env()?;
    ///
    /// client.download()
    ///     .download_single("/small.txt", "./small.txt")
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn download_single(
        &self,
        path: &str,
        save_path: impl AsRef<Path>,
    ) -> NetDiskResult<()> {
        let file_meta = self.get_dlink_from_path(path).await?;
        self.download_single_with_meta(&file_meta, save_path).await
    }

    /// Single-threaded download using file fs_id
    pub async fn download_single_by_fsid(
        &self,
        fs_id: u64,
        save_path: impl AsRef<Path>,
    ) -> NetDiskResult<()> {
        let file_meta = self.get_dlink_from_fsid(fs_id).await?;
        self.download_single_with_meta(&file_meta, save_path).await
    }

    /// Multi-threaded parallel download using producer-consumer pattern (using file path)
    ///
    /// Uses file path to locate the file.
    /// thread_num defaults to 4 if None.
    ///
    /// # Examples
    ///
    /// ```
    /// use baidu_netdisk_sdk::BaiduNetDiskClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = BaiduNetDiskClient::builder().build()?;
    /// client.load_token_from_env()?;
    ///
    /// // Use 4 threads (default)
    /// client.download()
    ///     .download_parallel("/large.zip", "./large.zip", None)
    ///     .await?;
    ///
    /// // Use 8 threads
    /// client.download()
    ///     .download_parallel("/large.zip", "./large.zip", Some(8))
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn download_parallel(
        &self,
        path: &str,
        save_path: impl AsRef<Path>,
        thread_num: Option<usize>,
    ) -> NetDiskResult<()> {
        let file_meta = self.get_dlink_from_path(path).await?;
        self.download_parallel_multi_threaded(&file_meta, save_path, thread_num)
            .await
    }

    /// Multi-threaded parallel download using producer-consumer pattern (using file fs_id)
    pub async fn download_parallel_by_fsid(
        &self,
        fs_id: u64,
        save_path: impl AsRef<Path>,
        thread_num: Option<usize>,
    ) -> NetDiskResult<()> {
        let file_meta = self.get_dlink_from_fsid(fs_id).await?;
        self.download_parallel_multi_threaded(&file_meta, save_path, thread_num)
            .await
    }

    /// Concurrent futures download using buffer_unordered (using file path)
    ///
    /// Uses file path to locate the file.
    ///
    /// # Examples
    ///
    /// ```
    /// use baidu_netdisk_sdk::BaiduNetDiskClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = BaiduNetDiskClient::builder().build()?;
    /// client.load_token_from_env()?;
    ///
    /// client.download()
    ///     .download_streaming("/large.zip", "./large.zip", 4)
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn download_streaming(
        &self,
        path: &str,
        save_path: impl AsRef<Path>,
        max_concurrency: usize,
    ) -> NetDiskResult<()> {
        let file_meta = self.get_dlink_from_path(path).await?;
        self.download_streaming_with_meta(&file_meta, save_path, max_concurrency)
            .await
    }

    /// Concurrent futures download using buffer_unordered (using file fs_id)
    ///
    /// Uses file fs_id to locate the file.
    pub async fn download_streaming_by_fsid(
        &self,
        fs_id: u64,
        save_path: impl AsRef<Path>,
        max_concurrency: usize,
    ) -> NetDiskResult<()> {
        let file_meta = self.get_dlink_from_fsid(fs_id).await?;
        self.download_streaming_with_meta(&file_meta, save_path, max_concurrency)
            .await
    }

    // --- Core download implementations (accept FileMeta) ---

    /// Single-threaded download (core implementation)
    ///
    /// Accepts FileMeta directly. Most users should use download_single() instead.
    pub async fn download_single_with_meta(
        &self,
        file_meta: &FileMeta,
        save_path: impl AsRef<Path>,
    ) -> NetDiskResult<()> {
        let token = self.token_getter.get_token().await?;
        let dlink = file_meta
            .dlink
            .as_ref()
            .ok_or_else(|| NetDiskError::api_error(-1, "Failed to get download link"))?;

        let download_url = if dlink.contains('?') {
            format!("{}&access_token={}", dlink, token.access_token)
        } else {
            format!("{}?access_token={}", dlink, token.access_token)
        };

        let client = reqwest::Client::new();
        let response = client.get(&download_url).send().await?;

        if !response.status().is_success() {
            return Err(NetDiskError::Unknown {
                message: format!("Failed to download file: {}", response.status()),
            });
        }

        let mut file = File::create(&save_path)?;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk)?;
        }

        info!(
            "Single download completed: {}",
            save_path.as_ref().display()
        );
        Ok(())
    }

    /// Multi-threaded parallel download using producer-consumer pattern (core implementation)
    ///
    /// Accepts FileMeta directly. Most users should use download_parallel() instead.
    pub async fn download_parallel_multi_threaded(
        &self,
        file_meta: &FileMeta,
        save_path: impl AsRef<Path>,
        thread_num: Option<usize>,
    ) -> NetDiskResult<()> {
        let token = self.token_getter.get_token().await?;
        let thread_num = thread_num.unwrap_or(4);
        let max_concurrent = thread_num * 3; // concurrency = thread_num * 3
        let max_queue_chunks = max_concurrent; // queue size = concurrency (1x buffer)

        let file_size = file_meta.size.unwrap_or(0);
        let dlink = file_meta
            .dlink
            .as_ref()
            .ok_or_else(|| NetDiskError::api_error(-1, "Failed to get download link"))?;

        const CHUNK_SIZE: u64 = 4 * 1024 * 1024; // 4MB per chunk

        let download_url = if dlink.contains('?') {
            format!("{}&access_token={}", dlink, token.access_token)
        } else {
            format!("{}?access_token={}", dlink, token.access_token)
        };

        let total_chunks = file_size.div_ceil(CHUNK_SIZE);
        debug!(
            "Producer-consumer download: {} bytes, {} chunks of {} bytes each, {} concurrent",
            file_size, total_chunks, CHUNK_SIZE, max_concurrent
        );

        let final_file = File::create(&save_path)?;
        final_file.set_len(file_size)?;
        drop(final_file);

        let (sender, receiver) = mpsc::channel(max_queue_chunks);
        let semaphore = Arc::new(Semaphore::new(max_concurrent));

        let save_path_clone = save_path.as_ref().to_path_buf();
        let consumer_handle =
            task::spawn(
                async move { consume_chunks(receiver, &save_path_clone, total_chunks).await },
            );

        let mut producer_handles = Vec::with_capacity(total_chunks as usize);

        for i in 0..total_chunks {
            let start = i * CHUNK_SIZE;
            let end = std::cmp::min((i + 1) * CHUNK_SIZE, file_size) - 1;

            let sender_clone = sender.clone();
            let semaphore_clone = Arc::clone(&semaphore);
            let url_clone = download_url.clone();

            producer_handles.push(task::spawn(async move {
                produce_chunk(
                    i as usize,
                    start,
                    end,
                    &url_clone,
                    &sender_clone,
                    &semaphore_clone,
                )
                .await
            }));
        }

        for handle in producer_handles {
            handle.await??;
        }

        drop(sender);
        consumer_handle.await??;

        info!(
            "Producer-consumer download completed: {}",
            save_path.as_ref().display()
        );
        Ok(())
    }

    /// Concurrent futures download using buffer_unordered (core implementation)
    ///
    /// Accepts FileMeta directly. Most users should use download_concurrent_futures() instead.
    pub async fn download_streaming_with_meta(
        &self,
        file_meta: &FileMeta,
        save_path: impl AsRef<Path>,
        max_concurrency: usize,
    ) -> NetDiskResult<()> {
        let token = self.token_getter.get_token().await?;
        let dlink = file_meta
            .dlink
            .as_ref()
            .ok_or_else(|| NetDiskError::api_error(-1, "Failed to get download link"))?;

        let download_url = if dlink.contains('?') {
            format!("{}&access_token={}", dlink, token.access_token)
        } else {
            format!("{}?access_token={}", dlink, token.access_token)
        };

        let file_size = file_meta.size.unwrap_or(0);
        const CHUNK_SIZE: u64 = 4 * 1024 * 1024;
        let total_chunks = file_size.div_ceil(CHUNK_SIZE);

        debug!(
            "Streaming download: {} bytes, {} chunks, max_concurrency={}",
            file_size, total_chunks, max_concurrency
        );

        let final_file = File::create(&save_path)?;
        final_file.set_len(file_size)?;
        drop(final_file);

        let save_path_clone = save_path.as_ref().to_path_buf();

        let chunks: Vec<(usize, u64, u64)> = (0..total_chunks)
            .map(|i| {
                let start = i * CHUNK_SIZE;
                let end = std::cmp::min((i + 1) * CHUNK_SIZE, file_size) - 1;
                (i as usize, start, end)
            })
            .collect();

        stream::iter(chunks)
            .map(|(index, start, end)| {
                let url = download_url.clone();
                async move {
                    let range = format!("bytes={}-{}", start, end);
                    let client = reqwest::Client::new();
                    let response = client.get(&url).header("Range", &range).send().await?;

                    if !response.status().is_success()
                        && response.status() != reqwest::StatusCode::PARTIAL_CONTENT
                    {
                        return Err(NetDiskError::Unknown {
                            message: format!(
                                "Failed to download chunk {}: {}",
                                index,
                                response.status()
                            ),
                        });
                    }

                    let data = response.bytes().await?.to_vec();
                    debug!("Downloaded chunk {} ({} bytes)", index, data.len());
                    Ok((index, start, data))
                }
            })
            .buffer_unordered(max_concurrency)
            .try_for_each(|(index, start, data)| {
                let save_path = save_path_clone.clone();
                async move {
                    let mut file = File::options().write(true).open(&save_path)?;
                    file.seek(SeekFrom::Start(start))?;
                    file.write_all(&data)?;
                    debug!("Written chunk {} at offset {}", index, start);
                    Ok::<(), NetDiskError>(())
                }
            })
            .await?;

        info!(
            "Streaming download completed: {}",
            save_path.as_ref().display()
        );
        Ok(())
    }
}

/// Download chunk data with position info
#[derive(Debug)]
struct DownloadChunk {
    index: usize,  // Chunk index for tracking
    offset: u64,   // Offset in final file
    data: Vec<u8>, // Chunk data
}

/// Producer: Download chunk and send to queue
async fn produce_chunk(
    index: usize,
    start: u64,
    end: u64,
    url: &str,
    sender: &mpsc::Sender<DownloadChunk>,
    semaphore: &Arc<Semaphore>,
) -> NetDiskResult<()> {
    let permit = semaphore.acquire().await.unwrap();

    let range = format!("bytes={}-{}", start, end);

    let client = reqwest::Client::new();
    let response = client.get(url).header("Range", &range).send().await?;

    if !response.status().is_success() && response.status() != reqwest::StatusCode::PARTIAL_CONTENT
    {
        drop(permit);
        return Err(NetDiskError::Unknown {
            message: format!("Failed to download chunk {}: {}", index, response.status()),
        });
    }

    let data = response.bytes().await?.to_vec();
    let data_len = data.len();

    drop(permit);

    sender
        .send(DownloadChunk {
            index,
            offset: start,
            data,
        })
        .await
        .map_err(|e| {
            NetDiskError::MpscSendError(format!("Failed to send chunk {}: {}", index, e))
        })?;

    debug!("Produced chunk {} ({} bytes)", index, data_len);
    Ok(())
}

/// Consumer: Receive chunks and write to file sequentially
async fn consume_chunks(
    mut receiver: mpsc::Receiver<DownloadChunk>,
    save_path: &Path,
    total_chunks: u64,
) -> NetDiskResult<()> {
    let mut file = File::options().write(true).open(save_path)?;
    let mut received_chunks = 0;

    while let Some(chunk) = receiver.recv().await {
        debug!(
            "Consuming chunk {} ({} bytes)",
            chunk.index,
            chunk.data.len()
        );

        file.seek(SeekFrom::Start(chunk.offset))?;
        file.write_all(&chunk.data)?;

        received_chunks += 1;
        if received_chunks % 10 == 0 || received_chunks == total_chunks {
            info!(
                "Download progress: {}/{} chunks",
                received_chunks, total_chunks
            );
        }
    }

    file.flush()?;
    Ok(())
}
