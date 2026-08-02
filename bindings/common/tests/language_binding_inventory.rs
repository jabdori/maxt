//! Cross-language public API inventory parity.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use maxt::{Exchange, Feature};
use syn::{
    Fields, FnArg, GenericArgument, ImplItem, ImplItemFn, Item, Pat, PathArguments, ReturnType,
    TraitItem, Type, UseTree, Visibility,
};

const CORE_ADAPTER: &str = include_str!("../../../src/adapter.rs");
const CORE_CLIENT: &str = include_str!("../../../src/client.rs");
const COMMON_CONTRACT: &str = include_str!("../src/contract.rs");
const PYTHON_RUST_ADAPTER: &str = include_str!("../../python/src/adapter.rs");
const PYTHON_BUILTIN: &str = include_str!("../../python/src/builtin.rs");
const PYTHON_MODELS: &str = include_str!("../../python/python/maxt/models.py");
const PYTHON_API: &str = include_str!("../../python/python/maxt/_api.py");
const PYTHON_ADAPTERS: &str = include_str!("../../python/python/maxt/adapters.py");
const DART_RUST_ADAPTER: &str = include_str!("../../dart/rust/src/adapter.rs");
const DART_RUST_API: &str = include_str!("../../dart/rust/src/api.rs");
const DART_MODELS: &str = include_str!("../../dart/lib/src/models.dart");
const DART_ADAPTER: &str = include_str!("../../dart/lib/src/adapter.dart");
const DART_CLIENT: &str = include_str!("../../dart/lib/src/client.dart");
const DART_ADAPTERS: &str = include_str!("../../dart/lib/src/adapters.dart");
const DART_ERRORS: &str = include_str!("../../dart/lib/src/errors.dart");
const DART_PROVIDERS: &str = include_str!("../../dart/lib/src/providers.dart");
const DART_GENERATED_CONVERT: &str = include_str!("../../dart/lib/src/rust/convert.dart");
const DART_NATIVE_CONSTRUCTOR_EXCLUSIONS: &[&str] = &["from_dart_adapter"];

struct ProviderDescriptor {
    exchange: &'static str,
    adapter: &'static str,
    rust_source: &'static str,
    python_native: &'static str,
    dart_native_methods: &'static [&'static str],
    modes: &'static [&'static str],
    mode_axis: Option<&'static str>,
    python_default_mode: &'static str,
    python_factories: &'static [&'static str],
}

const PROVIDERS: &[ProviderDescriptor] = &[
    ProviderDescriptor {
        exchange: "upbit",
        adapter: "UpbitAdapter",
        rust_source: include_str!("../../../src/adapters/upbit/mod.rs"),
        python_native: "NativeUpbitAdapter",
        dart_native_methods: &["upbit"],
        modes: &["new"],
        mode_axis: None,
        python_default_mode: "new",
        python_factories: &[],
    },
    ProviderDescriptor {
        exchange: "bithumb",
        adapter: "BithumbAdapter",
        rust_source: include_str!("../../../src/adapters/bithumb/mod.rs"),
        python_native: "NativeBithumbAdapter",
        dart_native_methods: &["bithumb"],
        modes: &["new"],
        mode_axis: None,
        python_default_mode: "new",
        python_factories: &[],
    },
    ProviderDescriptor {
        exchange: "binance",
        adapter: "BinanceAdapter",
        rust_source: include_str!("../../../src/adapters/binance/mod.rs"),
        python_native: "NativeBinanceAdapter",
        dart_native_methods: &["binance_spot", "binance_usd_m_futures"],
        modes: &["spot", "usd_m_futures"],
        mode_axis: Some("venue"),
        python_default_mode: "spot",
        python_factories: &["spot", "usd_m_futures"],
    },
    ProviderDescriptor {
        exchange: "hyperliquid",
        adapter: "HyperliquidAdapter",
        rust_source: include_str!("../../../src/adapters/hyperliquid/mod.rs"),
        python_native: "NativeHyperliquidAdapter",
        dart_native_methods: &["hyperliquid"],
        modes: &["new", "testnet"],
        mode_axis: Some("testnet"),
        python_default_mode: "new",
        python_factories: &["testnet"],
    },
];

fn rust_trait_methods(source: &str, name: &str) -> BTreeSet<String> {
    syn::parse_file(source)
        .expect("Rust trait source must parse")
        .items
        .into_iter()
        .find_map(|item| match item {
            Item::Trait(item) if item.ident == name => Some(
                item.items
                    .into_iter()
                    .filter_map(|item| match item {
                        TraitItem::Fn(method) => Some(method.sig.ident.to_string()),
                        _ => None,
                    })
                    .collect(),
            ),
            _ => None,
        })
        .unwrap_or_else(|| panic!("Rust trait {name} must exist"))
}

fn rust_enum_variants(source: &str, name: &str) -> BTreeSet<String> {
    syn::parse_file(source)
        .expect("Rust enum source must parse")
        .items
        .into_iter()
        .find_map(|item| match item {
            Item::Enum(item) if item.ident == name => Some(
                item.variants
                    .into_iter()
                    .map(|variant| variant.ident.to_string())
                    .collect(),
            ),
            _ => None,
        })
        .unwrap_or_else(|| panic!("Rust enum {name} must exist"))
}

fn rust_struct_fields(source: &str, name: &str) -> BTreeSet<String> {
    syn::parse_file(source)
        .expect("Rust struct source must parse")
        .items
        .into_iter()
        .find_map(|item| match item {
            Item::Struct(item)
                if item.ident == name && matches!(item.vis, Visibility::Public(_)) =>
            {
                Some(match item.fields {
                    Fields::Named(fields) => fields
                        .named
                        .into_iter()
                        .map(|field| {
                            field
                                .ident
                                .expect("named field has an identifier")
                                .to_string()
                        })
                        .collect(),
                    Fields::Unnamed(_) => {
                        panic!(
                            "public Rust struct {name} is a tuple struct; classify it explicitly"
                        )
                    }
                    Fields::Unit => {
                        panic!("public Rust struct {name} is a unit struct; classify it explicitly")
                    }
                })
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("public Rust struct {name} must exist"))
}

fn rust_module_path(root: &Path, path: &Path) -> Vec<String> {
    let relative = path
        .strip_prefix(root)
        .unwrap_or_else(|_| panic!("{} must be below {}", path.display(), root.display()));
    let file_name = relative
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_else(|| panic!("{} must have a UTF-8 file name", path.display()));
    let mut module = relative
        .parent()
        .into_iter()
        .flat_map(Path::components)
        .map(|component| {
            component
                .as_os_str()
                .to_str()
                .unwrap_or_else(|| panic!("{} must have a UTF-8 module path", path.display()))
                .to_owned()
        })
        .collect::<Vec<_>>();
    match file_name {
        "lib.rs" => assert!(module.is_empty(), "lib.rs must be the crate root"),
        "mod.rs" => {}
        _ => module.push(
            path.file_stem()
                .and_then(|name| name.to_str())
                .unwrap_or_else(|| panic!("{} must have a UTF-8 file stem", path.display()))
                .to_owned(),
        ),
    }
    module
}

fn collect_rust_sources(
    root: &Path,
    directory: &Path,
    sources: &mut BTreeMap<Vec<String>, String>,
) {
    let mut entries = fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", directory.display()))
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or_else(|error| panic!("failed to read {} entry: {error}", directory.display()));
    entries.sort_by_key(std::fs::DirEntry::path);

    for entry in entries {
        let path = entry.path();
        let file_type = entry
            .file_type()
            .unwrap_or_else(|error| panic!("failed to inspect {}: {error}", path.display()));
        if file_type.is_dir() {
            collect_rust_sources(root, &path, sources);
        } else if file_type.is_file() && path.extension().is_some_and(|extension| extension == "rs")
        {
            let module = rust_module_path(root, &path);
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            assert!(
                sources.insert(module.clone(), source).is_none(),
                "duplicate Rust module path {}",
                module.join("::")
            );
        }
    }
}

fn collect_use_bindings(
    tree: UseTree,
    prefix: Vec<String>,
    bindings: &mut Vec<(String, Vec<String>)>,
) {
    match tree {
        UseTree::Path(path) => {
            let mut prefix = prefix;
            prefix.push(path.ident.to_string());
            collect_use_bindings(*path.tree, prefix, bindings);
        }
        UseTree::Name(name) => {
            let name = name.ident.to_string();
            if name == "self" {
                let public_name = prefix
                    .last()
                    .unwrap_or_else(|| panic!("public self re-export needs a path"))
                    .clone();
                bindings.push((public_name, prefix));
            } else {
                let mut target = prefix;
                target.push(name.clone());
                bindings.push((name, target));
            }
        }
        UseTree::Rename(rename) => {
            let public_name = rename.rename.to_string();
            if public_name == "_" {
                return;
            }
            let mut target = prefix;
            let name = rename.ident.to_string();
            if name != "self" {
                target.push(name);
            }
            bindings.push((public_name, target));
        }
        UseTree::Group(group) => {
            for tree in group.items {
                collect_use_bindings(tree, prefix.clone(), bindings);
            }
        }
        UseTree::Glob(_) => panic!("public glob re-exports need explicit inventory handling"),
    }
}

fn resolve_use_path(module: &[String], path: &[String]) -> Vec<String> {
    let mut resolved = module.to_vec();
    let mut index = 0;
    match path.first().map(String::as_str) {
        Some("crate") => {
            resolved.clear();
            index = 1;
        }
        Some("self") => index = 1,
        Some("super") => {
            while path.get(index).is_some_and(|part| part == "super") {
                assert!(
                    resolved.pop().is_some(),
                    "public re-export has too many super segments"
                );
                index += 1;
            }
        }
        _ => {}
    }
    resolved.extend_from_slice(&path[index..]);
    resolved
}

fn public_definitions(
    modules: &BTreeMap<Vec<String>, String>,
    include_enums_and_macros: bool,
) -> BTreeMap<Vec<String>, (String, String)> {
    let mut definitions = BTreeMap::new();
    for (module, source) in modules {
        for item in syn::parse_file(source)
            .expect("scanned Rust source must parse")
            .items
        {
            let (name, public) = match item {
                Item::Struct(item) => (
                    item.ident.to_string(),
                    matches!(item.vis, Visibility::Public(_)),
                ),
                Item::Enum(item) if include_enums_and_macros => (
                    item.ident.to_string(),
                    matches!(item.vis, Visibility::Public(_)),
                ),
                _ => continue,
            };
            if !public {
                continue;
            }
            let mut path = module.clone();
            path.push(name.clone());
            assert!(
                definitions
                    .insert(path.clone(), (name, source.clone()))
                    .is_none(),
                "duplicate public Rust struct path {}",
                path.join("::")
            );
        }

        if include_enums_and_macros && source.contains("macro_rules!") {
            for line in source.lines().map(str::trim) {
                let Some(name) = ["pub struct ", "pub enum "]
                    .into_iter()
                    .find_map(|prefix| line.strip_prefix(prefix))
                    .map(|declaration| {
                        declaration
                            .chars()
                            .take_while(|character| {
                                *character == '_' || character.is_ascii_alphanumeric()
                            })
                            .collect::<String>()
                    })
                else {
                    continue;
                };
                if name.is_empty() {
                    continue;
                }
                let mut path = module.clone();
                path.push(name.clone());
                definitions.entry(path).or_insert((name, source.clone()));
            }
        }
    }
    definitions
}

type UseBindingGraph = (BTreeMap<Vec<String>, Vec<String>>, BTreeSet<Vec<String>>);

fn use_binding_graph(modules: &BTreeMap<Vec<String>, String>) -> UseBindingGraph {
    let mut edges = BTreeMap::new();
    let mut public_bindings = BTreeSet::new();
    for (module, source) in modules {
        for item in syn::parse_file(source)
            .expect("Rust re-export source must parse")
            .items
        {
            let Item::Use(item) = item else {
                continue;
            };
            let is_public = matches!(item.vis, Visibility::Public(_));
            let prefix = if item.leading_colon.is_some() {
                vec!["$external".to_owned()]
            } else {
                Vec::new()
            };
            let mut bindings = Vec::new();
            collect_use_bindings(item.tree, prefix, &mut bindings);
            for (public_name, target) in bindings {
                let mut binding = module.clone();
                binding.push(public_name);
                let target = resolve_use_path(module, &target);
                if is_public {
                    public_bindings.insert(binding.clone());
                }
                if let Some(previous) = edges.insert(binding.clone(), target.clone()) {
                    assert_eq!(
                        previous,
                        target,
                        "duplicate public re-export binding {}",
                        binding.join("::")
                    );
                }
            }
        }
    }
    (edges, public_bindings)
}

fn alias_target(
    path: &[String],
    edges: &BTreeMap<Vec<String>, Vec<String>>,
) -> Option<Vec<String>> {
    for prefix_length in (1..=path.len()).rev() {
        if let Some(target) = edges.get(&path[..prefix_length]) {
            let mut resolved = target.clone();
            resolved.extend_from_slice(&path[prefix_length..]);
            return Some(resolved);
        }
    }
    None
}

fn resolve_struct_source(
    path: &[String],
    definitions: &BTreeMap<Vec<String>, (String, String)>,
    edges: &BTreeMap<Vec<String>, Vec<String>>,
    visiting: &mut BTreeSet<Vec<String>>,
) -> Option<(String, String)> {
    if let Some(definition) = definitions.get(path) {
        return Some(definition.clone());
    }
    let target = alias_target(path, edges)?;
    assert!(
        visiting.insert(path.to_vec()),
        "public re-export cycle detected at {}",
        path.join("::")
    );
    let resolved = resolve_struct_source(&target, definitions, edges, visiting);
    visiting.remove(path);
    resolved
}

fn resolve_public_struct_sources(
    modules: &BTreeMap<Vec<String>, String>,
    entry_modules: &[Vec<String>],
) -> BTreeMap<String, (String, String)> {
    resolve_public_sources(
        modules,
        entry_modules,
        public_definitions(modules, false),
        "struct",
    )
}

fn resolve_public_type_sources(
    modules: &BTreeMap<Vec<String>, String>,
    entry_modules: &[Vec<String>],
) -> BTreeMap<String, (String, String)> {
    resolve_public_sources(
        modules,
        entry_modules,
        public_definitions(modules, true),
        "type",
    )
}

fn resolve_public_sources(
    modules: &BTreeMap<Vec<String>, String>,
    entry_modules: &[Vec<String>],
    definitions: BTreeMap<Vec<String>, (String, String)>,
    definition_kind: &str,
) -> BTreeMap<String, (String, String)> {
    let (edges, public_bindings) = use_binding_graph(modules);
    let mut exported = BTreeMap::new();

    for entry_module in entry_modules {
        let starts = public_bindings
            .iter()
            .chain(definitions.keys())
            .filter(|path| {
                path.len() == entry_module.len() + 1 && path.starts_with(entry_module.as_slice())
            })
            .cloned()
            .collect::<BTreeSet<_>>();
        for start in starts {
            let Some(source) =
                resolve_struct_source(&start, &definitions, &edges, &mut BTreeSet::new())
            else {
                continue;
            };
            let public_name = start.last().expect("public binding has a name").clone();
            if let Some(previous) = exported.insert(public_name.clone(), source.clone()) {
                assert_eq!(
                    previous, source,
                    "duplicate public Rust {definition_kind} name {public_name}"
                );
            }
        }
    }
    exported
}

fn exported_public_struct_sources(root: &Path) -> BTreeMap<String, (String, String)> {
    let src = root.join("src");
    let mut modules = BTreeMap::new();
    collect_rust_sources(&src, &src, &mut modules);
    resolve_public_struct_sources(&modules, &[Vec::new(), vec!["adapters".to_owned()]])
}

fn exported_public_type_sources(root: &Path) -> BTreeMap<String, (String, String)> {
    let src = root.join("src");
    let mut modules = BTreeMap::new();
    collect_rust_sources(&src, &src, &mut modules);
    resolve_public_type_sources(&modules, &[Vec::new(), vec!["adapters".to_owned()]])
}

fn qualified_variants(source: &str, prefix: &str) -> BTreeSet<String> {
    let mut variants = BTreeSet::new();
    let mut remainder = source;
    while let Some(index) = remainder.find(prefix) {
        remainder = &remainder[index + prefix.len()..];
        let variant = remainder
            .chars()
            .take_while(|character| character.is_ascii_alphanumeric() || *character == '_')
            .collect::<String>();
        if !variant.is_empty() {
            variants.insert(variant);
        }
    }
    variants
}

fn rust_impl_methods(source: &str, name: &str) -> Vec<ImplItemFn> {
    syn::parse_file(source)
        .expect("Rust impl source must parse")
        .items
        .into_iter()
        .filter_map(|item| match item {
            Item::Impl(item)
                if item.trait_.is_none()
                    && matches!(
                        item.self_ty.as_ref(),
                        Type::Path(path)
                            if path.path.segments.last().is_some_and(
                                |segment| segment.ident == name
                            )
                    ) =>
            {
                Some(item.items)
            }
            _ => None,
        })
        .flatten()
        .filter_map(|item| match item {
            ImplItem::Fn(method) => Some(method),
            _ => None,
        })
        .collect()
}

fn rust_public_methods(source: &str, name: &str, async_only: bool) -> BTreeSet<String> {
    rust_impl_methods(source, name)
        .into_iter()
        .filter(|method| {
            matches!(method.vis, Visibility::Public(_))
                && (!async_only || method.sig.asyncness.is_some())
        })
        .map(|method| method.sig.ident.to_string())
        .collect()
}

fn rust_without_outer_attributes(mut declaration: &str) -> &str {
    while declaration.starts_with("#[") {
        let mut nesting = DartNesting::default();
        let end = declaration
            .char_indices()
            .find_map(|(offset, character)| {
                let syntax = nesting.consume(character);
                (syntax && character == ']' && nesting.brackets == 0)
                    .then_some(offset + character.len_utf8())
            })
            .expect("Rust outer attribute must close");
        declaration = declaration[end..].trim_start();
    }
    declaration
}

fn rust_macro_public_methods(source: &str, name: &str) -> Vec<ImplItemFn> {
    if !source.contains("macro_rules!") {
        return Vec::new();
    }
    let marker = format!("impl {name}");
    let mut remainder = source;
    let mut methods = Vec::new();
    while let Some(index) = remainder.find(&marker) {
        let candidate = &remainder[index..];
        remainder = &candidate[marker.len()..];
        if remainder
            .chars()
            .next()
            .is_some_and(|character| character == '_' || character.is_ascii_alphanumeric())
        {
            continue;
        }
        let block = dart_block(candidate, &marker);
        let mut without_lifetime_apostrophes = String::with_capacity(block.len());
        for (offset, character) in block.char_indices() {
            if character == '\'' {
                let lifetime = block[offset + character.len_utf8()..]
                    .chars()
                    .take_while(|character| *character == '_' || character.is_ascii_alphanumeric())
                    .collect::<String>();
                let after_lifetime = offset + character.len_utf8() + lifetime.len();
                if !lifetime.is_empty() && !block[after_lifetime..].starts_with('\'') {
                    without_lifetime_apostrophes.push_str("__RUST_LIFETIME_");
                    continue;
                }
            }
            without_lifetime_apostrophes.push(character);
        }
        methods.extend(
            dart_top_level_members(&without_lifetime_apostrophes, name)
                .into_iter()
                .filter_map(|declaration| {
                    let declaration = rust_without_outer_attributes(&declaration);
                    if !declaration.split_whitespace().any(|token| token == "fn") {
                        return None;
                    }
                    let declaration = declaration.replace("__RUST_LIFETIME_", "'");
                    let method = syn::parse_str::<ImplItemFn>(&format!("{declaration} {{}}"))
                        .unwrap_or_else(|error| {
                            panic!("Rust macro method must parse: {declaration}: {error}")
                        });
                    matches!(method.vis, Visibility::Public(_)).then_some(method)
                }),
        );
    }
    methods
}

fn rust_public_value_methods(source: &str, name: &str) -> BTreeSet<String> {
    let mut methods = rust_public_methods(source, name, false);
    methods.extend(
        rust_macro_public_methods(source, name)
            .into_iter()
            .map(|method| method.sig.ident.to_string()),
    );
    methods
}

#[derive(Clone, Copy)]
enum BindingMemberKind {
    Method,
    Property,
    ClassMethod,
    StaticMethod,
    Constructor,
    Getter,
    StaticGetter,
    Factory,
    Field,
    EnumValue,
    IntRepresentation,
    StringRepresentation,
}

impl BindingMemberKind {
    const fn label(self) -> &'static str {
        match self {
            Self::Method => "method",
            Self::Property => "property",
            Self::ClassMethod => "class_method",
            Self::StaticMethod => "static_method",
            Self::Constructor => "constructor",
            Self::Getter => "getter",
            Self::StaticGetter => "static_getter",
            Self::Factory => "factory",
            Self::Field => "field",
            Self::EnumValue => "enum_value",
            Self::IntRepresentation => "int_representation",
            Self::StringRepresentation => "string_representation",
        }
    }
}

#[derive(Clone, Copy)]
struct BindingMember {
    kind: BindingMemberKind,
    name: &'static str,
}

#[derive(Clone, Copy)]
struct ValueMemberContract {
    rust: &'static str,
    rust_shape: Option<&'static str>,
    python: Option<BindingMember>,
    dart: Option<BindingMember>,
}

#[derive(Clone, Copy)]
enum RustValueSurface {
    Methods,
    Variants,
}

#[derive(Clone, Copy)]
enum PythonValueSurface {
    Class,
    Dataclass { fields: bool },
    String,
    Enum,
    Int,
}

#[derive(Clone, Copy)]
enum DartValueSurface {
    Class { fields: bool },
    ConstructibleClass { fields: bool },
    Extension,
}

struct ValueTypeContract<'a> {
    name: &'static str,
    rust_source: &'a str,
    python_source: &'a str,
    dart_source: &'a str,
    rust_surface: RustValueSurface,
    python_surface: PythonValueSurface,
    dart_surface: DartValueSurface,
    members: &'a [ValueMemberContract],
}

