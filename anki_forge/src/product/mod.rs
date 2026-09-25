pub mod assets;
pub mod builders;
pub mod content;
pub mod diagnostics;
pub mod helpers;
pub mod lowering;
pub mod metadata;
pub mod model;
pub mod stock;
pub mod template;
pub mod template_engine;

pub use diagnostics::{LoweringDiagnostic, ProductDiagnostic, ProductLoweringError};
pub use model::ProductDocument;
pub use template::stable_config_id;
pub use template_engine::{TemplateEngine, TemplateIssueSeverity};
