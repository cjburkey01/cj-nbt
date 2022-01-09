//! cj-nbt Minecraft NBT editor

mod ui;

use byteorder::{BigEndian, ReadBytesExt};
use flate2::read::GzDecoder;
use fltk::app;
use fltk::app::Sender;
use fltk::dialog::{FileDialog, FileDialogOptions, FileDialogType};
use fltk::menu::MenuBar;
use fltk::prelude::*;
use fltk::tree::Tree;
use std::fs::File;
use std::path::{Path, PathBuf};
use ui::UserInterface;

// Temporary lang handling :P
const MENU_FILE_OPEN: &'static str = "File/Open";
const TREE_EMPTY_LABEL: &'static str = "No file open";

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

    /// The current open file, or None if no file is open.
    current_file: Option<PathBuf>,
    /// The current root of the NBT data structure, if a file is open.
    current_root: Option<nbt::Value>,
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

            current_file: None,
            current_root: None,
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

    /// Take control and start listening for events from our FLTK GUI.
    pub fn run(mut self) {
        while self.app.wait() {
            if let Some(msg) = self.receiver.recv() {
                match msg {
                    // Try to load a new file into the editor
                    CJNbtAppEvent::OpenFile => {
                        if let Some(selected_path) = show_open_nbt_file_dialog() {
                            println!("parsing nbt");
                            match Self::parse_nbt_file(&selected_path) {
                                Ok((name, root_node)) => {
                                    println!("successfully parsed NBT");
                                    Self::load_nbt_root_into_tree(
                                        self.ui.nbt_tag_tree.clone(),
                                        &name,
                                        &root_node,
                                        true,
                                    );
                                    self.current_root = Some(root_node);
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
        match Self::from_gzip_reader(&mut file) {
            v @ Ok(_) => v,
            // If that fails, try to read it as if it weren't compressed.
            Err(err) => {
                println!(
                    "failed to read nbt file with gzip stream reader ({:?}), trying to read uncompressed.",
                    err
                );
                Self::from_reader(&mut file)
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
        }
        tree.add(current_level);
        let _ = tree.close(current_level, false);
        if let nbt::Value::Compound(map) = root {
            map.iter().for_each(|(key, val)| {
                Self::load_nbt_root_into_tree(
                    tree.clone(),
                    &format!("{}/{}", current_level, key),
                    val,
                    false,
                );
            });
        }
    }

    /// Extracts an `Blob` object from an `io::Read` source that is
    /// compressed using the Gzip format.
    fn from_gzip_reader<R>(src: &mut R) -> nbt::Result<(String, nbt::Value)>
    where
        R: std::io::Read,
    {
        // Reads the gzip header, and fails if it is incorrect.
        let mut data = GzDecoder::new(src);
        Self::from_reader(&mut data)
    }

    /// Ripped from [`nbt::blob::Blob::from_reader`]
    #[inline]
    fn from_reader<R>(src: &mut R) -> nbt::Result<(String, nbt::Value)>
    where
        R: std::io::Read,
    {
        // Try to read the first tag (should be a compound tag)
        let tag = src.read_u8()?;
        // We must at least consume this title
        let title = match tag {
            0x00 => "".to_string(),
            _ => Self::read_bare_string(src)?,
        };

        // Although it would be possible to read NBT format files composed of
        // arbitrary objects using the current API, by convention all files
        // have a top-level Compound.
        if tag != 0x0a {
            return Err(nbt::Error::NoRootCompound);
        }
        let content = nbt::Value::from_reader(tag, src)?;
        match content {
            val @ nbt::Value::Compound(_) => Ok((title, val)),
            _ => Err(nbt::Error::NoRootCompound),
        }
    }

    /// Ripped from [`nbt::blob::Blob::from_reader`]
    #[inline]
    fn read_bare_string<R>(src: &mut R) -> nbt::Result<String>
    where
        R: std::io::Read,
    {
        let len = src.read_u16::<BigEndian>()? as usize;

        if len == 0 {
            return Ok("".to_string());
        }

        let mut bytes = vec![0; len];
        let mut n_read = 0usize;
        while n_read < bytes.len() {
            match src.read(&mut bytes[n_read..])? {
                0 => return Err(nbt::Error::IncompleteNbtValue),
                n => n_read += n,
            }
        }

        let decoded = cesu8::from_java_cesu8(&bytes)?;
        Ok(decoded.into_owned())
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