const fn binding_member(kind: BindingMemberKind, name: &'static str) -> BindingMember {
    BindingMember { kind, name }
}

const fn value_member(
    rust: &'static str,
    python: BindingMember,
    dart: BindingMember,
) -> ValueMemberContract {
    ValueMemberContract {
        rust,
        rust_shape: None,
        python: Some(python),
        dart: Some(dart),
    }
}

const fn shaped_value_member(
    rust: &'static str,
    rust_shape: &'static str,
    python: BindingMember,
    dart: BindingMember,
) -> ValueMemberContract {
    ValueMemberContract {
        rust,
        rust_shape: Some(rust_shape),
        python: Some(python),
        dart: Some(dart),
    }
}

const fn rust_only_value_member(rust: &'static str) -> ValueMemberContract {
    ValueMemberContract {
        rust,
        rust_shape: None,
        python: None,
        dart: None,
    }
}

const EXCHANGE_VALUE_MEMBERS: &[ValueMemberContract] = &[
    value_member(
        "id",
        binding_member(BindingMemberKind::EnumValue, "value"),
        binding_member(BindingMemberKind::Getter, "id"),
    ),
    value_member(
        "display_name",
        binding_member(BindingMemberKind::Method, "display_name"),
        binding_member(BindingMemberKind::Getter, "displayName"),
    ),
];

const FEATURE_VALUE_MEMBERS: &[ValueMemberContract] = &[
    value_member(
        "id",
        binding_member(BindingMemberKind::EnumValue, "value"),
        binding_member(BindingMemberKind::Getter, "wireName"),
    ),
    value_member(
        "needs_credentials",
        binding_member(BindingMemberKind::Method, "needs_credentials"),
        binding_member(BindingMemberKind::Getter, "needsCredentials"),
    ),
    value_member(
        "is_derivatives_only",
        binding_member(BindingMemberKind::Method, "is_derivatives_only"),
        binding_member(BindingMemberKind::Getter, "isDerivativesOnly"),
    ),
];

const MARKET_KIND_VALUE_MEMBERS: &[ValueMemberContract] = &[value_member(
    "is_derivative",
    binding_member(BindingMemberKind::Method, "is_derivative"),
    binding_member(BindingMemberKind::Getter, "isDerivative"),
)];

const BINANCE_LISTEN_KEY_VALUE_MEMBERS: &[ValueMemberContract] = &[value_member(
    "as_str",
    binding_member(BindingMemberKind::Property, "value"),
    binding_member(BindingMemberKind::Getter, "value"),
)];

const MARKET_VALUE_MEMBERS: &[ValueMemberContract] = &[
    shaped_value_member(
        "spot",
        "associated:spot(pos:exchange,pos:base,pos:quote)",
        binding_member(
            BindingMemberKind::ClassMethod,
            "spot(arg:exchange,arg:base,arg:quote)",
        ),
        binding_member(
            BindingMemberKind::Factory,
            "spot(pos:exchange,pos:base,pos:quote)",
        ),
    ),
    shaped_value_member(
        "perpetual",
        "associated:perpetual(pos:exchange,pos:base,pos:quote)",
        binding_member(
            BindingMemberKind::ClassMethod,
            "perpetual(arg:exchange,arg:base,arg:quote)",
        ),
        binding_member(
            BindingMemberKind::Factory,
            "perpetual(pos:exchange,pos:base,pos:quote)",
        ),
    ),
    shaped_value_member(
        "new",
        "associated:new(pos:exchange,pos:kind,pos:base,pos:quote)",
        binding_member(
            BindingMemberKind::Constructor,
            "Market(arg:exchange,arg:kind,arg:base,arg:quote)",
        ),
        binding_member(
            BindingMemberKind::Constructor,
            "Market(pos:exchange,pos:kind,pos:base,pos:quote)",
        ),
    ),
];

const SIDE_VALUE_MEMBERS: &[ValueMemberContract] = &[value_member(
    "flip",
    binding_member(BindingMemberKind::Method, "flip"),
    binding_member(BindingMemberKind::Getter, "flipped"),
)];

const ORDER_BOOK_VALUE_MEMBERS: &[ValueMemberContract] = &[
    value_member(
        "best_bid",
        binding_member(BindingMemberKind::Method, "best_bid"),
        binding_member(BindingMemberKind::Getter, "bestBid"),
    ),
    value_member(
        "best_ask",
        binding_member(BindingMemberKind::Method, "best_ask"),
        binding_member(BindingMemberKind::Getter, "bestAsk"),
    ),
    value_member(
        "spread",
        binding_member(BindingMemberKind::Method, "spread"),
        binding_member(BindingMemberKind::Getter, "spread"),
    ),
    value_member(
        "mid_price",
        binding_member(BindingMemberKind::Method, "mid_price"),
        binding_member(BindingMemberKind::Getter, "midPrice"),
    ),
];

const INTERVAL_VALUE_MEMBERS: &[ValueMemberContract] = &[
    value_member(
        "as_secs",
        binding_member(BindingMemberKind::Method, "as_secs"),
        binding_member(BindingMemberKind::Getter, "seconds"),
    ),
    shaped_value_member(
        "advance",
        "instance:advance(pos:at,pos:count)",
        binding_member(
            BindingMemberKind::Method,
            "advance(arg:timestamp_ns,arg:count)",
        ),
        binding_member(BindingMemberKind::Method, "advance(pos:at,pos:count)"),
    ),
];

const BALANCE_VALUE_MEMBERS: &[ValueMemberContract] = &[value_member(
    "total",
    binding_member(BindingMemberKind::Method, "total"),
    binding_member(BindingMemberKind::Getter, "total"),
)];

const ORDER_STATUS_VALUE_MEMBERS: &[ValueMemberContract] = &[value_member(
    "is_live",
    binding_member(BindingMemberKind::Method, "is_live"),
    binding_member(BindingMemberKind::Getter, "isLive"),
)];

const EXCHANGE_ERROR_KIND_VALUE_MEMBERS: &[ValueMemberContract] = &[value_member(
    "is_retryable",
    binding_member(BindingMemberKind::Method, "is_retryable"),
    binding_member(BindingMemberKind::Getter, "isRetryable"),
)];

const POSITION_VALUE_MEMBERS: &[ValueMemberContract] = &[value_member(
    "is_flat",
    binding_member(BindingMemberKind::Method, "is_flat"),
    binding_member(BindingMemberKind::Getter, "isFlat"),
)];

const PAGE_VALUE_MEMBERS: &[ValueMemberContract] = &[value_member(
    "has_more",
    binding_member(BindingMemberKind::Method, "has_more"),
    binding_member(BindingMemberKind::Getter, "hasMore"),
)];

const CURSOR_VALUE_MEMBERS: &[ValueMemberContract] = &[
    shaped_value_member(
        "new",
        "associated:new(pos:cursor)",
        binding_member(BindingMemberKind::StringRepresentation, "Cursor(pos:value)"),
        binding_member(BindingMemberKind::Constructor, "Cursor(pos:value)"),
    ),
    value_member(
        "as_str",
        binding_member(BindingMemberKind::Method, "as_str"),
        binding_member(BindingMemberKind::Field, "value"),
    ),
];

const SUBSCRIPTION_VALUE_MEMBERS: &[ValueMemberContract] = &[
    shaped_value_member(
        "new",
        "associated:new",
        binding_member(
            BindingMemberKind::Constructor,
            "Subscription(arg:markets,arg:feeds)",
        ),
        binding_member(BindingMemberKind::Constructor, "Subscription"),
    ),
    shaped_value_member(
        "market",
        "instance:market(pos:market)",
        binding_member(BindingMemberKind::Field, "markets"),
        binding_member(BindingMemberKind::Method, "withMarket(pos:market)"),
    ),
    shaped_value_member(
        "markets_iter",
        "instance:markets_iter(pos:markets)",
        binding_member(BindingMemberKind::Field, "markets"),
        binding_member(BindingMemberKind::Method, "withMarkets(pos:values)"),
    ),
    shaped_value_member(
        "feed",
        "instance:feed(pos:feed)",
        binding_member(BindingMemberKind::Field, "feeds"),
        binding_member(BindingMemberKind::Method, "withFeed(pos:feed)"),
    ),
    value_member(
        "markets",
        binding_member(BindingMemberKind::Field, "markets"),
        binding_member(BindingMemberKind::Field, "markets"),
    ),
    value_member(
        "feeds",
        binding_member(BindingMemberKind::Field, "feeds"),
        binding_member(BindingMemberKind::Field, "feeds"),
    ),
];

const TIMESTAMP_VALUE_MEMBERS: &[ValueMemberContract] = &[
    shaped_value_member(
        "from_nanos",
        "associated:from_nanos(pos:nanos)",
        binding_member(BindingMemberKind::IntRepresentation, "Timestamp"),
        binding_member(
            BindingMemberKind::Factory,
            "fromNanoseconds(pos:nanoseconds)",
        ),
    ),
    shaped_value_member(
        "from_micros",
        "associated:from_micros(pos:micros)",
        binding_member(BindingMemberKind::IntRepresentation, "Timestamp"),
        binding_member(
            BindingMemberKind::Factory,
            "fromMicroseconds(pos:microseconds)",
        ),
    ),
    shaped_value_member(
        "from_millis",
        "associated:from_millis(pos:millis)",
        binding_member(BindingMemberKind::IntRepresentation, "Timestamp"),
        binding_member(
            BindingMemberKind::Factory,
            "fromMilliseconds(pos:milliseconds)",
        ),
    ),
    shaped_value_member(
        "from_secs",
        "associated:from_secs(pos:secs)",
        binding_member(BindingMemberKind::IntRepresentation, "Timestamp"),
        binding_member(BindingMemberKind::Factory, "fromSeconds(pos:seconds)"),
    ),
    value_member(
        "as_nanos",
        binding_member(BindingMemberKind::IntRepresentation, "Timestamp"),
        binding_member(BindingMemberKind::Field, "nanosecondsSinceEpoch"),
    ),
    value_member(
        "as_millis",
        binding_member(BindingMemberKind::IntRepresentation, "Timestamp"),
        binding_member(BindingMemberKind::Getter, "millisecondsSinceEpoch"),
    ),
    value_member(
        "as_secs",
        binding_member(BindingMemberKind::IntRepresentation, "Timestamp"),
        binding_member(BindingMemberKind::Getter, "secondsSinceEpoch"),
    ),
    shaped_value_member(
        "now",
        "associated:now",
        binding_member(BindingMemberKind::IntRepresentation, "Timestamp"),
        binding_member(BindingMemberKind::Factory, "now"),
    ),
    rust_only_value_member("into_system_time"),
];

const MARKET_EVENT_VALUE_MEMBERS: &[ValueMemberContract] = &[
    value_member(
        "Trade",
        binding_member(BindingMemberKind::ClassMethod, "trade(arg:trade)"),
        binding_member(BindingMemberKind::Factory, "trade(pos:value)"),
    ),
    value_member(
        "OrderBook",
        binding_member(BindingMemberKind::ClassMethod, "order_book(arg:order_book)"),
        binding_member(BindingMemberKind::Factory, "orderBook(pos:value)"),
    ),
    value_member(
        "Ticker",
        binding_member(BindingMemberKind::ClassMethod, "ticker(arg:ticker)"),
        binding_member(BindingMemberKind::Factory, "ticker(pos:value)"),
    ),
    value_member(
        "Candle",
        binding_member(BindingMemberKind::ClassMethod, "candle(arg:candle)"),
        binding_member(BindingMemberKind::Factory, "candle(pos:value)"),
    ),
    value_member(
        "Reconnected",
        binding_member(BindingMemberKind::ClassMethod, "reconnected"),
        binding_member(BindingMemberKind::Factory, "reconnected"),
    ),
];

const ACCOUNT_EVENT_VALUE_MEMBERS: &[ValueMemberContract] = &[
    value_member(
        "Balance",
        binding_member(BindingMemberKind::ClassMethod, "balance(arg:balance)"),
        binding_member(BindingMemberKind::Factory, "balance(pos:value)"),
    ),
    value_member(
        "Order",
        binding_member(BindingMemberKind::ClassMethod, "order(arg:order)"),
        binding_member(BindingMemberKind::Factory, "order(pos:value)"),
    ),
    value_member(
        "Reconnected",
        binding_member(BindingMemberKind::ClassMethod, "reconnected"),
        binding_member(BindingMemberKind::Factory, "reconnected"),
    ),
];

