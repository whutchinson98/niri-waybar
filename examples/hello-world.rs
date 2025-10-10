use serde::Deserialize;
use waybar_cffi::{
    InitInfo, Module,
    gtk::{Label, prelude::ContainerExt},
    waybar_module,
};

struct HelloWorld;

impl Module for HelloWorld {
    type Config = Config;

    fn init(info: &InitInfo, config: Config) -> Self {
        let container = info.get_root_widget();

        let mut socket =
            niri_ipc::socket::Socket::connect().expect("failed to connect to niri-ipc");

        let workspaces = socket
            .send(niri_ipc::Request::Workspaces)
            .expect("failed to send request");

        match workspaces {
            Ok(niri_ipc::Response::Workspaces(workspaces)) => {
                println!("{workspaces:?}");
                let label = Label::new(Some(&format_workspaces(&workspaces)));
                container.add(&label);
            }
            Ok(_) => unreachable!(),
            Err(e) => println!("Error: {e}"),
        }

        HelloWorld
    }
}

fn format_workspaces(workspaces: &[niri_ipc::Workspace]) -> String {
    let result = workspaces.iter().map(|w| {
        let name = w.name.clone().unwrap_or(w.id.to_string());
        if w.is_active {
            format!("* {name}")
        } else {
            name
        }
    }).collect::<Vec<_>>().join(" ");

    result
}

waybar_module!(HelloWorld);

#[derive(Deserialize)]
struct Config {
    name: Option<String>,
}
