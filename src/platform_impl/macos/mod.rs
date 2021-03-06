mod accelerator;
mod menu_item;

use crate::accelerator::{RawMods, SysMods};
use crate::NativeMenuItem;
use crate::{accelerator::Accelerator, platform_impl::platform_impl::menu_item::make_menu_item};
use cocoa::{
    appkit::{NSApp, NSApplication, NSMenu, NSMenuItem},
    base::{id, nil, selector, NO},
    foundation::{NSAutoreleasePool, NSString},
};
use keyboard_types::Code;
use objc::{class, msg_send, sel, sel_impl};

use self::accelerator::remove_mnemonic;
pub use menu_item::CheckMenuItem;
pub use menu_item::MenuItem;

#[derive(Debug, Clone)]
pub struct Menu(id);

impl Menu {
    pub fn new() -> Self {
        unsafe {
            let ns_menu = NSMenu::alloc(nil).autorelease();
            let () = msg_send![ns_menu, setAutoenablesItems: NO];
            Self(ns_menu)
        }
    }

    pub fn add_submenu<S: AsRef<str>>(&mut self, label: S, enabled: bool) -> Submenu {
        let menu = Menu::new();
        let menu_item = MenuItem::new("", enabled, sel!(fireMenubarAction:), None);

        unsafe {
            menu_item.ns_menu_item.setSubmenu_(menu.0);
            self.0.addItem_(menu_item.ns_menu_item);
        }

        let mut sub_menu = Submenu { menu, menu_item };
        sub_menu.set_label(label);

        sub_menu
    }

    pub fn init_for_nsapp(&self) {
        unsafe {
            NSApp().setMainMenu_(self.0);
        }
    }

    pub fn remove_for_nsapp(&self) {
        unsafe {
            NSApp().setMainMenu_(std::ptr::null_mut());
        }
    }
}

#[derive(Debug, Clone)]
pub struct Submenu {
    pub(crate) menu: Menu,
    pub(crate) menu_item: MenuItem,
}

impl Submenu {
    pub fn label(&self) -> String {
        self.menu_item.label()
    }

    pub fn set_label<S: AsRef<str>>(&mut self, label: S) {
        let label = remove_mnemonic(label);
        self.menu_item.set_label(&label);
        unsafe {
            let menu_title = NSString::alloc(nil).init_str(&label);
            let () = msg_send![self.menu.0, setTitle: menu_title];
        }
    }

    pub fn enabled(&self) -> bool {
        self.menu_item.enabled()
    }

    pub fn set_enabled(&mut self, _enabled: bool) {
        self.menu_item.set_enabled(_enabled)
    }

    pub fn add_submenu<S: AsRef<str>>(&mut self, label: S, enabled: bool) -> Submenu {
        self.menu.add_submenu(label, enabled)
    }

    pub fn add_item<S: AsRef<str>>(
        &mut self,
        label: S,
        enabled: bool,
        accelerator: Option<Accelerator>,
    ) -> MenuItem {
        let item = MenuItem::new(label, enabled, sel!(fireMenubarAction:), accelerator);
        unsafe {
            self.menu.0.addItem_(item.ns_menu_item);
        }
        item
    }

    pub fn add_native_item(&mut self, item: NativeMenuItem) {
        let (_, native_menu_item) = match item {
            NativeMenuItem::Separator => unsafe { (0, NSMenuItem::separatorItem(nil)) },
            NativeMenuItem::About(app_name, _) => {
                let title = format!("About {}", app_name);
                make_menu_item(
                    title.as_str(),
                    selector("orderFrontStandardAboutPanel:"),
                    None,
                )
            }
            NativeMenuItem::CloseWindow => make_menu_item(
                "Close Window",
                selector("performClose:"),
                Some(Accelerator::new(SysMods::Cmd, Code::KeyW)),
            ),
            NativeMenuItem::Quit => make_menu_item(
                "Quit",
                selector("terminate:"),
                Some(Accelerator::new(SysMods::Cmd, Code::KeyQ)),
            ),
            NativeMenuItem::Hide => make_menu_item(
                "Hide",
                selector("hide:"),
                Some(Accelerator::new(SysMods::Cmd, Code::KeyH)),
            ),
            NativeMenuItem::HideOthers => make_menu_item(
                "Hide Others",
                selector("hideOtherApplications:"),
                Some(Accelerator::new(RawMods::Alt, Code::KeyH)),
            ),
            NativeMenuItem::ShowAll => {
                make_menu_item("Show All", selector("unhideAllApplications:"), None)
            }
            NativeMenuItem::ToggleFullScreen => make_menu_item(
                "Toggle Full Screen",
                selector("toggleFullScreen:"),
                Some(Accelerator::new(RawMods::Ctrl, Code::KeyF)),
            ),
            NativeMenuItem::Minimize => make_menu_item(
                "Minimize",
                selector("performMiniaturize:"),
                Some(Accelerator::new(SysMods::Cmd, Code::KeyM)),
            ),
            NativeMenuItem::Zoom => make_menu_item("Zoom", selector("performZoom:"), None),
            NativeMenuItem::Copy => make_menu_item(
                "Copy",
                selector("copy:"),
                Some(Accelerator::new(SysMods::Cmd, Code::KeyC)),
            ),
            NativeMenuItem::Cut => make_menu_item(
                "Cut",
                selector("cut:"),
                Some(Accelerator::new(SysMods::Cmd, Code::KeyX)),
            ),
            NativeMenuItem::Paste => make_menu_item(
                "Paste",
                selector("paste:"),
                Some(Accelerator::new(SysMods::Cmd, Code::KeyV)),
            ),
            NativeMenuItem::Undo => make_menu_item(
                "Undo",
                selector("undo:"),
                Some(Accelerator::new(SysMods::Cmd, Code::KeyZ)),
            ),
            NativeMenuItem::Redo => make_menu_item(
                "Redo",
                selector("redo:"),
                Some(Accelerator::new(SysMods::CmdShift, Code::KeyZ)),
            ),
            NativeMenuItem::SelectAll => make_menu_item(
                "Select All",
                selector("selectAll:"),
                Some(Accelerator::new(SysMods::Cmd, Code::KeyA)),
            ),
            NativeMenuItem::Services => unsafe {
                let (_, item) = make_menu_item("Services", sel!(fireMenubarAction:), None);
                let app_class = class!(NSApplication);
                let app: id = msg_send![app_class, sharedApplication];
                let services: id = msg_send![app, servicesMenu];
                let _: () = msg_send![&*item, setSubmenu: services];
                (0, item)
            },
        };
        unsafe {
            self.menu.0.addItem_(native_menu_item);
        }
    }

    pub fn add_check_item<S: AsRef<str>>(
        &mut self,
        label: S,
        enabled: bool,
        checked: bool,
        accelerator: Option<Accelerator>,
    ) -> CheckMenuItem {
        let item = CheckMenuItem::new(
            label,
            enabled,
            checked,
            sel!(fireMenubarAction:),
            accelerator,
        );
        unsafe {
            self.menu.0.addItem_(item.ns_menu_item);
        }
        item
    }
}