const MARKET_PYTHON_ONLY_MEMBERS: &[BindingMember] = &[binding_member(
    BindingMemberKind::ClassMethod,
    "from_wire(arg:value)",
)];
const CURSOR_PYTHON_ONLY_MEMBERS: &[BindingMember] =
    &[binding_member(BindingMemberKind::Method, "to_wire")];

fn python_language_only_members(name: &str) -> &'static [BindingMember] {
    match name {
        "Market" => MARKET_PYTHON_ONLY_MEMBERS,
        "Cursor" => CURSOR_PYTHON_ONLY_MEMBERS,
        _ => &[],
    }
}

struct ExactPublicMethodType {
    name: &'static str,
    rust_source: &'static str,
    methods: &'static [&'static str],
}

const VERIFIED_PUBLIC_METHOD_TYPE_EXCLUSIONS: &[ExactPublicMethodType] = &[
    ExactPublicMethodType {
        name: "AccountStream",
        rust_source: include_str!("../../../src/stream.rs"),
        methods: &["new", "new_with_close", "close"],
    },
    ExactPublicMethodType {
        name: "CandleRequest",
        rust_source: include_str!("../../../src/request.rs"),
        methods: &["new", "from", "to", "limit"],
    },
    ExactPublicMethodType {
        name: "Error",
        rust_source: include_str!("../../../src/error.rs"),
        methods: &["is_retryable", "is_rate_limited", "adapter"],
    },
    ExactPublicMethodType {
        name: "HistoryRequest",
        rust_source: include_str!("../../../src/request.rs"),
        methods: &["new", "from", "to", "cursor", "limit"],
    },
    ExactPublicMethodType {
        name: "MarginRequest",
        rust_source: include_str!("../../../src/request.rs"),
        methods: &["new", "leverage", "margin_mode"],
    },
    ExactPublicMethodType {
        name: "MarketStream",
        rust_source: include_str!("../../../src/stream.rs"),
        methods: &["new", "new_with_close", "close"],
    },
    ExactPublicMethodType {
        name: "OrderRequest",
        rust_source: include_str!("../../../src/request.rs"),
        methods: &["market", "limit", "time_in_force", "reduce_only"],
    },
];

const CLIENT_METHOD_PARITY_TYPE: &str = "Client";

const VALUE_TYPE_CONTRACTS: &[ValueTypeContract<'static>] = &[
    ValueTypeContract {
        name: "Exchange",
        rust_source: include_str!("../../../src/types/market.rs"),
        python_source: PYTHON_MODELS,
        dart_source: DART_MODELS,
        rust_surface: RustValueSurface::Methods,
        python_surface: PythonValueSurface::Enum,
        dart_surface: DartValueSurface::Extension,
        members: EXCHANGE_VALUE_MEMBERS,
    },
    ValueTypeContract {
        name: "Feature",
        rust_source: include_str!("../../../src/feature.rs"),
        python_source: PYTHON_MODELS,
        dart_source: DART_MODELS,
        rust_surface: RustValueSurface::Methods,
        python_surface: PythonValueSurface::Enum,
        dart_surface: DartValueSurface::Extension,
        members: FEATURE_VALUE_MEMBERS,
    },
    ValueTypeContract {
        name: "MarketKind",
        rust_source: include_str!("../../../src/types/market.rs"),
        python_source: PYTHON_MODELS,
        dart_source: DART_MODELS,
        rust_surface: RustValueSurface::Methods,
        python_surface: PythonValueSurface::Class,
        dart_surface: DartValueSurface::Extension,
        members: MARKET_KIND_VALUE_MEMBERS,
    },
    ValueTypeContract {
        name: "BinanceListenKey",
        rust_source: include_str!("../../../src/adapters/binance/private.rs"),
        python_source: PYTHON_ADAPTERS,
        dart_source: DART_ADAPTERS,
        rust_surface: RustValueSurface::Methods,
        python_surface: PythonValueSurface::Class,
        dart_surface: DartValueSurface::Class { fields: false },
        members: BINANCE_LISTEN_KEY_VALUE_MEMBERS,
    },
    ValueTypeContract {
        name: "Market",
        rust_source: include_str!("../../../src/types/market.rs"),
        python_source: PYTHON_MODELS,
        dart_source: DART_MODELS,
        rust_surface: RustValueSurface::Methods,
        python_surface: PythonValueSurface::Dataclass { fields: false },
        dart_surface: DartValueSurface::ConstructibleClass { fields: false },
        members: MARKET_VALUE_MEMBERS,
    },
    ValueTypeContract {
        name: "Side",
        rust_source: include_str!("../../../src/types/data.rs"),
        python_source: PYTHON_MODELS,
        dart_source: DART_MODELS,
        rust_surface: RustValueSurface::Methods,
        python_surface: PythonValueSurface::Class,
        dart_surface: DartValueSurface::Extension,
        members: SIDE_VALUE_MEMBERS,
    },
    ValueTypeContract {
        name: "OrderBook",
        rust_source: include_str!("../../../src/types/data.rs"),
        python_source: PYTHON_MODELS,
        dart_source: DART_MODELS,
        rust_surface: RustValueSurface::Methods,
        python_surface: PythonValueSurface::Class,
        dart_surface: DartValueSurface::Class { fields: false },
        members: ORDER_BOOK_VALUE_MEMBERS,
    },
    ValueTypeContract {
        name: "Interval",
        rust_source: include_str!("../../../src/types/data.rs"),
        python_source: PYTHON_MODELS,
        dart_source: DART_MODELS,
        rust_surface: RustValueSurface::Methods,
        python_surface: PythonValueSurface::Class,
        dart_surface: DartValueSurface::Extension,
        members: INTERVAL_VALUE_MEMBERS,
    },
    ValueTypeContract {
        name: "Subscription",
        rust_source: include_str!("../../../src/types/stream.rs"),
        python_source: PYTHON_MODELS,
        dart_source: DART_MODELS,
        rust_surface: RustValueSurface::Methods,
        python_surface: PythonValueSurface::Dataclass { fields: true },
        dart_surface: DartValueSurface::ConstructibleClass { fields: true },
        members: SUBSCRIPTION_VALUE_MEMBERS,
    },
    ValueTypeContract {
        name: "Balance",
        rust_source: include_str!("../../../src/types/account.rs"),
        python_source: PYTHON_MODELS,
        dart_source: DART_MODELS,
        rust_surface: RustValueSurface::Methods,
        python_surface: PythonValueSurface::Class,
        dart_surface: DartValueSurface::Class { fields: false },
        members: BALANCE_VALUE_MEMBERS,
    },
    ValueTypeContract {
        name: "OrderStatus",
        rust_source: include_str!("../../../src/types/account.rs"),
        python_source: PYTHON_MODELS,
        dart_source: DART_MODELS,
        rust_surface: RustValueSurface::Methods,
        python_surface: PythonValueSurface::Class,
        dart_surface: DartValueSurface::Extension,
        members: ORDER_STATUS_VALUE_MEMBERS,
    },
    ValueTypeContract {
        name: "Position",
        rust_source: include_str!("../../../src/types/account.rs"),
        python_source: PYTHON_MODELS,
        dart_source: DART_MODELS,
        rust_surface: RustValueSurface::Methods,
        python_surface: PythonValueSurface::Class,
        dart_surface: DartValueSurface::Class { fields: false },
        members: POSITION_VALUE_MEMBERS,
    },
    ValueTypeContract {
        name: "Page",
        rust_source: include_str!("../../../src/types/account.rs"),
        python_source: PYTHON_MODELS,
        dart_source: DART_MODELS,
        rust_surface: RustValueSurface::Methods,
        python_surface: PythonValueSurface::Class,
        dart_surface: DartValueSurface::Class { fields: false },
        members: PAGE_VALUE_MEMBERS,
    },
    ValueTypeContract {
        name: "Cursor",
        rust_source: include_str!("../../../src/types/account.rs"),
        python_source: PYTHON_MODELS,
        dart_source: DART_MODELS,
        rust_surface: RustValueSurface::Methods,
        python_surface: PythonValueSurface::String,
        dart_surface: DartValueSurface::ConstructibleClass { fields: true },
        members: CURSOR_VALUE_MEMBERS,
    },
    ValueTypeContract {
        name: "Timestamp",
        rust_source: include_str!("../../../src/types/time.rs"),
        python_source: PYTHON_MODELS,
        dart_source: DART_MODELS,
        rust_surface: RustValueSurface::Methods,
        python_surface: PythonValueSurface::Int,
        dart_surface: DartValueSurface::Class { fields: true },
        members: TIMESTAMP_VALUE_MEMBERS,
    },
    ValueTypeContract {
        name: "MarketEvent",
        rust_source: include_str!("../../../src/types/stream.rs"),
        python_source: PYTHON_MODELS,
        dart_source: DART_MODELS,
        rust_surface: RustValueSurface::Variants,
        python_surface: PythonValueSurface::Class,
        dart_surface: DartValueSurface::Class { fields: false },
        members: MARKET_EVENT_VALUE_MEMBERS,
    },
    ValueTypeContract {
        name: "AccountEvent",
        rust_source: include_str!("../../../src/types/stream.rs"),
        python_source: PYTHON_MODELS,
        dart_source: DART_MODELS,
        rust_surface: RustValueSurface::Variants,
        python_surface: PythonValueSurface::Class,
        dart_surface: DartValueSurface::Class { fields: false },
        members: ACCOUNT_EVENT_VALUE_MEMBERS,
    },
    ValueTypeContract {
        name: "ExchangeErrorKind",
        rust_source: include_str!("../../../src/error.rs"),
        python_source: PYTHON_API,
        dart_source: DART_ERRORS,
        rust_surface: RustValueSurface::Methods,
        python_surface: PythonValueSurface::Class,
        dart_surface: DartValueSurface::Extension,
        members: EXCHANGE_ERROR_KIND_VALUE_MEMBERS,
    },
];

fn member_shape_key<T: AsRef<str>>(kind: &str, name: &str, required: &[T]) -> String {
    if required.is_empty() {
        return format!("{kind}:{name}");
    }
    format!(
        "{kind}:{name}({})",
        required
            .iter()
            .map(AsRef::as_ref)
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn binding_member_key(member: BindingMember) -> String {
    format!("{}:{}", member.kind.label(), member.name)
}

fn assert_value_type_contract(contract: &ValueTypeContract<'_>) {
    let mut expected = BTreeSet::new();
    for member in contract.members {
        assert!(
            expected.insert(member.rust.to_owned()),
            "Rust {} value mapping repeats {}",
            contract.name,
            member.rust
        );
        assert_eq!(
            member.python.is_some(),
            member.dart.is_some(),
            "Rust {}::{} must map to both languages or be rust_only",
            contract.name,
            member.rust
        );
    }
    let actual = match contract.rust_surface {
        RustValueSurface::Methods => rust_public_value_methods(contract.rust_source, contract.name),
        RustValueSurface::Variants => rust_enum_variants(contract.rust_source, contract.name),
    };
    assert_inventory(
        &format!("Rust {} value members", contract.name),
        &expected,
        &actual,
    );
    if matches!(contract.rust_surface, RustValueSurface::Methods) {
        let expected_rust_shapes = contract
            .members
            .iter()
            .map(|member| {
                member
                    .rust_shape
                    .map_or_else(|| format!("instance:{}", member.rust), str::to_owned)
            })
            .collect::<BTreeSet<_>>();
        assert_inventory(
            &format!("Rust {} value member shapes", contract.name),
            &expected_rust_shapes,
            &rust_public_value_method_shapes(contract.rust_source, contract.name),
        );
    }

    let mut expected_python = contract
        .members
        .iter()
        .filter_map(|member| member.python.map(binding_member_key))
        .collect::<BTreeSet<_>>();
    expected_python.extend(
        python_language_only_members(contract.name)
            .iter()
            .copied()
            .map(binding_member_key),
    );
    assert_inventory(
        &format!("Python {} value members", contract.name),
        &expected_python,
        &python_value_members(
            contract.python_source,
            contract.name,
            contract.python_surface,
        ),
    );

    let expected_dart = contract
        .members
        .iter()
        .filter_map(|member| member.dart.map(binding_member_key))
        .collect::<BTreeSet<_>>();
    assert_inventory(
        &format!("Dart {} value members", contract.name),
        &expected_dart,
        &dart_value_members(contract.dart_source, contract.name, contract.dart_surface),
    );
}

fn rust_typed_parameters(method: &ImplItemFn) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut parameters = Vec::new();
    for input in &method.sig.inputs {
        let FnArg::Typed(input) = input else {
            continue;
        };
        let Pat::Ident(pattern) = input.pat.as_ref() else {
            panic!(
                "Rust method {} parameter must use an identifier pattern",
                method.sig.ident
            );
        };
        assert!(
            pattern.subpat.is_none(),
            "Rust method {} parameter must use an identifier pattern",
            method.sig.ident
        );
        let parameter = pattern.ident.to_string();
        assert!(
            seen.insert(parameter.clone()),
            "Rust method {} has duplicate parameter {}",
            method.sig.ident,
            pattern.ident
        );
        parameters.push(parameter);
    }
    parameters
}

fn rust_method_shape(method: &ImplItemFn) -> String {
    let instance = method
        .sig
        .inputs
        .iter()
        .any(|input| matches!(input, FnArg::Receiver(_)));
    let required = rust_typed_parameters(method)
        .into_iter()
        .map(|parameter| format!("pos:{parameter}"))
        .collect::<Vec<_>>();
    member_shape_key(
        if instance { "instance" } else { "associated" },
        &method.sig.ident.to_string(),
        &required,
    )
}

fn rust_public_value_method_shapes(source: &str, name: &str) -> BTreeSet<String> {
    let mut shapes = rust_impl_methods(source, name)
        .into_iter()
        .filter(|method| matches!(method.vis, Visibility::Public(_)))
        .map(|method| rust_method_shape(&method))
        .collect::<BTreeSet<_>>();
    shapes.extend(
        rust_macro_public_methods(source, name)
            .iter()
            .map(rust_method_shape),
    );
    shapes
}

fn rust_method_parameters(source: &str, name: &str, method: &str) -> BTreeSet<String> {
    let mut matches = rust_impl_methods(source, name)
        .into_iter()
        .filter(|candidate| candidate.sig.ident == method)
        .collect::<Vec<_>>();
    assert_eq!(
        matches.len(),
        1,
        "Rust {name}::{method} must exist exactly once"
    );
    rust_typed_parameters(&matches.pop().expect("one Rust method"))
        .into_iter()
        .collect()
}

fn rust_type_contains_self(r#type: &Type) -> bool {
    let Type::Path(path) = r#type else {
        return false;
    };
    path.qself
        .as_ref()
        .is_some_and(|qself| rust_type_contains_self(&qself.ty))
        || path.path.is_ident("Self")
        || path.path.segments.iter().any(|segment| {
            let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
                return false;
            };
            arguments.args.iter().any(|argument| {
                matches!(
                    argument,
                    GenericArgument::Type(r#type) if rust_type_contains_self(r#type)
                )
            })
        })
}

fn rust_public_associated_constructors(
    source: &str,
    name: &str,
) -> BTreeMap<String, BTreeSet<String>> {
    rust_impl_methods(source, name)
        .into_iter()
        .filter(|method| {
            matches!(method.vis, Visibility::Public(_))
                && !method
                    .sig
                    .inputs
                    .iter()
                    .any(|input| matches!(input, FnArg::Receiver(_)))
                && matches!(
                    &method.sig.output,
                    ReturnType::Type(_, output) if rust_type_contains_self(output)
                )
        })
        .map(|method| {
            let name = method.sig.ident.to_string();
            let parameters = rust_typed_parameters(&method).into_iter().collect();
            (name, parameters)
        })
        .collect()
}

fn rust_self_returning_methods(source: &str, name: &str) -> BTreeMap<String, BTreeSet<String>> {
    rust_impl_methods(source, name)
        .into_iter()
        .filter(|method| {
            matches!(method.vis, Visibility::Public(_))
                && matches!(
                    &method.sig.output,
                    ReturnType::Type(_, output)
                        if matches!(
                            output.as_ref(),
                            Type::Path(path)
                                if path.qself.is_none() && path.path.is_ident("Self")
                        )
                )
        })
        .map(|method| {
            let name = method.sig.ident.to_string();
            let parameters = rust_typed_parameters(&method).into_iter().collect();
            (name, parameters)
        })
        .collect()
}

fn rust_constructor_settings(
    name: &str,
    methods: &BTreeMap<String, BTreeSet<String>>,
    modes: &[&str],
    mode_axis: Option<&str>,
) -> BTreeSet<String> {
    let actual_modes = methods
        .iter()
        .filter(|(_, parameters)| parameters.is_empty())
        .map(|(method, _)| method.clone())
        .collect::<BTreeSet<_>>();
    let expected_modes = modes
        .iter()
        .map(|mode| (*mode).to_owned())
        .collect::<BTreeSet<_>>();
    assert_inventory(
        &format!("Rust {name} constructor modes"),
        &expected_modes,
        &actual_modes,
    );

    let mut settings = methods.values().flatten().cloned().collect::<BTreeSet<_>>();
    if let Some(mode_axis) = mode_axis {
        settings.insert(mode_axis.to_owned());
    }
    settings
}

