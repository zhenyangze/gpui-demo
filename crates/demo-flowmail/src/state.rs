//! FlowMail 数据层:邮件/文件夹模型、种子数据、过滤排序、JSON 持久化、模拟收信。

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub name: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mail {
    pub id: u64,
    pub from: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub subject: String,
    pub body: String,
    pub time_unix: i64,
    pub folder: String,
    pub read: bool,
    pub starred: bool,
    pub attachments: Vec<Attachment>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub id: String,
    pub name: String,
    pub builtin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub theme_dark: bool,
    pub accent_index: usize,
    pub compact: bool,
    pub font_scale: usize, // 0/1/2 → 13/14/15
    pub sort_desc: bool,
    pub sort_by_sender: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme_dark: true,
            accent_index: 0,
            compact: false,
            font_scale: 1,
            sort_desc: true,
            sort_by_sender: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Persisted {
    pub mails: Vec<Mail>,
    pub custom_folders: Vec<Folder>,
    pub settings: Settings,
    pub selected_folder: String,
    pub next_id: u64,
}

pub fn data_dir() -> PathBuf {
    let mut p = dirs_home();
    p.push(".gpui-demo");
    p.push("flowmail");
    std::fs::create_dir_all(&p).ok();
    p
}

fn dirs_home() -> PathBuf {
    std::env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("."))
}

pub fn state_path() -> PathBuf {
    let mut p = data_dir();
    p.push("state.json");
    p
}

pub fn load_state() -> Option<Persisted> {
    let text = std::fs::read_to_string(state_path()).ok()?;
    serde_json::from_str(&text).ok()
}

pub fn save_state(persisted: &Persisted) {
    if let Ok(json) = serde_json::to_string_pretty(persisted) {
        let _ = std::fs::write(state_path(), json);
    }
}

pub fn builtin_folders() -> Vec<Folder> {
    vec![
        Folder { id: "inbox".into(), name: "收件箱".into(), builtin: true },
        Folder { id: "starred".into(), name: "星标".into(), builtin: true },
        Folder { id: "sent".into(), name: "已发送".into(), builtin: true },
        Folder { id: "drafts".into(), name: "草稿".into(), builtin: true },
        Folder { id: "trash".into(), name: "垃圾箱".into(), builtin: true },
    ]
}

/// 全量邮件列表(种子数据,覆盖 markdown 元素/附件/多标签)
pub fn seed_mails(now: i64) -> Vec<Mail> {
    let mut v = Vec::new();
    let mut id = 1u64;
    let mut push = |v: &mut Vec<Mail>,
                    id: &mut u64,
                    from: &str,
                    to: &[&str],
                    cc: &[&str],
                    subject: &str,
                    body: &str,
                    mins_ago: i64,
                    folder: &str,
                    read: bool,
                    starred: bool,
                    atts: &[(&str, u64)],
                    tags: &[&str]| {
        v.push(Mail {
            id: *id,
            from: from.into(),
            to: to.iter().map(|s| s.to_string()).collect(),
            cc: cc.iter().map(|s| s.to_string()).collect(),
            subject: subject.into(),
            body: body.into(),
            time_unix: now - mins_ago * 60,
            folder: folder.into(),
            read,
            starred,
            attachments: atts
                .iter()
                .map(|(n, s)| Attachment { name: n.to_string(), size: *s })
                .collect(),
            tags: tags.iter().map(|s| s.to_string()).collect(),
        });
        *id += 1;
    };

    push(&mut v, &mut id, "林一 <linyi@example.com>", &["我"], &[],
        "Q4 路线图评审会议纪要",
        "## 会议纪要\n\n各位好,以下是本次评审的要点:\n\n- **目标一**:Q4 完成桌面端 1.0\n- **目标二**:智能体接入稳定\n- `重点` 项目排期见附件\n\n> 风险:资源紧张,优先级需要再对齐\n\n1. 请各位今天下班前确认\n2. 有异议直接回复\n\n---\n谢谢!",
        42, "inbox", false, true,
        &[("roadmap-q4.pdf", 2048000)], &["工作", "重要"]);

    push(&mut v, &mut id, "GitHub <noreply@github.com>", &["我"], &[],
        "[gpui-demo] New release: gpui-kit v0.1.0",
        "Your repository has a new release.\n\n- Tag: `v0.1.0`\n- Components: **30**\n\nSee the changelog for details.",
        90, "inbox", false, false, &[], &["通知"]);

    push(&mut v, &mut id, "张三 <zhangsan@example.com>", &["我"], &["李四"],
        "关于接口联调的时间安排",
        "Hi,\n\n接口已经就绪:\n\n- 登录 `/api/login`\n- 用户信息 `/api/user`\n\n```json\n{\"ok\": true, \"code\": 0}\n```\n\n请安排联调时间。",
        160, "inbox", false, false, &[], &["工作"]);

    push(&mut v, &mut id, "李四 <lisi@example.com>", &["我"], &[],
        "Re: 关于接口联调的时间安排",
        "收到,我这边 **周四上午** 有空,可以直接约。",
        150, "inbox", true, false, &[], &["工作"]);

    push(&mut v, &mut id, "产品组 <pm@example.com>", &["all"], &[],
        "本周产品动态(第 46 周)",
        "## 动态\n\n- 新用户引导改版上线\n- 转化率 *+3.2%*\n\n下周五评审新方案。",
        260, "inbox", true, false, &[], &["产品"]);

    push(&mut v, &mut id, "王五 <wangwu@example.com>", &["我"], &[],
        "设计稿:Fmail 深色主题",
        "深色主题初稿已上传蓝湖,主要调整了:\n\n1. 背景层级\n2. 主色饱和度\n3. 圆角统一 8px\n\n麻烦看看。",
        400, "inbox", false, true, &[("dark-theme.sketch", 8192000)], &["设计"]);

    push(&mut v, &mut id, "云服务 <billing@example.com>", &["我"], &[],
        "11 月账单已生成",
        "您的 11 月账单金额为 **¥128.00**,详情请登录控制台查看。",
        700, "inbox", true, false, &[], &["账单"]);

    push(&mut v, &mut id, "赵六 <zhaoliu@example.com>", &["我"], &[],
        "旅行照片分享",
        "上周团建的照片整理好了,云盘链接 7 天有效,记得下载~",
        900, "inbox", true, false, &[("photos.zip", 52428800)], &["生活"]);

    push(&mut v, &mut id, "孙七 <sunqi@example.com>", &["我"], &[],
        "面试邀请:高级 Rust 工程师",
        "您好,我们希望邀请您参加本周五 15:00 的面试,请确认时间。",
        1300, "inbox", false, false, &[], &["机会"]);

    push(&mut v, &mut id, "周八 <zhouba@example.com>", &["我"], &[],
        "关于周报模板的更新",
        "周报模板已更新,新增了「风险与求助」栏目。",
        1500, "inbox", true, false, &[], &[]);

    push(&mut v, &mut id, "我 <me@example.com>", &["张三"], &[],
        "已发送:接口文档链接",
        "文档在此:https://docs.example.com/api",
        300, "sent", true, false, &[], &[]);

    push(&mut v, &mut id, "我 <me@example.com>", &["李四"], &[],
        "已发送:Re: 联调时间",
        "那就周四上午 10 点,会议室 A02。",
        140, "sent", true, false, &[], &[]);

    push(&mut v, &mut id, "我 <me@example.com>", &["产品组"], &[],
        "草稿:关于新版排期的反馈",
        "(未写完)关于排期,我有几点想法……",
        200, "drafts", true, false, &[], &[]);

    push(&mut v, &mut id, "营销 <ads@example.com>", &["我"], &[],
        "限时优惠:年度会员 5 折",
        "不要错过!年度会员限时 5 折……",
        2000, "trash", true, false, &[], &["广告"]);

    v
}

