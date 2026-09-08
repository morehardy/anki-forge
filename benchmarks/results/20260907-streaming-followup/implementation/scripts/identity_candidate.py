"""Private candidate: resolve per-build note identities once, carry them forward."""
from pathlib import Path

WORK = Path(__file__).resolve().parent
SRC = WORK / 'profile-source/anki_forge/src'


def edit(file, before, after):
    path = SRC / file
    text = path.read_text()
    assert text.count(before) == 1, (file, before, text.count(before))
    path.write_text(text.replace(before, after))


f = 'product/project.rs'
edit(f, '''        let mut plan = self.lower_product_document();
        self.apply_note_source_paths(&mut plan);
        self.apply_notetype_source_paths(&mut plan);''',
     '''        let (mut plan, _) = self.lower_product_document();''')
edit(f, '''    fn lower_product_document(&self) -> LoweringPlan {
        let (document_id, payload) = self.to_product_v3_payload();
        crate::product::lowering::lower_owned_product_v2_document(document_id, payload)
    }

    fn to_product_v3_payload(&self) -> (String, crate::product::model::ProductDocumentV2Payload) {''',
     '''    fn lower_product_document(&self) -> (LoweringPlan, ResolvedNoteIdentities) {
        let counts = self.note_stable_id_counts();
        let identities = self.resolve_note_identities(&counts);
        let (document_id, payload) = self.product_payload_with_identities(&identities);
        let mut plan = crate::product::lowering::lower_owned_product_v2_document(document_id, payload);
        self.apply_prepared_note_source_paths(&mut plan, &counts, &identities);
        self.apply_notetype_source_paths(&mut plan);
        let identities = identities.into_iter()
            .map(|identity| (identity.stable_id.clone(), identity)).collect();
        (plan, identities)
    }

    #[cfg(test)]
    fn to_product_v3_payload(&self) -> (String, crate::product::model::ProductDocumentV2Payload) {
        let counts = self.note_stable_id_counts();
        self.product_payload_with_identities(&self.resolve_note_identities(&counts))
    }

    fn product_payload_with_identities(
        &self,
        identities: &[crate::update_safety::model::ResolvedNoteIdentity],
    ) -> (String, crate::product::model::ProductDocumentV2Payload) {''')
edit(f, '''        let stable_id_counts = self.note_stable_id_counts();
        let mut notes = Vec::with_capacity(self.notes.len());
        for (index, note) in self.notes.iter().enumerate() {
            let note_id =
                resolve_product_note_identity(self, note, index, &stable_id_counts).stable_id;''',
     '''        let mut notes = Vec::with_capacity(self.notes.len());
        for (index, (note, identity)) in self.notes.iter().zip(identities).enumerate() {
            let note_id = identity.stable_id.clone();''')
edit(f, '''    fn resolved_note_identities(
        &self,
    ) -> BTreeMap<String, crate::update_safety::model::ResolvedNoteIdentity> {
        let stable_id_counts = self.note_stable_id_counts();
        self.notes
            .iter()
            .enumerate()
            .map(|(index, note)| {
                let identity = resolve_product_note_identity(self, note, index, &stable_id_counts);
                (identity.stable_id.clone(), identity)
            })
            .collect()
    }''',
     '''    fn resolve_note_identities(
        &self,
        stable_id_counts: &BTreeMap<&str, usize>,
    ) -> Vec<crate::update_safety::model::ResolvedNoteIdentity> {
        self.notes.iter().enumerate()
            .map(|(index, note)| resolve_product_note_identity(self, note, index, stable_id_counts))
            .collect()
    }''')
