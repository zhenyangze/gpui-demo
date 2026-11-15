//! # gpui-kit 工具函数
//!
//! 数字/时间/字节格式化、颜色工具、哈希取色、四则表达式求值器。

use gpui::Hsla;

// ------------------------- 格式化 -------------------------

/// 千分位格式化整数/浮点
pub fn format_thousands(n: i64) -> String {
    let neg = n < 0;
    let s = n.unsigned_abs().to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3 + 2);
    if neg {
        out.push('-');
    }
    let bytes = s.as_bytes();
    let first_group = if s.len() % 3 == 0 { 3 } else { s.len() % 3 };
    out.push_str(unsafe { std::str::from_utf8_unchecked(&bytes[..first_group]) });
    let mut rest = &bytes[first_group..];
    while !rest.is_empty() {
        out.push(',');
        out.push_str(unsafe { std::str::from_utf8_unchecked(&rest[..3]) });
        rest = &rest[3..];
    }
    out
}

/// 字节大小人性化
pub fn format_bytes(n: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    if n < 1024 {
        return format!("{} B", n);
    }
    let mut v = n as f64;
    let mut u = 0;
    while v >= 1024.0 && u < UNITS.len() - 1 {
        v /= 1024.0;
        u += 1;
    }
    if v >= 100.0 {
        format!("{:.0} {}", v, UNITS[u])
    } else {
        format!("{:.1} {}", v, UNITS[u])
    }
}

/// 相对时间(秒差 → "刚刚/3 分钟前/2 小时前/昨天/5 天前/2025-01-02")
pub fn relative_time(seconds_ago: i64, now_unix: i64) -> String {
    if seconds_ago < 60 {
        "刚刚".to_string()
    } else if seconds_ago < 3600 {
        format!("{} 分钟前", seconds_ago / 60)
    } else if seconds_ago < 86400 {
        format!("{} 小时前", seconds_ago / 3600)
    } else if seconds_ago < 172800 {
        "昨天".to_string()
    } else if seconds_ago < 30 * 86400 {
        format!("{} 天前", seconds_ago / 86400)
    } else {
        unix_to_date(now_unix - seconds_ago)
    }
}

/// 简化的 civil-date 转换(UTC 意义,显示用足够)
pub fn unix_to_date(unix: i64) -> String {
    let days = unix.div_euclid(86400);
    let (y, m, d) = civil_from_days(days);
    format!("{y:04}-{m:02}-{d:02}")
}

pub fn unix_to_datetime(unix: i64) -> String {
    let secs_of_day = unix.rem_euclid(86400);
    let days = unix.div_euclid(86400);
    let (y, m, d) = civil_from_days(days);
    format!(
        "{y:04}-{m:02}-{d:02} {:02}:{:02}:{:02}",
        secs_of_day / 3600,
        (secs_of_day % 3600) / 60,
        secs_of_day % 60
    )
}

/// Howard Hinnant 的 civil_from_days 算法
pub fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

/// date → days since epoch(用于反向转换)
pub fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let mp = if m > 2 { m - 3 } else { m + 9 } as i64;
    let doy = (153 * mp + 2) / 5 + d as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

/// 星期几(1=周一 … 7=周日;1970-01-01 是周四)
pub fn weekday_of(unix: i64) -> u32 {
    let days = unix.div_euclid(86400);
    ((days.rem_euclid(7) + 3) % 7) as u32 + 1
}

// ------------------------- 颜色 -------------------------

pub fn hsla_hex(h: f64, s: f64, l: f64) -> String {
    let (r, g, b) = hsl_to_rgb(h, s, l);
    format!("#{:02X}{:02X}{:02X}", (r * 255.0).round() as u8, (g * 255.0).round() as u8, (b * 255.0).round() as u8)
}

pub fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (f64, f64, f64) {
    if s == 0.0 {
        return (l, l, l);
    }
    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;
    let hue = |t: f64| -> f64 {
        let mut t = t;
        if t < 0.0 { t += 1.0; }
        if t > 1.0 { t -= 1.0; }
        if t < 1.0 / 6.0 {
            p + (q - p) * 6.0 * t
        } else if t < 0.5 {
            q
        } else if t < 2.0 / 3.0 {
            p + (q - p) * (2.0 / 3.0 - t) * 6.0
        } else {
            p
        }
    };
    (hue(h + 1.0 / 3.0), hue(h), hue(h - 1.0 / 3.0))
}

