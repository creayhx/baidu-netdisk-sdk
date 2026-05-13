# 百度网盘 Rust SDK

[English](README.md) | [中文](README_zh.md)

百度网盘开放平台 API 的 Rust SDK，提供文件管理、上传下载等功能。

## 特性

- **API 覆盖**：文件管理、上传下载、媒体处理等
- **高性能**：支持并行上传、流式下载、多线程下载
- **优雅的错误处理**：分层的错误类型，提供中文错误描述
- **线程安全**：使用 `RwLock` 保证并发安全
- **灵活配置**：Builder 模式便于客户端配置
- **异步优先**：基于 `tokio` 异步运行时
- **便捷快捷 API**：Client 已封装各子模块方法，内部自动管理 token，无需显式传递

## 安装

- **异步优先**：基于 `tokio` 异步运行时

添加到 `Cargo.toml`:

```toml
[dependencies]
baidu-netdisk-sdk = "0.1.3"
tokio = { version = "1.0", features = ["full"] }
```

## 快速开始

> **提示**：`BaiduNetDiskClient` 已封装各子模块方法，内部自动管理 token。大部分操作可直接在 client 上调用，无需访问子模块或显式传递 token。

### 1. 创建客户端

```rust
use baidu_netdisk_sdk::BaiduNetDiskClient;

let client = BaiduNetDiskClient::builder()
    .app_key("your_app_key")
    .app_secret("your_app_secret")
    .build()?;
```

### 2. 授权

```rust
use baidu_netdisk_sdk::BaiduNetDiskClient;

// 获取设备码进行授权
let device_code = client.authorize().get_device_code().await?;
println!("请访问: {}", device_code.verification_url);
println!("输入用户码: {}", device_code.user_code);

// 轮询获取访问令牌
let token = loop {
    match client.authorize().request_access_token(&device_code).await? {
        Some(t) => break t,
        None => tokio::time::sleep(std::time::Duration::from_secs(5)).await,
    }
};
```

### 3. 文件操作

```rust
// 列出目录 - 使用快捷 API（无需显式传递 token）
let files = client.list_directory("/").await?;

// 搜索文件 - 使用快捷 API
let (results, has_more) = client.search_files("文档", "/").await?;

// 上传文件 - 使用快捷 API
client.upload_file("local/path.txt", "/remote/path.txt").await?;

// 下载文件 - 使用快捷 API
client.download_single("/remote/file.txt", "./local/file.txt").await?;
```

### 4. TokenScopedClient 多用户场景

对于需要同时使用多个令牌的多用户场景，可以创建 `TokenScopedClient`：

```rust
// 创建绑定到特定令牌的作用域客户端
let scoped_client = client.with_token(token);

// 使用作用域客户端 - 令牌会自动使用
let files = scoped_client.list_directory("/").await?;
let quota = scoped_client.get_quota().await?;

// 每个作用域客户端都有独立的令牌上下文
// 多个作用域客户端可以并发使用而不会冲突
```

**TokenScopedClient 的主要优势：**

- **线程安全**：每个作用域客户端有独立的令牌上下文
- **隔离性**：修改一个作用域客户端不影响其他客户端
- **便捷性**：无需为每个调用显式传递令牌
- **可缓存**：可以按用户缓存作用域客户端以提高性能

## 配置

### 环境变量

#### 构建器初始化时静默读取

这些变量在调用 `ClientBuilder::default()` 时（即 `BaiduNetDiskClient::builder()`）会自动读取。环境变量的值作为默认值，但可以通过显式的构建器调用覆盖。

| 变量                      | 描述                |
| ----------------------- | ----------------- |
| `BD_NETDISK_APP_ID`     | 应用 ID（可选）         |
| `BD_NETDISK_APP_KEY`    | 应用 Key            |
| `BD_NETDISK_SECRET_KEY` | 应用 Secret         |
| `BD_NETDISK_APP_NAME`   | 应用名称（可选，用于识别多个应用） |

**注意**：如果您通过构建器显式设置这些值（例如 `.app_key("...")`），它们将覆盖任何环境变量的值。

#### 显式加载（手动读取）

这些变量在构建器初始化时不会自动读取。您必须显式调用 `load_token_from_env()` 才能加载它们。

