use std::fs::File;
use std::io::{Write, BufRead, BufReader};
use reqwest::blocking::get;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://raw.githubusercontent.com/Loyalsoldier/v2ray-rules-dat/release/direct-list.txt";
    let response = get(url)?;
    let content = response.text()?;

    let mut output = File::create("direct-list.txt")?;
    
    // AutoProxy 头部标识
    writeln!(output, "[AutoProxy 0.2.9]")?;
    writeln!(output, "! Updated: {}", chrono::Utc::now().to_rfc3339())?;
    writeln!(output, "! Source: {}", url)?;

    for line in content.lines() {
        let line = line.trim();
        
        // 跳过空行和原始注释
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let rule = if line.starts_with("domain:") {
            // domain:google.com -> ||google.com
            format!("||{}", &line[7..])
        } else if line.starts_with("full:") {
            // full:www.google.com -> |http://www.google.com 这种通常表示精确匹配
            // 在 AutoProxy 中，通常用 |https:// 这种前缀，或者直接 || 表示
            format!("|{}", &line[5..]) 
        } else if line.starts_with("regexp:") {
            // regexp:.*\.google\.com$ -> /.*\.google\.com$/
            format!("/{}/", &line[7..])
        } else if line.starts_with("keyword:") {
            // keyword:google -> google
            line[8..].to_string()
        } else {
            // 默认为 domain 匹配
            format!("||{}", line)
        };

        writeln!(output, "{}", rule)?;
    }

    println!("Conversion completed successfully.");
    Ok(())
}