fn is_identifier(value: &str) -> bool {
    let mut characters = value.chars();
    characters
        .next()
        .is_some_and(|character| character == '_' || character.is_ascii_alphabetic())
        && characters.all(|character| character == '_' || character.is_ascii_alphanumeric())
}

fn python_class_body<'a>(source: &'a str, name: &str) -> &'a str {
    let lines = source.lines().collect::<Vec<_>>();
    let start = lines
        .iter()
        .position(|line| {
            line.strip_prefix("class ").is_some_and(|declaration| {
                declaration
                    .chars()
                    .take_while(|character| *character == '_' || character.is_ascii_alphanumeric())
                    .eq(name.chars())
            })
        })
        .unwrap_or_else(|| panic!("Python class {name} must exist"));
    let end = lines
        .iter()
        .enumerate()
        .skip(start + 1)
        .find_map(|(index, line)| (!line.is_empty() && !line.starts_with(' ')).then_some(index))
        .unwrap_or(lines.len());
    let start_offset = lines[..=start]
        .iter()
        .map(|line| line.len() + 1)
        .sum::<usize>()
        .min(source.len());
    let end_offset = lines[..end]
        .iter()
        .map(|line| line.len() + 1)
        .sum::<usize>()
        .min(source.len());
    &source[start_offset..end_offset]
}

fn python_wire_name(line: &str) -> Option<&str> {
    ["\"wire_name\"", "'wire_name'"]
        .into_iter()
        .find_map(|marker| {
            let value = line.split_once(marker)?.1.split_once(':')?.1.trim_start();
            let quote = value
                .chars()
                .next()
                .filter(|quote| matches!(quote, '\'' | '"'))?;
            value[1..].split_once(quote).map(|(name, _)| name)
        })
}

fn python_member_depth(body: &str) -> usize {
    body.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.len() - line.trim_start_matches([' ', '\t']).len())
        .filter(|depth| *depth > 0)
        .min()
        .unwrap_or(0)
}

fn python_class_fields(source: &str, name: &str) -> BTreeSet<String> {
    let body = python_class_body(source, name);
    let member_depth = python_member_depth(body);

    body.lines()
        .filter(|line| line.len() - line.trim_start_matches([' ', '\t']).len() == member_depth)
        .filter_map(|line| {
            let member = line.trim();
            let (field, annotation) = member.split_once(':')?;
            if !is_identifier(field)
                || annotation
                    .split(|character: char| character != '_' && !character.is_ascii_alphanumeric())
                    .any(|part| part == "ClassVar")
            {
                return None;
            }
            Some(python_wire_name(member).unwrap_or(field).to_owned())
        })
        .collect()
}

fn python_parameters(source: &str, name: &str, method: &str, required_only: bool) -> Vec<String> {
    let body = python_class_body(source, name);
    let member_depth = python_member_depth(body);
    let marker = format!("def {method}(");
    let mut offset = 0;
    let declaration = body
        .split_inclusive('\n')
        .find_map(|line| {
            let depth = line.len() - line.trim_start_matches([' ', '\t']).len();
            let trimmed = line.trim_start();
            let result = (depth == member_depth && trimmed.starts_with(&marker))
                .then_some(&body[offset + depth..]);
            offset += line.len();
            result
        })
        .unwrap_or_else(|| panic!("Python class {name} method {method} must exist"));

    let label = format!("Python {name}.{method}");
    let declarations = split_top_level_commas(parenthesized_contents(declaration, &label));
    let positional_only_end = declarations
        .iter()
        .position(|parameter| parameter.trim() == "/");
    let keyword_only_start = declarations
        .iter()
        .position(|parameter| parameter.trim_start().starts_with('*'));
    let mut seen = BTreeMap::new();
    let mut parameters = Vec::new();
    for (index, parameter) in declarations.into_iter().enumerate() {
        let parameter = parameter.trim();
        if matches!(parameter, "" | "/" | "*") || (required_only && parameter.contains('=')) {
            continue;
        }
        if required_only && parameter.starts_with('*') {
            continue;
        }
        let parameter = parameter.trim_start_matches('*').trim_start();
        let end = parameter.find([':', '=']).unwrap_or(parameter.len());
        let identifier = parameter[..end].trim();
        assert!(
            is_identifier(identifier),
            "Python {name}.{method} parameter must use an identifier: {parameter}"
        );
        if identifier != "self" && identifier != "cls" {
            let identifier = insert_normalized_parameter(&mut seen, &label, identifier);
            let group = if positional_only_end.is_some_and(|end| index < end) {
                "pos"
            } else if keyword_only_start.is_some_and(|start| index > start) {
                "kw"
            } else {
                "arg"
            };
            parameters.push(format!("{group}:{identifier}"));
        }
    }
    parameters
}

fn python_method_parameters(source: &str, name: &str, method: &str) -> BTreeSet<String> {
    python_parameters(source, name, method, false)
        .into_iter()
        .map(|parameter| {
            parameter
                .split_once(':')
                .expect("Python parameter shape has a group")
                .1
                .to_owned()
        })
        .collect()
}

fn python_required_method_parameters(source: &str, name: &str, method: &str) -> Vec<String> {
    python_parameters(source, name, method, true)
}

fn python_decorated_method_names(source: &str, name: &str, decorator: &str) -> BTreeSet<String> {
    let body = python_class_body(source, name);
    let member_depth = python_member_depth(body);
    let mut decorated = false;
    let mut methods = BTreeSet::new();
    for line in body.lines() {
        let depth = line.len() - line.trim_start_matches([' ', '\t']).len();
        if depth != member_depth {
            continue;
        }
        let member = line.trim();
        if member == decorator {
            decorated = true;
        } else if member.starts_with('@') {
            continue;
        } else if let Some(declaration) = member.strip_prefix("def ") {
            let method = declaration
                .split_once('(')
                .map(|(method, _)| method)
                .unwrap_or(declaration);
            if decorated && is_identifier(method) && !method.starts_with('_') {
                methods.insert(method.to_owned());
            }
            decorated = false;
        } else {
            decorated = false;
        }
    }
    methods
}

fn python_classmethod_names(source: &str, name: &str) -> BTreeSet<String> {
    python_decorated_method_names(source, name, "@classmethod")
}

fn python_property_names(source: &str, name: &str) -> BTreeSet<String> {
    python_decorated_method_names(source, name, "@property")
}

fn python_staticmethod_names(source: &str, name: &str) -> BTreeSet<String> {
    python_decorated_method_names(source, name, "@staticmethod")
}

fn python_classmethod_parameters(source: &str, name: &str) -> BTreeMap<String, BTreeSet<String>> {
    python_classmethod_names(source, name)
        .into_iter()
        .map(|method| {
            let parameters = python_method_parameters(source, name, &method);
            (method, parameters)
        })
        .collect()
}

fn python_class_methods(source: &str, name: &str, async_only: bool) -> BTreeSet<String> {
    let body = python_class_body(source, name);
    let member_depth = python_member_depth(body);
    let mut methods = BTreeSet::new();
    for line in body.lines() {
        let depth = line.len() - line.trim_start_matches([' ', '\t']).len();
        if depth != member_depth {
            continue;
        }
        let member = line.trim();
        let declaration = if let Some(value) = member.strip_prefix("async def ") {
            value
        } else if !async_only {
            match member.strip_prefix("def ") {
                Some(value) => value,
                None => continue,
            }
        } else {
            continue;
        };
        let method = declaration.split('(').next().expect("method has a name");
        if !method.starts_with('_') {
            methods.insert(method.to_owned());
        }
    }
    methods
}

fn python_class_field_names(source: &str, name: &str, required_only: bool) -> Vec<String> {
    let body = python_class_body(source, name);
    let member_depth = python_member_depth(body);
    body.lines()
        .filter(|line| line.len() - line.trim_start_matches([' ', '\t']).len() == member_depth)
        .filter_map(|line| {
            let member = line.trim();
            let (field, annotation) = member.split_once(':')?;
            if !is_identifier(field)
                || field.starts_with('_')
                || annotation
                    .split(|character: char| character != '_' && !character.is_ascii_alphanumeric())
                    .any(|part| part == "ClassVar")
                || (required_only && annotation.contains('='))
            {
                return None;
            }
            Some(field.to_owned())
        })
        .collect()
}

fn python_value_members(source: &str, name: &str, surface: PythonValueSurface) -> BTreeSet<String> {
    if matches!(surface, PythonValueSurface::Int) {
        let expected = format!("{name} = int");
        let actual = source
            .lines()
            .filter(|line| line.trim_start().starts_with(&format!("{name} =")))
            .map(str::trim)
            .collect::<Vec<_>>();
        assert_eq!(
            actual,
            vec![expected.as_str()],
            "Python {name} must be represented exactly as int"
        );
        return BTreeSet::from([format!(
            "{}:{name}",
            BindingMemberKind::IntRepresentation.label()
        )]);
    }

    let classmethods = python_classmethod_names(source, name);
    let properties = python_property_names(source, name);
    let staticmethods = python_staticmethod_names(source, name);
    assert!(
        classmethods.is_disjoint(&properties)
            && classmethods.is_disjoint(&staticmethods)
            && properties.is_disjoint(&staticmethods),
        "Python {name} method decorators must select one call form"
    );
    let mut members = python_class_methods(source, name, false)
        .into_iter()
        .map(|method| {
            let kind = if classmethods.contains(&method) {
                BindingMemberKind::ClassMethod
            } else if staticmethods.contains(&method) {
                BindingMemberKind::StaticMethod
            } else if properties.contains(&method) {
                BindingMemberKind::Property
            } else {
                BindingMemberKind::Method
            };
            let required = if matches!(kind, BindingMemberKind::Property) {
                Vec::new()
            } else {
                python_required_method_parameters(source, name, &method)
            };
            member_shape_key(kind.label(), &method, &required)
        })
        .collect::<BTreeSet<_>>();
    match surface {
        PythonValueSurface::Enum => {
            let declaration = source
                .lines()
                .find(|line| line.starts_with(&format!("class {name}(")))
                .unwrap_or_else(|| panic!("Python enum {name} must exist"));
            assert!(
                parenthesized_contents(declaration, &format!("Python enum {name}"))
                    .split(',')
                    .any(|base| base.trim() == "Enum"),
                "Python {name} enum values require an Enum class"
            );
            assert!(
                !python_enum_values(source, name).is_empty(),
                "Python enum {name} must contain values"
            );
            members.insert(binding_member_key(binding_member(
                BindingMemberKind::EnumValue,
                "value",
            )));
        }
        PythonValueSurface::Dataclass { fields } => {
            let required = python_class_field_names(source, name, true)
                .into_iter()
                .map(|parameter| format!("arg:{parameter}"))
                .collect::<Vec<_>>();
            members.insert(member_shape_key(
                BindingMemberKind::Constructor.label(),
                name,
                &required,
            ));
            if fields {
                members.extend(
                    python_class_field_names(source, name, false)
                        .into_iter()
                        .map(|field| {
                            member_shape_key::<String>(
                                BindingMemberKind::Field.label(),
                                &field,
                                &[],
                            )
                        }),
                );
            }
        }
        PythonValueSurface::String => {
            let declaration = source
                .lines()
                .find(|line| line.starts_with(&format!("class {name}(")))
                .unwrap_or_else(|| panic!("Python string value {name} must exist"));
            assert!(
                parenthesized_contents(declaration, &format!("Python string value {name}"))
                    .split(',')
                    .any(|base| base.trim() == "str"),
                "Python {name} string representation requires a str class"
            );
            members.insert(member_shape_key(
                BindingMemberKind::StringRepresentation.label(),
                name,
                &["pos:value"],
            ));
        }
        PythonValueSurface::Class | PythonValueSurface::Int => {}
    }
    members
}

fn python_enum_values(source: &str, name: &str) -> Vec<String> {
    let marker = format!("class {name}(");
    let mut found = false;
    let mut values = Vec::new();
    for line in source.lines() {
        if !found {
            found = line.starts_with(&marker);
            continue;
        }
        if !line.is_empty() && !line.starts_with(' ') {
            break;
        }
        let Some(member) = line.strip_prefix("    ") else {
            continue;
        };
        if member.starts_with(' ') {
            continue;
        }
        if let Some((_, value)) = member.split_once(" = ") {
            values.push(value.trim_matches('"').to_owned());
        }
    }
    assert!(found, "Python enum {name} must exist");
    values
}

fn python_enum_names(source: &str, name: &str) -> BTreeSet<String> {
    let marker = format!("class {name}(");
    let mut found = false;
    let mut names = BTreeSet::new();
    for line in source.lines() {
        if !found {
            found = line.starts_with(&marker);
            continue;
        }
        if !line.is_empty() && !line.starts_with(' ') {
            break;
        }
        let Some(member) = line.strip_prefix("    ") else {
            continue;
        };
        if member.starts_with(' ') {
            continue;
        }
        if let Some((name, _)) = member.split_once(" = ") {
            names.insert(name.to_ascii_lowercase());
        }
    }
    assert!(found, "Python enum {name} must exist");
    names
}

fn python_function<'a>(source: &'a str, name: &str) -> &'a str {
    let marker = format!("def {name}(");
    let start = source
        .lines()
        .enumerate()
        .find_map(|(index, line)| line.starts_with(&marker).then_some(index))
        .unwrap_or_else(|| panic!("Python function {name} must exist"));
    let lines = source.lines().collect::<Vec<_>>();
    let end = lines
        .iter()
        .enumerate()
        .skip(start + 1)
        .find_map(|(index, line)| (!line.is_empty() && !line.starts_with(' ')).then_some(index))
        .unwrap_or(lines.len());
    let start_offset: usize = lines[..=start].iter().map(|line| line.len() + 1).sum();
    let end_offset: usize = lines[..end].iter().map(|line| line.len() + 1).sum();
    &source[start_offset..end_offset.min(source.len())]
}

fn python_event_kinds(source: &str, function: &str) -> BTreeSet<String> {
    let function = python_function(source, function);
    let start = function
        .find("model = {")
        .expect("event decoder must contain a model map")
        + "model = {".len();
    let end = function[start..]
        .find("}.get(kind)")
        .expect("event decoder model map must end")
        + start;
    let mut values = function[start..end]
        .split(',')
        .filter_map(|entry| entry.split_once(':').map(|(key, _)| key))
        .map(str::trim)
        .map(|key| key.trim_matches('"').to_owned())
        .collect::<BTreeSet<_>>();
    values.insert("reconnected".to_owned());
    values
}

fn python_error_kinds(source: &str) -> BTreeSet<String> {
    python_function(source, "_error_from_wire")
        .lines()
        .filter_map(|line| line.trim().strip_prefix("if kind == \""))
        .filter_map(|value| value.split_once('"').map(|(kind, _)| kind.to_owned()))
        .collect()
}

fn python_assigned_constructor_values(source: &str, prefix: &str) -> BTreeSet<String> {
    source
        .lines()
        .filter(|line| line.starts_with(prefix))
        .filter_map(|line| line.split_once("(\"").map(|(_, value)| value))
        .filter_map(|value| value.split_once('"').map(|(value, _)| value.to_owned()))
        .collect()
}

fn dart_block<'a>(source: &'a str, marker: &str) -> &'a str {
    let start = source
        .find(marker)
        .unwrap_or_else(|| panic!("missing {marker}"));
    let source = &source[start..];
    let open = source
        .find('{')
        .expect("Dart declaration must open a block");
    let mut depth = 0_usize;
    for (offset, character) in source[open..].char_indices() {
        match character {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &source[open + 1..open + offset];
                }
            }
            _ => {}
        }
    }
    panic!("Dart declaration {marker} must close its block")
}

fn dart_class_name(line: &str) -> Option<&str> {
    if line.chars().next().is_some_and(char::is_whitespace) {
        return None;
    }
    let mut declaration = line;
    loop {
        let (modifier, rest) = declaration.split_once(' ')?;
        if matches!(
            modifier,
            "abstract" | "base" | "final" | "interface" | "sealed"
        ) {
            declaration = rest.trim_start();
        } else {
            break;
        }
    }
    declaration.strip_prefix("class ").map(|rest| {
        let end = rest
            .find(|character: char| character != '_' && !character.is_ascii_alphanumeric())
            .unwrap_or(rest.len());
        &rest[..end]
    })
}

fn find_dart_class_body<'a>(source: &'a str, name: &str) -> Option<&'a str> {
    let mut offset = 0;
    for line in source.split_inclusive('\n') {
        let declaration = line.trim_end_matches(['\r', '\n']);
        if dart_class_name(declaration) == Some(name) {
            return Some(dart_block(&source[offset..], declaration));
        }
        offset += line.len();
    }
    None
}

#[derive(Default)]
struct DartNesting {
    angle: u32,
    parentheses: u32,
    brackets: u32,
    braces: u32,
    quote: Option<char>,
    escaped: bool,
}

impl DartNesting {
    fn is_top_level(&self) -> bool {
        self.angle == 0 && self.parentheses == 0 && self.brackets == 0 && self.braces == 0
    }

