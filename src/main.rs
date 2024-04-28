use notify_rust::{Notification, Timeout};
use nu_plugin::{self, serve_plugin, EvaluatedCall, MsgPackSerializer};
use nu_plugin::{EngineInterface, Plugin, PluginCommand, SimplePluginCommand};
use nu_protocol::{Category, LabeledError, Signature, SyntaxShape, Value};
use std::time::Duration;

struct DesktopNotificationPlugin;

impl Plugin for DesktopNotificationPlugin {
    fn commands(&self) -> Vec<Box<dyn PluginCommand<Plugin = Self>>> {
        vec![Box::new(DesktopNotification)]
    }
}

struct DesktopNotification;

impl SimplePluginCommand for DesktopNotification {
    type Plugin = DesktopNotificationPlugin;

    fn name(&self) -> &str {
        "notify"
    }

    fn usage(&self) -> &str {
        "sends notification with given parameters"
    }

    fn signature(&self) -> Signature {
        Signature::build(PluginCommand::name(self))
            .named(
                "summary",
                SyntaxShape::String,
                "summary of the notification",
                Some('s'),
            )
            .named(
                "body",
                SyntaxShape::String,
                "body of the notification",
                Some('t'),
            )
            .named(
                "subtitle",
                SyntaxShape::String,
                "subtitle of the notification [macOS only]",
                None,
            )
            .named(
                "app-name",
                SyntaxShape::String,
                "app name of the notification",
                Some('a'),
            )
            .named(
                "icon",
                SyntaxShape::Filepath,
                "path to the icon of the notification",
                Some('i'),
            )
            .named(
                "timeout",
                SyntaxShape::Duration,
                "duration of the notification [XDG Desktops only] (defaults to system default)",
                None,
            )
            
            .usage(PluginCommand::usage(self))
            .category(Category::Experimental)
    }

    fn run(
        &self,
        _plugin: &DesktopNotificationPlugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        input: &Value,
    ) -> Result<Value, LabeledError> {
        let mut notification = Notification::new();
        if let Some(summary) = load_string(call, "summary") {
            notification.summary(&summary);
        }
        if let Some(body) = load_string(call, "body") {
            notification.body(&body);
        }
        if let Some(subtitle) = load_string(call, "subtitle") {
            notification.subtitle(&subtitle);
        }
        if let Some(app_name) = load_string(call, "app-name") {
            notification.appname(&app_name);
        }

        if let Some(icon) = load_string(call, "icon") {
            notification.icon(&icon);
        } else {
            notification.auto_icon();
        }

        if let Some(duration_value) = call.get_flag_value("timeout") {
            match duration_value.as_duration() {
                Ok(timeout) => {
                    if let Ok(nanos) = timeout.try_into() {
                        let duration = Timeout::from(Duration::from_nanos(nanos));
                        notification.timeout(duration);
                    }
                }
                Err(_) => {}
            }
        }
        
        match notification.show() {
            Ok(_) => Ok(input.clone()),
            Err(err) => {
                return Err(LabeledError::new(err.to_string()).with_label(
                    "Notification Exception".to_string(),
                    call.head))
            }
        }
        
        /* let span = input.span();
        match input {
            Value::String { val, .. } => Ok(Value::int(val.len() as i64, span)),
            _ => Err(
                LabeledError::new("Expected String input from pipeline").with_label(
                    format!("requires string input; got {}", input.get_type()),
                    call.head,
                ),
            ),
        } */
    }
}

fn main() {
    serve_plugin(&DesktopNotificationPlugin, MsgPackSerializer)
}

pub fn load_string(call: &EvaluatedCall, name: &str) -> Option<String> {
    let value = call.get_flag_value(name);
    match value {
        Some(Value::String { val, .. }) => Some(val),
        _ => None,
    }
}
