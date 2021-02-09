use crate::backend;
use crate::frontend;

pub trait Mappings {
    fn rename_packet(
        &self,
        unformatted_name: &str,
        name: &str,
        bound: &frontend::Bound,
        protocol: &backend::Protocol,
    ) -> String;

    fn change_field_type(&self, packet_name: &str, field: frontend::Field) -> frontend::Field;
}

pub struct CodeMappings {}

impl CodeMappings {
    pub fn new() -> CodeMappings {
        CodeMappings {}
    }
}

impl Mappings for CodeMappings {
    fn rename_packet(
        &self,
        unformatted_name: &str,
        name: &str,
        bound: &frontend::Bound,
        protocol: &backend::Protocol,
    ) -> String {
        let new_name = match (name, bound) {
            ("EncryptionBegin", frontend::Bound::Server) => "EncryptionResponse",
            ("EncryptionBegin", frontend::Bound::Client) => "EncryptionRequest",
            ("PingStart", frontend::Bound::Server) => "StatusRequest",
            ("Ping", frontend::Bound::Server) => "PingRequest",
            ("ServerInfo", frontend::Bound::Client) => "StatusResponse",
            ("Ping", frontend::Bound::Client) => "PingResponse",
            ("Login", frontend::Bound::Client) => "JoinGame",
            _ => name,
        }
        .to_owned();

        if new_name == name
            && protocol.to_client.types.contains_key(unformatted_name)
            && protocol.to_server.types.contains_key(unformatted_name)
        {
            match bound {
                frontend::Bound::Server => format!("ServerBound{}", name),
                frontend::Bound::Client => format!("ClientBound{}", name),
            }
        } else {
            new_name.to_owned()
        }
    }

    fn change_field_type(&self, packet_name: &str, field: frontend::Field) -> frontend::Field {
        match (packet_name, field.name.as_str()) {
            // ("StatusResponse", "response") => field.change_type(frontend::DataType::RefType {
            //     ref_name: "ServerStatus".to_owned(),
            // }),
            // ("Success", "uuid") => field.change_type(frontend::DataType::Uuid { hyphenated: true }),
            // ("Disconnect", "reason") => field.change_type(frontend::DataType::Chat),
            // ("ClientBoundChat", "message") => field.change_type(frontend::DataType::Chat),
            // ("ClientBoundChat", "position") => field.change_type(frontend::DataType::RefType {
            //     ref_name: "MessagePosition".to_owned(),
            // }),
            _ => field,
        }
    }
}