    fn is_member_top_level(&self) -> bool {
        self.parentheses == 0 && self.brackets == 0 && self.braces == 0
    }

    fn consume(&mut self, character: char) -> bool {
        if let Some(active_quote) = self.quote {
            if self.escaped {
                self.escaped = false;
            } else if character == '\\' {
                self.escaped = true;
            } else if character == active_quote {
                self.quote = None;
            }
            return false;
        }
        match character {
            '\'' | '"' => {
                self.quote = Some(character);
                return false;
            }
            '<' => self.angle += 1,
            '>' => self.angle = self.angle.saturating_sub(1),
            '(' => self.parentheses += 1,
            ')' => self.parentheses = self.parentheses.saturating_sub(1),
            '[' => self.brackets += 1,
            ']' => self.brackets = self.brackets.saturating_sub(1),
            '{' => self.braces += 1,
            '}' => self.braces = self.braces.saturating_sub(1),
            _ => {}
        }
        true
    }
}

fn parenthesized_contents<'a>(source: &'a str, label: &str) -> &'a str {
    let open = source
        .find('(')
        .unwrap_or_else(|| panic!("{label} signature must have parameters"));
    let mut nesting = DartNesting::default();
    for (offset, character) in source[open..].char_indices() {
        nesting.consume(character);
        if character == ')' && nesting.parentheses == 0 {
            return &source[open + 1..open + offset];
        }
    }
    panic!("{label} signature has unterminated parameters")
}

fn split_top_level_commas(source: &str) -> Vec<&str> {
    let mut values = Vec::new();
    let mut start = 0;
    let mut nesting = DartNesting::default();
    for (offset, character) in source.char_indices() {
        let top_level = nesting.is_top_level();
        if nesting.consume(character) && top_level && character == ',' {
            values.push(&source[start..offset]);
            start = offset + character.len_utf8();
        }
    }
    values.push(&source[start..]);
    values
}

fn dart_has_top_level_comma(declaration: &str) -> bool {
    let mut nesting = DartNesting::default();
    declaration.chars().any(|character| {
        let top_level = nesting.is_top_level();
        nesting.consume(character) && top_level && character == ','
    })
}

fn dart_is_constructor(declaration: &str, name: &str) -> bool {
    let mut declaration = declaration.trim_start();
    while let Some((modifier, rest)) = declaration.split_once(' ') {
        if matches!(modifier, "const" | "external" | "factory") {
            declaration = rest.trim_start();
        } else {
            break;
        }
    }
    declaration.strip_prefix(name).is_some_and(|rest| {
        let rest = rest.trim_start();
        rest.starts_with('(') || rest.starts_with('.')
    })
}

fn dart_instance_field_name(declaration: &str, class_name: &str) -> Option<String> {
    let declaration = declaration.trim().trim_end_matches(';').trim();
    if dart_is_constructor(declaration, class_name) {
        return None;
    }
    let field_declaration = declaration
        .split_once('=')
        .map_or(declaration, |(before, _)| before)
        .trim();
    let tokens = field_declaration.split_whitespace().collect::<Vec<_>>();
    if tokens.contains(&"static")
        || tokens
            .iter()
            .any(|token| matches!(*token, "get" | "set" | "operator" | "typedef"))
    {
        return None;
    }
    tokens
        .last()
        .filter(|field| is_identifier(field))
        .map(|field| snake_case(field))
}

fn dart_top_level_members(block: &str, class_name: &str) -> Vec<String> {
    let mut members = Vec::new();
    let mut member = String::new();
    let mut nesting = DartNesting::default();
    let mut assignment_prefix = None;
    let mut member_block_header = None;

    for line in block.lines() {
        let line = line.trim();
        if line.is_empty() || (nesting.quote.is_none() && line.starts_with("//")) {
            continue;
        }
        if !member.is_empty() {
            member.push(' ');
        }
        for character in line.chars() {
            let top_level = nesting.is_member_top_level();
            let syntax = nesting.consume(character);
            if syntax && top_level && character == '=' && assignment_prefix.is_none() {
                assignment_prefix = Some(member.clone());
            }
            if syntax && top_level && character == '{' && assignment_prefix.is_none() {
                member_block_header = Some(member.trim().to_owned());
            }
            member.push(character);
            if !syntax {
                continue;
            }
            if character == '}' && nesting.is_member_top_level() {
                if let Some(declaration) = member_block_header.take() {
                    member.clear();
                    if !declaration.is_empty() {
                        members.push(declaration);
                    }
                    assignment_prefix = None;
                }
            } else if character == ';' && nesting.is_member_top_level() {
                let declaration = std::mem::take(&mut member);
                if !declaration.trim_matches(';').trim().is_empty() {
                    members.push(declaration);
                }
                assignment_prefix = None;
                member_block_header = None;
            }
        }
    }
    assert!(
        member.trim().is_empty(),
        "Dart class {class_name} has an unterminated top-level member"
    );
    members
}

fn dart_extension_target(declaration: &str) -> Option<&str> {
    let declaration = declaration.strip_prefix("extension ")?;
    let target = declaration
        .strip_prefix("on ")
        .or_else(|| declaration.split_once(" on ").map(|(_, target)| target))?;
    let end = target
        .find(|character: char| character != '_' && !character.is_ascii_alphanumeric())
        .unwrap_or(target.len());
    Some(&target[..end])
}

fn dart_extension_bodies<'a>(source: &'a str, name: &str) -> Vec<&'a str> {
    let mut bodies = Vec::new();
    let mut offset = 0;
    for line in source.split_inclusive('\n') {
        let declaration = line.trim_end_matches(['\r', '\n']);
        if !declaration.starts_with(char::is_whitespace)
            && dart_extension_target(declaration) == Some(name)
        {
            bodies.push(dart_block(&source[offset..], declaration));
        }
        offset += line.len();
    }
    assert!(!bodies.is_empty(), "Dart extension on {name} must exist");
    bodies
}

fn dart_value_member(
    declaration: &str,
    class_name: &str,
    include_fields: bool,
    include_constructor: bool,
) -> Option<String> {
    let declaration = declaration.trim().trim_end_matches(';').trim();
    if declaration.starts_with("@override ") {
        return None;
    }
    assert!(
        !declaration.starts_with('@'),
        "Dart {class_name} annotated member must be classified explicitly: {declaration}"
    );

    let mut unmodified = declaration;
    for modifier in ["const ", "external "] {
        if let Some(rest) = unmodified.strip_prefix(modifier) {
            unmodified = rest.trim_start();
        }
    }
    if let Some(factory) = unmodified.strip_prefix("factory ") {
        let factory = factory.strip_prefix(class_name)?.strip_prefix('.')?;
        let name = factory.split_once('(')?.0.trim();
        return (!name.starts_with('_') && is_identifier(name)).then(|| {
            member_shape_key(
                BindingMemberKind::Factory.label(),
                name,
                &dart_required_parameter_names(declaration, &format!("Dart {class_name}.{name}")),
            )
        });
    }

    if dart_is_constructor(declaration, class_name) {
        if !include_constructor {
            return None;
        }
        let constructor = unmodified.strip_prefix(class_name)?;
        let name = if constructor.trim_start().starts_with('(') {
            class_name
        } else {
            let name = constructor.trim_start().strip_prefix('.')?;
            name.split_once('(')?.0.trim()
        };
        if name.starts_with('_') || !is_identifier(name) {
            return None;
        }
        return Some(member_shape_key(
            BindingMemberKind::Constructor.label(),
            name,
            &dart_required_parameter_names(declaration, &format!("Dart {class_name}.{name}")),
        ));
    }

    let tokens = declaration.split_whitespace().collect::<Vec<_>>();
    if let Some(getter) = tokens.iter().position(|token| *token == "get") {
        let name = tokens.get(getter + 1)?.trim_end_matches([';', '{']);
        let kind = if tokens.contains(&"static") {
            BindingMemberKind::StaticGetter
        } else {
            BindingMemberKind::Getter
        };
        return (!name.starts_with('_') && is_identifier(name))
            .then(|| format!("{}:{name}", kind.label()));
    }
    if tokens.contains(&"operator") {
        return None;
    }
    if let Some((prefix, _)) = declaration.split_once('(') {
        let name = prefix.split_whitespace().next_back()?;
        let kind = if tokens.contains(&"static") {
            BindingMemberKind::StaticMethod
        } else {
            BindingMemberKind::Method
        };
        return (!name.starts_with('_') && is_identifier(name)).then(|| {
            member_shape_key(
                kind.label(),
                name,
                &dart_required_parameter_names(declaration, &format!("Dart {class_name}.{name}")),
            )
        });
    }
    if !include_fields {
        return None;
    }
    if tokens.contains(&"static") {
        return None;
    }

    let field = declaration
        .split_once('=')
        .map_or(declaration, |(before, _)| before)
        .split_whitespace()
        .next_back()?;
    (!field.starts_with('_') && is_identifier(field))
        .then(|| format!("{}:{field}", BindingMemberKind::Field.label()))
}

fn dart_value_members(source: &str, name: &str, surface: DartValueSurface) -> BTreeSet<String> {
    let (blocks, include_fields, include_constructor) = match surface {
        DartValueSurface::Class { fields } => (
            vec![
                find_dart_class_body(source, name)
                    .unwrap_or_else(|| panic!("Dart class {name} must exist")),
            ],
            fields,
            false,
        ),
        DartValueSurface::ConstructibleClass { fields } => (
            vec![
                find_dart_class_body(source, name)
                    .unwrap_or_else(|| panic!("Dart class {name} must exist")),
            ],
            fields,
            true,
        ),
        DartValueSurface::Extension => (dart_extension_bodies(source, name), false, false),
    };
    let mut members = BTreeSet::new();
    for block in blocks {
        for declaration in dart_top_level_members(block, name) {
            let Some(member) =
                dart_value_member(&declaration, name, include_fields, include_constructor)
            else {
                continue;
            };
            assert!(
                members.insert(member.clone()),
                "Dart {name} repeats value member {member}"
            );
        }
    }
    members
}

fn dart_class_fields(source: &str, name: &str) -> BTreeSet<String> {
    let block = find_dart_class_body(source, name)
        .unwrap_or_else(|| panic!("Dart class {name} must exist"));
    dart_top_level_members(block, name)
        .into_iter()
        .filter_map(|declaration| {
            let field = dart_instance_field_name(&declaration, name)?;
            assert!(
                !dart_has_top_level_comma(&declaration),
                "Dart class {name} has a comma-separated field declaration; split it into one field per line"
            );
            Some(field)
        })
        .collect()
}

fn dart_parameters(declaration: &str, label: &str, required_only: bool) -> Vec<String> {
    #[derive(Clone, Copy)]
    enum Group {
        Positional,
        Named,
        Optional,
    }

    fn collect(
        source: &str,
        label: &str,
        required_only: bool,
        group: Group,
        seen: &mut BTreeMap<String, String>,
        parameters: &mut Vec<String>,
    ) {
        for parameter in split_top_level_commas(source) {
            let parameter = parameter.trim();
            if parameter.is_empty() {
                continue;
            }
            if let Some(nested) = parameter
                .strip_prefix('{')
                .and_then(|parameter| parameter.strip_suffix('}'))
            {
                collect(nested, label, required_only, Group::Named, seen, parameters);
                continue;
            }
            if let Some(nested) = parameter
                .strip_prefix('[')
                .and_then(|parameter| parameter.strip_suffix(']'))
            {
                collect(
                    nested,
                    label,
                    required_only,
                    Group::Optional,
                    seen,
                    parameters,
                );
                continue;
            }

            let required = match group {
                Group::Positional => !parameter.contains('='),
                Group::Named => parameter.starts_with("required "),
                Group::Optional => false,
            };
            if required_only && !required {
                continue;
            }

            let declaration = parameter
                .split_once('=')
                .map_or(parameter, |(value, _)| value);
            let identifier = declaration
                .split_whitespace()
                .next_back()
                .unwrap_or_else(|| panic!("{label} parameter must have an identifier"));
            let identifier = identifier
                .strip_prefix("this.")
                .or_else(|| identifier.strip_prefix("super."))
                .unwrap_or(identifier);
            assert!(
                is_identifier(identifier),
                "{label} parameter must use an identifier: {parameter}"
            );
            let identifier = insert_normalized_parameter(seen, label, identifier);
            let group = match group {
                Group::Positional => "pos",
                Group::Named => "named",
                Group::Optional => "optional",
            };
            parameters.push(format!("{group}:{identifier}"));
        }
    }

    let mut seen = BTreeMap::new();
    let mut parameters = Vec::new();
    collect(
        parenthesized_contents(declaration, label),
        label,
        required_only,
        Group::Positional,
        &mut seen,
        &mut parameters,
    );
    parameters
}

fn dart_parameter_names(declaration: &str, label: &str) -> BTreeSet<String> {
    dart_parameters(declaration, label, false)
        .into_iter()
        .map(|parameter| {
            parameter
                .split_once(':')
                .expect("Dart parameter shape has a group")
                .1
                .to_owned()
        })
        .collect()
}

fn dart_required_parameter_names(declaration: &str, label: &str) -> Vec<String> {
    dart_parameters(declaration, label, true)
}

fn dart_factory_parameters(source: &str, class_name: &str) -> BTreeMap<String, BTreeSet<String>> {
    let block = find_dart_class_body(source, class_name)
        .unwrap_or_else(|| panic!("Dart class {class_name} must exist"));
    let mut factories = BTreeMap::new();
    for declaration in dart_top_level_members(block, class_name) {
        let mut declaration = declaration.trim_start();
        for modifier in ["const ", "external "] {
            if let Some(rest) = declaration.strip_prefix(modifier) {
                declaration = rest.trim_start();
            }
        }
        let Some(declaration) = declaration.strip_prefix("factory ") else {
            continue;
        };
        let Some(declaration) = declaration.strip_prefix(class_name) else {
            continue;
        };
        let declaration = declaration.trim_start();
        let factory = if declaration.starts_with('(') {
            "new"
        } else {
            let Some(factory) = declaration.strip_prefix('.') else {
                continue;
            };
            factory
                .split_once('(')
                .map(|(factory, _)| factory)
                .unwrap_or(factory)
        };
        if factory.starts_with('_') {
            continue;
        }
        assert!(
            is_identifier(factory),
            "Dart {class_name} factory must use an identifier: {factory}"
        );
        let parameters =
            dart_parameter_names(declaration, &format!("Dart {class_name}.{factory} factory"));
        assert!(
            factories.insert(snake_case(factory), parameters).is_none(),
            "Dart {class_name} has duplicate factory {factory}"
        );
    }
    factories
}

fn dart_self_returning_method_parameters(
    source: &str,
    class_name: &str,
) -> BTreeMap<String, BTreeSet<String>> {
    let block = find_dart_class_body(source, class_name)
        .unwrap_or_else(|| panic!("Dart class {class_name} must exist"));
    let mut methods = BTreeMap::new();
    for declaration in dart_top_level_members(block, class_name) {
        let Some((prefix, _)) = declaration.split_once('(') else {
            continue;
        };
        let mut tokens = prefix.split_whitespace().rev();
        let Some(method) = tokens.next() else {
            continue;
        };
        let Some(return_type) = tokens.next() else {
            continue;
        };
        if return_type != class_name || !method.starts_with("with") || method.starts_with('_') {
            continue;
        }
        assert!(
            is_identifier(method),
            "Dart {class_name} self-returning method must use an identifier: {method}"
        );
        let parameters =
            dart_parameter_names(&declaration, &format!("Dart {class_name}.{method} method"));
        assert!(
            methods.insert(snake_case(method), parameters).is_none(),
            "Dart {class_name} has duplicate self-returning method {method}"
        );
    }
    methods
}

fn dart_construction_method_parameters(
    source: &str,
    class_name: &str,
) -> BTreeMap<String, BTreeSet<String>> {
    let mut methods = dart_factory_parameters(source, class_name);
    for (method, parameters) in dart_self_returning_method_parameters(source, class_name) {
        assert!(
            methods.insert(method.clone(), parameters).is_none(),
            "Dart {class_name} has duplicate construction method {method}"
        );
    }
    methods
}

fn dart_public_model_fields(name: &str) -> BTreeSet<String> {
    let sources = [DART_MODELS, DART_PROVIDERS]
        .into_iter()
        .filter(|source| find_dart_class_body(source, name).is_some())
        .collect::<Vec<_>>();
    assert!(
        sources.len() == 1,
        "Dart class {name} must exist in exactly one of models.dart and providers.dart; found {}",
        sources.len()
    );
    dart_class_fields(sources[0], name)
}

fn assert_inventory(label: &str, expected: &BTreeSet<String>, actual: &BTreeSet<String>) {
    let missing = expected
        .difference(actual)
        .cloned()
        .collect::<BTreeSet<_>>();
    let extra = actual
        .difference(expected)
        .cloned()
        .collect::<BTreeSet<_>>();
    assert!(
        missing.is_empty() && extra.is_empty(),
        "{label} differ; missing: {missing:?}; extra: {extra:?}"
    );
}

fn assert_exact_public_methods(label: &str, source: &str, name: &str, allowed: &[&str]) {
    assert_inventory(
        label,
        &allowed.iter().map(|method| (*method).to_owned()).collect(),
        &rust_public_value_methods(source, name),
    );
}

