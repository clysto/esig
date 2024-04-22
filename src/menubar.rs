use eframe::egui::{self, Button, Key, KeyboardShortcut, Modifiers, Widget};
use std::collections::HashMap;

#[derive(Clone)]
pub struct MenuItem {
    id: u32,
    name: String,
    shortcut: Option<KeyboardShortcut>,
    children: Vec<MenuItem>,
}

pub struct MenuBar {
    menu_items: Vec<MenuItem>,
    shortcuts_map: HashMap<KeyboardShortcut, u32>,
    action: Option<u32>,
}

impl MenuItem {
    pub fn new(name: &str, children: &[MenuItem]) -> Self {
        Self {
            id: u32::MAX,
            name: name.to_string(),
            shortcut: None,
            children: children.to_vec(),
        }
    }

    pub fn separator() -> Self {
        Self {
            id: u32::MAX,
            name: "-".to_string(),
            shortcut: None,
            children: Vec::new(),
        }
    }

    pub fn single(id: u32, name: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            shortcut: None,
            children: Vec::new(),
        }
    }

    pub fn single_with_shortcut(id: u32, name: &str, modifiers: Modifiers, key: Key) -> Self {
        Self {
            id,
            name: name.to_string(),
            shortcut: Some(KeyboardShortcut::new(modifiers, key)),
            children: Vec::new(),
        }
    }

    pub fn register_shortcut(&self, shortcuts_map: &mut HashMap<KeyboardShortcut, u32>) {
        if let Some(shortcut) = self.shortcut.as_ref() {
            shortcuts_map.insert(shortcut.clone(), self.id);
        }
        for child in self.children.iter() {
            child.register_shortcut(shortcuts_map);
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Option<u32> {
        let mut id = None;
        if !self.children.is_empty() {
            ui.menu_button(&self.name, |ui| {
                for child in self.children.iter_mut() {
                    if let Some(child_id) = child.show(ui) {
                        id = Some(child_id);
                    }
                }
            });
        } else {
            if let Some(shortcut) = self.shortcut.as_ref() {
                if Button::new(&self.name)
                    .shortcut_text(ui.ctx().format_shortcut(shortcut))
                    .ui(ui)
                    .clicked()
                {
                    ui.close_menu();
                    id = Some(self.id);
                }
            } else if self.name == "-" {
                ui.separator();
            } else {
                if Button::new(&self.name).ui(ui).clicked() {
                    ui.close_menu();
                    id = Some(self.id);
                }
            }
        }
        return id;
    }
}

impl MenuBar {
    pub fn new() -> Self {
        Self {
            menu_items: Vec::new(),
            shortcuts_map: HashMap::new(),
            action: None,
        }
    }

    pub fn add(&mut self, item: MenuItem) {
        item.register_shortcut(&mut self.shortcuts_map);
        self.menu_items.push(item);
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        self.action = None;
        egui::menu::bar(ui, |ui| {
            for item in self.menu_items.iter_mut() {
                if let Some(id) = item.show(ui) {
                    self.action = Some(id);
                }
            }
        });
    }

    pub fn comsume_action(&mut self, ui: &egui::Ui) -> Option<&u32> {
        if self.action.is_some() {
            return self.action.as_ref();
        }
        for (shortcut, id) in self.shortcuts_map.iter() {
            if ui.ctx().input_mut(|input| input.consume_shortcut(shortcut)) {
                return Some(id);
            }
        }
        None
    }
}
