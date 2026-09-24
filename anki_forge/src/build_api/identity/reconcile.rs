use super::*;

impl PackageIdentity {
    pub(crate) fn prepare(
        project: &Project,
        baseline: Option<&PackageIdentity>,
    ) -> Result<Self, BuildError> {
        let fresh = Self::create(project)?;
        let Some(baseline) = baseline else {
            return Ok(fresh);
        };
        if baseline.namespace != project.namespace {
            return Err(BuildError::new(
                Kind::Validation,
                "UPDATE.NAMESPACE_MISMATCH",
                "baseline belongs to a different project namespace",
            ));
        }
        let mut result = baseline.clone();
        for model in result.models.values_mut() {
            model.active = false;
        }
        for note in result.notes.values_mut() {
            note.active = false;
        }
        for (key, current) in fresh.models {
            let Some(previous) = result.models.get_mut(&key) else {
                result.models.insert(key, current);
                continue;
            };
            if previous.kind != current.kind {
                return Err(BuildError::new(
                    Kind::Validation,
                    "UPDATE.MODEL_KIND_CHANGED",
                    "a model key cannot change between normal and cloze kinds",
                ));
            }
            previous.active = true;
            merge_symbols(
                &mut previous.fields,
                &mut previous.field_high_water,
                current.fields,
            )?;
            merge_symbols(
                &mut previous.templates,
                &mut previous.template_high_water,
                current.templates,
            )?;
        }
        for (key, current) in fresh.notes {
            let Some(previous) = result.notes.get_mut(&key) else {
                result.notes.insert(key, current);
                continue;
            };
            if previous.model != current.model {
                return Err(BuildError::new(
                    Kind::Validation,
                    "UPDATE.NOTE_MODEL_CHANGED",
                    "an existing note key cannot be reassigned to another model",
                ));
            }
            previous.active = true;
            for mask in previous.masks.values_mut() {
                mask.active = false;
            }
            // New masks follow authored order, never BTreeMap key order.
            let mut masks: Vec<_> = current.masks.into_iter().collect();
            masks.sort_by_key(|(_, mask)| mask.ordinal);
            for (key, _) in masks {
                if let Some(mask) = previous.masks.get_mut(&key) {
                    mask.active = true;
                    continue;
                }
                if previous.mask_high_water >= 500 {
                    return Err(BuildError::new(
                        Kind::Validation,
                        "NOTE.IO_ORDINAL_EXHAUSTED",
                        "mask history has used all 500 cloze ordinals; split this note",
                    ));
                }
                previous.masks.insert(
                    key,
                    MaskIdentity {
                        ordinal: previous.mask_high_water,
                        active: true,
                    },
                );
                previous.mask_high_water += 1;
            }
        }
        Ok(result)
    }

    pub(crate) fn mask_ordinals(&self, key: &str) -> BTreeMap<String, u16> {
        self.notes[key]
            .masks
            .iter()
            .filter(|(_, mask)| mask.active)
            .map(|(key, mask)| (key.clone(), mask.ordinal + 1))
            .collect()
    }
}

pub(super) fn advance_revision(
    previous_hash: &mut String,
    mtime: &mut i64,
    hash: String,
) -> Result<(), BuildError> {
    if !previous_hash.is_empty() && *previous_hash != hash {
        *mtime = timestamp()?.max(mtime.checked_add(1).ok_or_else(|| {
            BuildError::new(
                Kind::Validation,
                "UPDATE.REVISION_EXHAUSTED",
                "modification time cannot advance",
            )
        })?);
    }
    *previous_hash = hash;
    Ok(())
}

fn merge_symbols(
    previous: &mut BTreeMap<String, SymbolIdentity>,
    high_water: &mut u32,
    current: BTreeMap<String, SymbolIdentity>,
) -> Result<(), BuildError> {
    for symbol in previous.values_mut() {
        symbol.ordinal = None;
    }
    let mut current: Vec<_> = current.into_iter().collect();
    current.sort_by_key(|(_, symbol)| symbol.slot);
    for (key, mut symbol) in current {
        if let Some(previous) = previous.get_mut(&key) {
            previous.ordinal = Some(0);
        } else {
            symbol.slot = *high_water;
            *high_water = high_water.checked_add(1).ok_or_else(|| {
                BuildError::new(
                    Kind::Validation,
                    "UPDATE.IDENTITY_EXHAUSTED",
                    "symbol history cannot grow further",
                )
            })?;
            previous.insert(key, symbol);
        }
    }
    let mut active: Vec<_> = previous
        .values_mut()
        .filter(|s| s.ordinal.is_some())
        .collect();
    active.sort_by_key(|s| s.slot);
    for (ordinal, symbol) in active.into_iter().enumerate() {
        symbol.ordinal = Some(ordinal as u32);
    }
    Ok(())
}