fn dart_enum_values(source: &str, name: &str) -> Vec<String> {
    dart_block(source, &format!("enum {name}"))
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn dart_factory_names(source: &str, marker: &str) -> BTreeSet<String> {
    let type_name = marker.split_whitespace().last().expect("type name");
    let prefix = format!("const factory {type_name}.");
    dart_block(source, marker)
        .lines()
        .filter_map(|line| line.trim().strip_prefix(&prefix))
        .filter_map(|line| line.split_once('(').map(|(name, _)| name.to_owned()))
        .collect()
}

fn dart_static_and_factory_names(source: &str, marker: &str) -> BTreeSet<String> {
    let type_name = marker.split_whitespace().last().expect("type name");
    let factory_prefix = format!("factory {type_name}.");
    dart_block(source, marker)
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if let Some(value) = line.strip_prefix("static const ") {
                return value.split_whitespace().next().map(ToOwned::to_owned);
            }
            line.strip_prefix(&factory_prefix)
                .and_then(|value| value.split_once('(').map(|(name, _)| name.to_owned()))
        })
        .collect()
}

fn dart_class_members(source: &str, marker: &str, future_only: bool) -> BTreeSet<String> {
    let block = dart_block(source, marker);
    let mut depth = 0_isize;
    let mut members = BTreeSet::new();
    for line in block.lines() {
        let trimmed = line.trim();
        if depth == 0 && !trimmed.is_empty() {
            let name = if !future_only && trimmed.starts_with("final ") && trimmed.ends_with(';') {
                trimmed.trim_end_matches(';').split_whitespace().next_back()
            } else if !future_only {
                trimmed
                    .split_once(" get ")
                    .map(|(_, value)| value.split_whitespace().next().unwrap_or(value))
            } else {
                None
            }
            .or_else(|| {
                if future_only && !trimmed.starts_with("Future<") {
                    return None;
                }
                let prefix = trimmed.split_once('(')?.0;
                prefix.split_whitespace().next_back()
            });
            if let Some(name) = name.filter(|name| {
                !name.starts_with('_')
                    && !name.contains('.')
                    && !matches!(*name, "if" | "for" | "switch" | "while")
                    && !name
                        .chars()
                        .next()
                        .is_some_and(|character| character.is_ascii_uppercase())
            }) {
                members.insert(name.trim_end_matches(';').to_owned());
            }
        }
        depth += line.chars().filter(|value| *value == '{').count() as isize;
        depth -= line.chars().filter(|value| *value == '}').count() as isize;
    }
    members
}

fn camel_case(value: &str) -> String {
    let mut result = String::new();
    let mut uppercase = false;
    for character in value.chars() {
        if character == '_' {
            uppercase = true;
        } else if uppercase {
            result.extend(character.to_uppercase());
            uppercase = false;
        } else {
            result.push(character);
        }
    }
    result
}

fn snake_case(value: &str) -> String {
    let mut result = String::new();
    for character in value.chars() {
        if character.is_ascii_uppercase() {
            if !result.is_empty() {
                result.push('_');
            }
            result.push(character.to_ascii_lowercase());
        } else {
            result.push(character);
        }
    }
    result
}

fn insert_normalized_parameter(
    parameters: &mut BTreeMap<String, String>,
    label: &str,
    original: &str,
) -> String {
    let normalized = snake_case(original);
    if let Some(previous) = parameters.insert(normalized.clone(), original.to_owned()) {
        panic!("{label} parameters {previous} and {original} normalize to duplicate {normalized}");
    }
    normalized
}

fn rust_variant_ids(source: &str, name: &str) -> BTreeSet<String> {
    rust_enum_variants(source, name)
        .into_iter()
        .map(|value| snake_case(&value))
        .collect()
}

fn dart_variant_ids(source: &str, name: &str) -> BTreeSet<String> {
    dart_enum_values(source, name)
        .into_iter()
        .map(|value| snake_case(&value))
        .collect()
}

fn assert_provider_constructor_settings() {
    let exchanges = Exchange::ALL
        .map(|exchange| exchange.id().to_owned())
        .into_iter()
        .collect::<BTreeSet<_>>();
    let descriptors = PROVIDERS
        .iter()
        .map(|provider| provider.exchange.to_owned())
        .collect::<BTreeSet<_>>();
    assert_inventory("Provider descriptors", &exchanges, &descriptors);

    let mut dart_native_constructors =
        rust_public_associated_constructors(DART_RUST_API, "NativeClient");
    for exclusion in DART_NATIVE_CONSTRUCTOR_EXCLUSIONS {
        assert!(
            dart_native_constructors.remove(*exclusion).is_some(),
            "Dart NativeClient intentional constructor exclusion {exclusion} must exist"
        );
    }
    let expected_dart_native_constructors = PROVIDERS
        .iter()
        .flat_map(|provider| provider.dart_native_methods.iter())
        .map(|method| (*method).to_owned())
        .collect::<BTreeSet<_>>();
    assert_inventory(
        "Dart NativeClient provider constructors",
        &expected_dart_native_constructors,
        &dart_native_constructors.keys().cloned().collect(),
    );

    for provider in PROVIDERS {
        let rust_methods = rust_self_returning_methods(provider.rust_source, provider.adapter);
        let rust_settings = rust_constructor_settings(
            provider.adapter,
            &rust_methods,
            provider.modes,
            provider.mode_axis,
        );
        let rust_modes = provider
            .modes
            .iter()
            .map(|mode| (*mode).to_owned())
            .collect::<BTreeSet<_>>();

        let python_native = rust_method_parameters(PYTHON_BUILTIN, provider.python_native, "new");
        assert_inventory(
            &format!(
                "Python {}::new constructor settings",
                provider.python_native
            ),
            &rust_settings,
            &python_native,
        );

        for method in provider.dart_native_methods {
            let mut expected = rust_settings.clone();
            if provider.dart_native_methods.len() > 1 {
                expected.remove(
                    provider
                        .mode_axis
                        .expect("split Dart native constructors need a mode axis"),
                );
            }
            assert_inventory(
                &format!("Dart NativeClient::{method} constructor settings"),
                &expected,
                dart_native_constructors
                    .get(*method)
                    .unwrap_or_else(|| panic!("Dart NativeClient::{method} must exist")),
            );
        }

        let python_settings =
            python_method_parameters(PYTHON_ADAPTERS, provider.adapter, "__init__");
        assert_inventory(
            &format!("Python {} constructor settings", provider.adapter),
            &rust_settings,
            &python_settings,
        );

        let python_factories = python_classmethod_parameters(PYTHON_ADAPTERS, provider.adapter);
        let expected_python_factories = provider
            .python_factories
            .iter()
            .map(|factory| (*factory).to_owned())
            .collect::<BTreeSet<_>>();
        assert_inventory(
            &format!("Python {} classmethod factories", provider.adapter),
            &expected_python_factories,
            &python_factories.keys().cloned().collect(),
        );
        let mut python_modes = python_factories.keys().cloned().collect::<BTreeSet<_>>();
        python_modes.insert(provider.python_default_mode.to_owned());
        assert_inventory(
            &format!("Python {} constructor modes", provider.adapter),
            &rust_modes,
            &python_modes,
        );
        for (factory, parameters) in python_factories {
            let mut expected = rust_settings.clone();
            expected.remove(
                provider
                    .mode_axis
                    .expect("Python constructor factories need a mode axis"),
            );
            assert_inventory(
                &format!("Python {}.{factory} factory settings", provider.adapter),
                &expected,
                &parameters,
            );
        }

        let dart_factories = dart_factory_parameters(DART_ADAPTERS, provider.adapter);
        let dart_methods = dart_construction_method_parameters(DART_ADAPTERS, provider.adapter);
        let rust_method_names = rust_methods.keys().cloned().collect::<BTreeSet<_>>();
        let dart_method_names = dart_methods.keys().cloned().collect::<BTreeSet<_>>();
        assert_inventory(
            &format!("Dart {} construction methods", provider.adapter),
            &rust_method_names,
            &dart_method_names,
        );

        let dart_modes = dart_factories
            .keys()
            .filter(|factory| rust_modes.contains(*factory))
            .cloned()
            .collect::<BTreeSet<_>>();
        assert_inventory(
            &format!("Dart {} constructor modes", provider.adapter),
            &rust_modes,
            &dart_modes,
        );

        let mut dart_settings = dart_methods
            .values()
            .flatten()
            .cloned()
            .collect::<BTreeSet<_>>();
        if provider.modes.len() > 1 {
            dart_settings.insert(
                provider
                    .mode_axis
                    .expect("multiple Dart factories need a mode axis")
                    .to_owned(),
            );
        }
        assert_inventory(
            &format!("Dart {} constructor settings", provider.adapter),
            &rust_settings,
            &dart_settings,
        );
    }
}

#[test]
fn rust_struct_field_parser_includes_private_named_fields() {
    let source = r#"
pub struct Subscription {
    pub markets: Vec<String>,
    feeds: Vec<String>,
}
"#;

    assert_eq!(
        rust_struct_fields(source, "Subscription"),
        BTreeSet::from(["feeds".to_owned(), "markets".to_owned()])
    );
}

#[test]
fn python_enum_value_parser_preserves_declaration_order() {
    let python = r#"
class Exchange(str, Enum):
    UPBIT = "upbit"
    BITHUMB = "bithumb"
"#;

    assert_eq!(
        python_enum_values(python, "Exchange")
            .into_iter()
            .collect::<Vec<_>>(),
        ["upbit", "bithumb"]
    );
}

#[test]
fn dart_enum_value_parser_preserves_declaration_order() {
    let dart = r#"
enum Exchange {
  upbit,
  bithumb,
}
"#;

    assert_eq!(
        dart_enum_values(dart, "Exchange")
            .into_iter()
            .collect::<Vec<_>>(),
        ["upbit", "bithumb"]
    );
}

#[test]
fn rust_constructor_parser_extracts_self_returning_method_settings() {
    let source = r#"
struct Example;

impl Example {
    pub fn new() -> Self {
        Self
    }

    pub fn with_region(self, region: String) -> Self {
        self
    }

    pub fn with_credentials(
        self,
        access_key: String,
        secret_key: String,
    ) -> Self {
        self
    }

    pub fn operation(&self, request: String) {
        drop(request);
    }

    fn private_constructor() -> Self {
        Self
    }
}
"#;

    let methods = rust_self_returning_methods(source, "Example");
    assert_eq!(
        methods,
        BTreeMap::from([
            ("new".to_owned(), BTreeSet::new()),
            (
                "with_credentials".to_owned(),
                BTreeSet::from(["access_key".to_owned(), "secret_key".to_owned()]),
            ),
            (
                "with_region".to_owned(),
                BTreeSet::from(["region".to_owned()]),
            ),
        ])
    );
    assert_eq!(
        rust_constructor_settings("Example", &methods, &["new"], None),
        BTreeSet::from([
            "access_key".to_owned(),
            "region".to_owned(),
            "secret_key".to_owned(),
        ])
    );
}

#[test]
#[should_panic(expected = "Rust method with_pair parameter must use an identifier pattern")]
fn rust_constructor_parser_rejects_non_identifier_parameter_patterns() {
    let source = r#"
struct Example;

impl Example {
    pub fn with_pair(self, (access_key, secret_key): (String, String)) -> Self {
        self
    }
}
"#;

    rust_self_returning_methods(source, "Example");
}

#[test]
#[should_panic(
    expected = "Dart NativeClient provider constructors differ; missing: {}; extra: {\"binance_coin_m\"}"
)]
fn dart_native_constructor_inventory_rejects_unlisted_associated_constructor() {
    let source = r#"
struct NativeClient;
struct Error;

impl NativeClient {
    pub fn from_dart_adapter(adapter: String) -> Self {
        drop(adapter);
        Self
    }

    pub fn upbit(region: String) -> Result<Self, Error> {
        drop(region);
        Ok(Self)
    }

    pub fn binance_coin_m(api_key: String) -> Result<Self, Error> {
        drop(api_key);
        Ok(Self)
    }

    pub fn instance(&self) -> Self {
        Self
    }

    pub fn unrelated() -> Result<String, Error> {
        Ok(String::new())
    }

    fn private_constructor() -> Self {
        Self
    }
}
"#;

    let expected = BTreeSet::from(["upbit".to_owned()]);
    let mut constructors = rust_public_associated_constructors(source, "NativeClient");
    for exclusion in DART_NATIVE_CONSTRUCTOR_EXCLUSIONS {
        assert!(
            constructors.remove(*exclusion).is_some(),
            "fixture intentional exclusion {exclusion} must exist"
        );
    }
    let actual = constructors.keys().cloned().collect();
    assert_inventory(
        "Dart NativeClient provider constructors",
        &expected,
        &actual,
    );
}

#[test]
#[should_panic(
    expected = "Dart NativeClient::binance_usd_m_futures constructor settings differ; missing: {\"api_key\"}; extra: {}"
)]
fn dart_native_split_mode_settings_reject_per_mode_missing_parameters() {
    let source = r#"
struct NativeClient;
struct Error;

impl NativeClient {
    pub fn binance_spot(
        api_key: Option<String>,
        secret_key: Option<String>,
    ) -> Result<Self, Error> {
        Ok(Self)
    }

    pub fn binance_usd_m_futures(
        secret_key: Option<String>,
    ) -> Result<Self, Error> {
        Ok(Self)
    }
}
"#;

    let constructors = rust_public_associated_constructors(source, "NativeClient");
    let mut expected = BTreeSet::from([
        "api_key".to_owned(),
        "secret_key".to_owned(),
        "venue".to_owned(),
    ]);
    expected.remove("venue");
    for method in ["binance_spot", "binance_usd_m_futures"] {
        assert_inventory(
            &format!("Dart NativeClient::{method} constructor settings"),
            &expected,
            constructors.get(method).expect("fixture method must exist"),
        );
    }
}

#[test]
#[should_panic(expected = "tuple struct; classify it explicitly")]
fn rust_struct_field_parser_rejects_tuple_structs() {
    rust_struct_fields("pub struct Cursor(String);", "Cursor");
}

#[test]
#[should_panic(expected = "unit struct; classify it explicitly")]
fn rust_struct_field_parser_rejects_unit_structs() {
    rust_struct_fields("pub struct Marker;", "Marker");
}

#[test]
fn python_field_parser_reads_only_instance_annotations_and_wire_names() {
    let source = r#"
class Example(WireModel):
    from_: int = field(default=0, metadata={"wire_name": "from"})
    ordinary_: int
    shared: ClassVar[int] = 0

    def method(self) -> None:
        local: int = 0
"#;

    assert_eq!(
        python_class_fields(source, "Example"),
        BTreeSet::from(["from".to_owned(), "ordinary_".to_owned()])
    );
}

#[test]
fn python_constructor_parser_reads_multiline_parameters_and_factories() {
    let source = r#"
class Example(Base):
    def __init__(
        self,
        /,
        region: Region = Region.default(),
        *,
        accessKey: Optional[str] = None,
        secret_key: Optional[str] = None,
    ) -> None:
        pass

    @classmethod
    def spot(cls) -> Example:
        return cls()

    @classmethod
    def usd_m_futures(
        cls,
        *,
        accessKey: Optional[str] = None,
    ) -> Example:
        return cls(access_key=accessKey)

    @classmethod
    def _private_factory(cls) -> Example:
        return cls()
"#;

    assert_eq!(
        python_method_parameters(source, "Example", "__init__"),
        BTreeSet::from([
            "access_key".to_owned(),
            "region".to_owned(),
            "secret_key".to_owned(),
        ])
    );
    assert_eq!(
        python_classmethod_names(source, "Example"),
        BTreeSet::from(["spot".to_owned(), "usd_m_futures".to_owned()])
    );
}

#[test]
#[should_panic(
    expected = "Python Example.usd_m_futures factory settings differ; missing: {}; extra: {\"factory_only\"}"
)]
fn python_factory_settings_reject_factory_only_parameters() {
    let source = r#"
class Example(Base):
    @classmethod
    def usd_m_futures(
        cls,
        *,
        api_key: Optional[str] = None,
        secret_key: Optional[str] = None,
        factory_only: str = "extra",
    ) -> Example:
        return cls()
"#;

    let factories = python_classmethod_parameters(source, "Example");
    assert_inventory(
        "Python Example classmethod factories",
        &BTreeSet::from(["usd_m_futures".to_owned()]),
        &factories.keys().cloned().collect(),
    );
    assert_inventory(
        "Python Example.usd_m_futures factory settings",
        &BTreeSet::from(["api_key".to_owned(), "secret_key".to_owned()]),
        factories
            .get("usd_m_futures")
            .expect("fixture factory must exist"),
    );
}

#[test]
#[should_panic(
    expected = "Python Example.__init__ parameters accessKey and access_key normalize to duplicate access_key"
)]
fn python_parameter_parser_rejects_normalized_name_collisions() {
    let source = r#"
class Example(Base):
    def __init__(self, accessKey: str, access_key: str) -> None:
        pass
"#;

    python_method_parameters(source, "Example", "__init__");
}

