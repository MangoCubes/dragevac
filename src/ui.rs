mod card;
use std::fs;
use std::sync::{Arc, Mutex};

use gtk4::gdk::{
    ContentFormats, ContentProvider, Display, DragAction, FileList, Key, ModifierType,
};
use gtk4::gio::prelude::ApplicationExt;
use gtk4::gio::{Cancellable, File};
use gtk4::glib::value::ToValue;
use gtk4::glib::{self, Bytes, Priority, Propagation};
use gtk4::prelude::{
    BoxExt, EventControllerExt, FileExt, GestureExt, GtkWindowExt, ListBoxRowExt, StaticType,
    WidgetExt,
};
use gtk4::{Align, Box, EventSequenceState, SelectionMode, Separator};
use gtk4::{
    Application, ApplicationWindow, CssProvider, DragSource, DropTargetAsync, EventControllerKey,
    GestureClick, Label, ListBox, Orientation,
};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use std::path::{Path, PathBuf};

use crate::config::io::load_config;
use crate::state::{DropItem, ItemData, StateLocation};
use crate::ui::card::Card;
use crate::{debug, error};

pub fn build_ui(
    app: &Application,
    config_path: Option<&Path>,
    save: StateLocation,
    load_paths: Vec<PathBuf>,
    load_dirs: Vec<PathBuf>,
) {
    let config = load_config(config_path);
    let mut state = save.load_state();
    for path in load_paths {
        if let Ok(s) = fs::read_to_string(&path) {
            match serde_json::from_str::<Vec<DropItem>>(&s) {
                Ok(items) => state.extend(items),
                Err(e) => error!("Failed to parse load file: {}", e),
            }
        } else {
            error!("Failed to read load file: {:?}", path);
        }
    }

    load_dirs.into_iter().for_each(|d| match list_dir(&d) {
        Ok(mut items) => state.append(&mut items),
        Err(err) => error!("{}", err),
    });

    debug!("Loaded config: {:?}", config);
    debug!("Loaded state: {:?}", state);
    let window = ApplicationWindow::builder()
        .application(app)
        .title("DragEvac")
        .vexpand(true)
        .hexpand(true)
        .build();

    window.init_layer_shell();
    window.set_layer(Layer::Top);

    if config.exclusive {
        window.auto_exclusive_zone_enable();
    }
    window.set_keyboard_mode(KeyboardMode::OnDemand);

    let (top, bottom, left, right) = config.get_edges();
    window.set_anchor(Edge::Top, top);
    window.set_anchor(Edge::Bottom, bottom);
    window.set_anchor(Edge::Left, left);
    window.set_anchor(Edge::Right, right);

    let base_provider = CssProvider::new();
    base_provider.load_from_data(&config.css);
    let display = Display::default().unwrap();
    gtk4::style_context_add_provider_for_display(
        &display,
        &base_provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let key_controller = EventControllerKey::new();
    let a = app.clone();
    let w = window.clone();

    key_controller.connect_key_pressed(move |_, keyval, _, _| {
        // Quit on Escape
        if keyval == Key::Escape {
            w.set_visible(false);
            w.set_sensitive(false);
            a.quit();
            Propagation::Stop
        } else {
            Propagation::Proceed
        }
    });

    window.add_controller(key_controller);

    let items = Arc::new(Mutex::new(state.clone()));

    let vbox = Box::new(Orientation::Vertical, 0);

    let placeholder = Label::new(Some(&config.empty_text));
    if !state.is_empty() && !config.keep_text {
        placeholder.set_visible(false);
    }

    let list_box = ListBox::new();
    list_box.set_selection_mode(SelectionMode::Multiple);

    for item in state {
        add_row_to_list(&list_box, &items, item);
    }

    let click_controller = GestureClick::new();
    let lb2 = list_box.clone();
    click_controller.connect_released(move |gesture, _, _, _| {
        if !gesture
            .current_event_state()
            .contains(ModifierType::CONTROL_MASK)
        {
            lb2.unselect_all();
        }
    });
    list_box.add_controller(click_controller);

    vbox.append(&placeholder);
    vbox.append(&list_box);

    let text_formats = ContentFormats::new(&["text/uri-list", "text/plain"]);
    let file_formats = ContentFormats::for_type(FileList::static_type());
    let combined_formats = text_formats.union(&file_formats);

    let divider = Separator::builder().build();
    vbox.append(&divider);

    let cards_box = Box::new(Orientation::Horizontal, 0);
    cards_box.set_halign(Align::Center);
    for action in config.actions {
        let card = Card::new(
            action,
            items.clone(),
            list_box.clone(),
            save.clone(),
            placeholder.clone(),
            config.keep_text,
        );
        cards_box.append(&card);
    }

    vbox.append(&cards_box);

    if !matches!(save, StateLocation::ReadOnly(_)) {
        let drop_target = DropTargetAsync::new(Some(combined_formats), DragAction::COPY);

        drop_target.connect_drop(move |_target, drop, _x, _y| {
            // Ignore drops that originated from this program
            if drop.drag().is_some() {
                debug!("Ignoring self-drop.");
                drop.finish(DragAction::empty());
                return false;
            }

            let formats = drop.formats();
            let is_file = formats.contains_type(FileList::static_type());

            let items2 = items.clone();
            let list_box2 = list_box.clone();
            let placeholder2 = placeholder.clone();
            let drop_ref2 = drop.clone();
            let save2 = save.clone();

            if is_file {
                // Dropped item is a list of files
                debug!("File drop detected.");
                drop.read_value_async(
                    FileList::static_type(),
                    Priority::DEFAULT,
                    None::<&Cancellable>,
                    move |result| match result {
                        Ok(fl) => {
                            let file_list: FileList = fl.get().unwrap();
                            for file in file_list.files() {
                                let uri = file.uri().to_string();
                                let name = file
                                    .basename()
                                    .map(|p| p.display().to_string())
                                    .unwrap_or_else(|| uri.clone());
                                let item = DropItem {
                                    display_name: name,
                                    item: if file.path().map_or(false, |p| p.is_dir()) {
                                        ItemData::Dir(uri)
                                    } else {
                                        ItemData::File(uri)
                                    },
                                };

                                {
                                    let mut list = items2.lock().unwrap();
                                    list.push(item.clone());
                                    add_row_to_list(&list_box2, &items2, item);
                                    save2.write_state(&list);
                                }
                            }

                            if !config.keep_text {
                                placeholder2.set_visible(false);
                            }

                            drop_ref2.finish(DragAction::COPY);
                        }
                        Err(err) => {
                            error!("Failed to read dropped files: {err}");
                            drop_ref2.finish(DragAction::empty());
                        }
                    },
                );
            } else {
                // Dropped item is text
                debug!("Text drop detected.");
                drop.read_value_async(
                    glib::Type::STRING,
                    Priority::DEFAULT,
                    None::<&Cancellable>,
                    move |result| match result {
                        Ok(value) => {
                            let text: String = value.get().unwrap_or_default();
                            if text.is_empty() {
                                drop_ref2.finish(DragAction::empty());
                                return;
                            }

                            let item = DropItem {
                                display_name: text.clone(),
                                item: ItemData::File(text),
                            };

                            {
                                let mut list = items2.lock().unwrap();
                                list.push(item.clone());
                                add_row_to_list(&list_box2, &items2, item);
                                save2.write_state(&list);
                            }

                            if !config.keep_text {
                                placeholder2.set_visible(false);
                            }

                            drop_ref2.finish(DragAction::COPY);
                        }
                        Err(err) => {
                            error!("Failed to read dropped text: {err}");
                            drop_ref2.finish(DragAction::empty());
                        }
                    },
                );
            }

            true
        });

        vbox.add_controller(drop_target);
    }

    window.set_child(Some(&vbox));
    window.present();
}

fn list_dir(dir: &Path) -> Result<Vec<DropItem>, String> {
    match fs::read_dir(dir) {
        Err(e) => Err(format!("Failed to read directory {:?}: {}", dir, e)),
        Ok(entries) => {
            let mut items: Vec<DropItem> = entries
                .flatten()
                .filter_map(|entry| {
                    let path = fs::canonicalize(&entry.path()).ok()?;
                    let file = File::for_path(&path);
                    let uri = file.uri().to_string();
                    let name = file
                        .basename()
                        .map(|b| b.display().to_string())
                        .unwrap_or_else(|| uri.clone());
                    Some(DropItem {
                        display_name: name,
                        item: if path.is_dir() {
                            ItemData::Dir(uri)
                        } else {
                            ItemData::File(uri)
                        },
                    })
                })
                .collect();
            items.sort_by(|a, b| {
                b.item
                    .is_dir()
                    .cmp(&a.item.is_dir())
                    .then_with(|| a.display_name.cmp(&b.display_name))
            });
            Ok(items)
        }
    }
}

fn navigate_into_dir(
    listbox: &ListBox,
    items: &Arc<Mutex<Vec<DropItem>>>,
    dir: &Path,
) -> Result<(), String> {
    while let Some(child) = listbox.first_child() {
        listbox.remove(&child);
    }

    let new_items = list_dir(dir)?;

    let mut lock = items.lock().unwrap();
    *lock = new_items.clone();
    drop(lock);

    for item in new_items {
        add_row_to_list(listbox, items, item);
    }

    Ok(())
}

pub fn add_row_to_list(listbox: &ListBox, items: &Arc<Mutex<Vec<DropItem>>>, item: DropItem) {
    let row = Box::new(Orientation::Horizontal, 8);

    let name = Label::new(Some(&item.display_name));
    name.set_hexpand(true);
    name.set_halign(Align::Start);

    let mime = Label::new(Some(item.item.mime()));

    row.append(&name);
    row.append(&mime);

    let double_click = GestureClick::new();
    let listbox2 = listbox.clone();
    let items2 = items.clone();
    let item2 = item.clone();

    double_click.connect_released(move |gesture, count, _, _| {
        if count < 2 {
            return;
        }
        gesture.set_state(EventSequenceState::Claimed);
        if item2.item.mime() != "text/uri-list" {
            return;
        }
        if let Some(p) = File::for_uri(item2.item.uri()).path() {
            if p.is_dir() {
                if let Err(e) = navigate_into_dir(&listbox2, &items2, &p) {
                    error!("{}", e);
                };
            }
        }
    });

    row.add_controller(double_click);

    let drag_source = DragSource::new();
    drag_source.set_actions(DragAction::COPY);

    let list_box_clone = listbox.clone();
    let items_clone = items.clone();

    drag_source.connect_prepare(move |_, _, _| {
        let selected_items: Vec<DropItem> = list_box_clone
            .selected_rows()
            .iter()
            .map(|row| {
                let items_lock = items_clone.lock().unwrap();
                items_lock[row.index() as usize].clone()
            })
            .collect();
        let all_uris = selected_items
            .iter()
            .all(|i| i.item.mime() == "text/uri-list");
        let text = selected_items
            .iter()
            .map(|i| i.item.uri().to_owned())
            .collect::<Vec<String>>()
            .join("\n");
        if all_uris {
            Some(ContentProvider::for_bytes(
                "text/uri-list",
                &Bytes::from(text.as_bytes()),
            ))
        } else {
            Some(ContentProvider::for_value(&text.to_value()))
        }
    });
    row.add_controller(drag_source);

    listbox.append(&row);
}
