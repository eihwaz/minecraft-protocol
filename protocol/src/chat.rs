//! Minecraft chat are represented as json object. It's used in different packets.
//! Information about format can be found at https://wiki.vg/Chat.
//!
//! # Example
//!
//! ## Serialize
//!
//! ```
//! use minecraft_protocol::chat::{Payload, Color, MessageBuilder};
//!
//! let message = MessageBuilder::builder(Payload::text("Hello"))
//!    .color(Color::Yellow)
//!    .bold(true)
//!    .then(Payload::text("world"))
//!    .color(Color::Green)
//!    .bold(true)
//!    .italic(true)
//!    .then(Payload::text("!"))
//!    .color(Color::Blue)
//!    .build();
//!
//! println!("{}", message.to_json().unwrap());
//! ```
//!
//! ## Deserialize
//!
//! ```
//! use minecraft_protocol::chat::{MessageBuilder, Color, Payload, Message};
//!
//! let json = r#"
//! {
//!   "bold":true,
//!   "color":"yellow",
//!   "text":"Hello",
//!   "extra":[
//!      {
//!         "bold":true,
//!         "italic":true,
//!         "color":"green",
//!         "text":"world"
//!      },
//!      {
//!         "color":"blue",
//!         "text":"!"
//!      }
//!   ]
//! }
//! "#;
//!
//! let expected_message = MessageBuilder::builder(Payload::text("Hello"))
//!    .color(Color::Yellow)
//!    .bold(true)
//!    .then(Payload::text("world"))
//!    .color(Color::Green)
//!    .bold(true)
//!    .italic(true)
//!    .then(Payload::text("!"))
//!    .color(Color::Blue)
//!    .build();
//!
//! assert_eq!(expected_message, Message::from_json(json).unwrap());
//! ```

use crate::impl_json_encoder_decoder;
use serde::{
    de::{self, Visitor},
    Deserialize, Serialize,
};
use serde_json::Error;

#[derive(Debug, Eq, PartialEq)]
pub enum Color {
    Black,
    DarkBlue,
    DarkGreen,
    DarkAqua,
    DarkRed,
    DarkPurple,
    Gold,
    Gray,
    DarkGray,
    Blue,
    Green,
    Aqua,
    Red,
    LightPurple,
    Yellow,
    White,
    /// A hex color string
    ///
    /// Support for this was added in 1.16.
    ///
    /// # Examples
    ///
    /// ```
    /// use minecraft_protocol::chat::Color;
    ///
    /// let color = Color::Hex("#f98aff".into());
    /// ```
    Hex(String),
}

impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(match self {
            Color::Black => "black",
            Color::DarkBlue => "dark_blue",
            Color::DarkGreen => "dark_green",
            Color::DarkAqua => "dark_aqua",
            Color::DarkRed => "dark_red",
            Color::DarkPurple => "dark_purple",
            Color::Gold => "gold",
            Color::Gray => "gray",
            Color::DarkGray => "dark_gray",
            Color::Blue => "blue",
            Color::Green => "green",
            Color::Aqua => "aqua",
            Color::Red => "red",
            Color::LightPurple => "light_purple",
            Color::Yellow => "yellow",
            Color::White => "white",
            Color::Hex(val) => val,
        })
    }
}

struct ColorVisitor;