| 变量                          | 是否必需 | 描述                       |
| --------------------------- | -------- | ------------------------ |
| `BD_NETDISK_ACCESS_TOKEN`   | 是                            | 访问令牌                     |
| `BD_NETDISK_REFRESH_TOKEN`  | 是                            | 刷新令牌                     |
| `BD_NETDISK_EXPIRES_IN`     | 是                            | 令牌过期秒数                   |
| `BD_NETDISK_SCOPE`          | 否                            | 权限范围（默认："basic netdisk"） |
| `BD_NETDISK_SESSION_KEY`    | 否                            | 会话 Key                   |
| `BD_NETDISK_SESSION_SECRET` | 否                            | 会话 Secret                |
| `BD_NETDISK_ACQUIRED_AT`    | 否                            | 令牌获取时间戳（用于测试）            |

### 避免重复配置

如果您想避免环境变量干扰，只需通过构建器显式设置所有必需的值：

```rust
let client = BaiduNetDiskClient::builder()
    .app_key("your_app_key")           // 显式设置，覆盖 BD_NETDISK_APP_KEY
    .app_secret("your_app_secret")     // 显式设置，覆盖 BD_NETDISK_SECRET_KEY
    .build()?;

// 令牌仍需显式加载或手动设置
client.load_token_from_env()?;  // 加载令牌变量
// 或
client.set_access_token(manual_token)?;  // 手动设置
```

### 客户端构建器选项

```rust
let client = BaiduNetDiskClient::builder()
    .app_key("your_app_key")              // 应用 Key
    .app_secret("your_app_secret")        // 应用 Secret
    .app_name("My App")                   // 应用名称（可选）
    .timeout(Duration::from_secs(30))     // 请求超时时间
    .auto_refresh(true)                   // 自动刷新令牌
    .refresh_ahead_seconds(86400)         // 过期前24小时开始刷新
    .max_retries(3)                      // 最大重试次数
    .build()?;
```

## API 模块

### 文件管理 (`client.file()`)

核心文件操作：

- `list_directory()` - 列出目录文件
- `list_all()` - 递归列出所有文件
- `get_file_info()` - 根据路径获取文件信息
- `get_file_meta()` - 根据 fs\_id 获取文件元信息（**包含下载链接 dlink**）
- `search_files()` - 按关键词搜索文件
- `semantic_search()` - 语义搜索
- `create_folder()` - 创建目录
- `rename()` - 重命名文件/文件夹
- `move_file()` - 移动文件/文件夹
- `copy_file()` - 复制文件/文件夹
- `delete()` - 删除文件/文件夹

### 下载 (`client.download()`)

下载方法：

- `get_dlink_from_path()` - 根据文件路径获取下载链接
- `get_dlink_from_fsid()` - 根据 fs\_id 获取下载链接
- `auto_download()` - 根据文件大小自动选择最佳方法
- `auto_download_by_fsid()` - 通过 fs\_id 自动选择
- `download_single()` - 单线程下载（通过路径）
- `download_single_by_fsid()` - 单线程下载（通过 fs\_id）
- `download_single_with_meta()` - 单线程下载（使用 FileMeta）
- `download_parallel()` - 多线程并行下载（通过路径）
- `download_parallel_by_fsid()` - 多线程并行下载（通过 fs\_id）
- `download_parallel_multi_threaded()` - 多线程并行下载（使用 FileMeta）
- `download_streaming()` - 异步并发下载（通过路径）
- `download_streaming_by_fsid()` - 异步并发下载（通过 fs\_id）
- `download_streaming_with_meta()` - 流式下载（使用 FileMeta）

### 上传 (`client.upload()`)

SDK 提供多种上传方法以适应不同场景：

#### 上传方法对比

