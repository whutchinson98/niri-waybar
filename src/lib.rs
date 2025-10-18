use serde::Deserialize;
use std::collections::HashMap;
use std::thread;
use waybar_cffi::gtk::gdk::EventMask;
use waybar_cffi::gtk::{Box as GtkBox, EventBox, Label, Orientation};
use waybar_cffi::{
    InitInfo, Module,
    gtk::{glib, prelude::*},
    waybar_module,
};

struct NiriWaybar;

impl Module for NiriWaybar {
    type Config = Config;

    fn init(info: &InitInfo, config: Config) -> Self {
        let format_icons = config.format_icons.unwrap_or_default();
        let root = info.get_root_widget();

        let container = GtkBox::new(Orientation::Horizontal, 0);
        root.add(&container);
        container.show();

        let mut socket =
            niri_ipc::socket::Socket::connect().expect("failed to connect to niri-ipc");

        // Get initial workspaces
        let workspaces = socket
            .send(niri_ipc::Request::Workspaces)
            .expect("failed to send request");

        let initial_workspaces = match workspaces {
            Ok(niri_ipc::Response::Workspaces(workspaces)) => workspaces,
            Ok(_) => unreachable!(),
            Err(e) => {
                println!("Error: {e}");
                vec![]
            }
        };

        update_workspace_labels(&container, &initial_workspaces, &format_icons);

        let (sender, receiver) = async_channel::unbounded::<Vec<niri_ipc::Workspace>>();

        let container_clone = container.clone();
        glib::MainContext::default().spawn_local(async move {
            while let Ok(workspaces) = receiver.recv().await {
                update_workspace_labels(&container_clone, &workspaces, &format_icons);
            }
        });

        thread::spawn(move || {
            if let Err(e) = niri_event_stream(sender) {
                eprintln!("Event stream error: {e}");
            }
        });

        NiriWaybar
    }

    /// Called when the module should be updated.
    fn update(&mut self) {}

    /// Called when the module should be refreshed in response to a signal.
    fn refresh(&mut self, _signal: i32) {}

    /// Called when an action is called on the module.
    fn do_action(&mut self, _action: &str) {}
}

fn update_workspace_labels(container: &GtkBox, workspaces: &[niri_ipc::Workspace], format_icons: &HashMap<String, String>) {
    // Remove all existing labels
    for child in container.children() {
        container.remove(&child);
    }

    let mut sorted_workspaces: Vec<_> = workspaces.iter().collect();
    sorted_workspaces.sort_by_key(|w| w.id);

    for workspace in sorted_workspaces {
        let name = workspace.name.clone().unwrap_or(workspace.id.to_string());
        let label = Label::new(Some(&format_icons.get(&name).unwrap_or(&name)));

        let style_context = label.style_context();
        style_context.add_class("niri_workspace");

        if workspace.is_active {
            style_context.add_class("focused");
        }

        // Wrap label in EventBox to handle events
        let event_box = EventBox::new();
        event_box.add(&label);
        event_box.add_events(EventMask::BUTTON_PRESS_MASK | EventMask::BUTTON_RELEASE_MASK);

        let workspace_id = workspace.id;
        event_box.connect_button_press_event(move |_event_box, event| {
            if event.button() == 1
                && let Err(e) = goto_workspace(workspace_id)
            {
                eprintln!("Failed to switch to workspace {}: {}", workspace_id, e);
            }

            glib::Propagation::Proceed
        });

        container.add(&event_box);
        event_box.show_all();
    }
}

fn niri_event_stream(
    sender: async_channel::Sender<Vec<niri_ipc::Workspace>>,
) -> anyhow::Result<()> {
    let mut socket = niri_ipc::socket::Socket::connect()?;
    let reply = socket.send(niri_ipc::Request::EventStream)?;

    if matches!(reply, Ok(niri_ipc::Response::Handled)) {
        let mut read_event = socket.read_events();

        while let Ok(event) = read_event() {
            match event {
                niri_ipc::Event::WorkspacesChanged { .. }
                | niri_ipc::Event::WorkspaceActivated { .. }
                | niri_ipc::Event::WorkspaceActiveWindowChanged { .. } => {
                    // Fetch updated workspaces
                    if let Ok(mut new_socket) = niri_ipc::socket::Socket::connect()
                        && let Ok(Ok(niri_ipc::Response::Workspaces(ws))) =
                            new_socket.send(niri_ipc::Request::Workspaces)
                    {
                        // Send to main thread via channel (ignore errors if receiver dropped)
                        let _ = sender.send_blocking(ws);
                    }
                }
                _ => {} // Ignore other events
            }
        }
    }

    Ok(())
}

fn goto_workspace(workspace_id: u64) -> anyhow::Result<()> {
    let mut socket = niri_ipc::socket::Socket::connect()?;
    let _ = socket
        .send(niri_ipc::Request::Action(
            niri_ipc::Action::FocusWorkspace {
                reference: niri_ipc::WorkspaceReferenceArg::Id(workspace_id),
            },
        ))
        .inspect_err(|e| eprintln!("Error: {e}"))?;

    Ok(())
}

waybar_module!(NiriWaybar);

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "format-icons")]
    format_icons: Option<HashMap<String, String>>,
}
