use anyhow::{Context, Result};
use base64::{Engine as _, engine::general_purpose};
use chrono::{FixedOffset, Utc};
use reqwest::blocking::Client;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::time::Duration;

/// 下载最大尝试次数（首次 + 2 次重试）
const MAX_ATTEMPTS: u32 = 3;
/// 单次请求总超时（下载约 2 MB 列表）
const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);
/// 建立连接超时
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

fn main() -> Result<()> {
    // 定义任务列表：(源 V2Ray 格式 URL, 输出的文件名)
    let tasks = [
        (
            "https://raw.githubusercontent.com/Loyalsoldier/v2ray-rules-dat/release/direct-list.txt",
            "direct.txt",
        ),
        (
            "https://raw.githubusercontent.com/Loyalsoldier/v2ray-rules-dat/release/proxy-list.txt",
            "proxy.txt",
        ),
    ];

    // 遍历任务列表进行转换；收集失败项，避免部分失败时仍以退出码 0 结束（导致 CI 发布不完整的订阅）
    let mut failures: Vec<&str> = Vec::new();
    for (url, filename) in tasks {
        println!("正在处理: {} -> {}", url, filename);
        match convert_url_to_file(url, filename) {
            Ok(count) => println!("成功转换 {} 条规则到 {}", count, filename),
            Err(e) => {
                eprintln!("处理 {} 时出错: {:#}", filename, e);
                failures.push(filename);
            }
        }
    }

    if !failures.is_empty() {
        anyhow::bail!(
            "以下列表转换失败: {}，中止以免发布过期/不完整的订阅",
            failures.join(", ")
        );
    }

    Ok(())
}

/// 单次下载：发请求、校验状态码、读取响应体
fn fetch_once(client: &Client, url: &str) -> Result<String> {
    let response = client.get(url).send().context("请求失败")?;

    let status = response.status();
    if !status.is_success() {
        anyhow::bail!("HTTP {}", status);
    }

    response.text().context("读取响应体失败")
}

/// 带超时与指数退避重试的下载（1s、2s），抵御瞬时网络抖动
///
/// 所有失败（连接/超时/5xx/4xx）均重试，最多 3 次；
/// 若最终仍失败，返回最后一次的错误供上层汇总。
fn download_with_retry(url: &str) -> Result<String> {
    let client = Client::builder()
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(REQUEST_TIMEOUT)
        .build()
        .context("构建 HTTP 客户端失败")?;

    let mut last_err: Option<anyhow::Error> = None;
    for attempt in 1..=MAX_ATTEMPTS {
        match fetch_once(&client, url) {
            Ok(content) => {
                if attempt > 1 {
                    println!("第 {} 次尝试成功（共重试 {} 次）", attempt, attempt - 1);
                }
                return Ok(content);
            }
            Err(e) => {
                let retryable = attempt < MAX_ATTEMPTS;
                eprintln!(
                    "第 {}/{} 次下载 {} 失败: {:#}{}",
                    attempt,
                    MAX_ATTEMPTS,
                    url,
                    e,
                    if retryable {
                        ""
                    } else {
                        "（已耗尽重试）"
                    }
                );
                last_err = Some(e);
                if retryable {
                    // 指数退避：1s、2s
                    let backoff = 1u64 << (attempt - 1);
                    println!("{}s 后重试...", backoff);
                    std::thread::sleep(Duration::from_secs(backoff));
                }
            }
        }
    }

    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("未知错误")))
        .context(format!("下载 {} 重试 {} 次后仍失败", url, MAX_ATTEMPTS))
}

/// 核心转换函数
fn convert_url_to_file(url: &str, output_filename: &str) -> Result<usize> {
    // 1. 发起网络请求下载原始文件内容（内置超时 + 指数退避重试，失败时错误链中已含 URL）
    let content = download_with_retry(url)?;

    // 2. 初始化明文缓冲区，并添加 AutoProxy 必需的头部标识
    let mut raw_content = String::with_capacity(content.len() * 2);
    raw_content.push_str("[AutoProxy 0.2.9]\n"); // 插件识别标志
    // 生成北京时间（UTC+8）可读格式的时间戳
    let beijing_offset = FixedOffset::east_opt(8 * 3600).expect("无效的时区偏移");
    let beijing_time = Utc::now().with_timezone(&beijing_offset);
    raw_content.push_str(&format!(
        "! 更新时间: {}\n",
        beijing_time.format("%Y-%m-%d %H:%M:%S (北京时间)")
    ));
    raw_content.push_str(&format!("! 数据来源: {}\n", url));

    let mut count = 0;
    // 逐行解析转换
    for line in content.lines() {
        let line = line.trim();
        // 过滤空行和 V2Ray 原始注释
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // 格式转换逻辑：将 V2Ray 各种前缀转换为 AutoProxy 对应的语法
        let rule = if let Some(domain) = line.strip_prefix("domain:") {
            // domain:google.com -> ||google.com (匹配域名及其子域名)
            format!("||{}", domain)
        } else if let Some(full) = line.strip_prefix("full:") {
            // full:www.google.com -> |www.google.com (精确匹配)
            format!("|{}", full)
        } else if let Some(re) = line.strip_prefix("regexp:") {
            // regexp:^google -> /^google/ (正则表达式匹配)
            format!("/{}/", re)
        } else if let Some(kw) = line.strip_prefix("keyword:") {
            // keyword:google -> google (关键词匹配)
            kw.to_owned()
        } else {
            // 如果没有前缀，默认按域名匹配处理
            format!("||{}", line)
        };

        raw_content.push_str(&rule);
        raw_content.push('\n');
        count += 1;
    }

    // 3. 将整个明文内容进行标准 Base64 编码 (ZeroOmega 推荐格式)
    let b64_content = general_purpose::STANDARD.encode(raw_content);

    // 4. 将编码后的字符串写入本地文件
    let mut output = BufWriter::new(File::create(output_filename)?);
    output.write_all(b64_content.as_bytes())?;

    Ok(count)
}