| 方法                                  | 数据来源          | 内存占用   | 流式 | 适用场景       |
| ----------------------------------- | ------------- | ------ | -- | ---------- |
| [`upload_file()`](#1-从文件路径上传)       | 文件路径          | \~80MB | ✅  | 最常见的场景     |
| [`upload_reader()`](#2-从-reader-上传) | Reader + 文件大小 | \~80MB | ✅  | 自定义读取器、包装流 |
| [`upload_bytes()`](#3-从内存字节上传)      | `&[u8]` 切片    | 全量数据   | ❌  | 内存中的小数据    |

#### 核心特性

- **断点续传**：自动检测已上传的分片，只上传缺失的部分
- **并行上传**：支持多个分片同时上传（默认：10 并发）
- **内存优化**：大文件内存占用受批量大小限制（默认约 80MB）
- **自动分片**：文件自动切割为 4MB 分片

#### 1. 从文件路径上传

最简单的文件上传方式：

```rust
use baidu_netdisk_sdk::BaiduNetDiskClient;

let client = BaiduNetDiskClient::builder().build()?;
client.load_token_from_env()?;

// 简单上传 - 使用快捷 API（无需显式传递 token）
let response = client.upload_file("local_file.txt", "/remote/file.txt").await?;

println!("已上传: {} ({} 字节)", response.path, response.size);
```

自定义选项上传：

```rust
use baidu_netdisk_sdk::{BaiduNetDiskClient, upload::SimpleUploadOptions};

let options = SimpleUploadOptions::default()
    .chunk_size(8 * 1024 * 1024)  // 8MB 分片
    .max_concurrency(20);         // 20 并发上传

let response = client.upload_file_with_options("video.mp4", "/remote/video.mp4", options).await?;
```

#### 2. 从 Reader 上传

支持流式上传，适用需要自定义读取器的场景（需要 `Read + Seek`）：

```rust
use baidu_netdisk_sdk::BaiduNetDiskClient;
use std::io::BufReader;

let file = std::fs::File::open("local_file.txt")?;
let metadata = file.metadata()?;
let file_size = metadata.len();

let mut reader = BufReader::new(file);

let response = client.upload()
    .upload_reader(&token, &mut reader, file_size, "/remote/file.txt")
    .await?;
```

#### 3. 从内存字节上传

适用数据已经在内存中的场景：

```rust
use baidu_netdisk_sdk::BaiduNetDiskClient;

let data = b"Hello, World!";
// 使用快捷 API（无需显式传递 token）
let response = client.upload_bytes(data, "/remote/hello.txt").await?;

println!("已上传: {} 字节", response.size);
```

#### 断点续传原理

1. **第一次遍历**：读取文件 → 计算每个分片的 MD5
2. **预创建**：调用 API → 获取 uploadid 和已存在分片列表
3. **第二次遍历**：再次读取文件 → 只上传缺失的分片
4. **创建**：合并分片生成最终文件

这意味着如果上传中断，重启后只会上传缺失的分片。

### 授权 (`client.authorize()`)

- `get_device_code()` - 获取授权设备码
- `request_access_token()` - 轮询获取访问令牌

### 用户与配额

- `client.user().info()` - 获取用户信息
- `client.quota().info()` - 获取存储配额信息

### 播单 (`client.playlist()`)

播单和媒体功能：

**播单操作：**

- `get_playlist_list()` - 列出播单
- `get_playlist_file_list()` - 列出播单中的文件

**媒体播放：**

- `get_media_play_info()` - 获取媒体播放信息（支持路径或 fs\_id）
- `get_media_m3u8_content()` - 根据路径获取原始 m3u8 内容

**便捷方法（画质枚举）：**

- `get_video_m3u8()` - 使用 VideoQuality 获取视频 m3u8
- `get_video_m3u8_highest()` - 为给定 VIP 等级获取最高画质视频 m3u8
- `get_audio_m3u8()` - 使用 AudioQuality 获取音频 m3u8
- `get_audio_m3u8_default()` - 使用默认画质 (128K) 获取音频 m3u8

**转码状态检查：**

- `fetch_m3u8()` - 从 URL 获取 m3u8 内容
- `is_media_fully_transcoded()` - 检查媒体是否完全转码 (#EXT-X-ENDLIST)

**画质枚举：**

- `VideoQuality` - 视频画质级别（480P、720P、1080P）
- `AudioQuality` - 音频画质级别（MP3 128K）
- 画质方法：`to_media_type()`, `highest_for_vip_level()`, `available_for_vip_level()`
- 根据 VIP 等级自动选择最佳画质

## 错误处理

```rust
use baidu_netdisk_sdk::{NetDiskError, NetDiskResult};

match result {
    Ok(value) => println!("成功: {:?}", value),
    Err(e) => {
        eprintln!("错误: {}", e);
        if e.is_auth_error() {
            // 处理认证错误 - 重新授权
        } else if e.is_not_found_error() {
            // 处理文件不存在
        }
    }
}
```

## 令牌管理

```rust
// 手动设置令牌
let token = AccessToken::new(
    "access_token_string".to_string(),
    "refresh_token_string".to_string(),
    2592000,  // expires_in
    "basic netdisk".to_string(),
);
client.set_access_token(token)?;

// 从环境变量加载令牌
let token = client.load_token_from_env()?;

// 验证令牌状态
match client.validate_token() {
    Ok(TokenStatus::Valid) => println!("令牌有效"),
    Ok(TokenStatus::ExpiringSoon) => println!("令牌即将过期"),
    Ok(TokenStatus::Expired) => println!("令牌已过期"),
    Err(e) => eprintln!("错误: {}", e),
}
```

## 示例

运行示例:

```bash
# 授权流程
cargo run --example auth_flow

# 文件操作
cargo run --example file

# 搜索
cargo run --example search

# 上传
cargo run --example upload_file
cargo run --example upload_bytes
cargo run --example upload_reader
cargo run --example upload_file_options

# 下载
cargo run --example download

# 下载对比
cargo run --example download_compare

# 令牌测试
cargo run --example token_test

# 用户信息
cargo run --example user_info

# 配额信息
cargo run --example quota

# 播单
cargo run --example playlist
```

## 下载策略指南

本 SDK 提供多种下载策略以适应不同场景：

### 并发 vs 并行

**并发** (`download_streaming`)：

- 使用单线程（或线程池）上的异步任务
- 对多个小文件或网络是瓶颈时高效
- 内存开销更低
- 适用场景：下载多个小文件、内存受限环境

**并行** (`download_parallel`)：

- 使用真正的多线程和专用 OS 线程
- 大文件吞吐量更高（最大化网络带宽）
- 内存使用更高（每个线程有自己的堆栈）
- 适用场景：大文件 (>100MB)、追求最大速度

### 何时使用哪种方式

| 场景              | 推荐方式                        |
| --------------- | --------------------------- |
| 小文件 (<10MB)     | `auto_download()` 或并发流式下载   |
| 中等文件 (10-100MB) | `auto_download()` 会自动选择最佳方式 |
| 大文件 (>100MB)    | 多线程并行                       |
| 多个文件            | 并发流式下载                      |
| 内存受限            | 单线程或并发流式下载                  |
| 追求最大速度          | 多线程并行                       |

### 快速参考

```rust
// 根据文件大小自动选择（大多数情况推荐）
// - < 10MB: 单线程
// - > 10MB: futures 并发流式下载（无论CPU核心数量都能保持良好性能）
// 注意: 如需最大速度，请手动使用 `download_parallel`
// 使用快捷 API（无需显式传递 token）
client.auto_download("/remote/file.zip", "./local/file.zip").await?;

// 大文件追求最大速度（推荐 6+ 核心）
client.download_parallel("/remote/large.iso", "./local/large.iso", Some(8)).await?;

// 多个小文件或核心受限（<= 4）
client.download_streaming("/remote/small.txt", "./local/small.txt", 4).await?;
```

### 不确定用哪个？运行对比测试！

如果不确定哪种下载方法最适合您的硬件，请运行对比测试：

```bash
cargo run --example download_compare
```

测试会要求您输入 CPU 核心数（例如 4、8、12），然后：

1. 使用 **流式（Futures/并发）** 下载
2. 使用 **并行（多线程）** 下载
3. 显示并排速度对比

根据结果来决定哪种方法最适合您的特定硬件！

**测试的关键发现：**

- 4 核心: Futures（流式）通常比并行快 1.5\~2 倍
- 6-8 核心: 两种方法性能相似
- 8+ 核心: 并行由于更好的多核利用稍微领先

## 性能提示

1. **大文件上传**：使用 `upload_file()`，自动分片并行上传
2. **大文件下载**：使用 `download_parallel_multi_threaded()` 获得最大速度
3. **令牌刷新**：设置 `refresh_ahead_seconds` 匹配您的使用模式

## 许可证

MIT 许可证 - 详见 [LICENSE](LICENSE) 文件。

## 贡献

欢迎贡献！请提交 Pull Request。
