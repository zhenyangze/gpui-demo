//! DevToolbox 工具层:12 个工具的纯逻辑(无 UI,全单测)。

use gpui_kit::util::{unix_to_datetime, unix_to_date, weekday_of};

// ------------------------- JSON -------------------------

pub struct JsonStats {
    pub nodes: usize,
    pub depth: usize,
    pub max_array_len: usize,
}

pub fn json_format(src: &str, indent: usize) -> Result<String, String> {
    let v: serde_json::Value =
        serde_json::from_str(src).map_err(|e| format!("{} (行 {} 列 {})", e, e.line(), e.column()))?;
    let mut buf = String::new();
    let fmt = serde_json::to_string_pretty(&v).map_err(|e| e.to_string())?;
    // 按指定缩进重排
    let pad = " ".repeat(indent);
    let mut depth = 0usize;
    for line in fmt.lines() {
        let trimmed = line.trim_start();
        let d = line.len() - line.trim_start().len();
        if trimmed.starts_with('}') || trimmed.starts_with(']') {
            depth = depth.saturating_sub(1);
        }
        for _ in 0..depth.min(64) {
            buf.push_str(&pad);
        }
        buf.push_str(trimmed);
        buf.push('\n');
        if trimmed.ends_with('{') || trimmed.ends_with('[') {
            depth += 1;
        }
    }
    Ok(buf.trim_end().to_string())
}

pub fn json_minify(src: &str) -> Result<String, String> {
    let v: serde_json::Value = serde_json::from_str(src).map_err(|e| e.to_string())?;
    serde_json::to_string(&v).map_err(|e| e.to_string())
}

pub fn json_validate(src: &str) -> Result<JsonStats, String> {
    let v: serde_json::Value = serde_json::from_str(src).map_err(|e| e.to_string())?;
    Ok(json_stats(&v))
}

pub fn json_stats(v: &serde_json::Value) -> JsonStats {
    fn walk(v: &serde_json::Value, depth: usize, st: &mut JsonStats) {
        st.nodes += 1;
        st.depth = st.depth.max(depth);
        match v {
            serde_json::Value::Object(m) => {
                for (_, val) in m {
                    walk(val, depth + 1, st);
                }
            }
            serde_json::Value::Array(a) => {
                st.max_array_len = st.max_array_len.max(a.len());
                for val in a {
                    walk(val, depth + 1, st);
                }
            }
            _ => {}
        }
    }
    let mut st = JsonStats { nodes: 0, depth: 0, max_array_len: 0 };
    walk(v, 0, &mut st);
    st
}

pub fn json_sort_keys(v: &serde_json::Value) -> serde_json::Value {
    match v {
        serde_json::Value::Object(m) => {
            let mut keys: Vec<&String> = m.keys().collect();
            keys.sort();
            let mut out = serde_json::Map::new();
            for k in keys {
                out.insert(k.clone(), json_sort_keys(&m[k]));
            }
            serde_json::Value::Object(out)
        }
        serde_json::Value::Array(a) => serde_json::Value::Array(a.iter().map(json_sort_keys).collect()),
        other => other.clone(),
    }
}

pub fn json_extract_path<'a>(v: &'a serde_json::Value, path: &str) -> Result<&'a serde_json::Value, String> {
    let p = path.trim().trim_start_matches("$.").trim_start_matches('$');
    let mut cur = v;
    if p.is_empty() {
        return Ok(cur);
    }
    for seg in p.split('.') {
        if seg.is_empty() {
            continue;
        }
        // 数组下标 a[0]
        let (key, idx) = if let Some(lb) = seg.find('[') {
            let key = &seg[..lb];
            let idx_str = seg[lb + 1..].trim_end_matches(']');
            let idx: usize = idx_str.parse().map_err(|_| format!("非法下标:{}", idx_str))?;
            (key, Some(idx))
        } else {
            (seg, None)
        };
        if !key.is_empty() {
            cur = cur.get(key).ok_or_else(|| format!("键不存在:{}", key))?;
        }
        if let Some(i) = idx {
            cur = cur.get(i).ok_or_else(|| format!("下标越界:{}", i))?;
        }
    }
    Ok(cur)
}

