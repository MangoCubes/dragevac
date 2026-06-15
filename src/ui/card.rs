use gtk4::{
    Align, DropTargetAsync, Frame, Label, ListBox,
    gdk::{ContentFormats, DragAction, FileList},
    gio::{Cancellable, File, prelude::FileExt},
    glib::{Priority, types::StaticType},
    prelude::{FrameExt, WidgetExt},
};
use std::sync::{Arc, Mutex};

use crate::state::{DropItem, StateLocation};
use crate::{
    config::action::{Action, OnDrop},
    ui::add_row_to_list,
};
use crate::{debug, error};

pub struct Card {}

impl Card {
    pub fn new(
        action: Action,
        items: Arc<Mutex<Vec<DropItem>>>,
        list_box: ListBox,
        save: StateLocation,
        placeholder: Label,
        keep_text: bool,
    ) -> Frame {
        let card = Frame::builder()
            .css_classes(["action-card", "card"])
            .build();

        if let Some(class_name) = &action.class_name {
            card.add_css_class(class_name);
        }

        let text_formats = ContentFormats::new(&["text/uri-list", "text/plain"]);
        let file_formats = ContentFormats::for_type(FileList::static_type());
        let combined_formats = text_formats.union(&file_formats);
        let card_label = Label::new(Some(&action.title));
        card_label.set_halign(Align::Center);
        card_label.set_valign(Align::Center);
        card.set_child(Some(&card_label));

        let card_drop_target =
            DropTargetAsync::new(Some(combined_formats.clone()), DragAction::COPY);

        card_drop_target.connect_drop(move |_target, drop, _x, _y| {
            // Ignore self drop
            if action.block_self_drop && drop.drag().is_some() {
                debug!("Self drop into the card ignored.");
                drop.finish(DragAction::empty());
                return false;
            }

            let formats = drop.formats();

            // Check if the format is acceptable
            if !action.accept.is_empty() {
                let acceptable = action
                    .accept
                    .iter()
                    .any(|mime| formats.contain_mime_type(mime));
                if !acceptable {
                    drop.finish(DragAction::empty());
                    return false;
                }
            }

            let action_clone = action.clone();
            let items_clone = items.clone();
            let list_box_clone = list_box.clone();
            let save_clone = save.clone();
            let placeholder_clone = placeholder.clone();

            // Handler
            let handle_dropped_items = move |drops: Vec<DropItem>| {
                let uris = drops
                    .iter()
                    .map(|i| i.data.clone())
                    .collect::<Vec<String>>();
                let uris_str = uris.join(&action_clone.concat);
                let paths = drops
                    .iter()
                    .map(|i| {
                        if i.mime_type == "text/uri-list" {
                            File::for_uri(&i.data)
                                .path()
                                .map(|p| p.display().to_string())
                                .unwrap_or_else(|| i.data.clone())
                        } else {
                            i.data.clone()
                        }
                    })
                    .collect::<Vec<String>>();
                let paths_str = paths.join(&action_clone.concat);
                let home_str =
                    std::env::var("HOME").expect("Can't get environment variable $HOME.");

                let processed: Vec<String> =
                    action_clone
                        .command
                        .into_iter()
                        .fold(Vec::new(), |mut acc, arg| {
                            match arg.as_str() {
                                "%%ITEMS" => acc.push("%ITEMS".to_owned()),
                                "%ITEMS" => acc.extend(paths.clone()),
                                "%%URIS" => acc.push("%URIS".to_owned()),
                                "%URIS" => acc.extend(uris.clone()),
                                other => {
                                    let replaced = other
                                        .replace("%%ITEMSSTR", "\0ITEMSSTR\0")
                                        .replace("%%HOME", "\0HOME\0")
                                        .replace("%%URISSTR", "\0URISSTR\0")
                                        .replace("%ITEMSSTR", &paths_str)
                                        .replace("%HOME", &home_str)
                                        .replace("%URISSTR", &uris_str)
                                        .replace("\0ITEMSSTR\0", "%ITEMSSTR")
                                        .replace("\0HOME\0", "%HOME")
                                        .replace("\0URISSTR\0", "%URISSTR");

                                    acc.push(replaced);
                                }
                            };
                            acc
                        });

                if let Some((cmd, args_slice)) = processed.split_first() {
                    match std::process::Command::new(cmd).args(args_slice).spawn() {
                        Ok(_) => debug!("Executed command: {:?}", processed),
                        Err(e) => error!("Failed to execute command: {}", e),
                    }
                }

                let mut changed = false;
                let mut list = items_clone.lock().expect("Failed to lock list.");
                match action_clone.on_drop {
                    OnDrop::RemoveFromList => {
                        for dropped in &drops {
                            let orig_len = list.len();
                            list.retain(|i| i != dropped);
                            if list.len() != orig_len {
                                changed = true;
                            }
                        }
                    }
                    OnDrop::RemoveFirstFromList => {
                        for dropped in &drops {
                            if let Some(pos) = list.iter().position(|i| i == dropped) {
                                list.remove(pos);
                                changed = true;
                            }
                        }
                    }
                    OnDrop::AddToList => {
                        for dropped in &drops {
                            list.push(dropped.clone());
                            changed = true;
                        }
                    }
                    OnDrop::AddToListUnique => {
                        for dropped in &drops {
                            if !list.contains(dropped) {
                                list.push(dropped.clone());
                                changed = true;
                            }
                        }
                    }
                    OnDrop::NoAction => {}
                }

                if changed {
                    let list = items_clone.lock().unwrap();
                    for item in list.iter() {
                        add_row_to_list(&list_box_clone, &items_clone, item.clone());
                    }
                    if !list.is_empty() && !keep_text {
                        placeholder_clone.set_visible(false);
                    } else if list.is_empty() {
                        placeholder_clone.set_visible(true);
                    }
                    save_clone.write_state(&list);
                }
            };

            let drop_ref = drop.clone();
            drop.read_value_async(
                FileList::static_type(),
                Priority::DEFAULT,
                None::<&Cancellable>,
                move |result| match result {
                    Ok(fl) => {
                        let file_list: FileList = match fl.get() {
                            Ok(f) => f,
                            Err(err) => {
                                error!("Card failed to read files: {}", err);
                                drop_ref.finish(DragAction::empty());
                                return;
                            }
                        };
                        let drops = file_list
                            .files()
                            .iter()
                            .map(|f| {
                                let uri = f.uri().to_string();
                                DropItem {
                                    display_name: f
                                        .basename()
                                        .map(|p| p.display().to_string())
                                        .unwrap_or_else(|| uri.clone()),
                                    data: uri,
                                    mime_type: "text/uri-list".to_string(),
                                }
                            })
                            .collect();
                        handle_dropped_items(drops);
                        drop_ref.finish(DragAction::COPY);
                    }
                    Err(err) => {
                        error!("Card failed to read files: {}", err);
                        drop_ref.finish(DragAction::empty());
                    }
                },
            );
            true
        });

        card.add_controller(card_drop_target);
        card
    }
}
