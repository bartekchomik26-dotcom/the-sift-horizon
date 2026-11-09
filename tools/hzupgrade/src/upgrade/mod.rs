//! A 26.2 data pack's world generation, rewritten in 26.3's formats.
//!
//! 26.3 reworked the worldgen registries: configured carvers and features
//! became `carver` and `feature` with their `config` unwrapped, noise
//! parameters gained a base amplitude, density functions name their inputs
//! `input`/`left`/`right`, the noise settings moved aquifers, ore veins and
//! the surface rule out of the router into an `aquifers` object and a
//! `material_rule`, and biomes carry their mob spawns as attributes. Each
//! conversion here is the one vanilla's own data went through between 26.2
//! and 26.3, written so that the 26.3 pack generates what the 26.2 pack did:
//! the hard-coded parts of 26.2 (the beardifier added to the final density,
//! the deep dark aquifer exclusion, the ore veinifier, the interpolation
//! cell from `size_horizontal`/`size_vertical`) are spelled out as data.
//! Whatever has no faithful 26.3 form is kept as it is and listed in the
//! report.
mod density;
mod features;
mod noise;
mod registries;
mod settings;

use density::Context;
use noise::noise;
pub use noise::parity_base_amplitude;
use registries::{biome, carver, dimension_type, structure};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, io::Read, path::Path};

/// 26.3's data pack format (`version.json`'s `pack_version.data_major`).
pub const PACK_FORMAT: u64 = 121;

/// What the upgrade did and what it could not do faithfully.
#[derive(Default, Debug)]
pub struct Report {
    /// Files rewritten, per registry.
    pub upgraded: BTreeMap<String, usize>,
    /// Files written that the 26.2 pack did not have (named density
    /// functions the noise settings share).
    pub added: Vec<String>,
    /// Parts that changed meaning or were dropped, with how often.
    pub lossy: BTreeMap<String, usize>,
    /// Types the upgrade does not know, left as they are.
    pub unknown: BTreeMap<String, usize>,
}

impl Report {
    fn lossy(&mut self, what: impl Into<String>) {
        *self.lossy.entry(what.into()).or_default() += 1;
    }
    fn unknown(&mut self, what: impl Into<String>) {
        *self.unknown.entry(what.into()).or_default() += 1;
    }
}

/// Every file of a pack, by its path inside the pack (`data/...`,
/// `pack.mcmeta`), upgraded. Files outside world generation are kept as
/// they are.
pub fn upgrade_pack(files: &BTreeMap<String, Vec<u8>>) -> (BTreeMap<String, Vec<u8>>, Report) {
    let mut report = Report::default();
    let mut json: BTreeMap<String, Value> = BTreeMap::new();
    let mut out: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    for (path, bytes) in files {
        let parsed = path
            .ends_with(".json")
            .then(|| serde_json::from_slice::<Value>(bytes).ok())
            .flatten();
        match parsed {
            Some(v) if path.starts_with("data/") => {
                json.insert(path.clone(), v);
            }
            _ => {
                out.insert(path.clone(), bytes.clone());
            }
        }
    }
    let mut ctx = Context::new(&json);
    let mut written: BTreeMap<String, Value> = BTreeMap::new();
    for (path, v) in &json {
        let Some(file) = PackFile::parse(path) else {
            written.insert(path.clone(), v.clone());
            continue;
        };
        let upgraded = match file.registry.as_str() {
            "worldgen/noise" => Some(noise(v, &mut report)),
            "worldgen/density_function" => Some(ctx.named_function(&file.id(), v, &mut report)),
            "worldgen/noise_settings" => Some(ctx.settings(&file.id(), v, &mut report)),
            "worldgen/configured_carver" => Some(carver(v, &mut report)),
            "worldgen/configured_feature" => Some(features::feature(v, &mut report)),
            "worldgen/placed_feature" => Some(features::placed(v, &mut report)),
            "worldgen/biome" => Some(biome(v, &mut report)),
            "worldgen/structure" => Some(structure(v)),
            "worldgen/world_preset" => Some(ctx.world_preset(v, &mut report)),
            "dimension" => Some(ctx.dimension(v, &mut report)),
            "dimension_type" => Some(dimension_type(v)),
            "worldgen/template_pool" | "worldgen/processor_list" => {
                let mut v = v.clone();
                states(&mut v);
                Some(v)
            }
            _ => None,
        };
        let path = match file.registry.as_str() {
            "worldgen/configured_carver" => file.with_registry("worldgen/carver"),
            "worldgen/configured_feature" => file.with_registry("worldgen/feature"),
            _ => path.clone(),
        };
        match upgraded {
            Some(u) => {
                *report.upgraded.entry(file.registry.clone()).or_default() += 1;
                written.insert(path, u);
            }
            None => {
                written.insert(path, v.clone());
            }
        }
    }
    for (id, v) in std::mem::take(&mut ctx.generated) {
        let path = registry_path("worldgen/density_function", &id);
        report.added.push(path.clone());
        written.insert(path, v);
    }
    for (id, v) in std::mem::take(&mut ctx.rules) {
        let path = registry_path("worldgen/material_rule", &id);
        report.added.push(path.clone());
        written.insert(path, v);
    }
    for (path, v) in written {
        out.insert(path, pretty(&v));
    }
    if let Some(meta) = out
        .get("pack.mcmeta")
        .and_then(|b| serde_json::from_slice(b).ok())
    {
        out.insert("pack.mcmeta".into(), pretty(&mcmeta(meta)));
    }
    (out, report)
}

