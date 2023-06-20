// this is modified from https://github.com/Barugon/egui_file to allow it to work with bevy_egui
#![allow(dead_code)]

use bevy_egui::egui::{
    self, vec2, Align2, Context, Key, Layout, Pos2, RichText, ScrollArea, TextEdit, Ui, Vec2,
    Window,
};
use std::{
    env,
    fmt::Debug,
    fs,
    io::Error,
    path::{Path, PathBuf},
};

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
/// Dialog state.
pub enum State {
    /// Is open.
    Open,
    /// Was closed.
    Closed,
    /// Was canceled.
    Cancelled,
    /// File was selected.
    Selected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Dialog type.
pub enum DialogType {
    SelectFolder,
    OpenFile,
    SaveFile,
}

/// `egui` component that represents `OpenFileDialog` or `SaveFileDialog`.
pub struct FileDialog {
    /// Current opened path.
    path: PathBuf,
    /// Editable field with path.
    path_edit: String,

    /// Selected file path
    selected_file: Option<PathBuf>,
    /// Editable field with filename.
    filename_edit: String,

    /// Files in directory.
    files: Result<Vec<PathBuf>, Error>,
    /// Current dialog state.
    state: State,
    /// Dialog type.
    dialog_type: DialogType,

    current_pos: Option<Pos2>,
    default_size: Vec2,
    anchor: Option<(Align2, Vec2)>,
    filter: Option<Filter>,
    resizable: bool,
    rename: bool,
    new_folder: bool,

    // Show hidden files on unix systems.
    #[cfg(unix)]
    show_hidden: bool,
}

impl Debug for FileDialog {
    #[cfg(target_family = "unix")]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileDialog")
            .field("path", &self.path)
            .field("path_edit", &self.path_edit)
            .field("selected_file", &self.selected_file)
            .field("filename_edit", &self.filename_edit)
            .field("files", &self.files)
            .field("state", &self.state)
            .field("dialog_type", &self.dialog_type)
            .field("current_pos", &self.current_pos)
            .field("default_size", &self.default_size)
            .field("anchor", &self.anchor)
            // Closures don't implement std::fmt::Debug.
            // .field("filter", &self.filter)
            .field("resizable", &self.resizable)
            .field("rename", &self.rename)
            .field("new_folder", &self.new_folder)
            .field("show_hidden", &self.show_hidden)
            .finish()
    }

    #[cfg(not(target_family = "unix"))]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileDialog")
            .field("path", &self.path)
            .field("path_edit", &self.path_edit)
            .field("selected_file", &self.selected_file)
            .field("filename_edit", &self.filename_edit)
            .field("files", &self.files)
            .field("state", &self.state)
            .field("dialog_type", &self.dialog_type)
            .field("current_pos", &self.current_pos)
            .field("default_size", &self.default_size)
            .field("anchor", &self.anchor)
            // Closures don't implement std::fmt::Debug.
            // .field("filter", &self.filter)
            .field("resizable", &self.resizable)
            .field("rename", &self.rename)
            .field("new_folder", &self.new_folder)
            .finish()
    }
}

/// Function that returns `true` if the path is accepted.
pub type Filter = Box<dyn Fn(&Path) -> bool + Send + Sync + 'static>;

impl FileDialog {
    /// Create dialog that prompts the user to select a folder.
    pub fn select_folder(initial_path: Option<PathBuf>) -> Self {
        FileDialog::new(DialogType::SelectFolder, initial_path)
    }

    /// Create dialog that prompts the user to open a file.
    pub fn open_file(initial_path: Option<PathBuf>) -> Self {
        FileDialog::new(DialogType::OpenFile, initial_path)
    }

    /// Create dialog that prompts the user to save a file.
    pub fn save_file(initial_path: Option<PathBuf>) -> Self {
        FileDialog::new(DialogType::SaveFile, initial_path)
    }

    /// Constructs new file dialog. If no `initial_path` is passed,`env::current_dir` is used.
    fn new(dialog_type: DialogType, initial_path: Option<PathBuf>) -> Self {
        let mut path = initial_path.unwrap_or_else(|| env::current_dir().unwrap_or_default());
        let mut filename_edit = String::new();

        if path.is_file() {
            assert!(dialog_type != DialogType::SelectFolder);
            filename_edit = get_file_name(&path).to_string();
            path.pop();
        }

        let path_edit = path.to_str().unwrap_or_default().to_string();
        let files = read_folder(&path);

        Self {
            path,
            path_edit,
            selected_file: None,
            filename_edit,
            files,
            state: State::Closed,
            dialog_type,

            current_pos: None,
            default_size: vec2(512.0, 512.0),
            anchor: None,
            filter: None,
            resizable: true,
            rename: true,
            new_folder: true,

            #[cfg(unix)]
            show_hidden: false,
        }
    }

