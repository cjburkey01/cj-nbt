//! cj-nbt Minecraft NBT editor

mod nbt_util;
mod ui;

use fltk::app;
use fltk::app::Sender;
use fltk::dialog::{FileDialog, FileDialogOptions, FileDialogType};
use fltk::frame::Frame;
use fltk::menu::MenuBar;
use fltk::prelude::*;
use fltk::tree::{Tree, TreeConnectorStyle, TreeItemDrawMode, TreeSelect, TreeSort};
use std::fs::File;
use std::path::{Path, PathBuf};
use ui::UserInterface;

// Temporary lang handling :P
const MENU_FILE_OPEN: &str = "File/Open";
const TREE_EMPTY_LABEL: &str = "No file open";

/// Possible events to receive from the GUI
#[derive(Debug, Copy, Clone)]
enum CJNbtAppEvent {
    /// Open file
    OpenFile,
}

/// Container for our FLTK application and our app state.
struct CJNbtApp {
    /// The FLTK app.
    app: app::App,
    /// The user interface.
    ui: UserInterface,
    /// FLT event receiver.
    receiver: app::Receiver<CJNbtAppEvent>,

    /// The current open NBT file, or None if no file is open.
    current: Option<CurrentNbtFile>,
}

/// Information about the currently loaded NBT file.
#[allow(dead_code)]
struct CurrentNbtFile {
    /// Path to the file, or None if the file doesn't have a path yet.
    file_path: Option<PathBuf>,
    /// The root NBT node.
    current_root: nbt::Value,
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
            app,
            ui,
            receiver,

            current: None,
        }
    }

    fn init_main_ui_menubar(sender: Sender<CJNbtAppEvent>, menubar: MenuBar) {
        if let Some(mut item) = menubar.find_item(MENU_FILE_OPEN) {
            item.emit(sender, CJNbtAppEvent::OpenFile)
        }
    }

    fn init_main_ui_tree(mut tree: Tree) {
        tree.set_root_label(TREE_EMPTY_LABEL);
        tree.set_connector_style(TreeConnectorStyle::Solid);
        tree.set_item_draw_mode(TreeItemDrawMode::Default);
        tree.set_select_mode(TreeSelect::Single);
        tree.set_sort_order(TreeSort::Ascending);
    }

    /// Take control and start listening for events from our FLTK GUI.
    pub fn run(mut self) {
        while self.app.wait() {
            if let Some(msg) = self.receiver.recv() {
                match msg {
                    // Try to load a new file into the editor
                    CJNbtAppEvent::OpenFile => {
                        // Request a file from the user
                        if let Some(selected_path) = show_open_nbt_file_dialog() {
                            println!("parsing nbt");

                            // Try to parse the selected file
                            match Self::parse_nbt_file(&selected_path) {
                                Ok((name, root_node)) => {
                                    println!("successfully parsed NBT");

                                    // Load the NBT data into the tree
                                    Self::load_nbt_root_into_tree(
                                        self.ui.nbt_tag_tree.clone(),
                                        &name,
                                        &root_node,
                                        true,
                                    );

                                    // Update current open file in our app state
                                    self.current = Some(CurrentNbtFile {
                                        file_path: Some(selected_path),
                                        current_root: root_node,
                                    });
                                }
                                Err(e) => panic!("failed to parse nbt: {}", e),
                            }
                        }
                    }
                }
            }
        }
    }

    fn parse_nbt_file(file_path: &Path) -> nbt::Result<(String, nbt::Value)> {
        let mut file = File::open(file_path)?;
        // Try to interpret it as a compressed NBT file
        match nbt_util::from_gzip_reader(&mut file) {
            v @ Ok(_) => v,
            // If that fails, try to read it as if it weren't compressed.
            Err(err) => {
                println!(
                    "failed to read nbt file with gzip stream reader ({:?}), trying to read uncompressed.",
                    err
                );
                nbt_util::from_reader(&mut file)
            }
        }
    }

    // Function to load NBT elements recursively
    fn load_nbt_root_into_tree(
        mut tree: Tree,
        current_level: &str,
        root: &nbt::Value,
        clear_tree: bool,
    ) {
        if clear_tree {
            tree.clear();
            tree.set_show_root(false);
        }

        // Create this node
        if let Some(mut tree_item) = tree.add(current_level) {
            // Create a frame for our custom label to override the tree location
            let mut new_label_widget = Frame::default();
            let key = &current_level[current_level.rfind('/').map(|v| v + 1).unwrap_or(0)..];
            new_label_widget.set_label(&format!("{} ({})", key, &root.tag_name()[4..]));
            let (lw, lh) = new_label_widget.measure_label();
            new_label_widget.set_size(lw + 4, lh + 4);

            tree_item.set_widget(&new_label_widget);
            tree_item.close();
        }

        match root {
            // Make compound tags expandable
            nbt::Value::Compound(map) => map.iter().for_each(|(key, val)| {
                Self::load_nbt_root_into_tree(
                    tree.clone(),
                    &format!("{}/{}", current_level, key),
                    val,
                    false,
                );
            }),

            // Make arbitrary tag lists expandable
            nbt::Value::List(values) => {
                values.iter().enumerate().for_each(|(i, val)| {
                    Self::load_nbt_root_into_tree(
                        tree.clone(),
                        &format!("{}/{}", current_level, i),
                        val,
                        false,
                    );
                });
            }

            // Nothing else should expand (arrays can be edited with the array edit)
            _ => {}
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
        None
    } else {
        println!("load file: {:?}", path);
        Some(path)
    }
}
