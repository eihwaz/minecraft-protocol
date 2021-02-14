use handlebars::RenderError;

#[derive(Debug)]
pub enum FrontendError {
    TemplateRenderError { render_error: RenderError },
}

impl From<RenderError> for FrontendError {
    fn from(render_error: RenderError) -> Self {
        FrontendError::TemplateRenderError { render_error }
    }
}
