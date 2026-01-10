use gtk4::CssProvider;
use gtk4::gdk::Display;
use gtk4::prelude::GtkWindowExt;
use gtk4::{Application, ApplicationWindow};
use gtk4_layer_shell::{Layer, LayerShell};

pub fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("DragBox")
        .build();

    window.init_layer_shell();
    window.set_layer(Layer::Top);

    let base_provider = CssProvider::new();
    base_provider.load_from_data(include_str!("./default.css"));
    let display = Display::default().unwrap();
    gtk4::style_context_add_provider_for_display(
        &display,
        &base_provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let label = gtk4::Label::new(Some("Place items here"));
    window.set_child(Some(&label));

    window.present();
}
