use std::fs::File;
use std::io::Write;
use reqwest::blocking::get;
use chrono::Utc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://raw.githubusercontent.com/Loyalsoldier/v2ray-rules-dat/release/direct-list.txt";
    
    println!("Downloading rules from {}...", url);
    let response = get(url)?;
    let content = response.text()?;

    let mut output = File::create("rules.txt")?;
    
    // AutoProxy 必须的头部标识
    writeln!(output, "[AutoProxy 0.2.9]")?;
    writeln!(output, "! Updated: {}", Utc::now().to_rfc3339())?;
    writeln!(output, "! Source: {}", url)?;

    let mut count = 0;
    for line in content.lines() {
        let line = line.trim();
        
        // 跳过空行和原始注释
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // 转换逻辑
        let rule = if let Some(domain) = line.strip_prefix("domain:") {
            format!("||{}", domain)
        } else if let Some(full) = line.strip_prefix("full:") {
            format!("|{}", full)
        } else if let Some(re) = line.strip_prefix("regexp:") {
            format!("/{}/", re)
        } else if let Some(kw) = line.strip_prefix("keyword:") {
            kw.to_string()
        } else {
            // 如果没有前缀，通常 v2ray-rules-dat 默认也是 domain
            format!("||{}", line)
        };

        writeln!(output, "{}", rule)?;
        count += 1;
    }

    println!("Successfully converted {} rules to rules.txt", count);
    Ok(())
}
