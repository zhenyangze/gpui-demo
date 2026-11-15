//! # Store — 响应式状态容器(L2)
//!
//! 单向数据流:外部只能通过 `update`/`transaction` 修改状态,每次修改递增版本号;
//! 订阅者基于版本号去重,`select` 派生值深比较后才触发回调,避免无关重绘。

use std::marker::PhantomData;

use gpui::{App, AppContext, Context, Entity, Subscription};

/// 只读视图:阻止外部直接修改(单向数据流)
pub struct Reader<'a, T> {
    value: &'a T,
    version: u64,
}

impl<'a, T> Reader<'a, T> {
    pub fn value(&self) -> &T {
        self.value
    }
    pub fn version(&self) -> u64 {
        self.version
    }
}

impl<'a, T: Clone> Reader<'a, T> {
    pub fn cloned(&self) -> T {
        self.value.clone()
    }
}

pub struct StoreState<T> {
    value: T,
    version: u64,
}

/// 响应式 Store 实体句柄
#[derive(Clone)]
pub struct Store<T: 'static + Send + Sync> {
    entity: Entity<StoreState<T>>,
    _marker: PhantomData<*const ()>,
}

// Store 跨线程安全:Entity 本身受 GPUI 管理标记 Send/Sync
unsafe impl<T: Send + Sync> Send for Store<T> {}
unsafe impl<T: Send + Sync> Sync for Store<T> {}

impl<T: 'static + Send + Sync> Store<T> {
    pub fn new(value: T, cx: &mut Context<StoreState<T>>) -> Self {
        Self {
            entity: cx.new(|_| StoreState { value, version: 0 }),
            _marker: PhantomData,
        }
    }

    /// 从已存在的实体包装
    pub fn from_entity(entity: Entity<StoreState<T>>) -> Self {
        Self { entity, _marker: PhantomData }
    }

    pub fn entity(&self) -> &Entity<StoreState<T>> {
        &self.entity
    }

    /// 当前版本号
    pub fn version(&self, cx: &App) -> u64 {
        self.entity.read(cx).version
    }

    /// 只读快照
    pub fn read<'a>(&'a self, cx: &'a App) -> Reader<'a, T> {
        let st = self.entity.read(cx);
        Reader { value: &st.value, version: st.version }
    }

    /// 修改状态(单次通知)。接受任意 AppContext(实体 Context / App)。
    pub fn update<R, C: AppContext>(
        &self,
        cx: &mut C,
        f: impl FnOnce(&mut T, &mut Context<StoreState<T>>) -> R,
    ) -> C::Result<R> {
        self.entity.update(cx, |st: &mut StoreState<T>, cx: &mut Context<StoreState<T>>| {
            let r = f(&mut st.value, cx);
            st.version += 1;
            cx.notify();
            r
        })
    }

    /// 事务:一次 update 内执行多步修改,版本只递增一次、只通知一次
    /// (update 本身即事务边界;闭包内不应再触发任何通知)
    pub fn transaction<R, C: AppContext>(
        &self,
        cx: &mut C,
        f: impl FnOnce(&mut T) -> R,
    ) -> C::Result<R> {
        self.entity.update(cx, |st: &mut StoreState<T>, cx: &mut Context<StoreState<T>>| {
            let r = f(&mut st.value);
            st.version += 1;
            cx.notify();
            r
        })
    }

    /// 订阅版本变化(在实体上下文中)
    pub fn observe<O: 'static>(
        &self,
        cx: &mut Context<O>,
        mut f: impl FnMut(&mut Context<O>) + 'static,
    ) -> Subscription {
        let entity = self.entity.clone();
        cx.observe(&entity, move |_, _, cx| f(cx))
    }
}

/// 选择器:派生值 + 上次回调时的快照比较
pub struct Selector<S: 'static + Send + Sync, T: PartialEq + 'static + Send + Sync> {
    store: Store<S>,
    map: Box<dyn Fn(&S) -> T + 'static>,
    last: std::cell::RefCell<Option<(u64, T)>>,
}