pub fn rgb_to_hsl(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    if max == min {
        return (0.0, 0.0, l);
    }
    let d = max - min;
    let s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };
    let h = if max == r {
        (g - b) / d + if g < b { 6.0 } else { 0.0 }
    } else if max == g {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };
    (h / 6.0, s, l)
}

pub fn parse_hex_color(s: &str) -> Option<(f64, f64, f64)> {
    let s = s.trim().trim_start_matches('#');
    if s.len() == 6 {
        let r = u8::from_str_radix(&s[0..2], 16).ok()?;
        let g = u8::from_str_radix(&s[2..4], 16).ok()?;
        let b = u8::from_str_radix(&s[4..6], 16).ok()?;
        Some((r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0))
    } else {
        None
    }
}

/// 名字哈希取色(头像背景)
pub fn hash_color(name: &str) -> Hsla {
    let mut h: u32 = 5381;
    for b in name.bytes() {
        h = h.wrapping_mul(33).wrapping_add(b as u32);
    }
    let hue = (h % 360) as f64 / 360.0;
    gpui::hsla(hue as f32, 0.55, 0.55, 1.0)
}

// ------------------------- 四则表达式求值器 -------------------------

/// 支持四则、括号、幂(^)、取模(%)、一元负号与常量 pi/e/tau。
/// 返回 Result 以便 UI 友好展示错误。
pub fn eval_math(input: &str) -> Result<f64, String> {
    let filtered: String = input.chars().filter(|c| !c.is_whitespace()).collect();
    if filtered.is_empty() {
        return Err("空表达式".into());
    }
    let mut p = Parser { s: filtered.as_bytes(), pos: 0 };
    let v = p.parse_expr(0)?;
    if p.pos != p.s.len() {
        return Err(format!("位置 {} 处有非法字符", p.pos + 1));
    }
    if v.is_nan() || v.is_infinite() {
        return Err("结果溢出或未定义".into());
    }
    Ok(v)
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
}

/// 运算符优先级,数值越大越高
impl BinOp {
    fn prec(self) -> u8 {
        match self {
            BinOp::Add | BinOp::Sub => 1,
            BinOp::Mul | BinOp::Div | BinOp::Mod => 2,
            BinOp::Pow => 3,
        }
    }
    fn from_byte(b: u8) -> Option<BinOp> {
        match b {
            b'+' => Some(BinOp::Add),
            b'-' => Some(BinOp::Sub),
            b'*' => Some(BinOp::Mul),
            b'/' => Some(BinOp::Div),
            b'%' => Some(BinOp::Mod),
            b'^' => Some(BinOp::Pow),
            _ => None,
        }
    }
}

