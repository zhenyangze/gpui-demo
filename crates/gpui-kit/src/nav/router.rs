//! # Router — 栈式导航(L2)
//!
//! 页面枚举驱动的栈式导航:push/pop/replace/回首页,带转场动画句柄。
//! 纯逻辑,页面渲染由使用方根据 `current()` 完成。

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Router<P: Clone + PartialEq> {
    stack: Vec<P>,
    max_depth: usize,
    /// 转场标记:push=true(前进)pop=false(后退),UI 可据此选择动画方向
    last_transition: Option<bool>,
}

impl<P: Clone + PartialEq> Router<P> {
    pub fn new(root: P) -> Self {
        Self { stack: vec![root], max_depth: 32, last_transition: None }
    }

    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth.max(1);
        self
    }

    pub fn current(&self) -> &P {
        self.stack.last().expect("栈至少含根页面")
    }

    pub fn depth(&self) -> usize {
        self.stack.len()
    }

    pub fn can_pop(&self) -> bool {
        self.stack.len() > 1
    }

    /// 前进(若与当前页相同则忽略)
    pub fn push(&mut self, page: P) -> bool {
        if self.current() == &page {
            return false;
        }
        if self.stack.len() >= self.max_depth {
            self.stack.remove(0); // 环形淘汰最旧页
        }
        self.stack.push(page);
        self.last_transition = Some(true);
        true
    }

    /// 后退
    pub fn pop(&mut self) -> bool {
        if !self.can_pop() {
            return false;
        }
        self.stack.pop();
        self.last_transition = Some(false);
        true
    }

    /// 替换当前页(无动画语义)
    pub fn replace(&mut self, page: P) {
        *self.stack.last_mut().expect("栈非空") = page;
        self.last_transition = None;
    }

    /// 回到根页
    pub fn pop_to_root(&mut self) -> bool {
        if !self.can_pop() {
            return false;
        }
        let root = self.stack[0].clone();
        self.stack.truncate(1);
        *self.stack.last_mut().expect("栈非空") = root;
        self.last_transition = Some(false);
        true
    }

    /// 是否为刚 push 的页面(UI 转场动画判断)
    pub fn last_transition_forward(&self) -> Option<bool> {
        self.last_transition
    }

    pub fn clear_transition(&mut self) {
        self.last_transition = None;
    }

    /// 栈内是否有某页(去重导航)
    pub fn contains(&self, page: &P) -> bool {
        self.stack.contains(page)
    }

    /// 回退到栈中某页(不存在则不动作)
    pub fn pop_to(&mut self, page: &P) -> bool {
        if let Some(ix) = self.stack.iter().rposition(|p| p == page) {
            if ix + 1 == self.stack.len() {
                return false; // 已是当前页
            }
            self.stack.truncate(ix + 1);
            self.last_transition = Some(false);
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Page {
        Home,
        List,
        Detail(u32),
    }

    #[test]
    fn test_push_pop() {
        let mut r = Router::new(Page::Home);
        assert!(!r.can_pop());
        assert!(r.push(Page::List));
        assert_eq!(r.current(), &Page::List);
        assert!(r.can_pop());
        assert!(r.pop());
        assert_eq!(r.current(), &Page::Home);
        assert!(!r.pop(), "根页不可再 pop");
    }

    #[test]
    fn test_push_same_page_ignored() {
        let mut r = Router::new(Page::Home);
        assert!(!r.push(Page::Home));
        assert_eq!(r.depth(), 1);
    }

    #[test]
    fn test_replace() {
        let mut r = Router::new(Page::Home);
        r.push(Page::List);
        r.replace(Page::Detail(3));
        assert_eq!(r.current(), &Page::Detail(3));
        assert!(r.can_pop());
        assert!(r.pop());
        assert_eq!(r.current(), &Page::Home, "replace 不影响栈底");
    }

    #[test]
    fn test_pop_to_root_and_pop_to() {
        let mut r = Router::new(Page::Home);
        r.push(Page::List);
        r.push(Page::Detail(1));
        assert!(r.pop_to(&Page::Home));
        assert_eq!(r.current(), &Page::Home);
        assert!(!r.pop_to(&Page::List), "栈中不存在则不动作");
        // 重新构建并测试 pop_to 中间页
        r.push(Page::List);
        r.push(Page::Detail(2));
        assert!(r.pop_to(&Page::List));
        assert_eq!(r.current(), &Page::List);
        assert_eq!(r.depth(), 2);
    }

    #[test]
    fn test_max_depth_ring() {
        let mut r = Router::new(Page::Home).with_max_depth(3);
        r.push(Page::Detail(1));
        r.push(Page::Detail(2));
        r.push(Page::Detail(3));
        assert_eq!(r.depth(), 3);
        r.push(Page::Detail(4)); // 淘汰 Home
        assert_eq!(r.depth(), 3);
        assert_eq!(r.current(), &Page::Detail(4));
        assert!(!r.contains(&Page::Home), "环形淘汰后 Home 应已不在栈中");
        assert!(!r.pop_to(&Page::Home));
    }

    #[test]
    fn test_transition_flags() {
        let mut r = Router::new(Page::Home);
        assert_eq!(r.last_transition_forward(), None);
        r.push(Page::List);
        assert_eq!(r.last_transition_forward(), Some(true));
        r.clear_transition();
        assert_eq!(r.last_transition_forward(), None);
        r.pop();
        assert_eq!(r.last_transition_forward(), Some(false));
    }

    #[test]
    fn test_contains() {
        let mut r = Router::new(Page::Home);
        r.push(Page::Detail(9));
        assert!(r.contains(&Page::Detail(9)));
        assert!(!r.contains(&Page::List));
    }
}
