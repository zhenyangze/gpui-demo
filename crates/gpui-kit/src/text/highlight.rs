//! # Highlight — 语法高亮引擎(L2,零 GPUI 依赖)
//!
//! 独立 tokenizer:输入语言标识 + 源码,输出着色区间;渲染层按区间切分文本。
//! 支持:rust / json / regex-minimal(关键字/字符串/数字/注释/标点)。

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    Keyword,
    String,
    Number,
    Comment,
    Type,
    Function,
    Punct,
    Plain,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub kind: TokenKind,
}

/// 语言注册表:关键字集合 + 行注释前缀
pub struct LangDef {
    pub keywords: &'static [&'static str],
    pub types: &'static [&'static str],
    pub line_comment: Option<&'static str>,
    pub block_comment: Option<(&'static str, &'static str)>,
}

pub fn lang(name: &str) -> Option<LangDef> {
    match name {
        "rust" | "rs" => Some(LangDef {
            keywords: &[
                "fn", "let", "mut", "pub", "struct", "enum", "impl", "trait", "for", "in", "if",
                "else", "match", "while", "loop", "return", "use", "mod", "const", "static",
                "self", "Self", "where", "as", "dyn", "async", "await", "move", "ref", "true",
                "false", "crate", "super",
            ],
            types: &["u8", "u16", "u32", "u64", "i8", "i16", "i32", "i64", "f32", "f64", "bool",
                     "String", "Vec", "Option", "Result", "Box", "Rc", "Arc", "HashMap", "usize", "isize"],
            line_comment: Some("//"),
            block_comment: Some(("/*", "*/")),
        }),
        "json" => Some(LangDef {
            keywords: &["true", "false", "null"],
            types: &[],
            line_comment: None,
            block_comment: None,
        }),
        "toml" => Some(LangDef {
            keywords: &[],
            types: &[],
            line_comment: Some("#"),
            block_comment: None,
        }),
        _ => None,
    }
}

/// 词法切分:逐字符扫描,识别 标识符/数字/字符串/注释/标点
pub fn tokenize(lang_name: &str, src: &str) -> Vec<Span> {
    let Some(def) = lang(lang_name) else {
        return vec![];
    };
    let kw: HashMap<&str, ()> = def.keywords.iter().map(|k| (*k, ())).collect();
    let ty: HashMap<&str, ()> = def.types.iter().map(|k| (*k, ())).collect();

    let b = src.as_bytes();
    let mut spans = Vec::new();
    let mut i = 0usize;

    let is_ident_start = |c: u8| c.is_ascii_alphabetic() || c == b'_';
    let is_ident = |c: u8| c.is_ascii_alphanumeric() || c == b'_';
    let is_digit = |c: u8| c.is_ascii_digit();

    while i < b.len() {
        let c = b[i];
        // 行注释
        if let Some(lc) = def.line_comment
            && src[i..].starts_with(lc)
        {
            let end = src[i..].find('\n').map(|p| i + p).unwrap_or(b.len());
            spans.push(Span { start: i, end, kind: TokenKind::Comment });
            i = end;
            continue;
        }
        // 块注释
        if let Some((open, close)) = def.block_comment
            && src[i..].starts_with(open)
        {
            let end = src[i + open.len()..]
                .find(close)
                .map(|p| i + open.len() + p + close.len())
                .unwrap_or(b.len());
            spans.push(Span { start: i, end, kind: TokenKind::Comment });
            i = end;
            continue;
        }
        // 字符串("..." / 'c' / json 同)
        if c == b'"' {
            let mut j = i + 1;
            let mut escaped = false;
            while j < b.len() {
                if escaped {
                    escaped = false;
                } else if b[j] == b'\\' {
                    escaped = true;
                } else if b[j] == b'"' {
                    j += 1;
                    break;
                }
                j += 1;
            }
            spans.push(Span { start: i, end: j.min(b.len()), kind: TokenKind::String });
            i = j;
            continue;
        }
        // 标识符/关键字
        if is_ident_start(c) {
            let start = i;
            while i < b.len() && is_ident(b[i]) {
                i += 1;
            }
            let word = &src[start..i];
            let kind = if kw.contains_key(word) {
                TokenKind::Keyword
            } else if ty.contains_key(word) {
                TokenKind::Type
            } else if i < b.len() && b[i] == b'(' {
                TokenKind::Function
            } else {
                TokenKind::Plain
            };
            spans.push(Span { start, end: i, kind });
            continue;
        }
        // 数字
        if is_digit(c) {
            let start = i;
            while i < b.len() && (is_digit(b[i]) || b[i] == b'.' || b[i] == b'_') {
                i += 1;
            }
            spans.push(Span { start, end: i, kind: TokenKind::Number });
            continue;
        }
        // 标点
        if c.is_ascii_punctuation() {
            let start = i;
            while i < b.len() && b[i].is_ascii_punctuation() {
                // 标点聚合到词边界(避免吞掉注释/字符串起始)
                if def.line_comment.map(|lc| src[i..].starts_with(lc)).unwrap_or(false)
                    || src[i..].starts_with("\"")
                    || def.block_comment.map(|(op, _)| src[i..].starts_with(op)).unwrap_or(false)
                {
                    break;
                }
                i += 1;
            }
            spans.push(Span { start, end: i, kind: TokenKind::Punct });
            continue;
        }
        // 其他(空白/Unicode)并入 plain
        let start = i;
        i += 1;
        while i < b.len()
            && !is_ident_start(b[i])
            && !is_digit(b[i])
            && b[i] != b'"'
            && !b[i].is_ascii_punctuation()
        {
            i += 1;
        }
        spans.push(Span { start, end: i, kind: TokenKind::Plain });
    }
    spans
}

