//! # History — 通用撤销/重做框架(L2,纯逻辑零依赖)
//!
//! 任何"提交快照 → 变更"的场景都可复用;支持步数上限与连续输入合并。

#[derive(Debug, Clone)]
pub struct History<T: Clone> {
    undo_stack: Vec<T>,
    redo_stack: Vec<T>,
    limit: usize,
}

impl<T: Clone + PartialEq> History<T> {
    pub fn new(limit: usize) -> Self {
        Self { undo_stack: Vec::new(), redo_stack: Vec::new(), limit: limit.max(1) }
    }

    /// 变更前提交当前快照
    pub fn commit(&mut self, snapshot: T) {
        // 与栈顶相同则合并(避免空操作入栈)
        if let Some(top) = self.undo_stack.last() {
            if *top == snapshot {
                return;
            }
        }
        self.undo_stack.push(snapshot);
        if self.undo_stack.len() > self.limit {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    /// 连续输入合并:把 snapshot 与最近一次提交合并(输入框场景)
    pub fn merge_last(&mut self, snapshot: T) {
        if self.undo_stack.pop().is_none() {
            self.undo_stack.push(snapshot);
        }
    }

    /// 撤销:成功则把当前状态写回 current 并返回 true
    pub fn undo(&mut self, current: &mut T) -> bool {
        match self.undo_stack.pop() {
            Some(prev) => {
                let cur = std::mem::replace(current, prev);
                self.redo_stack.push(cur);
                true
            }
            None => false,
        }
    }

    /// 重做
    pub fn redo(&mut self, current: &mut T) -> bool {
        match self.redo_stack.pop() {
            Some(next) => {
                let cur = std::mem::replace(current, next);
                self.undo_stack.push(cur);
                true
            }
            None => false,
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
    pub fn undo_len(&self) -> usize {
        self.undo_stack.len()
    }
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commit_undo_redo() {
        let mut state = String::from("a");
        let mut hist = History::new(20);
        hist.commit(state.clone());
        state.push_str("bc");
        hist.commit(state.clone());
        state.push_str("de");

        assert!(hist.undo(&mut state));
        assert_eq!(state, "abc");
        assert!(hist.undo(&mut state));
        assert_eq!(state, "a");
        assert!(!hist.undo(&mut state), "栈空应返回 false");

        assert!(hist.redo(&mut state));
        assert_eq!(state, "abc");
        assert!(hist.redo(&mut state));
        assert_eq!(state, "abcde");
        assert!(!hist.redo(&mut state));
    }

    #[test]
    fn test_commit_clears_redo() {
        let mut state = 1i32;
        let mut hist = History::new(20);
        hist.commit(state);
        state = 2;
        hist.commit(state);
        state = 3;
        assert!(hist.undo(&mut state));
        assert_eq!(state, 2);
        // 撤销后提交新状态 → redo 栈清空
        hist.commit(state);
        state = 99;
        assert!(!hist.redo(&mut state));
    }

    #[test]
    fn test_limit() {
        let mut state = 0i32;
        let mut hist = History::new(3);
        for i in 0..10 {
            hist.commit(state);
            state = i + 1;
        }
        // 只能撤销 3 步
        let mut undos = 0;
        while hist.undo(&mut state) {
            undos += 1;
        }
        assert_eq!(undos, 3);
        assert_eq!(state, 7); // 回到第 7 次提交前的值
    }

    #[test]
    fn test_same_snapshot_merged() {
        let mut state = 5i32;
        let mut hist = History::new(10);
        hist.commit(state);
        hist.commit(state); // 重复提交被合并
        assert_eq!(hist.undo_len(), 1);
    }

    #[test]
    fn test_merge_last_for_typing() {
        let mut state = String::new();
        let mut hist = History::new(100);
        hist.commit(state.clone());
        state.push('h');
        hist.commit(state.clone());
        state.push('i');
        // 输入场景:合并上一个撤销单元
        hist.merge_last(state.clone());
        assert!(hist.undo(&mut state));
        assert_eq!(state, ""); // 一次撤销回到输入前
    }

    #[test]
    fn test_clear() {
        let mut state = 1i32;
        let mut hist = History::new(10);
        hist.commit(state);
        hist.clear();
        assert!(!hist.can_undo());
        assert!(!hist.can_redo());
    }
}