fn pretty(v: &Value) -> Vec<u8> {
    let mut bytes = serde_json::to_vec_pretty(v).unwrap_or_default();
    bytes.push(b'\n');
    bytes
}

/// `pack.mcmeta` for 26.3: the pack format range is 26.3's alone.
fn mcmeta(mut meta: Value) -> Value {
    if let Some(pack) = meta.get_mut("pack").and_then(Value::as_object_mut) {
        pack.remove("supported_formats");
        pack.insert("pack_format".into(), json!(PACK_FORMAT));
        pack.insert("min_format".into(), json!(PACK_FORMAT));
        pack.insert("max_format".into(), json!(PACK_FORMAT));
        if let Some(Value::String(d)) = pack.get_mut("description") {
            d.push_str(" (upgraded to 26.3)");
        }
    }
    meta
}

/// `data/<ns>/<registry...>/<path>.json`, split by the registries the
/// upgrade knows (a registry name has one or two segments).
struct PackFile {
    namespace: String,
    registry: String,
    path: String,
}

impl PackFile {
    fn parse(path: &str) -> Option<Self> {
        let rest = path.strip_prefix("data/")?.strip_suffix(".json")?;
        let (namespace, rest) = rest.split_once('/')?;
        let (registry, path) = if let Some(r) = rest.strip_prefix("worldgen/") {
            let (kind, path) = r.split_once('/')?;
            (format!("worldgen/{kind}"), path)
        } else {
            rest.split_once('/').map(|(k, p)| (k.to_owned(), p))?
        };
        Some(Self {
            namespace: namespace.into(),
            registry,
            path: path.into(),
        })
    }
    fn id(&self) -> String {
        format!("{}:{}", self.namespace, self.path)
    }
    fn with_registry(&self, registry: &str) -> String {
        format!("data/{}/{registry}/{}.json", self.namespace, self.path)
    }
}

fn registry_path(registry: &str, id: &str) -> String {
    let (ns, path) = id.split_once(':').unwrap_or(("minecraft", id));
    format!("data/{ns}/{registry}/{path}.json")
}

/// A type name without its `minecraft:` prefix, as the codecs read both.
fn kind(v: &Value) -> Option<&str> {
    let t = v.get("type")?.as_str()?;
    Some(t.strip_prefix("minecraft:").unwrap_or(t))
}

