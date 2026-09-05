use crate::build::{BuildOptions, BuildReport};
use crate::product::ProductDocument;
use crate::writer_core::{BuildContext, WriterPolicy};

pub fn build_product_document(
    document: ProductDocument,
    options: BuildOptions,
) -> Result<BuildReport, crate::build::BuildError> {
    document.build(options)
}

pub fn build_product_document_with_writer_stack(
    document: ProductDocument,
    options: BuildOptions,
    writer_policy: WriterPolicy,
    build_context: BuildContext,
) -> Result<BuildReport, crate::build::BuildError> {
    document.build_with_writer_stack(options, writer_policy, build_context)
}