edit(f, '''    fn apply_note_source_paths(&self, plan: &mut LoweringPlan) {
        let stable_id_counts = self.note_stable_id_counts();
        let mut project_indexes_by_authoring_id = BTreeMap::<String, Vec<usize>>::new();
        for (project_index, product_note) in self.notes.iter().enumerate() {
            let identity =
                resolve_product_note_identity(self, product_note, project_index, &stable_id_counts);
            project_indexes_by_authoring_id
                .entry(identity.stable_id)
                .or_default()
                .push(project_index);
        }
        for authoring_note in &plan.authoring_document.notes {
            let Some(project_index) = project_indexes_by_authoring_id
                .get(&authoring_note.id)
                .filter(|indexes| indexes.len() == 1)
                .map(|indexes| indexes[0])''',
     '''    #[cfg(test)]
    fn apply_note_source_paths(&self, plan: &mut LoweringPlan) {
        let counts = self.note_stable_id_counts();
        self.apply_prepared_note_source_paths(plan, &counts, &self.resolve_note_identities(&counts));
    }

    fn apply_prepared_note_source_paths(
        &self,
        plan: &mut LoweringPlan,
        stable_id_counts: &BTreeMap<&str, usize>,
        identities: &[crate::update_safety::model::ResolvedNoteIdentity],
    ) {
        let mut project_indexes_by_authoring_id = BTreeMap::new();
        for (project_index, identity) in identities.iter().enumerate() {
            project_indexes_by_authoring_id
                .entry(identity.stable_id.as_str())
                .and_modify(|index| *index = None)
                .or_insert(Some(project_index));
        }
        for authoring_note in &plan.authoring_document.notes {
            let Some(project_index) = project_indexes_by_authoring_id
                .get(authoring_note.id.as_str())
                .copied()
                .flatten()''')
edit(f, '''        return note
            .field_keys()
            .map(|field| (field.to_string(), field.to_string()))
            .collect();''',
     '''        // Without custom field aliases, the caller already uses the same
        // authoring field spelling as the source; no per-note identity map.
        return BTreeMap::new();''')
edit(f, '''struct ProjectNormalizeOutput {
    normalized_ir: crate::authoring_core::NormalizedIr,''',
     '''type ResolvedNoteIdentities = BTreeMap<String, crate::update_safety::model::ResolvedNoteIdentity>;

struct ProjectNormalizeOutput {
    resolved_note_identities: ResolvedNoteIdentities,
    normalized_ir: crate::authoring_core::NormalizedIr,''')

f = 'product/project/input.rs'
path = SRC / f
text = path.read_text()
start = text.index('    pub(super) fn resolved_note_identities(')
end = text.index('    fn lower_with_project_error(', start)
path.write_text(text[:start] + text[end:])
edit(f, 'fn lower_with_project_error(&self) -> Result<LoweringPlan, ProjectNormalizeError>',
     'fn lower_with_project_error(&self) -> Result<(LoweringPlan, ResolvedNoteIdentities), ProjectNormalizeError>')
edit(f, '''            Self::Project(project) => {
                let mut plan = project.lower_product_document();
                project.apply_note_source_paths(&mut plan);
                project.apply_notetype_source_paths(&mut plan);
                Ok(plan)
            }
            Self::Document(document) => document.lower(),''',
     '''            Self::Project(project) => Ok(project.lower_product_document()),
            Self::Document(document) => document.lower().map(|plan| {
                let identities = document.product_v2()
                    .map(product_v2_resolved_note_identities).unwrap_or_default();
                (plan, identities)
            }),''')
edit(f, '        let mut lowering = self.lower_with_project_error()?;',
     '        let (mut lowering, resolved_note_identities) = self.lower_with_project_error()?;')
edit(f, '        Ok(ProjectNormalizeOutput {\n            normalized_ir,',
     '        Ok(ProjectNormalizeOutput {\n            resolved_note_identities,\n            normalized_ir,')

f = 'product/project/pipeline.rs'
edit(f, '''        let prepared = self.prepare(writer_stack)?;
        let reconciled = self.reconcile(prepared)?;''',
     '''        let (prepared, identities) = self.prepare(writer_stack)?;
        let reconciled = self.reconcile(prepared, identities)?;''')
edit(f, '    ) -> Result<PreparedBuild, BuildFailureCause> {',
     '    ) -> Result<(PreparedBuild, ResolvedNoteIdentities), BuildFailureCause> {')
edit(f, '        let normalized = normalized_output.normalized_ir;',
     '        let normalized = normalized_output.normalized_ir;\n        let identities = normalized_output.resolved_note_identities;')
edit(f, '        Ok(PreparedBuild {', '        Ok((PreparedBuild {')
edit(f, '''            build_context,
        })
    }

    fn generate(''', '''            build_context,
        }, identities))
    }

    fn generate(''')

f = 'product/project/pipeline/reconcile.rs'
edit(f, '        prepared: PreparedBuild,', '        prepared: PreparedBuild,\n        resolved_note_identities: ResolvedNoteIdentities,')
edit(f, '        let resolved_note_identities = input.resolved_note_identities();\n', '')
print('Identity preparation candidate ready')