    pub fn default_filename(mut self, filename: impl Into<String>) -> Self {
        self.filename_edit = filename.into();
        self
    }

    /// Set the window anchor.
    pub fn anchor(mut self, align: Align2, offset: impl Into<Vec2>) -> Self {
        self.anchor = Some((align, offset.into()));
        self
    }

    /// Set the window position.
    pub fn current_pos(mut self, current_pos: impl Into<Pos2>) -> Self {
        self.current_pos = Some(current_pos.into());
        self
    }

    /// Set the window default size.
    pub fn default_size(mut self, default_size: impl Into<Vec2>) -> Self {
        self.default_size = default_size.into();
        self
    }

    /// Enable/disable resizing the window. Default is `true`.
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// Show the Rename button. Default is `true`.
    pub fn show_rename(mut self, rename: bool) -> Self {
        self.rename = rename;
        self
    }

    /// Show the New Folder button. Default is `true`.
    pub fn show_new_folder(mut self, new_folder: bool) -> Self {
        self.new_folder = new_folder;
        self
    }

    /// Set a function to filter shown files.
    pub fn filter(mut self, filter: Filter) -> Self {
        self.filter = Some(filter);
        self
    }

    /// Get the dialog type.
    pub fn dialog_type(&self) -> DialogType {
        self.dialog_type
    }

    /// Get the window's visibility.
    pub fn visible(&self) -> bool {
        self.state == State::Open
    }

    /// Opens the dialog.
    pub fn open(&mut self) {
        self.state = State::Open;
    }

    /// Resulting file path.
    pub fn path(&self) -> Option<PathBuf> {
        self.selected_file.clone()
    }

    /// Dialog state.
    pub fn state(&self) -> State {
        self.state
    }

    /// Returns true, if the file selection was confirmed.
    pub fn selected(&self) -> bool {
        self.state == State::Selected
    }

    fn open_selected(&mut self) {
        if let Some(path) = &self.selected_file {
            if path.is_dir() {
                self.path = path.clone();
                self.refresh();
            } else if path.is_file() && self.dialog_type == DialogType::OpenFile {
                self.confirm();
            }
        }
    }

    fn confirm(&mut self) {
        self.state = State::Selected;
    }

    fn refresh(&mut self) {
        self.files = read_folder(&self.path);
        self.path_edit = String::from(self.path.to_str().unwrap_or_default());
        self.select(None);
    }

    fn select(&mut self, file: Option<PathBuf>) {
        self.filename_edit = match &file {
            Some(path) => get_file_name(path).to_string(),
            None => String::new(),
        };
        self.selected_file = file;
    }

    fn can_save(&self) -> bool {
        self.selected_file.is_some() || !self.filename_edit.is_empty()
    }

    fn can_open(&self) -> bool {
        self.selected_file.is_some()
    }

    fn can_rename(&self) -> bool {
        if !self.filename_edit.is_empty() {
            if let Some(file) = &self.selected_file {
                return get_file_name(file) != self.filename_edit;
            }
        }
        false
    }

    fn title(&self) -> &str {
        match self.dialog_type {
            DialogType::SelectFolder => "📁  Select Folder",
            DialogType::OpenFile => "📂  Open File",
            DialogType::SaveFile => "💾  Save File",
        }
    }

    /// Shows the dialog if it is open. It is also responsible for state management.
    /// Should be called every ui update.
    pub fn show(&mut self, ctx: &Context) -> &Self {
        self.state = match self.state {
            State::Open => {
                if ctx.input(|state| state.key_pressed(Key::Escape)) {
                    self.state = State::Cancelled;
                }

                let mut is_open = true;
                self.ui(ctx, &mut is_open);
                match is_open {
                    true => self.state,
                    false => State::Closed,
                }
            }
            _ => State::Closed,
        };

        self
    }

    fn ui(&mut self, ctx: &Context, is_open: &mut bool) {
        let mut window = Window::new(RichText::new(self.title()).strong())
            .open(is_open)
            .default_size(self.default_size)
            .resizable(self.resizable)
            .collapsible(false);

        if let Some((align, offset)) = self.anchor {
            window = window.anchor(align, offset);
        }

        if let Some(current_pos) = self.current_pos {
            window = window.current_pos(current_pos);
        }

        window.show(ctx, |ui| self.ui_in_window(ui));
    }

