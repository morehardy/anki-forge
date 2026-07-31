pub mod assets;
pub mod builders;
pub mod comparison;
pub mod content;
pub mod diagnostics;
pub mod helpers;
pub mod identity;
pub mod lowering;
pub mod media_registry;
pub mod metadata;
pub mod model;
pub mod note;
pub mod notetype;
pub mod project;
pub mod stock;
pub mod template;
pub mod template_bundle;
pub mod template_engine;

pub use assets::{AssetSource, FontBinding};
pub use content::Content;
pub use diagnostics::{LoweringDiagnostic, ProductDiagnostic, ProductLoweringError};
pub use helpers::HelperDeclaration;
pub use identity::IdentityRecipe;
pub use lowering::{LoweringMapping, LoweringPlan};
pub use media_registry::{MediaRef, MediaRegistry};
pub use metadata::{
    FieldMetadataDeclaration, TemplateBrowserAppearanceDeclaration, TemplateTargetDeckDeclaration,
};
pub use model::{
    BasicNoteType, CustomField, CustomGenerationRule, CustomNote, CustomNoteType, CustomTemplate,
    ProductDocument, ProductNote, ProductNoteType,
};
pub use note::{ImageOcclusionNoteBuilder, Note, ProductNoteError};
pub use notetype::{Field, FieldKey, NoteType, NoteTypeKind};
pub use project::{Project, ProjectAddError};
pub use stock::{
    render_image_occlusion_cloze, STOCK_BASIC_ID, STOCK_CLOZE_ID, STOCK_IMAGE_OCCLUSION_ID,
};
pub use template::{stable_config_id, GenerationRule, Template, TemplateKey, TemplateSource};
pub use template_bundle::TemplateBundleError;
pub use template_engine::{TemplateEngine, TemplateIssue, TemplateIssueSeverity};