impl<'de> Visitor<'de> for ColorVisitor {
    type Value = Color;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a hex color string or a pre-defined color name")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(if v.starts_with("#") {
            Color::Hex(v.into())
        } else {
            match v {
                "black" => Color::Black,
                "dark_blue" => Color::DarkBlue,
                "dark_green" => Color::DarkGreen,
                "dark_aqua" => Color::DarkAqua,
                "dark_red" => Color::DarkRed,
                "dark_purple" => Color::DarkPurple,
                "gold" => Color::Gold,
                "gray" => Color::Gray,
                "dark_gray" => Color::DarkGray,
                "blue" => Color::Blue,
                "green" => Color::Green,
                "aqua" => Color::Aqua,
                "red" => Color::Red,
                "light_purple" => Color::LightPurple,
                "yellow" => Color::Yellow,
                "white" => Color::White,
                _ => return Err(E::invalid_value(de::Unexpected::Str(v), &self)),
            }
        })
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(ColorVisitor)
    }
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClickAction {
    OpenUrl,
    RunCommand,
    SuggestCommand,
    ChangePage,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClickEvent {
    pub action: ClickAction,
    pub value: String,
}

impl ClickEvent {
    pub fn new(action: ClickAction, value: &str) -> Self {
        ClickEvent {
            action,
            value: value.to_owned(),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HoverAction {
    ShowText,
    ShowItem,
    ShowEntity,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HoverEvent {
    pub action: HoverAction,
    pub value: String,
}

impl HoverEvent {
    pub fn new(action: HoverAction, value: &str) -> Self {
        HoverEvent {
            action,
            value: value.to_owned(),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Payload {
    Text {
        text: String,
    },
    Translation {
        translate: String,
        with: Vec<Message>,
    },
    Keybind {
        keybind: String,
    },
    Score {
        name: String,
        objective: String,
        value: String,
    },
    Selector {
        selector: String,
    },
}

impl Payload {
    pub fn text(text: &str) -> Self {
        Payload::Text {
            text: text.to_owned(),
        }
    }

    pub fn translation(translate: &str, with: Vec<Message>) -> Self {
        Payload::Translation {
            translate: translate.to_owned(),
            with,
        }
    }

    pub fn keybind(keybind: &str) -> Self {
        Payload::Keybind {
            keybind: keybind.to_owned(),
        }
    }

    pub fn score(name: &str, objective: &str, value: &str) -> Self {
        Payload::Score {
            name: name.to_owned(),
            objective: objective.to_owned(),
            value: value.to_owned(),
        }
    }

    pub fn selector(selector: &str) -> Self {
        Payload::Selector {
            selector: selector.to_owned(),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bold: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub italic: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub underlined: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strikethrough: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub obfuscated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insertion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub click_event: Option<ClickEvent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hover_event: Option<HoverEvent>,
    #[serde(flatten)]
    pub payload: Payload,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub extra: Vec<Message>,
}

impl Message {
    pub fn new(payload: Payload) -> Self {
        Message {
            bold: None,
            italic: None,
            underlined: None,
            strikethrough: None,
            obfuscated: None,
            color: None,
            insertion: None,
            click_event: None,
            hover_event: None,
            payload,
            extra: vec![],
        }
    }

    pub fn from_json(json: &str) -> Result<Self, Error> {
        serde_json::from_str(json)
    }

    pub fn to_json(&self) -> Result<String, Error> {
        serde_json::to_string(&self)
    }
}

impl_json_encoder_decoder!(Message);

pub struct MessageBuilder {
    current: Message,
    root: Option<Message>,
}

macro_rules! create_builder_style_method (
    ($style: ident) => (
        pub fn $style(mut self, value: bool) -> Self {
            self.current.$style = Some(value);
            self
        }
    );
);

macro_rules! create_builder_click_event_method (
    ($method_name: ident, $event: ident) => (
        pub fn $method_name(mut self, value: &str) -> Self {
            let click_event = ClickEvent::new(ClickAction::$event, value);
            self.current.click_event = Some(click_event);
            self
        }
    );
);

macro_rules! create_builder_hover_event_method (
    ($method_name: ident, $event: ident) => (
        pub fn $method_name(mut self, value: &str) -> Self {
            let hover_event = HoverEvent::new(HoverAction::$event, value);
            self.current.hover_event = Some(hover_event);
            self
        }
    );
);

impl MessageBuilder {
    pub fn builder(payload: Payload) -> Self {
        let current = Message::new(payload);

        MessageBuilder {
            current,
            root: None,
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.current.color = Some(color);
        self
    }

    pub fn insertion(mut self, insertion: &str) -> Self {
        self.current.insertion = Some(insertion.to_owned());
        self
    }

    create_builder_style_method!(bold);
    create_builder_style_method!(italic);
    create_builder_style_method!(underlined);
    create_builder_style_method!(strikethrough);
    create_builder_style_method!(obfuscated);

    create_builder_click_event_method!(click_open_url, OpenUrl);
    create_builder_click_event_method!(click_run_command, RunCommand);
    create_builder_click_event_method!(click_suggest_command, SuggestCommand);
    create_builder_click_event_method!(click_change_page, ChangePage);

    create_builder_hover_event_method!(hover_show_text, ShowText);
    create_builder_hover_event_method!(hover_show_item, ShowItem);
    create_builder_hover_event_method!(hover_show_entity, ShowEntity);

    pub fn then(mut self, payload: Payload) -> Self {
        match self.root.as_mut() {
            Some(root) => {
                root.extra.push(self.current);
            }
            None => {
                self.root = Some(self.current);
            }
        }

        self.current = Message::new(payload);
        self
    }

    pub fn build(self) -> Message {
        match self.root {
            Some(mut root) => {
                root.extra.push(self.current);
                root
            }
            None => self.current,
        }
    }
}

#[test]
fn test_serialize_text_hello_world() {
    let message = MessageBuilder::builder(Payload::text("Hello"))
        .color(Color::Yellow)
        .bold(true)
        .then(Payload::text("world"))
        .color(Color::Green)
        .bold(true)
        .italic(true)
        .then(Payload::text("!"))
        .color(Color::Blue)
        .build();

    assert_eq!(
        message.to_json().unwrap(),
        include_str!("../test/chat/text_hello_world.json")
    );
}

#[test]
fn test_deserialize_text_hello_world() {
    let expected_message = MessageBuilder::builder(Payload::text("Hello"))
        .color(Color::Yellow)
        .bold(true)
        .then(Payload::text("world"))
        .color(Color::Green)
        .bold(true)
        .italic(true)
        .then(Payload::text("!"))
        .color(Color::Blue)
        .build();

    assert_eq!(
        expected_message,
        Message::from_json(include_str!("../test/chat/text_hello_world.json")).unwrap()
    );
}

#[test]
fn test_serialize_translate_opped_steve() {
    let with = vec![Message::new(Payload::text("Steve"))];
    let message = Message::new(Payload::translation("Opped %s", with));

    assert_eq!(
        message.to_json().unwrap(),
        include_str!("../test/chat/translate_opped_steve.json")
    );
}

#[test]
fn test_deserialize_translate_opped_steve() {
    let with = vec![Message::new(Payload::text("Steve"))];
    let expected_message = Message::new(Payload::translation("Opped %s", with));

    assert_eq!(
        expected_message,
        Message::from_json(include_str!("../test/chat/translate_opped_steve.json")).unwrap()
    );
}

#[test]
fn test_serialize_keybind_jump() {
    let message = MessageBuilder::builder(Payload::text("Press \""))
        .color(Color::Yellow)
        .bold(true)
        .then(Payload::keybind("key.jump"))
        .color(Color::Blue)
        .bold(false)
        .underlined(true)
        .then(Payload::text("\" to jump!"))
        .build();

    assert_eq!(
        message.to_json().unwrap(),
        include_str!("../test/chat/keybind_jump.json")
    );
}

#[test]
fn test_deserialize_keybind_jump() {
    let expected_message = MessageBuilder::builder(Payload::text("Press \""))
        .color(Color::Yellow)
        .bold(true)
        .then(Payload::keybind("key.jump"))
        .color(Color::Blue)
        .bold(false)
        .underlined(true)
        .then(Payload::text("\" to jump!"))
        .build();

    assert_eq!(
        expected_message,
        Message::from_json(include_str!("../test/chat/keybind_jump.json")).unwrap()
    );
}

#[test]
fn test_serialize_click_open_url() {
    let message = MessageBuilder::builder(Payload::text("click me"))
        .color(Color::Yellow)
        .bold(true)
        .click_open_url("http://minecraft.net")
        .build();

    assert_eq!(
        message.to_json().unwrap(),
        include_str!("../test/chat/click_open_url.json")
    );
}

#[test]
fn test_deserialize_click_open_url() {
    let expected_message = MessageBuilder::builder(Payload::text("click me"))
        .color(Color::Yellow)
        .bold(true)
        .click_open_url("http://minecraft.net")
        .build();

    assert_eq!(
        expected_message,
        Message::from_json(include_str!("../test/chat/click_open_url.json")).unwrap()
    );
}

#[test]
fn test_serialize_click_run_command() {
    let message = MessageBuilder::builder(Payload::text("click me"))
        .color(Color::LightPurple)
        .italic(true)
        .click_run_command("/help")
        .build();

    assert_eq!(
        message.to_json().unwrap(),
        include_str!("../test/chat/click_run_command.json")
    );
}

#[test]
fn test_deserialize_click_run_command() {
    let expected_message = MessageBuilder::builder(Payload::text("click me"))
        .color(Color::LightPurple)
        .italic(true)
        .click_run_command("/help")
        .build();

    assert_eq!(
        expected_message,
        Message::from_json(include_str!("../test/chat/click_run_command.json")).unwrap()
    );
}

#[test]
fn test_serialize_click_suggest_command() {
    let message = MessageBuilder::builder(Payload::text("click me"))
        .color(Color::Blue)
        .obfuscated(true)
        .click_suggest_command("/help")
        .build();

    assert_eq!(
        message.to_json().unwrap(),
        include_str!("../test/chat/click_suggest_command.json")
    );
}

#[test]
fn test_deserialize_click_suggest_command() {
    let expected_message = MessageBuilder::builder(Payload::text("click me"))
        .color(Color::Blue)
        .obfuscated(true)
        .click_suggest_command("/help")
        .build();

    assert_eq!(
        expected_message,
        Message::from_json(include_str!("../test/chat/click_suggest_command.json")).unwrap()
    );
}

#[test]
fn test_serialize_click_change_page() {
    let message = MessageBuilder::builder(Payload::text("click me"))
        .color(Color::DarkGray)
        .underlined(true)
        .click_change_page("2")
        .build();

    assert_eq!(
        message.to_json().unwrap(),
        include_str!("../test/chat/click_change_page.json")
    );
}

#[test]
fn test_deserialize_click_change_page() {
    let expected_message = MessageBuilder::builder(Payload::text("click me"))
        .color(Color::DarkGray)
        .underlined(true)
        .click_change_page("2")
        .build();

    assert_eq!(
        expected_message,
        Message::from_json(include_str!("../test/chat/click_change_page.json")).unwrap()
    );
}

#[test]
fn test_serialize_hover_show_text() {
    let message = MessageBuilder::builder(Payload::text("hover at me"))
        .color(Color::DarkPurple)
        .bold(true)
        .hover_show_text("Herobrine behind you!")
        .build();

    assert_eq!(
        message.to_json().unwrap(),
        include_str!("../test/chat/hover_show_text.json")
    );
}

#[test]
fn test_deserialize_hover_show_text() {
    let expected_message = MessageBuilder::builder(Payload::text("hover at me"))
        .color(Color::DarkPurple)
        .bold(true)
        .hover_show_text("Herobrine behind you!")
        .build();

    assert_eq!(
        expected_message,
        Message::from_json(include_str!("../test/chat/hover_show_text.json")).unwrap()
    );
}

#[test]
fn test_serialize_hover_show_item() {
    let message = MessageBuilder::builder(Payload::text("hover at me"))
        .color(Color::DarkRed)
        .italic(true)
        .hover_show_item("{\"id\":\"stone\",\"Count\":1}")
        .build();

    assert_eq!(
        message.to_json().unwrap(),
        include_str!("../test/chat/hover_show_item.json")
    );
}

#[test]
fn test_deserialize_hover_show_item() {
    let expected_message = MessageBuilder::builder(Payload::text("hover at me"))
        .color(Color::DarkRed)
        .italic(true)
        .hover_show_item("{\"id\":\"stone\",\"Count\":1}")
        .build();

    assert_eq!(
        expected_message,
        Message::from_json(include_str!("../test/chat/hover_show_item.json")).unwrap()
    );
}

#[test]
fn test_serialize_hover_show_entity() {
    let message = MessageBuilder::builder(Payload::text("hover at me"))
        .color(Color::DarkAqua)
        .obfuscated(true)
        .hover_show_entity("{\"id\":\"7e4a61cc-83fa-4441-a299-bf69786e610a\",\"type\":\"minecraft:zombie\",\"name\":\"Zombie}\"")
        .build();

    assert_eq!(
        message.to_json().unwrap(),
        include_str!("../test/chat/hover_show_entity.json")
    );
}

#[test]
fn test_deserialize_hover_show_entity() {
    let expected_message = MessageBuilder::builder(Payload::text("hover at me"))
        .color(Color::DarkAqua)
        .obfuscated(true)
        .hover_show_entity("{\"id\":\"7e4a61cc-83fa-4441-a299-bf69786e610a\",\"type\":\"minecraft:zombie\",\"name\":\"Zombie}\"")
        .build();

    assert_eq!(
        expected_message,
        Message::from_json(include_str!("../test/chat/hover_show_entity.json")).unwrap()
    );
}

#[test]
fn test_serialize_hex_color() {
    let message = MessageBuilder::builder(Payload::text("Hello"))
        .color(Color::Hex("#ffffff".into()))
        .build();

    assert_eq!(
        message.to_json().unwrap(),
        include_str!("../test/chat/hex_color.json")
    );
}

#[test]
fn test_deserialize_hex_color() {
    let expected_message = MessageBuilder::builder(Payload::text("Hello"))
        .color(Color::Hex("#ffffff".into()))
        .build();

    assert_eq!(
        Message::from_json(include_str!("../test/chat/hex_color.json")).unwrap(),
        expected_message
    );
}