#[test]
fn dart_field_parser_reads_only_top_level_instance_fields() {
    let source = r#"
final class ExampleExtra {
  final String wrongField;
}

final class Example {
  final String lowerCamel;
  final String plain;
  String mutableField;
  late String lateField;
  var state;
  void Function() callback;
  static final String shared = 'shared';
  static String mutableStatic = 'shared';
  final values = <String>[
    'first',
    'second',
  ];
  final compute = () {
    final local = 'inside;{}\'';
    return local;
  };

  bool get derivedValue => true;
  String get label =>
      shared;
  set derivedValue(bool value) => state = value;
  String format() => lowerCamel;
  Example();

  void method() {
    final String localValue = 'local';
  }
}
"#;

    assert_eq!(
        dart_class_fields(source, "Example"),
        BTreeSet::from([
            "callback".to_owned(),
            "compute".to_owned(),
            "late_field".to_owned(),
            "lower_camel".to_owned(),
            "mutable_field".to_owned(),
            "plain".to_owned(),
            "state".to_owned(),
            "values".to_owned(),
        ])
    );
}

#[test]
fn dart_constructor_parser_reads_factories_and_self_returning_methods() {
    let source = r#"
final class Example {
  factory Example({String? accessKey}) => Example._();

  factory Example.testnet({
    String? address,
    String? privateKey,
  }) => Example._();

  Example._();

  Example withCredentials(
    String accessKey,
    String secretKey,
  ) => Example(accessKey: accessKey);

  Future<void> operation(String request) async {}
}
"#;

    let mut methods = dart_factory_parameters(source, "Example");
    methods.extend(dart_self_returning_method_parameters(source, "Example"));
    assert_eq!(
        methods,
        BTreeMap::from([
            ("new".to_owned(), BTreeSet::from(["access_key".to_owned()]),),
            (
                "testnet".to_owned(),
                BTreeSet::from(["address".to_owned(), "private_key".to_owned()]),
            ),
            (
                "with_credentials".to_owned(),
                BTreeSet::from(["access_key".to_owned(), "secret_key".to_owned()]),
            ),
        ])
    );
}

#[test]
fn dart_constructor_parser_reads_block_bodied_members() {
    let source = r#"
final class Example {
  factory Example.extra({String? accessKey}) {
    return Example._();
  }

  Example._();

  Example withCredentials(String accessKey, String secretKey) {
    return Example.extra(accessKey: accessKey);
  }
}
"#;

    assert_eq!(
        dart_construction_method_parameters(source, "Example"),
        BTreeMap::from([
            (
                "extra".to_owned(),
                BTreeSet::from(["access_key".to_owned()]),
            ),
            (
                "with_credentials".to_owned(),
                BTreeSet::from(["access_key".to_owned(), "secret_key".to_owned()]),
            ),
        ])
    );
}

#[test]
#[should_panic(expected = "Dart Example factories differ; missing: {}; extra: {\"extra\"}")]
fn dart_factory_inventory_rejects_block_bodied_extra_factory() {
    let source = r#"
final class Example {
  factory Example.extra(String factoryOnly) {
    return Example._();
  }

  Example._();
}
"#;

    assert_inventory(
        "Dart Example factories",
        &BTreeSet::new(),
        &dart_factory_parameters(source, "Example")
            .keys()
            .cloned()
            .collect(),
    );
}

fn assert_example_value_contract(
    rust_source: &str,
    python_source: &str,
    dart_source: &str,
    members: &[ValueMemberContract],
) {
    assert_value_type_contract(&ValueTypeContract {
        name: "Example",
        rust_source,
        python_source,
        dart_source,
        rust_surface: RustValueSurface::Methods,
        python_surface: PythonValueSurface::Class,
        dart_surface: DartValueSurface::Class { fields: false },
        members,
    });
}

#[test]
#[should_panic(expected = "Rust Example value members differ; missing: {}; extra: {\"future\"}")]
fn value_contract_rejects_unmapped_macro_method() {
    let rust = r#"
macro_rules! define_example {
    () => {
        impl Example {
            pub fn known(&self) {}
            pub fn future(&self) {}
            fn private(&self) {}
        }
    };
}

define_example!();
"#;
    let members = [value_member(
        "known",
        binding_member(BindingMemberKind::Method, "known"),
        binding_member(BindingMemberKind::Getter, "known"),
    )];
    let contract = ValueTypeContract {
        name: "Example",
        rust_source: rust,
        python_source: "class Example(object):\n    def known(self):\n        pass\n",
        dart_source: "final class Example { bool get known => true; }",
        rust_surface: RustValueSurface::Methods,
        python_surface: PythonValueSurface::Class,
        dart_surface: DartValueSurface::Class { fields: false },
        members: &members,
    };

    assert_value_type_contract(&contract);
}

#[test]
#[should_panic(
    expected = "Python Example value members differ; missing: {\"method:known\"}; extra: {}"
)]
fn value_contract_rejects_missing_python_method() {
    let rust = "struct Example; impl Example { pub fn known(&self) {} }";
    let members = [value_member(
        "known",
        binding_member(BindingMemberKind::Method, "known"),
        binding_member(BindingMemberKind::Getter, "known"),
    )];
    let contract = ValueTypeContract {
        name: "Example",
        rust_source: rust,
        python_source: "class Example(object):\n    pass\n",
        dart_source: "final class Example { bool get known => true; }",
        rust_surface: RustValueSurface::Methods,
        python_surface: PythonValueSurface::Class,
        dart_surface: DartValueSurface::Class { fields: false },
        members: &members,
    };

    assert_value_type_contract(&contract);
}

#[test]
#[should_panic(
    expected = "Dart Example value members differ; missing: {\"getter:ready\"}; extra: {\"method:ready\"}"
)]
fn value_contract_rejects_dart_member_kind_mismatch() {
    let rust = "struct Example; impl Example { pub fn ready(&self) -> bool { true } }";
    let members = [value_member(
        "ready",
        binding_member(BindingMemberKind::Method, "ready"),
        binding_member(BindingMemberKind::Getter, "ready"),
    )];
    let contract = ValueTypeContract {
        name: "Example",
        rust_source: rust,
        python_source: "class Example(object):\n    def ready(self):\n        return True\n",
        dart_source: "final class Example { bool ready() => true; }",
        rust_surface: RustValueSurface::Methods,
        python_surface: PythonValueSurface::Class,
        dart_surface: DartValueSurface::Class { fields: false },
        members: &members,
    };

    assert_value_type_contract(&contract);
}

#[test]
#[should_panic(
    expected = "Python Example value members differ; missing: {\"method:display_name\"}; extra: {\"property:display_name\"}"
)]
fn value_contract_rejects_python_property_for_method() {
    let rust = "struct Example; impl Example { pub fn display_name(&self) -> &'static str { \"Example\" } }";
    let members = [value_member(
        "display_name",
        binding_member(BindingMemberKind::Method, "display_name"),
        binding_member(BindingMemberKind::Getter, "displayName"),
    )];
    let contract = ValueTypeContract {
        name: "Example",
        rust_source: rust,
        python_source: "class Example(object):\n    @property\n    def display_name(self):\n        return 'Example'\n",
        dart_source: "final class Example { String get displayName => 'Example'; }",
        rust_surface: RustValueSurface::Methods,
        python_surface: PythonValueSurface::Class,
        dart_surface: DartValueSurface::Class { fields: false },
        members: &members,
    };

    assert_value_type_contract(&contract);
}

#[test]
#[should_panic(expected = "Rust Example value members differ; missing: {}; extra: {\"future\"}")]
fn value_contract_rejects_unmapped_attributed_macro_method() {
    let rust = r#"
macro_rules! define_example {
    () => {
        impl Example {
            pub fn known(&self) {}

            #[doc = "A future public method."]
            #[must_use]
            pub const fn future(&self) -> bool { true }
        }
    };
}

define_example!();
"#;
    let members = [value_member(
        "known",
        binding_member(BindingMemberKind::Method, "known"),
        binding_member(BindingMemberKind::Getter, "known"),
    )];
    let contract = ValueTypeContract {
        name: "Example",
        rust_source: rust,
        python_source: "class Example(object):\n    def known(self):\n        pass\n",
        dart_source: "final class Example { bool get known => true; }",
        rust_surface: RustValueSurface::Methods,
        python_surface: PythonValueSurface::Class,
        dart_surface: DartValueSurface::Class { fields: false },
        members: &members,
    };

    assert_value_type_contract(&contract);
}

#[test]
#[should_panic(expected = "Rust Example value members differ; missing: {}; extra: {\"future\"}")]
fn value_contract_rejects_unmapped_generic_macro_method() {
    let rust = r#"
macro_rules! define_example {
    () => {
        impl Example {
            pub fn known(&self) {}
            pub fn future<T>(&self, value: T) -> T { value }
        }
    };
}

define_example!();
"#;
    let members = [value_member(
        "known",
        binding_member(BindingMemberKind::Method, "known"),
        binding_member(BindingMemberKind::Getter, "known"),
    )];
    let contract = ValueTypeContract {
        name: "Example",
        rust_source: rust,
        python_source: "class Example(object):\n    def known(self):\n        pass\n",
        dart_source: "final class Example { bool get known => true; }",
        rust_surface: RustValueSurface::Methods,
        python_surface: PythonValueSurface::Class,
        dart_surface: DartValueSurface::Class { fields: false },
        members: &members,
    };

    assert_value_type_contract(&contract);
}

#[test]
#[should_panic(
    expected = "Rust Example value member shapes differ; missing: {\"associated:create\"}; extra: {\"instance:create\"}"
)]
fn value_contract_rejects_rust_call_form_mismatch() {
    let members = [shaped_value_member(
        "create",
        "associated:create",
        binding_member(BindingMemberKind::ClassMethod, "create"),
        binding_member(BindingMemberKind::Factory, "create"),
    )];
    assert_example_value_contract(
        "struct Example; impl Example { pub fn create(&self) {} }",
        "class Example(object):\n    @classmethod\n    def create(cls):\n        return cls()\n",
        "final class Example { factory Example.create() => Example._(); Example._(); }",
        &members,
    );
}

#[test]
#[should_panic(
    expected = "Python Example value members differ; missing: {\"class_method:create(arg:value)\"}; extra: {\"class_method:create\"}"
)]
fn value_contract_rejects_required_parameter_drift() {
    let members = [shaped_value_member(
        "create",
        "associated:create(pos:value)",
        binding_member(BindingMemberKind::ClassMethod, "create(arg:value)"),
        binding_member(BindingMemberKind::Factory, "create(pos:value)"),
    )];
    assert_example_value_contract(
        "struct Example; impl Example { pub fn create(value: u32) -> Self { Self } }",
        "class Example(object):\n    @classmethod\n    def create(cls):\n        return cls()\n",
        "final class Example { factory Example.create(int value) => Example._(); Example._(); }",
        &members,
    );
}

#[test]
#[should_panic(expected = "Rust Example value members differ; missing: {}; extra: {\"future\"}")]
fn value_contract_reads_every_macro_impl_for_one_type() {
    let rust = r#"
macro_rules! define_example {
    () => {
        impl Example {
            pub fn known(&self) {}
        }

        impl Example {
            pub fn future(&self) {}
        }
    };
}

define_example!();
"#;
    let members = [value_member(
        "known",
        binding_member(BindingMemberKind::Method, "known"),
        binding_member(BindingMemberKind::Getter, "known"),
    )];
    assert_example_value_contract(
        rust,
        "class Example(object):\n    def known(self):\n        pass\n",
        "final class Example { bool get known => true; }",
        &members,
    );
}

#[test]
fn required_parameter_shapes_preserve_order_and_call_groups() {
    let method =
        syn::parse_str::<ImplItemFn>("pub fn create(second: u8, first: u8) -> Self { Self }")
            .expect("fixture Rust method must parse");
    assert_eq!(
        rust_method_shape(&method),
        "associated:create(pos:second,pos:first)"
    );

    let python =
        "class Example(object):\n    def create(self, first, /, second, *, third):\n        pass\n";
    assert_eq!(
        python_required_method_parameters(python, "Example", "create")
            .into_iter()
            .collect::<Vec<_>>(),
        ["pos:first", "arg:second", "kw:third"]
    );

    let dart = "factory Example.create(int first, {required int second}) => Example._()";
    assert_eq!(
        dart_required_parameter_names(dart, "Dart Example.create"),
        ["pos:first", "named:second"]
    );
}

#[test]
fn dart_value_parser_distinguishes_static_getters() {
    assert_eq!(
        dart_value_member("static bool get ready => true", "Example", false, false).as_deref(),
        Some("static_getter:ready")
    );
}

#[test]
#[should_panic(expected = "Excluded Example methods differ; missing: {}; extra: {\"future\"}")]
fn excluded_public_method_inventory_rejects_drift() {
    assert_exact_public_methods(
        "Excluded Example methods",
        "struct Example; impl Example { pub fn known(&self) {} pub fn future(&self) {} }",
        "Example",
        &["known"],
    );
}

#[test]
fn public_type_inventory_reads_adapter_namespace() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    assert!(
        exported_public_type_sources(&root).contains_key("BinanceListenKey"),
        "public adapter namespace types must participate in method classification"
    );
}

#[test]
#[should_panic(
    expected = "Dart Example.new factory parameters accessKey and access_key normalize to duplicate access_key"
)]
fn dart_parameter_parser_rejects_normalized_name_collisions() {
    let source = r#"
final class Example {
  factory Example({String? accessKey, String? access_key}) => Example._();
}
"#;

    dart_factory_parameters(source, "Example");
}

#[test]
#[should_panic(expected = "comma-separated field declaration")]
fn dart_field_parser_rejects_comma_separated_fields() {
    dart_class_fields(
        "final class Example {\n  String first, second;\n}",
        "Example",
    );
}

#[test]
#[should_panic(
    expected = "Python Example fields differ; missing: {\"missing\"}; extra: {\"extra\"}"
)]
fn inventory_assertion_reports_language_model_missing_and_extra_fields() {
    assert_inventory(
        "Python Example fields",
        &BTreeSet::from(["missing".to_owned(), "shared".to_owned()]),
        &BTreeSet::from(["extra".to_owned(), "shared".to_owned()]),
    );
}

#[test]
fn public_reexports_resolve_same_named_structs_by_module_path() {
    let modules = BTreeMap::from([
        (
            Vec::<String>::new(),
            "pub use crate::right::Record as PublicRecord;".to_owned(),
        ),
        (
            vec!["left".to_owned()],
            "pub struct Record { pub left_value: String }".to_owned(),
        ),
        (
            vec!["right".to_owned()],
            "pub struct Record { pub right_value: String }".to_owned(),
        ),
    ]);

    let exported: BTreeMap<String, (String, String)> =
        resolve_public_struct_sources(&modules, &[Vec::new()]);
    let (rust_name, source) = exported
        .get("PublicRecord")
        .expect("PublicRecord must resolve");

    assert_eq!(rust_name, "Record");
    assert_eq!(
        rust_struct_fields(source, rust_name),
        BTreeSet::from(["right_value".to_owned()])
    );
}

#[test]
fn public_reexports_follow_two_alias_steps() {
    let modules = BTreeMap::from([
        (
            Vec::<String>::new(),
            "pub use self::inner::Public as PublicRecord;".to_owned(),
        ),
        (
            vec!["inner".to_owned()],
            "pub use super::leaf::Internal as Public;".to_owned(),
        ),
        (
            vec!["leaf".to_owned()],
            "pub struct Internal { pub value: String }".to_owned(),
        ),
    ]);

    let exported: BTreeMap<String, (String, String)> =
        resolve_public_struct_sources(&modules, &[Vec::new()]);
    let (rust_name, source) = exported
        .get("PublicRecord")
        .expect("PublicRecord must resolve");

    assert_eq!(rust_name, "Internal");
    assert_eq!(
        rust_struct_fields(source, rust_name),
        BTreeSet::from(["value".to_owned()])
    );
}

#[test]
fn public_module_alias_prefix_resolves_struct_reexport() {
    let modules = BTreeMap::from([
        (
            Vec::<String>::new(),
            "pub use crate::leaf as api;\n\
             pub use api::Record as PublicRecord;"
                .to_owned(),
        ),
        (
            vec!["leaf".to_owned()],
            "pub struct Record { pub public_alias: String }".to_owned(),
        ),
    ]);

    let exported = resolve_public_struct_sources(&modules, &[Vec::new()]);
    let (rust_name, source) = exported
        .get("PublicRecord")
        .expect("PublicRecord must resolve through a public module alias");

    assert!(!exported.contains_key("api"));
    assert_eq!(rust_name, "Record");
    assert_eq!(
        rust_struct_fields(source, rust_name),
        BTreeSet::from(["public_alias".to_owned()])
    );
}

#[test]
fn private_module_alias_prefix_resolves_struct_reexport() {
    let modules = BTreeMap::from([
        (
            Vec::<String>::new(),
            "use crate::leaf as api;\n\
             pub use api::Record as PublicRecord;"
                .to_owned(),
        ),
        (
            vec!["leaf".to_owned()],
            "pub struct Record { pub private_alias: String }".to_owned(),
        ),
    ]);

    let exported = resolve_public_struct_sources(&modules, &[Vec::new()]);
    let (rust_name, source) = exported
        .get("PublicRecord")
        .expect("PublicRecord must resolve through a private module alias");

    assert!(!exported.contains_key("api"));
    assert_eq!(rust_name, "Record");
    assert_eq!(
        rust_struct_fields(source, rust_name),
        BTreeSet::from(["private_alias".to_owned()])
    );
}