    fn ui_in_window(&mut self, ui: &mut Ui) {
        enum Command {
            Cancel,
            CreateDirectory,
            Folder,
            Open(PathBuf),
            OpenSelected,
            Refresh,
            Rename(PathBuf, PathBuf),
            Save(PathBuf),
            Select(PathBuf),
            UpDirectory,
        }
        let mut command: Option<Command> = None;

        // Top directory field with buttons.
        egui::TopBottomPanel::top("egui_file_top").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add_enabled_ui(self.path.parent().is_some(), |ui| {
                    let response = ui.button("⬆").on_hover_text("Parent Folder");
                    if response.clicked() {
                        command = Some(Command::UpDirectory);
                    }
                });
                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    let response = ui.button("⟲").on_hover_text("Refresh");
                    if response.clicked() {
                        command = Some(Command::Refresh);
                    }

                    let response = ui.add_sized(
                        ui.available_size(),
                        TextEdit::singleline(&mut self.path_edit),
                    );
                    if response.lost_focus() {
                        let path = PathBuf::from(&self.path_edit);
                        command = Some(Command::Open(path));
                    };
                });
            });
            ui.add_space(ui.spacing().item_spacing.y);
        });

        // Bottom file field.
        egui::TopBottomPanel::bottom("egui_file_bottom").show_inside(ui, |ui| {
            ui.add_space(ui.spacing().item_spacing.y * 2.0);
            ui.horizontal(|ui| {
                ui.label("File:");
                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.new_folder && ui.button("New Folder").clicked() {
                        command = Some(Command::CreateDirectory);
                    }

                    if self.rename {
                        ui.add_enabled_ui(self.can_rename(), |ui| {
                            if ui.button("Rename").clicked() {
                                if let Some(from) = self.selected_file.clone() {
                                    let to = from.with_file_name(&self.filename_edit);
                                    command = Some(Command::Rename(from, to));
                                }
                            }
                        });
                    }

                    let result = ui.add_sized(
                        ui.available_size(),
                        TextEdit::singleline(&mut self.filename_edit),
                    );

                    if result.lost_focus()
                        && result
                            .ctx
                            .input(|state| state.key_pressed(egui::Key::Enter))
                        && !self.filename_edit.is_empty()
                    {
                        let path = self.path.join(&self.filename_edit);
                        match self.dialog_type {
                            DialogType::SelectFolder => {
                                command = Some(Command::Folder);
                            }
                            DialogType::OpenFile => {
                                if path.exists() {
                                    command = Some(Command::Open(path));
                                }
                            }
                            DialogType::SaveFile => {
                                command = Some(match path.is_dir() {
                                    true => Command::Open(path),
                                    false => Command::Save(path),
                                });
                            }
                        }
                    }
                });
            });

            ui.add_space(ui.spacing().item_spacing.y);

            // Confirm, Cancel buttons.
            ui.horizontal(|ui| {
                match self.dialog_type {
                    DialogType::SelectFolder => {
                        ui.horizontal(|ui| {
                            if ui.button("Open").clicked() {
                                command = Some(Command::Folder);
                            };
                        });
                    }
                    DialogType::OpenFile => {
                        ui.horizontal(|ui| {
                            ui.set_enabled(self.can_open());
                            if ui.button("Open").clicked() {
                                command = Some(Command::OpenSelected);
                            };
                        });
                    }
                    DialogType::SaveFile => {
                        let should_open_directory = match &self.selected_file {
                            Some(file) => file.is_dir(),
                            None => false,
                        };

                        if should_open_directory {
                            if ui.button("Open").clicked() {
                                command = Some(Command::OpenSelected);
                            };
                        } else {
                            ui.horizontal(|ui| {
                                ui.set_enabled(self.can_save());
                                if ui.button("Save").clicked() {
                                    let filename = &self.filename_edit;
                                    let path = self.path.join(filename);
                                    command = Some(Command::Save(path));
                                };
                            });
                        }
                    }
                }

                if ui.button("Cancel").clicked() {
                    command = Some(Command::Cancel);
                }

                #[cfg(unix)]
                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.checkbox(&mut self.show_hidden, "Show Hidden");
                });
            });
        });

        // Rows with files.
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                match &self.files {
                    Ok(files) => {
                        for path in files {
                            let is_dir = path.is_dir();

                            if !is_dir {
                                // Do not show system files.
                                if !path.is_file() {
                                    continue;
                                }

                                // Filter.
                                if let Some(filter) = &self.filter {
                                    if !filter(path) {
                                        continue;
                                    }
                                } else if self.dialog_type == DialogType::SelectFolder {
                                    continue;
                                }
                            }

                            let filename = get_file_name(path);

                            #[cfg(unix)]
                            if !self.show_hidden && filename.starts_with('.') {
                                continue;
                            }

                            ui.with_layout(ui.layout().with_cross_justify(true), |ui| {
                                let label = match is_dir {
                                    true => "🗀 ",
                                    false => "🗋 ",
                                }
                                .to_string()
                                    + filename;

                                let is_selected = Some(path) == self.selected_file.as_ref();
                                let selectable_label = ui.selectable_label(is_selected, label);
                                if selectable_label.clicked() {
                                    command = Some(Command::Select(path.clone()));
                                }

                                if selectable_label.double_clicked() {
                                    command =
                                        Some(match self.dialog_type == DialogType::SaveFile {
                                            true => match is_dir {
                                                true => Command::OpenSelected,
                                                false => Command::Save(path.clone()),
                                            },
                                            false => Command::Open(path.clone()),
                                        });
                                }
                            });
                        }
                    }
                    Err(e) => {
                        ui.label(e.to_string());
                    }
                }
            });
        });

        if let Some(command) = command {
            match command {
                Command::Select(file) => {
                    self.select(Some(file));
                }
                Command::Folder => {
                    self.selected_file = Some(self.get_folder().to_owned());
                    self.confirm();
                }
                Command::Open(path) => {
                    self.select(Some(path));
                    self.open_selected();
                }
                Command::OpenSelected => {
                    self.open_selected();
                }
                Command::Save(file) => {
                    self.selected_file = Some(file);
                    self.confirm();
                }
                Command::Cancel => {
                    self.state = State::Cancelled;
                }
                Command::Refresh => {
                    self.refresh();
                }
                Command::UpDirectory => {
                    if self.path.pop() {
                        self.refresh();
                    }
                }
                Command::CreateDirectory => {
                    let mut path = self.path.clone();
                    let name = match self.filename_edit.is_empty() {
                        true => "New folder",
                        false => &self.filename_edit,
                    };
                    path.push(name);
                    match fs::create_dir(&path) {
                        Ok(_) => {
                            self.refresh();
                            self.select(Some(path));
                            // TODO: scroll to selected?
                        }
                        Err(err) => println!("Error while creating directory: {err}"),
                    }
                }
                Command::Rename(from, to) => match fs::rename(from, &to) {
                    Ok(_) => {
                        self.refresh();
                        self.select(Some(to));
                    }
                    Err(err) => println!("Error while renaming: {err}"),
                },
            };
        }
    }

    fn get_folder(&self) -> &Path {
        if let Some(file) = &self.selected_file {
            if file.is_dir() {
                return file.as_path();
            }
        }

        // No selected file or it's not a folder, so use the current path.
        &self.path
    }
}

