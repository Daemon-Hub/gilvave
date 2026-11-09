//! Context menu data models and reactive state management.

use std::rc::Rc;
use sycamore::prelude::*;

#[derive(Clone)]
pub struct ContextMenuItem {
    pub label: String,
    pub icon: &'static str,
    pub shortcut: Option<&'static str>,
    pub danger: bool,
    pub separator_above: bool,
    #[allow(dead_code)]
    pub on_click: Rc<dyn Fn()>,
}

#[derive(Clone, Copy)]
pub struct ContextMenuContext {
    pub is_open: Signal<bool>,
    pub x: Signal<i32>,
    pub y: Signal<i32>,
    pub items: Signal<Vec<ContextMenuItem>>,
}

impl ContextMenuContext {
    pub fn new() -> Self {
        Self {
            is_open: create_signal(false),
            x: create_signal(0),
            y: create_signal(0),
            items: create_signal(Vec::new()),
        }
    }

    pub fn show(&self, x: i32, y: i32, mut items: Vec<ContextMenuItem>) {
        if items.is_empty() {
            return;
        }
        // Первый элемент контекстного меню никогда не должен иметь разделитель сверху
        if let Some(first) = items.first_mut() {
            first.separator_above = false;
        }
        self.items.set(items);
        self.x.set(x);
        self.y.set(y);
        self.is_open.set(true);
    }

    pub fn close(&self) {
        self.is_open.set(false);
    }
}


