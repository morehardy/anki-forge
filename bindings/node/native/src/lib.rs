//! N-API transport over the default, public ankiforge API.
mod artifact;
mod options;
mod tasks;
use ankiforge::build::json::PathSnapshot;
use ankiforge::note::{Mask, OcclusionMode};
use ankiforge::schema::GenerationRule;
use ankiforge::{Content, Field, Media, Note, NoteType, Project, Template};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::Deserialize;
use serde_json::{json, Value};

fn configuration_error(message: impl Into<String>) -> Error {
    Error::from_reason(json!({"domain":"configuration","kind":"InvalidInput","code":"BINDING.INPUT_INVALID","message":message.into(),"causes":[]}).to_string())
}
fn parse<T: serde::de::DeserializeOwned>(s: &str) -> Result<T> {
    serde_json::from_str(s).map_err(|e| configuration_error(e.to_string()))
}
fn source_detail(error: &(dyn std::error::Error + 'static)) -> Value {
    if let Some(e) = error.downcast_ref::<ankiforge::media::MediaError>() {
        return json!({"type":"media","kind":format!("{:?}",e.kind()),"code":e.code(),"path":e.path().map(PathSnapshot::new),"limitExceeded":e.limit_exceeded().map(|l|json!({"resource":l.resource,"limit":l.limit,"observed":l.observed}))});
    }
    if let Some(e) = error.downcast_ref::<ankiforge::schema::SchemaError>() {
        return json!({"type":"schema","kind":format!("{:?}",e.kind()),"code":e.code(),"location":e.location().map(|l|json!({"template":l.template.as_str(),"side":format!("{:?}",l.side),"byteRange":{"start":l.byte_range.start,"end":l.byte_range.end}}))});
    }
    if let Some(e) = error.downcast_ref::<ankiforge::schema::TemplateBundleLimitExceeded>() {
        return json!({"type":"bundle_limit","limit":e.limit,"observed":e.observed});
    }
    if let Some(e) = error.downcast_ref::<ankiforge::build::InspectLimitExceeded>() {
        return json!({"type":"inspect_limit","resource":e.resource,"entry":e.entry,"limit":e.limit,"observed":e.observed});
    }
    if let Some(e) = error.downcast_ref::<std::io::Error>() {
        return json!({"type":"io","kind":format!("{:?}",e.kind()),"osCode":e.raw_os_error()});
    }
    json!({"type":"source","message":error.to_string()})
}
fn domain(
    domain: &str,
    kind: impl std::fmt::Debug,
    code: &str,
    error: &dyn std::error::Error,
    details: Value,
) -> Error {
    let mut causes = Vec::new();
    let mut source_details = Vec::new();
    let mut source = error.source();
    while let Some(e) = source {
        causes.push(e.to_string());
        source_details.push(source_detail(e));
        source = e.source();
    }
    Error::from_reason(json!({"domain":domain,"kind":format!("{kind:?}"),"code":code,"message":error.to_string(),"causes":causes,"sourceDetails":source_details,"details":details}).to_string())
}
macro_rules! err {
    ($domain:expr, $e:expr) => {
        domain($domain, $e.kind(), $e.code(), &$e, Value::Null)
    };
}
#[napi]
pub fn binding_metadata() -> String {
    json!({"bindingVersion":env!("CARGO_PKG_VERSION"),"bindingProtocolVersion":4,"target":env!("ANKI_FORGE_NODE_TARGET"),"nodeApiVersion":8}).to_string()
}

#[napi]
pub struct NativeContent {
    inner: Content,
}
#[napi]
impl NativeContent {
    #[napi(factory)]
    pub fn text(text: String) -> Self {
        Self {
            inner: Content::text(text),
        }
    }
    #[napi(factory)]
    pub fn html(text: String) -> Self {
        Self {
            inner: Content::html(text),
        }
    }
    #[napi(factory)]
    pub fn sequence(items: Vec<ClassInstance<NativeContent>>) -> Self {
        Self {
            inner: Content::sequence(items.iter().map(|v| v.inner.clone())),
        }
    }
}
#[napi]
pub struct NativeMedia {
    inner: Media,
}
#[napi]
impl NativeMedia {
    #[napi]
    pub fn file<'env>(env: &'env Env, path: String, limits: String) -> Result<Object<'env>> {
        tasks::spawn(
            env,
            MediaTask {
                source: Some(MediaSource::File(path)),
                limits: options::media_limits(&limits)?,
            },
        )
    }
    #[napi]
    pub fn bytes<'env>(
        env: &'env Env,
        bytes: Buffer,
        mime: String,
        limits: String,
    ) -> Result<Object<'env>> {
        let limits = options::media_limits(&limits)?;
        let observed = bytes.len() as u64;
        if observed > limits.max_bytes {
            // Reject before cloning the JS buffer into the worker's owned input.
            let code = "MEDIA.RESOURCE_LIMIT_EXCEEDED";
            return Err(Error::from_reason(
                json!({
                    "domain": "media",
                    "kind": "ResourceLimit",
                    "code": code,
                    "message": format!("{code}: media contains at least {observed} bytes; limit is {}", limits.max_bytes),
                    "causes": [],
                    "sourceDetails": [],
                    "details": {
                        "path": null,
                        "limitExceeded": {
                            "resource": "media_bytes",
                            "limit": limits.max_bytes,
                            "observed": observed
                        }
                    }
                })
                .to_string(),
            ));
        }
        tasks::spawn(
            env,
            MediaTask {
                // Capture accepted bytes synchronously so caller mutations after
                // this call cannot change the snapshot processed by the worker.
                source: Some(MediaSource::Bytes(bytes.to_vec(), mime)),
                limits,
            },
        )
    }
    #[napi]
    pub fn with_export_name(&self, name: String) -> Result<Self> {
        self.inner
            .clone()
            .with_export_name(name)
            .map(|inner| Self { inner })
            .map_err(|e| err!("media", e))
    }
    #[napi(getter)]
    pub fn filename(&self) -> &str {
        self.inner.filename()
    }
    #[napi(getter)]
    pub fn media_type(&self) -> &str {
        self.inner.media_type()
    }
    #[napi(getter)]
    pub fn byte_length(&self) -> f64 {
        self.inner.len() as f64
    }
    #[napi]
    pub fn image(&self) -> NativeContent {
        NativeContent {
            inner: self.inner.image(),
        }
    }
    #[napi]
    pub fn sound(&self) -> NativeContent {
        NativeContent {
            inner: self.inner.sound(),
        }
    }
}
enum MediaSource {
    File(String),
    Bytes(Vec<u8>, String),
}
struct MediaTask {
    source: Option<MediaSource>,
    limits: ankiforge::media::MediaLimits,
}
impl Task for MediaTask {
    type Output = Media;
    type JsValue = NativeMedia;
    fn compute(&mut self) -> Result<Media> {
        match self.source.take().expect("task runs once") {
        MediaSource::File(p)=>Media::file_with_limits(p,self.limits), MediaSource::Bytes(b,m)=>Media::bytes_with_limits(b,m,self.limits)
    }.map_err(|e|domain("media",e.kind(),e.code(),&e,json!({"path":e.path().map(PathSnapshot::new),"limitExceeded":e.limit_exceeded().map(|l|json!({"resource":l.resource,"limit":l.limit,"observed":l.observed}))})))
    }
    fn resolve(&mut self, _env: Env, inner: Media) -> Result<NativeMedia> {
        Ok(NativeMedia { inner })
    }
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct FieldInput {
    key: String,
    name: Option<String>,
    #[serde(default)]
    required: bool,
    #[serde(default)]
    sort: bool,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct TemplateInput {
    key: String,
    name: Option<String>,
    front: String,
    back: String,
    browser_front: Option<String>,
    browser_back: Option<String>,
    target_deck: Option<String>,
    generation: Option<GenerationInput>,
}
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum GenerationInput {
    AnkiDefault,
    All { fields: Vec<String> },
    Any { fields: Vec<String> },
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ModelInput {
    key: String,
    name: Option<String>,
    fields: Vec<FieldInput>,
    templates: Vec<TemplateInput>,
    css: Option<String>,
    cloze_field: Option<String>,
}
#[napi]
pub struct NativeNoteType {
    inner: NoteType,
}
#[napi]
impl NativeNoteType {
    #[napi(factory)]
    pub fn build(input: String, assets: Vec<ClassInstance<NativeMedia>>) -> Result<Self> {
        let i: ModelInput = parse(&input)?;
        let mut b = NoteType::builder(i.key);
        if let Some(n) = i.name {
            b = b.name(n)
        }
        for f in i.fields {
            let mut field = Field::new(f.key);
            if let Some(n) = f.name {
                field = field.name(n)
            }
            if f.required {
                field = field.required()
            }
            if f.sort {
                field = field.sort()
            }
            b = b.field(field);
        }
        for t in i.templates {
            let mut template = Template::new(t.key).front(t.front).back(t.back);
            if let Some(n) = t.name {
                template = template.name(n)
            }
            if let Some(s) = t.browser_front {
                template = template.browser_front(s)
            }
            if let Some(s) = t.browser_back {
                template = template.browser_back(s)
            }
            if let Some(s) = t.target_deck {
                template = template.target_deck(s)
            }
            if let Some(g) = t.generation {
                template = template.generate_when(match g {
                    GenerationInput::AnkiDefault => GenerationRule::AnkiDefault,
                    GenerationInput::All { fields } => GenerationRule::all(fields),
                    GenerationInput::Any { fields } => GenerationRule::any(fields),
                })
            }
            b = b.template(template);
        }
        if let Some(s) = i.css {
            b = b.css(s)
        }
        if let Some(s) = i.cloze_field {
            b = b.cloze_field(s)
        }
        for m in assets {
            b = b.asset(m.inner.clone())
        }
        b.build().map(|inner|Self{inner}).map_err(|e|domain("schema",e.kind(),e.code(),&e,json!({"location":e.location().map(|l|json!({"template":l.template.as_str(),"side":format!("{:?}",l.side),"byteRange":{"start":l.byte_range.start,"end":l.byte_range.end}}))})))
    }
    #[napi]
    pub fn from_bundle<'env>(env: &'env Env, path: String, limits: String) -> Result<Object<'env>> {
        tasks::spawn(
            env,
            BundleTask {
                path,
                limits: options::media_limits(&limits)?,
            },
        )
    }
    #[napi(getter)]
    pub fn key(&self) -> &str {
        self.inner.key()
    }
    #[napi(getter)]
    pub fn display_name(&self) -> &str {
        self.inner.display_name()
    }
    #[napi]
    pub fn note(&self) -> NativeNote {
        NativeNote {
            inner: self.inner.note(),
        }
    }
}
struct BundleTask {
    path: String,
    limits: ankiforge::media::MediaLimits,
}
impl Task for BundleTask {
    type Output = NoteType;
    type JsValue = NativeNoteType;
    fn compute(&mut self) -> Result<NoteType> {
        NoteType::from_bundle_with_limits(&self.path, self.limits).map_err(|e| {
            domain(
                "bundle",
                e.kind(),
                e.code(),
                &e,
                json!({"path":e.path().map(PathSnapshot::new),"byteOffset":e.byte_offset()}),
            )
        })
    }
    fn resolve(&mut self, _env: Env, inner: NoteType) -> Result<NativeNoteType> {
        Ok(NativeNoteType { inner })
    }
}
// Pass IEEE floats through N-API so Rust validates NaN/Infinity at IO build,
// preserving the same domain error and ordering as the native consumer API.
#[napi(object)]
pub struct MaskInput {
    pub key: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
#[napi]
pub struct NativeNote {
    inner: Note,
}
#[napi]
impl NativeNote {
    #[napi(factory)]
    pub fn basic(front: &NativeContent, back: &NativeContent) -> Self {
        Self {
            inner: Note::basic(front.inner.clone(), back.inner.clone()),
        }
    }
    #[napi(factory)]
    pub fn cloze(text: &NativeContent) -> Self {
        Self {
            inner: Note::cloze(text.inner.clone()),
        }
    }
    #[napi(factory)]
    pub fn image_occlusion(
        image: &NativeMedia,
        masks: Vec<MaskInput>,
        mode: String,
    ) -> Result<Self> {
        let mode = match mode.as_str() {
            "hide_all_guess_one" => OcclusionMode::HideAllGuessOne,
            "hide_one_guess_one" => OcclusionMode::HideOneGuessOne,
            _ => return Err(configuration_error("invalid occlusion mode")),
        };
        let mut b = Note::image_occlusion(image.inner.clone()).mode(mode);
        for m in masks {
            b = b.mask(Mask::rect(m.key, m.x, m.y, m.width, m.height))
        }
        b.build()
            .map(|inner| Self { inner })
            .map_err(|e| err!("occlusion", e))
    }
    #[napi]
    pub fn field(&self, key: String, value: &NativeContent) -> Self {
        Self {
            inner: self.inner.clone().field(key, value.inner.clone()),
        }
    }
    #[napi]
    pub fn deck(&self, name: String) -> Self {
        Self {
            inner: self.inner.clone().deck(name),
        }
    }
    #[napi]
    pub fn tag(&self, tag: String) -> Self {
        Self {
            inner: self.inner.clone().tag(tag),
        }
    }
}
#[napi]
pub struct NativeProject {
    inner: Project,
}
#[napi]
impl NativeProject {
    #[napi(constructor)]
    pub fn new(namespace: String) -> Result<Self> {
        Project::new(namespace)
            .map(|inner| Self { inner })
            .map_err(|e| err!("schema", e))
    }
    #[napi]
    pub fn name(&mut self, name: String) {
        self.inner = self.inner.clone().name(name)
    }
    #[napi]
    pub fn default_deck(&mut self, name: String) {
        self.inner = self.inner.clone().default_deck(name)
    }
    #[napi]
    pub fn add(&mut self, key: String, note: &NativeNote) -> Result<()> {
        self.inner
            .add(key, note.inner.clone())
            .map_err(|e| err!("add", e))
    }
    #[napi]
    pub fn add_asset(&mut self, media: &NativeMedia) -> Result<()> {
        self.inner
            .add_asset(media.inner.clone())
            .map_err(|e| err!("add", e))
    }
    #[napi(getter)]
    pub fn length(&self) -> u32 {
        self.inner.len() as u32
    }
    #[napi]
    pub fn clone_state(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
    #[napi]
    pub fn build<'env>(&self, env: &'env Env, input: String) -> Result<Object<'env>> {
        tasks::spawn(
            env,
            artifact::BuildTask {
                project: self.inner.clone(),
                options: options::build(&input)?,
            },
        )
    }
    #[napi]
    pub fn compare<'env>(&self, env: &'env Env, input: String) -> Result<Object<'env>> {
        tasks::spawn(
            env,
            CompareTask {
                project: self.inner.clone(),
                options: options::compare(&input)?,
            },
        )
    }
}
struct CompareTask {
    project: Project,
    options: ankiforge::update::CompareOptions,
}
impl Task for CompareTask {
    type Output = String;
    type JsValue = String;
    fn compute(&mut self) -> Result<String> {
        self.project.compare(self.options.clone()).map(|r|serde_json::to_string(&r.snapshot()).expect("serializable snapshot")).map_err(|e|domain("compare",e.kind(),e.code(),&e,json!({"report":e.report().snapshot(),"limitExceeded":e.limit_exceeded().map(|l|json!({"resource":l.resource,"entry":l.entry,"limit":l.limit,"observed":l.observed}))})))
    }
    fn resolve(&mut self, _env: Env, output: String) -> Result<String> {
        Ok(output)
    }
}
