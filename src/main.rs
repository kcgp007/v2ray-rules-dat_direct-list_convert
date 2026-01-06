use std::fs::File;
use std::io::Write;
use reqwest::blocking::get;
use chrono::Utc;
use base64::{Engine as _, engine::general_purpose};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 定义任务列表：(源 V2Ray 格式 URL, 输出的文件名)
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

    // 遍历任务列表进行转换
    for (url, filename) in tasks {
        println!("正在处理: {} -> {}", url, filename);
        match convert_url_to_file(url, filename) {
            Ok(count) => println!("成功转换 {} 条规则到 {}", count, filename),
            Err(e) => eprintln!("处理 {} 时出错: {}", filename, e),
        }
    }

    Ok(())
}

/// 核心转换函数
fn convert_url_to_file(url: &str, output_filename: &str) -> Result<usize, Box<dyn std::error::Error>> {
    // 1. 发起网络请求下载原始文件内容
    let response = get(url)?;
    let content = response.text()?;

    // 2. 初始化明文缓冲区，并添加 AutoProxy 必需的头部标识
    let mut raw_content = String::new();
    raw_content.push_str("[AutoProxy 0.2.9]\n"); // 插件识别标志
    raw_content.push_str(&format!("! 更新时间: {}\n", Utc::now().to_rfc3339()));
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
            kw.to_string()
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
    let mut output = File::create(output_filename)?;
    output.write_all(b64_content.as_bytes())?;

    Ok(count)
}