unsafe impl<S: Send + Sync, T: PartialEq + Send + Sync> Send for Selector<S, T> {}
unsafe impl<S: Send + Sync, T: PartialEq + Send + Sync> Sync for Selector<S, T> {}

impl<S: 'static + Send + Sync, T: PartialEq + Clone + 'static + Send + Sync> Selector<S, T> {
    pub fn new(store: Store<S>, map: impl Fn(&S) -> T + 'static) -> Self {
        Self { store, map: Box::new(map), last: std::cell::RefCell::new(None) }
    }

    /// 计算当前派生值;若版本与上次一致直接返回缓存
    pub fn current(&self, cx: &App) -> T {
        let st = self.store.entity.read(cx);
        let mut last = self.last.borrow_mut();
        if let Some((v, cached)) = last.as_ref() {
            if *v == st.version {
                return cached.clone();
            }
        }
        let derived = (self.map)(&st.value);
        *last = Some((st.version, derived.clone()));
        derived
    }

    /// 判断派生值是否变化(用于订阅回调内的去重)
    pub fn changed(&self, cx: &App) -> bool {
        let st = self.store.entity.read(cx);
        let derived = (self.map)(&st.value);
        let mut last = self.last.borrow_mut();
        match last.as_ref() {
            Some((v, cached)) if *v == st.version => false,
            Some((_, cached)) => {
                let same = *cached == derived;
                if !same {
                    *last = Some((st.version, derived));
                }
                !same
            }
            None => {
                *last = Some((st.version, derived));
                false // 首次不算变化
            }
        }
    }
}

/// 在 App 级创建 Store(不经 Context 泛型)
pub fn create_store<T: 'static + Send + Sync>(value: T, cx: &mut App) -> Store<T> {
    Store::from_entity(cx.new(|_| StoreState { value, version: 0 }))
}

/// 订阅 Store 版本变化(App 级)
pub fn observe_store<T: 'static + Send + Sync>(
    store: &Store<T>,
    cx: &mut App,
    mut f: impl FnMut(&mut App) + 'static,
) -> Subscription {
    let entity = store.entity().clone();
    cx.observe(&entity, move |_, cx| f(cx))
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::TestAppContext;

    #[gpui::test]
    fn test_store_update_and_version(cx: &mut TestAppContext) {
        cx.update(|cx| {
            let store = create_store(Vec::<String>::new(), cx);
            assert_eq!(store.version(cx), 0);
            store.update(cx, |v, _| v.push("a".into()));
            assert_eq!(store.version(cx), 1);
            assert_eq!(store.read(cx).value().len(), 1);
        });
    }

    #[gpui::test]
    fn test_selector_dedup(cx: &mut TestAppContext) {
        cx.update(|cx| {
            #[derive(Clone)]
            struct S {
                big: Vec<u32>,
                name: String,
            }
            let store = create_store(S { big: vec![], name: "n".into() }, cx);
            let sel = Selector::new(store.clone(), |s| s.name.clone());

            // 首次读取不算变化
            assert!(!sel.changed(cx));
            assert_eq!(sel.current(cx), "n");

            // 修改无关字段(big),版本变化但派生值相同 → 不应视为变化
            store.update(cx, |s, _| s.big.push(1));
            assert!(!sel.changed(cx), "无关字段变化不应触发选择器");

            // 修改相关字段 → 变化
            store.update(cx, |s, _| s.name = "m".into());
            assert!(sel.changed(cx));
            assert_eq!(sel.current(cx), "m");
            // 同版本重复查询 → 缓存,无变化
            assert!(!sel.changed(cx));
        });
    }

    #[gpui::test]
    fn test_reader_readonly(cx: &mut TestAppContext) {
        cx.update(|cx| {
            let store = create_store(42u32, cx);
            let r = store.read(cx);
            assert_eq!(*r.value(), 42);
            assert_eq!(r.version(), 0);
        });
    }
}
