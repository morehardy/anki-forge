pub mod builders;
pub mod export;
pub(crate) mod identity;
pub mod lowering;
pub mod media;
pub mod model;
pub mod validation;

pub use export::BuildResult;
pub use media::MediaSource;
pub use model::{
    BasicIdentityField, BasicIdentityOverride, BasicIdentitySelection, BasicNote, ClozeNote, Deck,
    DeckError, DeckIdentityPolicy, DeckNote, IdentityOverride, IdentityProvenance,
    IdentitySelection, IoMode, IoNote, IoRect, MediaRef, Package, ResolvedIdentitySnapshot,
};
pub use validation::{ValidationCode, ValidationDiagnostic, ValidationReport};
