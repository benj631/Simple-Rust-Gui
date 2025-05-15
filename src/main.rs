extern crate native_windows_gui as nwg;
use nwg::NativeUi;
use std::rc::Rc;
use std::cell::RefCell;

// use nwg::ListViewColumn;
// use nwg::ListViewColumnFlags;

use std::fs;
use std::path::Path;

// #[derive(Default)]
pub struct BasicApp {
    window: nwg::Window,
    layout: nwg::GridLayout,
    path_input: nwg::TextInput,
    load_button: nwg::Button,
    file_list: nwg::ListView,
    context_menu: Rc<nwg::Menu>,
    info_item: Rc<nwg::MenuItem>,   
}

impl Default for BasicApp {
    fn default() -> Self {
        Self {
            window: Default::default(),
            layout: Default::default(),
            path_input: Default::default(),
            load_button: Default::default(),
            file_list: Default::default(),
            context_menu: Rc::new(nwg::Menu::default()),
            info_item: Rc::new(nwg::MenuItem::default()),
        }
    }
}

impl nwg::NativeUi<()> for BasicApp {
    fn build_ui(mut data: BasicApp) -> Result<(), nwg::NwgError> {
        use nwg::Event as E;
    
        nwg::Window::builder()
            .size((600, 400))
            // .position((300, 300))
            .title("Simple App")
            .build(&mut data.window)?;

        nwg::GridLayout::builder()
            .parent(&data.window)
            .spacing(5)
            .build(&mut data.layout)?;

        nwg::TextInput::builder()
            .size((220, 25)) // make it wider
            .text("C:\\")
            .parent(&data.window)
            .build(&mut data.path_input)?;

        nwg::Button::builder()
            .text("Load")
            .size((60, 25))
            .parent(&data.window)
            .build(&mut data.load_button)?;

        nwg::ListView::builder()
            .parent(&data.window)
            .focus(true)
            .list_style(nwg::ListViewStyle::Detailed)
            .size((300, 200))
            .build(&mut data.file_list)?;

        data.context_menu = Rc::new(nwg::Menu::default());
        data.info_item = Rc::new(nwg::MenuItem::default());

        nwg::Menu::builder()
            .parent(&data.window)
            .popup(true)
            .build(Rc::get_mut(&mut data.context_menu).unwrap())?;

        nwg::MenuItem::builder()
            .text("Info")
            .parent(&*data.context_menu)
            .build(Rc::get_mut(&mut data.info_item).unwrap())?;

        data.file_list.set_headers_enabled(true);
        data.file_list.insert_column(nwg::InsertListViewColumn {
            index: None,
            text: Some("Name".to_string()),
            width: Some(250),
            fmt: Some(nwg::ListViewColumnFlags::LEFT),
        });
                
        data.layout.add_child(0, 0, &data.path_input);
        data.layout.add_child(0, 1, &data.load_button);

        data.layout.add_child_item(
            nwg::GridLayoutItem::new(&data.file_list, 1, 0, 1, 2) // row 1, col 0, span 1 row, 2 cols
        );

        let window_handle = data.window.handle.clone();
    
        let shared = Rc::new(RefCell::new(data));
        let load_evt = Rc::clone(&shared);
        let load_handle = load_evt.borrow().load_button.handle;
        
        // let handler_evt = Rc::clone(&shared);

        let menu_evt = Rc::clone(&shared);
        //let context_evt = Rc::clone(&shared);

        nwg::bind_event_handler(&window_handle, &window_handle, move |evt, _, handle| {
            
            // Exit app when window closed (Shouldn't have to put this but here we are)
            if evt == E::OnWindowClose {
            nwg::stop_thread_dispatch(); // Exit the event loop
            }
            
            // Load files when we press load
            if evt == E::OnButtonClick && handle == load_handle {
                {
                    let app = load_evt.borrow_mut();
                    let path = app.path_input.text();
                    app.file_list.clear();

                    if let Ok(entries) = fs::read_dir(&path) {
                        for entry in entries.flatten() {
                            let os_name = entry.file_name();
                            let name_string = os_name.to_string_lossy().into_owned();
                            app.file_list.insert_item(nwg::InsertListViewItem {
                                column_index: 0,
                                index: None,
                                text: Some(name_string),
                                image: None,
                            });
                        }
                    } else {
                        nwg::simple_message("Error", "Could not read that directory.");
                    }
                } // mutable borrow ends here
            }

            // Show option when we right click over a list view element
            if evt == E::OnListViewRightClick {
                let (x, y) = nwg::GlobalCursor::position();
                let app = menu_evt.borrow();
                nwg::Menu::popup(&app.context_menu, x, y);
            }

            // Gonna show file info
            if evt == E::OnButtonClick && handle == load_handle {

                let path = {
                    let app = load_evt.borrow();
                    app.path_input.text()
                };

                if let Ok(entries) = fs::read_dir(&path) {

                    let app = load_evt.borrow_mut();
                    app.file_list.clear();

                    for entry in entries.flatten() {
                        let name = entry.file_name().to_string_lossy().into_owned();
                        app.file_list.insert_item(nwg::InsertListViewItem {
                            column_index: 0,
                            index: None,
                            text: Some(name),
                            image: None,
                        });
                    }
                } else {
                    nwg::simple_message("Error", "Could not read that directory.");
                }
            }
            
            // Show file info after selecting info in context menu
            if evt == E::OnMenuItemSelected && handle == menu_evt.borrow().info_item.handle {
                let app = menu_evt.borrow();

                if let Some(index) = app.file_list.selected_item() {
                    if let Some(file_item) = app.file_list.item(index, 0, 256) {
                        let name = file_item.text;
                        let full_path = Path::new(&app.path_input.text()).join(&name);

                        if full_path.is_file() {
                            if let Ok(metadata) = fs::metadata(&full_path) {
                                let size = metadata.len();
                                nwg::simple_message("File Info", &format!("{}\n{} bytes", name, size));
                            } else {
                                nwg::simple_message("Error", "Could not read file metadata");
                            }
                        } else {
                            nwg::simple_message("Info", "Selected item is not a file");
                        }
                    } else {
                        nwg::simple_message("Error", "Could not get selected item");
                    }
                } else {
                    nwg::simple_message("Info", "No file selected");
                }
            }
        });
    
        Ok(())
    }
}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").unwrap();

    BasicApp::build_ui(Default::default()).expect("Failed to build UI");

    nwg::dispatch_thread_events();
}
