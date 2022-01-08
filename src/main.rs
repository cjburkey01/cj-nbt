//! cj-nbt Minecraft NBT editor

mod ui;

use fltk::app;
use fltk::app::Sender;
use fltk::dialog::{FileDialog, FileDialogOptions, FileDialogType};
use fltk::menu::MenuBar;
use fltk::prelude::*;
use fltk::tree::Tree;
use std::path::PathBuf;
use ui::UserInterface;

// Temporary lang :P
const MENU_FILE_OPEN: &'static str = "File/Open";
const TREE_EMPTY_LABEL: &'static str = "No file open";

#[derive(Debug, Copy, Clone)]
enum CJNbtAppEvent {
    /// Open file
    OpenFile,
}

struct CJNbtApp {
    /// The current open file, or None if no file is open.
    current_file: Option<PathBuf>,
    /// The FLTK app.
    app: app::App,
    /// The user interface.
    ui: UserInterface,
    /// FLT event receiver.
    receiver: app::Receiver<CJNbtAppEvent>,
}

impl CJNbtApp {
    pub fn new() -> Self {
        // Create the FLTK app
        let app = app::App::default();
        // Create the event channel
        let (sender, receiver) = app::channel();

        // Initialize the UI from the fluid project converted into rust
        let ui = UserInterface::make_window();
        Self::init_main_ui_menubar(sender, ui.main_ui_menubar.clone());
        Self::init_main_ui_tree(ui.nbt_tag_tree.clone());

        // Wrap and return
        Self {
            current_file: None,
            app,
            ui,
            receiver,
        }
    }

    fn init_main_ui_menubar(sender: Sender<CJNbtAppEvent>, menubar: MenuBar) {
        if let Some(mut item) = menubar.find_item(MENU_FILE_OPEN) {
            item.emit(sender, CJNbtAppEvent::OpenFile)
        }
    }

    fn init_main_ui_tree(mut tree: Tree) {
        tree.set_root_label(TREE_EMPTY_LABEL);
    }

    pub fn run(mut self) {
        while self.app.wait() {
            if let Some(msg) = self.receiver.recv() {
                match msg {
                    // Try to load a new file into the editor
                    CJNbtAppEvent::OpenFile => self.current_file = show_open_nbt_file_dialog(),
                }
            }
        }
    }
}

fn main() {
    // Initialize the app
    let cj_nbt_app = CJNbtApp::new();

    // Run the app
    cj_nbt_app.run();
}

fn show_open_nbt_file_dialog() -> Option<PathBuf> {
    // Create and show dialog for an NBT file selection
    let mut dlg = FileDialog::new(FileDialogType::BrowseFile);
    dlg.set_option(FileDialogOptions::NoOptions);
    dlg.set_filter(".Dat Files\t*.dat");
    dlg.show();

    // Get the path and determine whether the user cancelled
    let path = dlg.filename();
    if path.as_os_str().is_empty() {
        println!("user cancelled file load");
        Some(path)
    } else {
        println!("load file: {:?}", path);
        None
    }
}
