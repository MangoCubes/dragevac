mod dropitem;

use std::sync::{Arc, Mutex};

use gtk4::gdk;
use gtk4::gdk::{ContentFormats, ContentProvider, Display, DragAction, Key};
use gtk4::gio::prelude::ApplicationExt;
use gtk4::glib::{self, Propagation};
use gtk4::prelude::{BoxExt, FileExt, GtkWindowExt, StaticType, WidgetExt};
use gtk4::{
    Application, ApplicationWindow, CssProvider, DragSource, DropTargetAsync, EventControllerKey,
    Label, ListBox, Orientation,
};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use std::path::Path;

use crate::config::load_config;
use crate::ui::dropitem::DropItem;
use crate::{debug, error};

pub fn build_ui(app: &Application, config_path: Option<&Path>) {
    let config = load_config(config_path);
    let window = ApplicationWindow::builder()
        .application(app)
        .title("DragBox")
        .vexpand(true)
        .hexpand(true)
        .build();

    window.init_layer_shell();
    window.set_layer(Layer::Top);
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

    let items: Arc<Mutex<Vec<DropItem>>> = Arc::new(Mutex::new(Vec::new()));

    let vbox = gtk4::Box::new(Orientation::Vertical, 0);

    let placeholder = Label::new(Some(&config.empty_text));

    let list_box = ListBox::new();

    vbox.append(&placeholder);
    vbox.append(&list_box);

    let text_formats = ContentFormats::new(&["text/uri-list", "text/plain"]);
    let file_formats = ContentFormats::for_type(gdk::FileList::static_type());
    let combined_formats = text_formats.union(&file_formats);

    let drop_target = DropTargetAsync::new(Some(combined_formats), DragAction::COPY);

    drop_target.connect_drop(move |_target, drop, _x, _y| {
        let formats = drop.formats();
        let is_file = formats.contains_type(gdk::FileList::static_type());

        let items2 = items.clone();
        let list_box2 = list_box.clone();
        let placeholder2 = placeholder.clone();
        let drop_ref2 = drop.clone();

        if is_file {
            // Dropped item is a list of files
            debug!("File drop detected.");
            drop.read_value_async(
                gdk::FileList::static_type(),
                glib::Priority::DEFAULT,
                None::<&gtk4::gio::Cancellable>,
                move |result| match result {
                    Ok(fl) => {
                        let file_list: gdk::FileList = fl.get().unwrap();
                        for file in file_list.files() {
                            let uri = file.uri().to_string();
                            let name = file
                                .basename()
                                .map(|p| p.display().to_string())
                                .unwrap_or_else(|| uri.clone());

                            let item = DropItem {
                                display_name: name,
                                data: uri.as_bytes().to_vec(),
                                mime_type: "text/uri-list".to_string(),
                            };

                            {
                                items2.lock().unwrap().push(item.clone());
                                add_row_to_list(&list_box2, &items2, item);
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
                glib::Priority::DEFAULT,
                None::<&gtk4::gio::Cancellable>,
                move |result| match result {
                    Ok(value) => {
                        let text: String = value.get().unwrap_or_default();
                        if text.is_empty() {
                            drop_ref2.finish(DragAction::empty());
                            return;
                        }

                        let item = DropItem {
                            display_name: text.clone(),
                            data: text.as_bytes().to_vec(),
                            mime_type: "text/plain".to_string(),
                        };

                        {
                            items2.lock().unwrap().push(item.clone());
                            add_row_to_list(&list_box2, &items2, item);
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

    window.set_child(Some(&vbox));
    window.present();
}

fn add_row_to_list(list_box: &ListBox, _items: &Arc<Mutex<Vec<DropItem>>>, item: DropItem) {
    let row = gtk4::Box::new(Orientation::Horizontal, 8);

    let name = Label::new(Some(&item.display_name));
    name.set_hexpand(true);
    name.set_halign(gtk4::Align::Start);

    let mime = Label::new(Some(&item.mime_type));

    row.append(&name);
    row.append(&mime);

    let drag_source = DragSource::new();
    drag_source.set_actions(DragAction::COPY);

    drag_source.connect_prepare(move |_source, _x, _y| {
        let bytes = glib::Bytes::from(&item.data);
        Some(ContentProvider::for_bytes(&item.mime_type, &bytes))
    });

    row.add_controller(drag_source);

    list_box.append(&row);
}