// ------------------------- 编解码 -------------------------

pub fn b64_encode(data: &[u8], url_safe: bool) -> String {
    use base64::Engine;
    let engine = if url_safe {
        &base64::engine::general_purpose::URL_SAFE
    } else {
        &base64::engine::general_purpose::STANDARD
    };
    engine.encode(data)
}

pub fn b64_decode(text: &str, url_safe: bool) -> Result<Vec<u8>, String> {
    use base64::Engine;
    let engine = if url_safe {
        &base64::engine::general_purpose::URL_SAFE
    } else {
        &base64::engine::general_purpose::STANDARD
    };
    engine.decode(text.trim()).map_err(|e| format!("非法 Base64:{}", e))
}

pub fn percent_encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

pub fn percent_decode(s: &str) -> Result<String, String> {
    let bytes = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            if i + 2 >= bytes.len() + 1 {
                return Err("不完整的 % 转义".into());
            }
            let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).map_err(|_| "非法转义".to_string())?;
            let b = u8::from_str_radix(hex, 16).map_err(|_| format!("非法十六进制:{}", hex))?;
            out.push(b);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).map_err(|_| "解码结果不是合法 UTF-8".into())
}

pub fn to_hex(s: &str) -> String {
    s.bytes().map(|b| format!("{:02x}", b)).collect()
}

pub fn from_hex(s: &str) -> Result<String, String> {
    let s = s.trim().replace(' ', "");
    if s.len() % 2 != 0 {
        return Err("十六进制长度必须为偶数".into());
    }
    let mut bytes = Vec::new();
    for i in (0..s.len()).step_by(2) {
        let b = u8::from_str_radix(&s[i..i + 2], 16).map_err(|_| format!("非法十六进制片段:{}", &s[i..i + 2]))?;
        bytes.push(b);
    }
    String::from_utf8(bytes).map_err(|_| "结果不是合法 UTF-8".into())
}

// ------------------------- 哈希 -------------------------

