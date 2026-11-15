//! # Commands — 命令注册中心(L2)
//!
//! 一次注册,三处自动生成:命令面板条目、快捷键速查表、应用菜单数据。
//! 消灭 demo 中"面板/帮助/动作"三处手写不同步的问题。

use std::rc::Rc;

/// 命令定义
#[derive(Clone)]
pub struct CommandDef {
    pub id: &'static str,
    pub title: String,
    pub section: &'static str,
    /// 展示用快捷键(如 "⌘N"),None 表示无快捷键
    pub keystroke: Option<&'static str>,
    /// 是否出现在命令面板
    pub in_palette: bool,
    pub handler: Rc<dyn Fn(&mut gpui::Window, &mut gpui::App) + 'static>,
}

impl CommandDef {
    pub fn new(id: &'static str, title: impl Into<String>) -> Self {
        Self {
            id,
            title: title.into(),
            section: "通用",
            keystroke: None,
            in_palette: true,
            handler: Rc::new(|_, _| {}),
        }
    }

    pub fn section(mut self, s: &'static str) -> Self {
        self.section = s;
        self
    }

    pub fn keystroke(mut self, k: &'static str) -> Self {
        self.keystroke = Some(k);
        self
    }

    pub fn hidden_from_palette(mut self) -> Self {
        self.in_palette = false;
        self
    }

    pub fn handler(
        mut self,
        f: impl Fn(&mut gpui::Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.handler = Rc::new(f);
        self
    }
}

/// 命令注册中心(Global)
#[derive(Default)]
pub struct CommandRegistry {
    commands: Vec<CommandDef>,
}

impl gpui::Global for CommandRegistry {}

impl CommandRegistry {
    pub fn register(&mut self, def: CommandDef) {
        // 同 id 覆盖
        if let Some(slot) = self.commands.iter_mut().find(|c| c.id == def.id) {
            *slot = def;
        } else {
            self.commands.push(def);
        }
    }

    pub fn register_all(&mut self, defs: impl IntoIterator<Item = CommandDef>) {
        for d in defs {
            self.register(d);
        }
    }

    pub fn all(&self) -> &[CommandDef] {
        &self.commands
    }

    /// 命令面板条目(按 section 分组排序)
    pub fn palette_commands(&self) -> Vec<CommandDef> {
        let mut v: Vec<CommandDef> = self
            .commands
            .iter()
            .filter(|c| c.in_palette)
            .cloned()
            .collect();
        v.sort_by(|a, b| a.section.cmp(b.section).then(a.title.cmp(&b.title)));
        v
    }

    /// 快捷键速查表数据
    pub fn shortcut_table(&self) -> Vec<(&str, &str, &str)> {
        self.commands
            .iter()
            .filter_map(|c| c.keystroke.map(|k| (k, c.title.as_str(), c.section)))
            .collect()
    }

    /// 按 section 分组
    pub fn sections(&self) -> Vec<(&'static str, Vec<&CommandDef>)> {
        let mut sections: Vec<(&'static str, Vec<&CommandDef>)> = Vec::new();
        for c in &self.commands {
            match sections.iter_mut().find(|(s, _)| *s == c.section) {
                Some((_, v)) => v.push(c),
                None => sections.push((c.section, vec![c])),
            }
        }
        sections
    }

    pub fn execute(&self, id: &str, window: &mut gpui::Window, cx: &mut gpui::App) -> bool {
        match self.commands.iter().find(|c| c.id == id) {
            Some(c) => {
                (c.handler.clone())(window, cx);
                true
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> CommandRegistry {
        let mut r = CommandRegistry::default();
        r.register_all(vec![
            CommandDef::new("mail.compose", "写邮件").section("邮件").keystroke("C"),
            CommandDef::new("mail.delete", "删除邮件")
                .section("邮件")
                .keystroke("Del")
                .hidden_from_palette(),
            CommandDef::new("app.theme", "切换主题").section("视图"),
        ]);
        r
    }

    #[test]
    fn test_register_and_lookup() {
        let r = sample();
        assert_eq!(r.all().len(), 3);
        // execute 需要真实 Window,查询逻辑在此验证
        assert!(r.all().iter().any(|c| c.id == "mail.compose"));
    }

    #[test]
    fn test_palette_filter_and_sort() {
        let r = sample();
        let palette = r.palette_commands();
        assert_eq!(palette.len(), 2); // hidden_from_palette 被过滤
        assert!(palette.iter().all(|c| c.in_palette));
        // 按 section 名排序(Unicode 序:视 < 邮)
        assert_eq!(palette[0].section, "视图");
    }

    #[test]
    fn test_shortcut_table() {
        let r = sample();
        let table = r.shortcut_table();
        assert_eq!(table.len(), 2); // 只含有快捷键的
        assert!(table.iter().any(|(k, _, _)| *k == "C"));
    }

    #[test]
    fn test_sections_grouping() {
        let r = sample();
        let sections = r.sections();
        assert_eq!(sections.len(), 2);
        let mails = sections.iter().find(|(s, _)| *s == "邮件").unwrap();
        assert_eq!(mails.1.len(), 2);
    }

    #[test]
    fn test_same_id_overrides() {
        let mut r = CommandRegistry::default();
        r.register(CommandDef::new("x", "旧"));
        r.register(CommandDef::new("x", "新"));
        assert_eq!(r.all().len(), 1);
        assert_eq!(r.all()[0].title, "新");
    }
}
