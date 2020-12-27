use std::path::PathBuf;

use irc_rust::Message;
use mlua::{prelude::*, Function, Result, Table, UserData};
use regex::internal::Exec;
use tokio::sync::{broadcast, mpsc};

use crate::{
    client::Client,
    connection::{TaggedCommand, TaggedEvent},
};

pub enum ExecutorCommand {
    ResetGlobals,
    ExecuteFile(PathBuf),
    HandleEvent(Client, Message),
}

pub struct PluginManager {
    executor: PluginExecutor,
}

impl PluginManager {
    pub fn new() -> Self {
        let (exec_ctrl_tx, exec_ctrl_rx) = mpsc::channel::<ExecutorCommand>(32);

        let executor = PluginExecutor::new(exec_ctrl_rx);

        Self { executor }
    }
}

pub struct PluginExecutor {
    lua: Lua,
    ctrl_rx: mpsc::Receiver<ExecutorCommand>,
}

impl PluginExecutor {
    pub fn new(ctrl_rx: mpsc::Receiver<ExecutorCommand>) -> Result<Self> {
        let lua = Lua::new();

        let manager = Self { lua, ctrl_rx };

        Ok(manager)
    }

    pub async fn handle_event(
        &self,
        client: &Client,
        event: &TaggedEvent,
        command_tx: broadcast::Sender<TaggedCommand>,
    ) -> Result<()> {
        Ok(())
    }

    fn setup(&self) -> Result<()> {
        self.setup_plugin_registry()?;
        self.setup_plugin_methods()?;

        Ok(())
    }

    fn setup_plugin_registry(&self) -> Result<()> {
        let globals = self.lua.globals();

        let plugins = self.lua.create_table()?;
        let plugin_handlers = self.lua.create_table()?;

        let command_handlers = self.lua.create_table()?;
        plugin_handlers.set("commands", command_handlers)?;

        let regex_handlers = self.lua.create_table()?;
        plugin_handlers.set("regex", regex_handlers)?;

        let event_handlers = self.lua.create_table()?;
        plugin_handlers.set("events", event_handlers)?;

        let filter_handlers = self.lua.create_table()?;
        plugin_handlers.set("filters", filter_handlers)?;

        plugins.set("_handlers", plugin_handlers)?;

        globals.set("plugins", plugins)?;

        Ok(())
    }

    fn setup_plugin_methods(&self) -> Result<()> {
        let globals = self.lua.globals();
        let plugins: Table = globals.get("plugins")?;

        let register_command_handler = self.lua.create_function_mut(
            |lua, (name, triggers, callback): (String, Vec<String>, Function)| {
                let globals = lua.globals();
                let plugins: Table = globals.get("plugins")?;
                let handlers: Table = plugins.get("_handlers")?;
                let command_handlers: Table = handlers.get("commands")?;

                let plugin: Table = lua.create_table()?;
                plugin.set("name", name.clone())?;
                plugin.set("triggers", triggers)?;
                plugin.set("callback", callback)?;

                command_handlers.set(name.clone(), plugin)?;

                Ok(())
            },
        )?;
        plugins.set("register_command", register_command_handler)?;

        let register_regex_handler = self.lua.create_function_mut(
            |lua, (name, regex, callback): (String, String, Function)| {
                let globals = lua.globals();
                let plugins: Table = globals.get("plugins")?;
                let handlers: Table = plugins.get("_handlers")?;
                let regex_handlers: Table = handlers.get("regex")?;

                let plugin: Table = lua.create_table()?;
                plugin.set("name", name.clone())?;
                plugin.set("regex", regex)?;
                plugin.set("callback", callback)?;

                regex_handlers.set(name.clone(), plugin)?;

                Ok(())
            },
        )?;
        plugins.set("register_regex", register_regex_handler)?;

        let register_event_handler = self.lua.create_function_mut(
            |lua, (name, event, callback): (String, String, Function)| {
                let globals = lua.globals();
                let plugins: Table = globals.get("plugins")?;
                let handlers: Table = plugins.get("_handlers")?;
                let event_handlers: Table = handlers.get("event")?;

                let plugin: Table = lua.create_table()?;
                plugin.set("name", name.clone())?;
                plugin.set("event", event)?;
                plugin.set("callback", callback)?;

                event_handlers.set(name.clone(), plugin)?;

                Ok(())
            },
        )?;
        plugins.set("register_event", register_event_handler)?;

        let register_filter_handler =
            self.lua
                .create_function_mut(|lua, (name, callback): (String, Function)| {
                    let globals = lua.globals();
                    let plugins: Table = globals.get("plugins")?;
                    let handlers: Table = plugins.get("_handlers")?;
                    let filter_handlers: Table = handlers.get("filter")?;

                    let plugin: Table = lua.create_table()?;
                    plugin.set("name", name.clone())?;
                    plugin.set("callback", callback)?;

                    filter_handlers.set(name.clone(), plugin)?;

                    Ok(())
                })?;
        plugins.set("register_filter", register_filter_handler)?;

        Ok(())
    }
}