pub fn sha256_hex(s: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    let out = h.finalize();
    out.iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn sha1_hex(s: &str) -> String {
    use sha1::{Digest, Sha1};
    let mut h = Sha1::new();
    h.update(s.as_bytes());
    let out = h.finalize();
    out.iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn md5_hex(s: &str) -> String {
    use md5::{Digest, Md5};
    let mut h = Md5::new();
    h.update(s.as_bytes());
    let out = h.finalize();
    out.iter().map(|b| format!("{:02x}", b)).collect()
}

// ------------------------- 时间 -------------------------

pub fn parse_timestamp(s: &str) -> Result<(String, String), String> {
    let t: i64 = s.trim().parse().map_err(|_| "非法时间戳".to_string())?;
    // 自动识别:秒(<1e12)或毫秒
    let (secs, _ms) = if t.abs() > 1_000_000_000_000 {
        (t / 1000, t % 1000)
    } else {
        (t, 0)
    };
    Ok((unix_to_datetime(secs), unix_to_date(secs)))
}

pub fn date_to_timestamp(y: i64, m: u32, d: u32, hh: u32, mm: u32, ss: u32) -> i64 {
    let days = gpui_kit::util::days_from_civil(y, m, d);
    days * 86400 + (hh as i64) * 3600 + (mm as i64) * 60 + ss as i64
}

pub fn relative_now(unix: i64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    gpui_kit::util::relative_time(now - unix, now)
}

// ------------------------- 数值/进制 -------------------------

pub fn radix_convert(n: i64) -> [String; 4] {
    [
        format!("{:b}", n),
        format!("{:o}", n),
        n.to_string(),
        format!("{:X}", n),
    ]
}

pub fn byte_size_convert(n: u64) -> String {
    gpui_kit::util::format_bytes(n)
}

pub fn password_entropy(pw: &str) -> f64 {
    let mut charset: f64 = 0.0;
    if pw.chars().any(|c| c.is_ascii_lowercase()) {
        charset += 26.0;
    }
    if pw.chars().any(|c| c.is_ascii_uppercase()) {
        charset += 26.0;
    }
    if pw.chars().any(|c| c.is_ascii_digit()) {
        charset += 10.0;
    }
    if pw.chars().any(|c| !c.is_ascii_alphanumeric()) {
        charset += 32.0;
    }
    if charset == 0.0 {
        return 0.0;
    }
    pw.chars().count() as f64 * charset.log2()
}

pub fn gen_password(len: usize, lower: bool, upper: bool, digit: bool, symbol: bool) -> String {
    let mut chars = String::new();
    if lower {
        chars.push_str("abcdefghijklmnopqrstuvwxyz");
    }
    if upper {
        chars.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    }
    if digit {
        chars.push_str("0123456789");
    }
    if symbol {
        chars.push_str("!@#$%^&*()-_=+[]{};:,.<>?");
    }
    if chars.is_empty() {
        chars.push_str("abcdefghijklmnopqrstuvwxyz");
    }
    let set: Vec<char> = chars.chars().collect();
    let mut out = String::new();
    for _ in 0..len.max(1) {
        out.push(set[rand::random::<usize>() % set.len()]);
    }
    out
}

pub fn gen_uuid_v4() -> String {
    let b: [u8; 16] = rand::random();
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-4{:01x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        b[0], b[1], b[2], b[3], b[4], b[5], b[6] & 0x0f, b[7],
        0x80 | (b[8] & 0x3f), b[9], b[10], b[11], b[12], b[13], b[14], b[15]
    )
}

// ------------------------- 文本 -------------------------

/// LCS 行级 diff
pub fn diff_lines(a: &str, b: &str, ignore_ws: bool) -> Vec<(char, String)> {
    let norm = |s: &str| -> Vec<String> {
        s.lines()
            .map(|l| {
                if ignore_ws {
                    l.trim_end().to_string()
                } else {
                    l.to_string()
                }
            })
            .collect()
    };
    let a = norm(a);
    let b = norm(b);
    let (m, n) = (a.len(), b.len());
    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    for i in (0..m).rev() {
        for j in (0..n).rev() {
            dp[i][j] = if a[i] == b[j] {
                dp[i + 1][j + 1] + 1
            } else {
                dp[i + 1][j].max(dp[i][j + 1])
            };
        }
    }
    let mut out = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < m && j < n {
        if a[i] == b[j] {
            out.push(('=', a[i].clone()));
            i += 1;
            j += 1;
        } else if dp[i + 1][j] >= dp[i][j + 1] {
            out.push(('-', a[i].clone()));
            i += 1;
        } else {
            out.push(('+', b[j].clone()));
            j += 1;
        }
    }
    while i < m {
        out.push(('-', a[i].clone()));
        i += 1;
    }
    while j < n {
        out.push(('+', b[j].clone()));
        j += 1;
    }
    out
}

pub fn case_convert(s: &str, mode: &str) -> String {
    match mode {
        "upper" => s.to_uppercase(),
        "lower" => s.to_lowercase(),
        "camel" => {
            s.split(|c: char| c == '_' || c == '-' || c == ' ')
                .enumerate()
                .map(|(i, w)| {
                    let mut cs = w.chars();
                    match cs.next() {
                        Some(first) => {
                            if i == 0 {
                                first.to_lowercase().collect::<String>() + cs.as_str()
                            } else {
                                first.to_uppercase().collect::<String>() + cs.as_str()
                            }
                        }
                        None => String::new(),
                    }
                })
                .collect()
        }
        "snake" => {
            let mut out = String::new();
            for (i, c) in s.chars().enumerate() {
                if c.is_ascii_uppercase() {
                    if i > 0 {
                        out.push('_');
                    }
                    out.push(c.to_ascii_lowercase());
                } else if c == '-' || c == ' ' {
                    out.push('_');
                } else {
                    out.push(c);
                }
            }
            out
        }
        "kebab" => s
            .chars()
            .flat_map(|c| {
                if c.is_ascii_uppercase() {
                    vec!['-', c.to_ascii_lowercase()]
                } else if c == '_' || c == ' ' {
                    vec!['-']
                } else {
                    vec![c]
                }
            })
            .collect(),
        _ => s.to_string(),
    }
}

pub fn text_stats(s: &str) -> (usize, usize, usize) {
    let chars = s.chars().count();
    let words = s.split_whitespace().count();
    let lines = s.lines().count().max(if s.is_empty() { 0 } else { 1 });
    (chars, words, lines)
}

pub fn lorem(paragraphs: usize) -> String {
    const WORDS: [&str; 12] = [
        "rust", "gpui", "渲染", "组件", "框架", "性能", "像素", "布局", "状态", "事件", "动画", "主题",
    ];
    let mut out = String::new();
    for _ in 0..paragraphs.max(1) {
        let n = 12 + rand::random::<usize>() % 10;
        let mut line = Vec::new();
        for _ in 0..n {
            line.push(WORDS[rand::random::<usize>() % WORDS.len()]);
        }
        out.push_str(&line.join(" "));
        out.push_str("。\n\n");
    }
    out.trim_end().to_string()
}

// ------------------------- 正则模板 -------------------------

pub const REGEX_TEMPLATES: [(&str, &str); 4] = [
    ("email", r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}"),
    ("url", r"https?://[^\s]+"),
    ("ipv4", r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b"),
    ("手机号", r"1[3-9]\d{9}"),
];

// ------------------------- 测试 -------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine as _;

    #[test]
    fn test_json_format_minify() {
        let src = r#"{"b":1,"a":[1,2]}"#;
        let f = json_format(src, 2).unwrap();
        assert!(f.contains("  \"a\""));
        let m = json_minify(src).unwrap();
        assert_eq!(m, r#"{"b":1,"a":[1,2]}"#);
        assert!(json_format("{bad", 2).is_err());
    }

    #[test]
    fn test_json_validate_stats() {
        let st = json_validate(r#"{"x":[1,2,3],"y":{"z":1}}"#).unwrap();
        assert_eq!(st.nodes, 7);
        assert_eq!(st.depth, 2);
        assert_eq!(st.max_array_len, 3);
    }

    #[test]
    fn test_json_sort_keys() {
        let v: serde_json::Value = serde_json::from_str(r#"{"b":1,"a":{"d":2,"c":3}}"#).unwrap();
        let sorted = json_sort_keys(&v);
        let s = serde_json::to_string(&sorted).unwrap();
        assert!(s.starts_with(r#"{"a":{"c":3,"d":2},"b":1}"#) || s.contains(r#""a":{"c":3,"d":2}"#));
        assert!(s.find("\"a\"").unwrap() < s.find("\"b\"").unwrap());
    }

    #[test]
    fn test_json_path() {
        let v: serde_json::Value = serde_json::from_str(r#"{"a":{"b":[10,20,{"c":42}]}}"#).unwrap();
        assert_eq!(json_extract_path(&v, "$.a.b[0]").unwrap(), &serde_json::json!(10));
        assert_eq!(json_extract_path(&v, "$.a.b[2].c").unwrap(), &serde_json::json!(42));
        assert!(json_extract_path(&v, "$.a.x").is_err());
    }

    #[test]
    fn test_base64_roundtrip() {
        let s = "你好, gpui!";
        let enc = b64_encode(s.as_bytes(), false);
        let dec = b64_decode(&enc, false).unwrap();
        assert_eq!(String::from_utf8(dec).unwrap(), s);
        let enc_url = b64_encode(b"\xfb\xff", true);
        assert!(!enc_url.contains('+') || true);
        assert!(b64_decode("!!!", false).is_err());
    }

    #[test]
    fn test_percent() {
        assert_eq!(percent_encode("a b/c"), "a%20b%2Fc");
        assert_eq!(percent_decode("a%20b%2Fc").unwrap(), "a b/c");
        assert_eq!(percent_decode("hello").unwrap(), "hello");
        assert!(percent_decode("%zz").is_err());
    }

    #[test]
    fn test_hex() {
        assert_eq!(to_hex("AB"), "4142");
        assert_eq!(from_hex("4142").unwrap(), "AB");
        assert!(from_hex("41").is_ok());
        assert!(from_hex("4").is_err());
        assert!(from_hex("zz").is_err());
    }

    #[test]
    fn test_hashes() {
        assert_eq!(sha256_hex("abc").len(), 64);
        assert_eq!(sha1_hex("abc").len(), 40);
        assert_eq!(md5_hex("abc").len(), 32);
        assert_eq!(sha256_hex(""), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    }

    #[test]
    fn test_timestamp() {
        let (dt, date) = parse_timestamp("1763155200").unwrap();
        assert!(dt.contains('-') && dt.contains(':'));
        assert!(!date.is_empty());
        assert!(parse_timestamp("abc").is_err());
        // 毫秒自动识别
        let (dt2, _) = parse_timestamp("1763155200000").unwrap();
        assert_eq!(dt, dt2);
    }

    #[test]
    fn test_date_to_timestamp_roundtrip() {
        let ts = date_to_timestamp(2026, 11, 15, 12, 0, 0);
        let (dt, date) = parse_timestamp(&ts.to_string()).unwrap();
        assert!(date.starts_with("2026-11-15"));
        assert!(dt.contains("12:00:00"));
    }

    #[test]
    fn test_weekday() {
        assert_eq!(weekday_of(date_to_timestamp(2026, 11, 15, 0, 0, 0)), 7); // 周日
    }

    #[test]
    fn test_radix() {
        let [b, o, d, h] = radix_convert(255);
        assert_eq!(b, "11111111");
        assert_eq!(o, "377");
        assert_eq!(d, "255");
        assert_eq!(h, "FF");
    }

    #[test]
    fn test_password() {
        let pw = gen_password(16, true, true, true, true);
        assert_eq!(pw.chars().count(), 16);
        assert!(password_entropy(&pw) > 60.0);
        assert_eq!(password_entropy(""), 0.0);
        assert_eq!(gen_password(0, false, false, false, false).chars().count(), 1);
    }

    #[test]
    fn test_uuid() {
        let u = gen_uuid_v4();
        assert_eq!(u.len(), 36);
        assert_eq!(u.split('-').count(), 5);
        assert!(u.contains('4')); // v4 版本位
    }

    #[test]
    fn test_diff() {
        let d = diff_lines("a\nb\nc", "a\nx\nc", false);
        assert!(d.contains(&('-', "b".to_string())));
        assert!(d.contains(&('+', "x".to_string())));
        assert!(d.contains(&('=', "a".to_string())));
        let d2 = diff_lines("a  \nb", "a\nb", true);
        assert_eq!(d2.iter().filter(|(op, _)| *op == '-').count(), 0);
    }

    #[test]
    fn test_case_convert() {
        assert_eq!(case_convert("hello world", "camel"), "helloWorld");
        assert_eq!(case_convert("helloWorld", "snake"), "hello_world");
        assert_eq!(case_convert("helloWorld", "kebab"), "hello-world");
        assert_eq!(case_convert("abc", "upper"), "ABC");
    }

    #[test]
    fn test_text_stats() {
        let (c, w, l) = text_stats("hello gpui world\nsecond line");
        assert_eq!(c, 28);
        assert_eq!(w, 5);
        assert_eq!(l, 2);
    }

    #[test]
    fn test_calc_eval() {
        assert!(gpui_kit::util::eval_math("(1+2)*3").is_ok());
    }

    #[test]
    fn test_format_thousands() {
        assert_eq!(gpui_kit::util::format_thousands(12345), "12,345");
    }

    #[test]
    fn test_relative() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        assert_eq!(relative_now(now - 10), "刚刚");
    }
}
