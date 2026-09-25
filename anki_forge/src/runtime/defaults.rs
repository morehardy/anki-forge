use crate::writer_core::{BuildContext, WriterPolicy};

pub(crate) fn load_embedded_writer_defaults() -> anyhow::Result<(WriterPolicy, BuildContext)> {
    super::embedded::load_writer_defaults()
}