#[test]
#[should_panic(expected = "public re-export cycle detected")]
fn public_reexport_cycles_fail_explicitly() {
    let modules = BTreeMap::from([
        (
            Vec::<String>::new(),
            "pub use first::Public as PublicRecord;".to_owned(),
        ),
        (
            vec!["first".to_owned()],
            "pub use super::second::Public;".to_owned(),
        ),
        (
            vec!["second".to_owned()],
            "pub use super::first::Public;".to_owned(),
        ),
    ]);

    resolve_public_struct_sources(&modules, &[Vec::new()]);
}

#[test]
fn public_alias_reexports_map_original_struct_to_exported_name() {
    let modules = BTreeMap::from([
        (
            Vec::<String>::new(),
            "pub use module::{InternalRecord as PublicRecord, OtherRecord};".to_owned(),
        ),
        (
            vec!["module".to_owned()],
            "pub struct InternalRecord { pub value: String }\n\
             pub struct OtherRecord { pub id: String }"
                .to_owned(),
        ),
    ]);
    let exported = resolve_public_struct_sources(&modules, &[Vec::new()]);

    let (rust_name, source) = exported
        .get("PublicRecord")
        .expect("PublicRecord must resolve");
    assert_eq!(rust_name, "InternalRecord");
    assert_eq!(
        rust_struct_fields(source, rust_name),
        BTreeSet::from(["value".to_owned()])
    );
    let (rust_name, source) = exported
        .get("OtherRecord")
        .expect("OtherRecord must resolve");
    assert_eq!(rust_name, "OtherRecord");
    assert_eq!(
        rust_struct_fields(source, rust_name),
        BTreeSet::from(["id".to_owned()])
    );
}

#[test]
fn public_data_model_fields_match_every_language() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let exported_structs = exported_public_struct_sources(&root);
    let opaque_values = BTreeSet::from([
        "BinanceListenKey".to_owned(),
        "Cursor".to_owned(),
        "Timestamp".to_owned(),
    ]);
    let runtime_handles = BTreeSet::from([
        "AccountStream".to_owned(),
        "Client".to_owned(),
        "MarketStream".to_owned(),
    ]);
    let mut record_count = 0;

    for classified in opaque_values.iter().chain(&runtime_handles) {
        assert!(
            exported_structs.contains_key(classified),
            "classified Rust struct {classified} must remain publicly exported"
        );
    }

    for (model, (rust_model, source)) in exported_structs {
        if opaque_values.contains(&model)
            || runtime_handles.contains(&model)
            || model.ends_with("Adapter")
        {
            continue;
        }
        record_count += 1;

        let rust_fields = rust_struct_fields(&source, &rust_model);
        assert!(
            !rust_fields.is_empty(),
            "public fieldless Rust struct {model} needs explicit runtime handle or opaque value classification"
        );
        let python_fields = python_class_fields(PYTHON_MODELS, &model);
        assert_inventory(
            &format!("Python {model} fields"),
            &rust_fields,
            &python_fields,
        );

        let mut dart_expected = rust_fields;
        if matches!(model.as_str(), "UpbitMarketEvent" | "BithumbMarketAlert") {
            dart_expected.insert("market".to_owned());
        }
        let dart_fields = dart_public_model_fields(&model);
        assert_inventory(
            &format!("Dart {model} fields"),
            &dart_expected,
            &dart_fields,
        );
    }
    assert_eq!(
        record_count, 26,
        "all current public record DTOs must resolve"
    );
}

#[test]
fn opaque_value_representations_match_every_language() {
    assert!(PYTHON_MODELS.lines().any(|line| line == "Timestamp = int"));
    assert!(
        PYTHON_MODELS
            .lines()
            .any(|line| line == "class Cursor(str):")
    );
    assert!(
        python_class_body(PYTHON_ADAPTERS, "BinanceListenKey")
            .contains("    @property\n    def value(self) -> str:")
    );

    let dart_timestamp =
        find_dart_class_body(DART_MODELS, "Timestamp").expect("Dart Timestamp class must exist");
    assert!(
        dart_timestamp
            .lines()
            .any(|line| line.trim() == "final int nanosecondsSinceEpoch;")
    );
    let dart_cursor =
        find_dart_class_body(DART_MODELS, "Cursor").expect("Dart Cursor class must exist");
    assert!(
        dart_cursor
            .lines()
            .any(|line| line.trim() == "final String value;")
    );
    let dart_listen_key = find_dart_class_body(DART_ADAPTERS, "BinanceListenKey")
        .expect("Dart BinanceListenKey class must exist");
    assert!(
        dart_listen_key
            .lines()
            .any(|line| line.trim().starts_with("String get value =>"))
    );
}

#[test]
fn exchange_and_feature_inventories_match_every_language() {
    let exchanges = Exchange::ALL
        .map(|value| value.id().to_owned())
        .into_iter()
        .collect::<Vec<_>>();
    let features = Feature::ALL
        .map(|value| value.id().to_owned())
        .into_iter()
        .collect::<Vec<_>>();

    assert_eq!(python_enum_values(PYTHON_MODELS, "Exchange"), exchanges);
    assert_eq!(python_enum_values(PYTHON_MODELS, "Feature"), features);
    assert_eq!(dart_enum_values(DART_MODELS, "Exchange"), exchanges);
    assert_eq!(
        dart_enum_values(DART_MODELS, "Feature")
            .into_iter()
            .map(|value| snake_case(&value))
            .collect::<Vec<_>>(),
        features
    );
}

#[test]
fn adapter_and_client_methods_match_every_language() {
    let adapter = rust_trait_methods(CORE_ADAPTER, "Adapter");
    let mut python_adapter = adapter.clone();
    python_adapter.insert("features".to_owned());
    assert_eq!(
        python_class_methods(PYTHON_API, "Adapter", false),
        python_adapter
    );
    assert_eq!(
        dart_class_members(DART_ADAPTER, "abstract interface class Adapter", false),
        python_adapter
            .iter()
            .map(|value| camel_case(value))
            .collect()
    );

    let mut client = rust_public_methods(CORE_CLIENT, CLIENT_METHOD_PARITY_TYPE, false);
    client.remove("new");
    assert_eq!(
        python_class_methods(PYTHON_API, CLIENT_METHOD_PARITY_TYPE, false),
        client
    );
    client.remove("into_adapter");
    assert_eq!(
        dart_class_members(
            DART_CLIENT,
            &format!("final class {CLIENT_METHOD_PARITY_TYPE}"),
            false,
        ),
        client.iter().map(|value| camel_case(value)).collect()
    );
}

#[test]
fn adapter_calls_and_replies_match_every_language() {
    let calls = rust_enum_variants(COMMON_CONTRACT, "AdapterCall");
    let python_production = PYTHON_RUST_ADAPTER
        .split("#[cfg(test)]")
        .next()
        .expect("Python adapter production source");
    assert_eq!(
        qualified_variants(python_production, "AdapterCall::"),
        calls
    );

    let mut dart_calls = calls.clone();
    dart_calls.insert("CancelStream".to_owned());
    assert_eq!(
        rust_enum_variants(DART_RUST_ADAPTER, "AdapterCall"),
        dart_calls
    );
    let dart_production = DART_RUST_ADAPTER
        .split("#[cfg(test)]")
        .next()
        .expect("Dart adapter production source");
    assert_eq!(
        qualified_variants(dart_production, "CommonAdapterCall::"),
        calls
    );

    let replies = rust_enum_variants(COMMON_CONTRACT, "AdapterReply");
    assert_eq!(
        rust_enum_variants(PYTHON_RUST_ADAPTER, "ReplyKind"),
        replies
    );
    let mut dart_replies = replies;
    dart_replies.remove("MarketStream");
    dart_replies.remove("AccountStream");
    assert_eq!(
        rust_enum_variants(DART_RUST_ADAPTER, "AdapterReply"),
        dart_replies
    );
}

#[test]
fn stream_event_and_error_variants_match_every_language() {
    let stream_types = include_str!("../../../src/types/stream.rs");
    let market_events = rust_variant_ids(stream_types, "MarketEvent");
    assert_eq!(
        python_event_kinds(PYTHON_ADAPTERS, "_decode_market_event"),
        market_events
    );
    assert_eq!(
        dart_factory_names(DART_MODELS, "sealed class MarketEvent")
            .into_iter()
            .map(|value| snake_case(&value))
            .collect::<BTreeSet<_>>(),
        market_events
    );

    let account_events = rust_variant_ids(stream_types, "AccountEvent");
    assert_eq!(
        python_event_kinds(PYTHON_ADAPTERS, "_decode_account_event"),
        account_events
    );
    assert_eq!(
        dart_factory_names(DART_MODELS, "sealed class AccountEvent")
            .into_iter()
            .map(|value| snake_case(&value))
            .collect::<BTreeSet<_>>(),
        account_events
    );

    let errors = rust_variant_ids(include_str!("../../../src/error.rs"), "Error");
    assert_eq!(python_error_kinds(PYTHON_API), errors);
    assert_eq!(
        dart_variant_ids(DART_GENERATED_CONVERT, "NativeErrorKind"),
        errors
    );
}

#[test]
fn public_enum_variants_match_every_language() {
    let inventories = [
        (
            "MarketKind",
            include_str!("../../../src/types/market.rs"),
            PYTHON_MODELS,
            DART_MODELS,
        ),
        (
            "MarketStatus",
            include_str!("../../../src/types/market.rs"),
            PYTHON_MODELS,
            DART_MODELS,
        ),
        (
            "Side",
            include_str!("../../../src/types/data.rs"),
            PYTHON_MODELS,
            DART_MODELS,
        ),
        (
            "Interval",
            include_str!("../../../src/types/data.rs"),
            PYTHON_MODELS,
            DART_MODELS,
        ),
        (
            "Overflow",
            include_str!("../../../src/types/stream.rs"),
            PYTHON_MODELS,
            DART_MODELS,
        ),
        (
            "MarginMode",
            include_str!("../../../src/types/account.rs"),
            PYTHON_MODELS,
            DART_MODELS,
        ),
        (
            "OrderStatus",
            include_str!("../../../src/types/account.rs"),
            PYTHON_MODELS,
            DART_MODELS,
        ),
        (
            "OrderType",
            include_str!("../../../src/types/account.rs"),
            PYTHON_MODELS,
            DART_MODELS,
        ),
        (
            "TimeInForce",
            include_str!("../../../src/types/account.rs"),
            PYTHON_MODELS,
            DART_MODELS,
        ),
        (
            "ExchangeErrorKind",
            include_str!("../../../src/error.rs"),
            PYTHON_API,
            DART_ERRORS,
        ),
        (
            "UpbitRegion",
            include_str!("../../../src/adapters/upbit/mod.rs"),
            PYTHON_MODELS,
            DART_PROVIDERS,
        ),
        (
            "BithumbAlertStep",
            include_str!("../../../src/adapters/bithumb/mod.rs"),
            PYTHON_MODELS,
            DART_PROVIDERS,
        ),
        (
            "BinanceMarket",
            include_str!("../../../src/adapters/binance/mod.rs"),
            PYTHON_MODELS,
            DART_PROVIDERS,
        ),
    ];

    for (name, rust_source, python_source, dart_source) in inventories {
        let variants = rust_variant_ids(rust_source, name);
        assert_eq!(
            python_enum_names(python_source, name),
            variants,
            "Python {name} variants differ"
        );
        assert_eq!(
            dart_variant_ids(dart_source, name),
            variants,
            "Dart {name} variants differ"
        );
    }

    let sizes = rust_variant_ids(include_str!("../../../src/types/account.rs"), "Size");
    assert_eq!(python_enum_names(PYTHON_MODELS, "SizeKind"), sizes);
    assert_eq!(
        dart_factory_names(DART_MODELS, "sealed class Size")
            .into_iter()
            .map(|value| snake_case(&value))
            .collect::<BTreeSet<_>>(),
        sizes
    );

    let feeds = rust_variant_ids(include_str!("../../../src/types/stream.rs"), "Feed");
    let mut python_feeds = python_assigned_constructor_values(PYTHON_MODELS, "Feed.");
    if python_class_methods(PYTHON_MODELS, "Feed", false).contains("candles") {
        python_feeds.insert("candles".to_owned());
    }
    assert_eq!(python_feeds, feeds);
    assert_eq!(dart_variant_ids(DART_MODELS, "FeedKind"), feeds);

    let ledger_kinds = rust_variant_ids(
        include_str!("../../../src/adapters/hyperliquid/native.rs"),
        "HyperliquidLedgerKind",
    );
    let mut python_ledger_kinds = python_enum_names(PYTHON_MODELS, "HyperliquidLedgerKind");
    assert!(PYTHON_MODELS.contains("def _missing_(cls, value: object) -> HyperliquidLedgerKind:"));
    python_ledger_kinds.insert("other".to_owned());
    assert_eq!(python_ledger_kinds, ledger_kinds);
    assert_eq!(
        dart_static_and_factory_names(DART_PROVIDERS, "final class HyperliquidLedgerKind",)
            .into_iter()
            .map(|value| snake_case(&value))
            .collect::<BTreeSet<_>>(),
        ledger_kinds
    );
}

#[test]
fn provider_specific_methods_match_every_language() {
    for provider in PROVIDERS {
        let mut methods = rust_public_methods(provider.rust_source, provider.adapter, false);
        for constructor in
            rust_self_returning_methods(provider.rust_source, provider.adapter).keys()
        {
            methods.remove(constructor);
        }

        let mut python_methods = python_class_methods(PYTHON_ADAPTERS, provider.adapter, false);
        for factory in python_classmethod_names(PYTHON_ADAPTERS, provider.adapter) {
            python_methods.remove(&factory);
        }
        assert_eq!(
            python_methods, methods,
            "Python {} provider methods differ",
            provider.adapter
        );

        let mut dart_methods = dart_class_members(
            DART_ADAPTERS,
            &format!("final class {}", provider.adapter),
            false,
        );
        for constructor in
            dart_construction_method_parameters(DART_ADAPTERS, provider.adapter).keys()
        {
            dart_methods.remove(&camel_case(constructor));
        }
        assert_eq!(
            dart_methods,
            methods.iter().map(|value| camel_case(value)).collect(),
            "Dart {} provider methods differ",
            provider.adapter
        );
    }
}

fn public_inherent_method_types(exported: &BTreeMap<String, (String, String)>) -> BTreeSet<String> {
    exported
        .iter()
        .filter(|(_, (rust_name, source))| !rust_public_value_methods(source, rust_name).is_empty())
        .map(|(public_name, _)| public_name.clone())
        .collect()
}

#[test]
#[should_panic(
    expected = "Public Rust inherent method types differ; missing: {}; extra: {\"Future\"}"
)]
fn public_method_type_inventory_rejects_unclassified_export() {
    let modules = BTreeMap::from([(
        Vec::new(),
        "pub struct Known; impl Known { pub fn current(&self) {} }\n\
         pub struct Future; impl Future { pub fn added(&self) {} }"
            .to_owned(),
    )]);
    let actual =
        public_inherent_method_types(&resolve_public_type_sources(&modules, &[Vec::new()]));

    assert_inventory(
        "Public Rust inherent method types",
        &BTreeSet::from(["Known".to_owned()]),
        &actual,
    );
}

#[test]
fn every_public_inherent_method_type_is_classified() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let exported = exported_public_type_sources(&root);
    let actual = public_inherent_method_types(&exported);
    let mut expected = VALUE_TYPE_CONTRACTS
        .iter()
        .filter(|contract| matches!(contract.rust_surface, RustValueSurface::Methods))
        .map(|contract| contract.name.to_owned())
        .collect::<BTreeSet<_>>();
    for exclusion in VERIFIED_PUBLIC_METHOD_TYPE_EXCLUSIONS {
        assert!(
            !expected.contains(exclusion.name),
            "Public method type {} cannot be both contracted and excluded",
            exclusion.name
        );
        assert!(
            expected.insert(exclusion.name.to_owned()),
            "Public method type exclusion {} repeats",
            exclusion.name
        );
        assert_exact_public_methods(
            &format!("Rust {} classified methods", exclusion.name),
            exclusion.rust_source,
            exclusion.name,
            exclusion.methods,
        );
    }
    assert!(
        expected.insert(CLIENT_METHOD_PARITY_TYPE.to_owned()),
        "Client method parity classification repeats"
    );
    for provider in PROVIDERS {
        assert!(
            expected.insert(provider.adapter.to_owned()),
            "provider method parity classification repeats {}",
            provider.adapter
        );
    }

    assert_inventory("Public Rust inherent method types", &expected, &actual);
}

#[test]
fn public_value_methods_and_event_constructors_match_every_language() {
    for contract in VALUE_TYPE_CONTRACTS {
        assert_value_type_contract(contract);
    }
}

#[test]
fn provider_constructor_settings_match_every_language() {
    assert_provider_constructor_settings();
}
