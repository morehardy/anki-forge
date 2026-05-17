use super::FieldKey;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityRecipe {
    field_keys: Vec<FieldKey>,
}

impl IdentityRecipe {
    pub fn fields<I, S>(fields: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut field_keys = fields
            .into_iter()
            .map(|field| FieldKey::new(field.into()))
            .collect::<Vec<_>>();
        field_keys.sort();
        field_keys.dedup();
        Self { field_keys }
    }

    pub fn field_keys(&self) -> Vec<FieldKey> {
        self.field_keys.clone()
    }
}
