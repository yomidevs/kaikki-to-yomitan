use crate::dict::writer::renderer::Renderer;

// The closest rendering to the yomitan popup.
// It is used as the default for the Renderer trait.
pub struct HtmlRenderer;

impl Renderer for HtmlRenderer {}