/// TokenKind → 语义名(供主题映射)
impl TokenKind {
    pub fn name(self) -> &'static str {
        match self {
            TokenKind::Keyword => "keyword",
            TokenKind::String => "string",
            TokenKind::Number => "number",
            TokenKind::Comment => "comment",
            TokenKind::Type => "type",
            TokenKind::Function => "function",
            TokenKind::Punct => "punct",
            TokenKind::Plain => "plain",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(spans: &[Span]) -> Vec<TokenKind> {
        spans.iter().map(|s| s.kind).collect()
    }

    #[test]
    fn test_rust_basic() {
        let src = "let x = 42;";
        let spans = tokenize("rust", src);
        let k = kinds(&spans);
        assert!(k.contains(&TokenKind::Keyword), "let 应为关键字: {spans:?}");
        assert!(k.contains(&TokenKind::Number), "42 应为数字: {spans:?}");
        assert!(k.contains(&TokenKind::Punct));
    }

    #[test]
    fn test_rust_string_and_comment() {
        let src = "let s = \"hi // 不是注释\"; // 真注释\n/* 块 */";
        let spans = tokenize("rust", src);
        assert!(spans.iter().any(|s| s.kind == TokenKind::String && &src[s.start..s.end] == "\"hi // 不是注释\""));
        assert!(spans.iter().any(|s| s.kind == TokenKind::Comment && src[s.start..s.end].starts_with("// 真注释")));
        assert!(spans.iter().any(|s| s.kind == TokenKind::Comment && src[s.start..s.end].contains("块")));
    }

    #[test]
    fn test_rust_type_and_fn() {
        let src = "fn main() -> Vec<u8> { let app = Application::new(); }";
        let spans = tokenize("rust", src);
        let text_of = |kind: TokenKind| -> Vec<String> {
            spans.iter().filter(|s| s.kind == kind).map(|s| src[s.start..s.end].to_string()).collect()
        };
        assert!(text_of(TokenKind::Keyword).contains(&"fn".to_string()));
        assert!(text_of(TokenKind::Type).contains(&"Vec".to_string()));
        assert!(text_of(TokenKind::Function).contains(&"main".to_string()));
    }

    #[test]
    fn test_json() {
        let src = r#"{"key": "value", "n": 3.14, "ok": true}"#;
        let spans = tokenize("json", src);
        let text_of = |kind: TokenKind| -> Vec<String> {
            spans.iter().filter(|s| s.kind == kind).map(|s| src[s.start..s.end].to_string()).collect()
        };
        assert!(text_of(TokenKind::String).contains(&"\"key\"".to_string()));
        assert!(text_of(TokenKind::Number).contains(&"3.14".to_string()));
        assert!(text_of(TokenKind::Keyword).contains(&"true".to_string()));
    }

    #[test]
    fn test_unknown_lang_empty() {
        assert!(tokenize("cobol", "LET X = 1").is_empty());
    }

    #[test]
    fn test_spans_cover_all_and_disjoint() {
        let src = "fn f(x: u32) -> u32 { x + 1 } // end";
        let spans = tokenize("rust", src);
        // 覆盖连续性:排序后相邻 span 无缝隙、不重叠
        let mut sorted = spans.clone();
        sorted.sort_by_key(|s| s.start);
        let mut cursor = 0usize;
        for s in &sorted {
            assert!(s.start >= cursor, "span 重叠");
            cursor = s.end;
        }
        assert_eq!(cursor, src.len(), "span 应覆盖全文本");
    }

    #[test]
    fn test_unicode_identifiers_fallback() {
        // 中文等非 ASCII 字符并入 plain,不 panic
        let src = "let 你好 = 1; // 注释";
        let spans = tokenize("rust", src);
        assert!(!spans.is_empty());
    }
}