#[cfg(windows)]
fn is_drive_root(path: &Path) -> bool {
    if let Some(path) = path.to_str() {
        if let Some(ch) = path.chars().next() {
            return ('A'..='Z').contains(&ch) && &path[1..] == ":\\";
        }
    }
    false
}

fn get_file_name(path: &Path) -> &str {
    match path.is_dir() {
        #[cfg(windows)]
        true if is_drive_root(path) => path.to_str().unwrap_or_default(),
        _ => path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default(),
    }
}

#[cfg(windows)]
extern "C" {
    pub fn GetLogicalDrives() -> u32;
}

fn read_folder(path: &Path) -> Result<Vec<PathBuf>, Error> {
    #[cfg(windows)]
    let drives = {
        let mut drives = unsafe { GetLogicalDrives() };
        let mut letter = b'A';
        let mut drive_names = Vec::new();
        while drives > 0 {
            if drives & 1 != 0 {
                drive_names.push(format!("{}:\\", letter as char).into());
            }
            drives >>= 1;
            letter += 1;
        }
        drive_names
    };

    match fs::read_dir(path) {
        Ok(paths) => {
            let mut result: Vec<PathBuf> = paths
                .filter_map(|result| result.ok())
                .map(|entry| entry.path())
                .collect();
            result.sort_by(|a, b| {
                let da = a.is_dir();
                let db = b.is_dir();
                match da == db {
                    true => a.file_name().cmp(&b.file_name()),
                    false => db.cmp(&da),
                }
            });

            #[cfg(windows)]
            let result = {
                let mut items = drives;
                items.reserve(result.len());
                items.append(&mut result);
                items
            };

            Ok(result)
        }
        Err(e) => Err(e),
    }
}