use handlebars::{Context, Handlebars, Helper, Output, RenderContext, RenderError};
use heck::SnakeCase;

pub fn create_template_engine(templates_folder: &str) -> Handlebars<'static> {
    let mut template_engine = Handlebars::new();

    template_engine.register_helper("snake_case", Box::new(format_snake_case));
    template_engine.register_helper("packet_id", Box::new(format_packet_id));
    template_engine.register_helper(
        "protocol_version_module",
        Box::new(format_protocol_version_module),
    );
    template_engine.register_escape_fn(|s| s.to_owned());

    register_template_file(&mut template_engine, templates_folder, "packet_struct");

    template_engine
}

fn register_template_file(template_engine: &mut Handlebars, templates_folder: &str, name: &str) {
    let tpl_path = format!("{}/{}.hbs", templates_folder, name);

    template_engine
        .register_template_file(name, tpl_path)
        .expect("Failed to register template");
}

fn format_snake_case(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> Result<(), RenderError> {
    let str = h
        .param(0)
        .and_then(|v| v.value().as_str())
        .ok_or(RenderError::new(
            "Param 0 with str type is required for snake case helper.",
        ))? as &str;

    let snake_case_str = str.to_snake_case();

    out.write(snake_case_str.as_ref())?;
    Ok(())
}

fn format_packet_id(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> Result<(), RenderError> {
    let id = h
        .param(0)
        .and_then(|v| v.value().as_u64())
        .ok_or(RenderError::new(
            "Param 0 with u64 type is required for packet id helper.",
        ))? as u64;

    let packet_id_str = format!("{:#04X}", id);

    out.write(packet_id_str.as_ref())?;
    Ok(())
}

fn format_protocol_version_module(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> Result<(), RenderError> {
    let version = h
        .param(0)
        .and_then(|v| v.value().as_str())
        .ok_or(RenderError::new(
            "Param 0 with str type is required for packet id helper.",
        ))? as &str;

    let formatted_protocol_module_version =
        format!("v_{}", version.replace(".", "_").replace("-", "_"));

    out.write(formatted_protocol_module_version.as_ref())?;
    Ok(())
}
