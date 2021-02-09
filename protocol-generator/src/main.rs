use std::fs::File;

use crate::mappings::CodeMappings;
use std::fs;
use std::io::Error;
use std::path::Path;

pub mod backend;
pub mod frontend;
pub mod mappings;
pub mod templates;
pub mod transformers;

pub fn main() {
    let paths = fs::read_dir("protocol-generator/minecraft-data/data/pc/")
        .expect("Failed to open data folder");

    let versions_data = paths
        .into_iter()
        .map(|entry| {
            entry
                .expect("Failed to get dir entry")
                .file_name()
                .into_string()
                .expect("Failed to get version string")
        })
        .filter(|version| match version.as_str() {
            "0.30c" => false, // A very old version with a lot of incompatibility.
            "1.7" => false,   // Requires some fixes to support.
            _ => true,
        })
        .filter_map(|version| {
            let protocol_data_file_name = format!(
                "protocol-generator/minecraft-data/data/pc/{}/protocol.json",
                version
            );

            let protocol_data_file_path = Path::new(&protocol_data_file_name);

            match protocol_data_file_path.exists() {
                true => {
                    let protocol_data_file = File::open(protocol_data_file_path)
                        .expect("Failed to open protocol data file");

                    Some((version, protocol_data_file))
                }
                false => None,
            }
        })
        .collect();

    let template_engine = templates::create_template_engine();
    let mappings = CodeMappings::new();

    frontend::generate_rust_files(versions_data, &template_engine, &mappings)
        .expect("Failed to generate rust files");
}
