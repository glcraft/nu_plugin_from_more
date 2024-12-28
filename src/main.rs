mod error;
mod plugin;

use nu_plugin::{serve_plugin, MsgPackSerializer};

fn main() {
    serve_plugin(&plugin::FromAdvPlugin, MsgPackSerializer)
}