/// An id with its namespace (`minecraft:` when it has none).
fn full_id(id: &str) -> String {
    if id.contains(':') {
        id.to_owned()
    } else {
        format!("minecraft:{id}")
    }
}

/// Whether an object of type `t` appears anywhere in `v`.
fn contains_type(v: &Value, t: &str) -> bool {
    match v {
        Value::Object(o) => kind(v) == Some(t) || o.values().any(|x| contains_type(x, t)),
        Value::Array(a) => a.iter().any(|x| contains_type(x, t)),
        _ => false,
    }
}

fn vanilla(name: &str) -> Value {
    Value::String(format!("minecraft:{name}"))
}

// ---------------------------------------------------------------------------
// Block states

/// 26.2 writes a block or fluid state as `{"Name": ..., "Properties":
/// {...}}`; 26.3 reads `{"id": ..., "properties": {...}}` (or the id alone
/// for the default state).
fn states(v: &mut Value) {
    match v {
        Value::Object(o) => {
            let state = o.get("Name").is_some_and(Value::is_string)
                && o.keys().all(|k| k == "Name" || k == "Properties");
            if state {
                let name = o.remove("Name").unwrap_or_default();
                match o.remove("Properties") {
                    Some(Value::Object(p)) if !p.is_empty() => {
                        o.insert("id".into(), name);
                        o.insert("properties".into(), Value::Object(p));
                    }
                    _ => *v = name,
                }
                return;
            }
            o.values_mut().for_each(states);
        }
        Value::Array(a) => a.iter_mut().for_each(states),
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Files

/// A pack's files by their path inside it, from a zip or a folder. A zip
/// whose `pack.mcmeta` sits in one top folder is read from that folder.
pub fn read_pack(path: &Path) -> Result<BTreeMap<String, Vec<u8>>, String> {
    let mut files = BTreeMap::new();
    fn walk(root:&Path, d:&Path, f:&mut BTreeMap<String,Vec<u8>>){ for e in fs::read_dir(d).unwrap(){let p=e.unwrap().path(); if p.is_dir(){walk(root,&p,f)} else {f.insert(p.strip_prefix(root).unwrap().to_string_lossy().replace('\\',"/"), fs::read(&p).unwrap());}}}
    walk(path, path, &mut files); Ok(files)
}

/// Upgrades the pack at `input` into the folder `out`, which must not
/// exist yet (a world's `datapacks/<name>`).
pub fn upgrade_into(input: &Path, out: &Path) -> Result<Report, String> {
    if out.exists() {
        return Err(format!("{} already exists", out.display()));
    }
    let files = read_pack(input)?;
    let (upgraded, report) = upgrade_pack(&files);
    for (path, bytes) in upgraded {
        if !is_inside(Path::new(&path)) {
            return Err(format!("{path}: leaves {}", out.display()));
        }
        let target = out.join(&path);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        }
        fs::write(&target, bytes).map_err(|e| format!("{}: {e}", target.display()))?;
    }
    Ok(report)
}

/// Whether `path` is a plain relative path: no root, drive, `.` or `..`.
fn is_inside(path: &Path) -> bool {
    path.components().next().is_some()
        && path
            .components()
            .all(|c| matches!(c, std::path::Component::Normal(_)))
}

impl Report {
    /// The report as lines for the terminal.
    pub fn lines(&self) -> Vec<String> {
        let mut lines: Vec<String> = self
            .upgraded
            .iter()
            .map(|(r, n)| format!("upgraded {n} {r}"))
            .collect();
        if !self.added.is_empty() {
            lines.push(format!(
                "added {} shared density functions",
                self.added.len()
            ));
        }
        lines.extend(
            self.lossy
                .iter()
                .map(|(w, n)| format!("not faithful ({n}x): {w}")),
        );
        lines.extend(
            self.unknown
                .iter()
                .map(|(w, n)| format!("left as it is ({n}x): {w}")),
        );
        lines
    }
}

