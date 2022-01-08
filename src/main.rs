//! cj-nbt Minecraft NBT editor

mod ui;

use fltk::app::App;
use fltk::dialog::{FileDialog, FileDialogOptions, FileDialogType};
use fltk::prelude::*;
use ui::UserInterface;

fn main() {
    let app = App::default();

    let mut ui = UserInterface::make_window();
    init_main_ui_menubar(&mut ui);

    app.run().expect("an fltk error occurred");
}

fn init_main_ui_menubar(ui: &mut UserInterface) {
    let menu = ui.main_ui_menubar.clone();

    if let Some(mut item) = menu.find_item("File/Open") {
        item.set_callback(|_| show_open_nbt_file_dialog());
    }
}

fn show_open_nbt_file_dialog() {
    let mut dlg = FileDialog::new(FileDialogType::BrowseFile);
    dlg.set_option(FileDialogOptions::NoOptions);
    dlg.set_filter(".Dat Files\t*.dat\nOther Files\t*.*");
    dlg.show();
    let filename = dlg.filename().to_string_lossy().to_string();
    if !filename.is_empty() {
        println!("load file: {}", filename);
    }
}
