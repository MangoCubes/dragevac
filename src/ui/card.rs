use gtk4::{
    Align, DropTargetAsync, Label,
    gdk::{ContentFormats, DragAction, FileList},
    gio::{Cancellable, prelude::FileExt},
    glib::{self, Priority, types::StaticType},
    prelude::{FrameExt, WidgetExt},
};

use crate::{debug, error};

pub struct Card {}

impl Card {
    pub fn new(name: String) -> gtk4::Frame {
        let card = gtk4::Frame::builder().css_classes(["card"]).build();

        let text_formats = ContentFormats::new(&["text/uri-list", "text/plain"]);
        let file_formats = ContentFormats::for_type(FileList::static_type());
        let combined_formats = text_formats.union(&file_formats);
        let card_label = Label::new(Some(&name));
        card_label.set_halign(Align::Center);
        card_label.set_valign(Align::Center);
        card.set_child(Some(&card_label));

        let card_drop_target =
            DropTargetAsync::new(Some(combined_formats.clone()), DragAction::COPY);

        card_drop_target.connect_drop(move |_target, drop, _x, _y| {
            let formats = drop.formats();
            let is_file = formats.contains_type(FileList::static_type());
            let drop_ref = drop.clone();

            if is_file {
                drop.read_value_async(
                    FileList::static_type(),
                    Priority::DEFAULT,
                    None::<&Cancellable>,
                    move |result| match result {
                        Ok(fl) => {
                            let file_list: FileList = fl.get().unwrap();
                            for file in file_list.files() {
                                debug!("Card received file: {:?}", file.uri());
                            }
                            drop_ref.finish(DragAction::COPY);
                        }
                        Err(err) => {
                            error!("Card failed to read files: {}", err);
                            drop_ref.finish(DragAction::empty());
                        }
                    },
                );
            } else {
                drop.read_value_async(
                    glib::Type::STRING,
                    Priority::DEFAULT,
                    None::<&Cancellable>,
                    move |result| match result {
                        Ok(value) => {
                            let text: String = value.get().unwrap_or_default();
                            debug!("Card received text: {}", text);
                            drop_ref.finish(DragAction::COPY);
                        }
                        Err(err) => {
                            error!("Card failed to read text: {}", err);
                            drop_ref.finish(DragAction::empty());
                        }
                    },
                );
            }
            true
        });

        card.add_controller(card_drop_target);
        card
    }
}
