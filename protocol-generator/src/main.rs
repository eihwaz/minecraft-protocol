use std::fs::File;

use crate::mappings::CodeMappings;
use structopt::StructOpt;

pub mod backend;
pub mod frontend;
pub mod mappings;
pub mod templates;
pub mod transformers;

#[derive(StructOpt)]
#[structopt(name = "protocol-generator")]
struct Opt {
    #[structopt(short, long, default_value = "1.14.4")]
    protocol_version: String,
}

pub fn main() {
    let opt: Opt = Opt::from_args();
    let template_engine = templates::create_template_engine();

    let protocol_data_file_name = format!(
        "protocol-generator/minecraft-data/data/pc/{}/protocol.json",
        opt.protocol_version
    );

    let protocol_data_file =
        File::open(protocol_data_file_name).expect("Failed to open protocol data file");

    let protocol_handler: backend::ProtocolHandler =
        serde_json::from_reader(protocol_data_file).expect("Failed to parse protocol data");

    let mappings = CodeMappings {};

    let protocols = vec![
        (
            transformers::transform_protocol(
                &mappings,
                frontend::State::Handshake,
                &protocol_handler.handshaking,
            ),
            frontend::State::Handshake,
        ),
        (
            transformers::transform_protocol(
                &mappings,
                frontend::State::Status,
                &protocol_handler.status,
            ),
            frontend::State::Status,
        ),
        (
            transformers::transform_protocol(
                &mappings,
                frontend::State::Login,
                &protocol_handler.login,
            ),
            frontend::State::Login,
        ),
        (
            transformers::transform_protocol(
                &mappings,
                frontend::State::Game,
                &protocol_handler.game,
            ),
            frontend::State::Game,
        ),
    ];

    for (protocol, state) in protocols {
        let file_name = format!(
            "protocol/src/packet/{}.rs",
            state.to_string().to_lowercase()
        );

        let file = File::create(file_name).expect("Failed to create file");

        frontend::generate_rust_file(&protocol, &template_engine, &file)
            .expect("Failed to generate rust file");
    }
}
