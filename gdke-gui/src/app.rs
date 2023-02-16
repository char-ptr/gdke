use std::{borrow::BorrowMut, ops::Deref, sync::mpsc::{Receiver, Sender}, rc::Rc, cell::RefCell};

use eframe::CreationContext;
use egui::{TextStyle, TextEdit};
use poggers::external::process::{ExPartialProcess, ExProcess};

use crate::Data;

#[derive(Debug)]
pub struct gdkeApp {
    procs : Rc<RefCell<Vec<ExPartialProcess>>>,
    selected: Option<ExPartialProcess>,
    awaiting: bool,
    last_key: String,
    process: Option<ExProcess>,
    search_query: String,
    rx: Option<std::sync::mpsc::Receiver<Data>>,
    tx: Option<std::sync::mpsc::Sender<Data>>
}
impl Default for gdkeApp {
    fn default() -> Self {
        let procs = if let Ok(procs) = ExProcess::get_processes() {
            procs
        } else {
            Vec::new()
        };
        Self {
            procs: Rc::new(RefCell::new(procs)),
            selected: None,
            process: None,
            search_query: String::new(),
            rx: None,
            awaiting: false,
            last_key: String::new(),
            tx: None
        }
    }
}
impl gdkeApp {
    pub fn new(cc: &CreationContext<'_>, rx: Receiver<Data>,tx: Sender<Data>) -> gdkeApp {
        Self {
            tx: Some(tx),
            rx: Some(rx),
            ..Default::default()
        }
    }
}
impl eframe::App for gdkeApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let Self {last_key, awaiting, rx,tx, procs, selected, process, search_query } = self;
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("GDKE");
            ui.separator();
            egui::Window::new("Key").collapsible(false).resizable(true).open(awaiting).show(ctx, |ui| {
                ui.label("Getting key, please wait...");

                if !last_key.is_empty() {
                    let mut keyda = last_key.clone();
                    TextEdit::singleline(&mut keyda).show(ui);
                    ui.label("Close this window when done.");
                }
                else if let Ok(data) = rx.as_ref().unwrap().try_recv() {
                    match data {
                        Data::Key(key) => {
                            println!("Got key: {}", key);
                            *last_key = key;
                        },
                        Data::Failure(e) => {
                            println!("Failed to get key");
                            *last_key = format!("Failed to get key: {}", e);
                        }
                        Data::Pid(_) => {unreachable!()}
                    }
                } else{
                    ui.centered_and_justified(|ui| {
                        ui.spinner();
                    });
                }
            });
            if !*awaiting {


                ui.label("Select a Godot process to find the encryption key for.");
                egui::TextEdit::singleline(&mut self.search_query).hint_text("Search...").show(ui);
                let text_style = TextStyle::Body;
                let row_height = ui.text_style_height(&text_style);
                if ui.button("refresh processes").clicked() {
                    procs.clone().borrow_mut().replace(if let Ok(procs) = ExProcess::get_processes() {
                        procs
                    } else {
                        Vec::new()
                    });
                }
                let mut procsrn = procs.clone();
                let proca = procsrn.borrow();
                let filtered_procs = if self.search_query.is_empty() {proca.iter().collect::<Vec::<&ExPartialProcess>>()} else {proca.iter()
                    .filter(|p| p.name.contains(&self.search_query) || p.pid.to_string().contains(&self.search_query)).collect()
                };
                let selval = selected.clone();
                ui.separator();
                egui::ScrollArea::vertical().max_height(if selval.is_none() {f32::INFINITY} else {260f32}).auto_shrink([false;2])
                .show_rows(ui, row_height, filtered_procs.len(), move |ui,row_range| {
                    for row in row_range {
                        if let Some(proc) = (&filtered_procs).get(row) {
                            let owner_proc = proc.deref();
                            ui.selectable_value(selected, Some(owner_proc.clone()) , proc.name.clone());
                        }
                    }
                });
                if let Some(selected) = selval {
                    ui.separator();
                    if ui.button(format!("get key for {}",selected.name)).clicked() {
                        tx.as_ref().unwrap().send(Data::Pid(selected.pid)).unwrap();
                        *awaiting = true;
                        last_key.clear();
                    }
                }
            }
        });
    }
}