struct Parser<'a> {
    s: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    /// 递归下降:parse_expr(min_prec) 处理二元运算
    fn parse_expr(&mut self, min_prec: u8) -> Result<f64, String> {
        let mut lhs = self.parse_unary()?;
        loop {
            let Some(op) = self.peek_op() else { break };
            if op.prec() < min_prec {
                break;
            }
            self.pos += 1;
            // 右结合仅用于 ^(幂)
            let next_min = if op == BinOp::Pow { op.prec() } else { op.prec() + 1 };
            let rhs = self.parse_expr(next_min)?;
            lhs = match op {
                BinOp::Add => lhs + rhs,
                BinOp::Sub => lhs - rhs,
                BinOp::Mul => lhs * rhs,
                BinOp::Div => {
                    if rhs == 0.0 {
                        return Err("除以零".into());
                    }
                    lhs / rhs
                }
                BinOp::Mod => {
                    if rhs == 0.0 {
                        return Err("模零".into());
                    }
                    lhs % rhs
                }
                BinOp::Pow => lhs.powf(rhs),
            };
        }
        Ok(lhs)
    }

    fn parse_unary(&mut self) -> Result<f64, String> {
        if self.peek() == Some(b'-') {
            self.pos += 1;
            return Ok(-self.parse_unary()?);
        }
        if self.peek() == Some(b'+') {
            self.pos += 1;
            return self.parse_unary();
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<f64, String> {
        match self.peek() {
            Some(b'(') => {
                self.pos += 1;
                let v = self.parse_expr(0)?;
                if self.peek() != Some(b')') {
                    return Err(format!("位置 {} 缺少右括号", self.pos + 1));
                }
                self.pos += 1;
                Ok(v)
            }
            Some(c) if c.is_ascii_digit() || c == b'.' => self.parse_number(),
            _ => self.parse_constant(),
        }
    }

    fn parse_number(&mut self) -> Result<f64, String> {
        let start = self.pos;
        let mut dot = false;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                self.pos += 1;
            } else if c == b'.' && !dot {
                dot = true;
                self.pos += 1;
            } else {
                break;
            }
        }
        let s = std::str::from_utf8(&self.s[start..self.pos]).unwrap();
        s.parse::<f64>().map_err(|_| format!("非法数字 '{s}'"))
    }

    fn parse_constant(&mut self) -> Result<f64, String> {
        let rest = std::str::from_utf8(&self.s[self.pos..]).unwrap();
        for (name, val) in [("pi", std::f64::consts::PI), ("tau", std::f64::consts::TAU), ("e", std::f64::consts::E)] {
            if rest.to_ascii_lowercase().starts_with(name) {
                self.pos += name.len();
                return Ok(val);
            }
        }
        Err(format!("位置 {} 处非法输入", self.pos + 1))
    }

    fn peek(&self) -> Option<u8> {
        self.s.get(self.pos).copied()
    }

    fn peek_op(&self) -> Option<BinOp> {
        self.peek().and_then(BinOp::from_byte)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thousands() {
        assert_eq!(format_thousands(0), "0");
        assert_eq!(format_thousands(1234), "1,234");
        assert_eq!(format_thousands(-1234567), "-1,234,567");
        assert_eq!(format_thousands(100), "100");
    }

    #[test]
    fn test_bytes() {
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(2048), "2.0 KB");
        assert_eq!(format_bytes(3 * 1024 * 1024), "3.0 MB");
    }

    #[test]
    fn test_relative_time() {
        assert_eq!(relative_time(5, 1_000_000), "刚刚");
        assert_eq!(relative_time(120, 1_000_000), "2 分钟前");
        assert_eq!(relative_time(7200, 1_000_000), "2 小时前");
        assert_eq!(relative_time(90000, 1_000_000), "昨天");
    }

    #[test]
    fn test_unix_date_roundtrip() {
        // 纪元锚点
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        // 已知日期往返
        let days = days_from_civil(2026, 11, 14);
        assert_eq!(civil_from_days(days), (2026, 11, 14));
        // 2026-11-14 是周六
        assert_eq!(weekday_of(days * 86400), 6);
        // 随机多年往返一致性
        for y in [1999, 2000, 2024, 2031, 2100] {
            for m in [1u32, 6, 12] {
                let d = days_from_civil(y, m, 15);
                assert_eq!(civil_from_days(d), (y, m, 15));
            }
        }
    }

    #[test]
    fn test_colors() {
        let hex = hsla_hex(0.0, 1.0, 0.5);
        assert_eq!(hex, "#FF0000");
        let (r, g, b) = parse_hex_color("#00FF00").unwrap();
        let (h, s, l) = rgb_to_hsl(r, g, b);
        assert!((h - 1.0 / 3.0).abs() < 1e-6);
        assert!((s - 1.0).abs() < 1e-6);
        assert!((l - 0.5).abs() < 1e-6);
        assert!(parse_hex_color("#GGG").is_none());
    }

    #[test]
    fn test_eval_basic() {
        assert_eq!(eval_math("1+2").unwrap(), 3.0);
        assert_eq!(eval_math("2*3+4").unwrap(), 10.0);
        assert_eq!(eval_math("(1+2)*3").unwrap(), 9.0);
        assert_eq!(eval_math("10/4").unwrap(), 2.5);
        assert_eq!(eval_math("2^10").unwrap(), 1024.0);
        assert_eq!(eval_math("10%3").unwrap(), 1.0);
        assert_eq!(eval_math("-5+3").unwrap(), -2.0);
        assert_eq!(eval_math("2^-2").unwrap(), 0.25);
        assert_eq!(eval_math("pi").unwrap(), std::f64::consts::PI);
        assert_eq!(eval_math(" 12 + 8 * 3 ").unwrap(), 36.0);
    }

    #[test]
    fn test_eval_precedence_and_right_assoc() {
        assert_eq!(eval_math("2^3^2").unwrap(), 512.0); // 右结合
        assert_eq!(eval_math("8-2-3").unwrap(), 3.0); // 左结合
        assert_eq!(eval_math("6/2*3").unwrap(), 9.0);
    }

    #[test]
    fn test_eval_errors() {
        assert!(eval_math("1/0").is_err());
        assert!(eval_math("(1+2").is_err());
        assert!(eval_math("1+abc").is_err());
        assert!(eval_math("").is_err());
        assert!(eval_math("1+").is_err());
    }
}
