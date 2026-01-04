use std::fs::File;
use std::io::Write;
use reqwest::blocking::get;
use chrono::Utc;
use base64::{Engine as _, engine::general_purpose};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 定义任务列表：(源URL, 目标文件名)
    let tasks = [
        (
            "https://raw.githubusercontent.com/Loyalsoldier/v2ray-rules-dat/release/direct-list.txt",
            "direct.txt"
        ),
        (
            "https://raw.githubusercontent.com/Loyalsoldier/v2ray-rules-dat/release/proxy-list.txt",
            "proxy.txt"
        ),
    ];

    for (url, filename) in tasks {
        println!("Processing {} -> {}...", url, filename);
        match convert_url_to_file(url, filename) {
            Ok(count) => println!("Successfully converted {} rules to {}.", count, filename),
            Err(e) => eprintln!("Error processing {}: {}", filename, e),
        }
    }

    Ok(())
}

fn convert_url_to_file(url: &str, output_filename: &str) -> Result<usize, Box<dyn std::error::Error>> {
    // 1. 下载内容
    let response = get(url)?;
    let content = response.text()?;

    // 2. 构建明文内容缓冲区
    let mut raw_content = String::new();
    raw_content.push_str("[AutoProxy 0.2.9]\n");
    raw_content.push_str(&format!("! Updated: {}\n", Utc::now().to_rfc3339()));
    raw_content.push_str(&format!("! Source: {}\n", url));

    let mut count = 0;
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // V2Ray 格式转换逻辑
        let rule = if let Some(domain) = line.strip_prefix("domain:") {
            format!("||{}", domain)
        } else if let Some(full) = line.strip_prefix("full:") {
            format!("|{}", full)
        } else if let Some(re) = line.strip_prefix("regexp:") {
            format!("/{}/", re)
        } else if let Some(kw) = line.strip_prefix("keyword:") {
            kw.to_string()
        } else {
            // 默认处理
            format!("||{}", line)
        };

        raw_content.push_str(&rule);
        raw_content.push('\n');
        count += 1;
    }

    // 3. Base64 编码
    let b64_content = general_purpose::STANDARD.encode(raw_content);

    // 4. 写入文件
    let mut output = File::create(output_filename)?;
    output.write_all(b64_content.as_bytes())?;

    Ok(count)
}