/// 模拟收信:生成一封新邮件
pub fn random_incoming(now: i64, next_id: u64) -> Mail {
    const NAMES: [&str; 4] = ["陈九 <chenjiu@example.com>", "吴十 <wushi@example.com>", "CI 机器人 <ci@example.com>", "监控 <alert@example.com>"];
    const SUBJECTS: [&str; 4] = ["部署完成:flowmail v0.1.3", "告警:CPU 使用率超阈值", "新的评论回复", "每周摘要"];
    let pick = (next_id as usize) % NAMES.len();
    Mail {
        id: next_id,
        from: NAMES[pick].into(),
        to: vec!["我".into()],
        cc: vec![],
        subject: SUBJECTS[pick].into(),
        body: "## 自动通知\n\n这是一封模拟收到的邮件,用于演示 FlowMail 的实时收信。\n\n- 时间戳已记录\n- 未读计数会自动增加".into(),
        time_unix: now,
        folder: "inbox".into(),
        read: false,
        starred: false,
        attachments: vec![],
        tags: vec!["自动".into()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seed_and_filter() {
        let mails = seed_mails(1_000_000);
        assert_eq!(mails.len(), 14);
        let inbox: Vec<&Mail> = mails.iter().filter(|m| m.folder == "inbox").collect();
        assert_eq!(inbox.len(), 10);
        let unread: Vec<&&Mail> = inbox.iter().filter(|m| !m.read).collect();
        assert_eq!(unread.len(), 5);
        let starred: Vec<&Mail> = mails.iter().filter(|m| m.starred).collect();
        assert_eq!(starred.len(), 2);
    }

    #[test]
    fn test_search_filter() {
        let mails = seed_mails(0);
        let q = "接口";
        let hits: Vec<&Mail> = mails
            .iter()
            .filter(|m| m.subject.contains(q) || m.body.contains(q) || m.from.contains(q))
            .collect();
        assert!(hits.len() >= 3, "搜索「接口」应命中 ≥3 封,实际 {}", hits.len());
    }

    #[test]
    fn test_starred_folder_view() {
        let mails = seed_mails(0);
        let starred_ids: Vec<u64> = mails.iter().filter(|m| m.starred).map(|m| m.id).collect();
        assert_eq!(starred_ids.len(), 2);
    }

    #[test]
    fn test_persistence_roundtrip() {
        let mails = seed_mails(42);
        let p = Persisted {
            mails: mails.clone(),
            custom_folders: vec![Folder { id: "f1".into(), name: "项目A".into(), builtin: false }],
            settings: Settings::default(),
            selected_folder: "inbox".into(),
            next_id: 100,
        };
        let json = serde_json::to_string(&p).unwrap();
        let back: Persisted = serde_json::from_str(&json).unwrap();
        assert_eq!(back.mails.len(), 14);
        assert_eq!(back.custom_folders.len(), 1);
        assert_eq!(back.mails[0].subject, mails[0].subject);
    }

    #[test]
    fn test_random_incoming() {
        let m = random_incoming(1000, 99);
        assert_eq!(m.id, 99);
        assert!(!m.read);
        assert_eq!(m.folder, "inbox");
    }
}
