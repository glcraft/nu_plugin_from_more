mod from_kdl;
use nu_plugin::{Plugin, PluginCommand};

pub struct FromAdvPlugin;

impl Plugin for FromAdvPlugin {
    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").into()
    }

    fn commands(&self) -> Vec<Box<dyn PluginCommand<Plugin = Self>>> {
        vec![Box::new(from_kdl::FromKdl)]
    }
}
