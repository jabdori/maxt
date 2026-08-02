//! Cross-language public API inventory parity.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use maxt::{Exchange, Feature};
use syn::{Fields, ImplItem, Item, TraitItem, Type, UseTree, Visibility};

const CORE_ADAPTER: &str = include_str!("../../../src/adapter.rs");
const CORE_CLIENT: &str = include_str!("../../../src/client.rs");
const COMMON_CONTRACT: &str = include_str!("../src/contract.rs");
const PYTHON_RUST_ADAPTER: &str = include_str!("../../python/src/adapter.rs");
const PYTHON_MODELS: &str = include_str!("../../python/python/maxt/models.py");
const PYTHON_API: &str = include_str!("../../python/python/maxt/_api.py");
const PYTHON_ADAPTERS: &str = include_str!("../../python/python/maxt/adapters.py");
const DART_RUST_ADAPTER: &str = include_str!("../../dart/rust/src/adapter.rs");
const DART_MODELS: &str = include_str!("../../dart/lib/src/models.dart");
const DART_ADAPTER: &str = include_str!("../../dart/lib/src/adapter.dart");
const DART_CLIENT: &str = include_str!("../../dart/lib/src/client.dart");
const DART_ADAPTERS: &str = include_str!("../../dart/lib/src/adapters.dart");
const DART_ERRORS: &str = include_str!("../../dart/lib/src/errors.dart");
const DART_PROVIDERS: &str = include_str!("../../dart/lib/src/providers.dart");
const DART_GENERATED_CONVERT: &str = include_str!("../../dart/lib/src/rust/convert.dart");

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

fn public_struct_definitions(
    modules: &BTreeMap<Vec<String>, String>,
) -> BTreeMap<Vec<String>, (String, String)> {
    let mut definitions = BTreeMap::new();
    for (module, source) in modules {
        for item in syn::parse_file(source)
            .expect("scanned Rust source must parse")
            .items
        {
            let Item::Struct(item) = item else {
                continue;
            };
            if !matches!(item.vis, Visibility::Public(_)) {
                continue;
            }
            let name = item.ident.to_string();
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
    let definitions = public_struct_definitions(modules);
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
                    "duplicate public Rust struct name {public_name}"
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

fn rust_public_methods(source: &str, name: &str, async_only: bool) -> BTreeSet<String> {
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
            ImplItem::Fn(method)
                if matches!(method.vis, Visibility::Public(_))
                    && (!async_only || method.sig.asyncness.is_some()) =>
            {
                Some(method.sig.ident.to_string())
            }
            _ => None,
        })
        .collect()
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

fn python_class_fields(source: &str, name: &str) -> BTreeSet<String> {
    let body = python_class_body(source, name);
    let member_depth = body
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.len() - line.trim_start_matches([' ', '\t']).len())
        .filter(|depth| *depth > 0)
        .min()
        .unwrap_or(0);

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

fn python_class_methods(source: &str, name: &str, async_only: bool) -> BTreeSet<String> {
    let marker = format!("class {name}(");
    let mut found = false;
    let mut methods = BTreeSet::new();
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
    assert!(found, "Python class {name} must exist");
    methods
}

fn python_enum_values(source: &str, name: &str) -> BTreeSet<String> {
    let marker = format!("class {name}(");
    let mut found = false;
    let mut values = BTreeSet::new();
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
            values.insert(value.trim_matches('"').to_owned());
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

fn dart_has_top_level_comma(declaration: &str) -> bool {
    let mut nesting = DartNesting::default();
    declaration.chars().any(|character| {
        let top_level = nesting.is_top_level();
        nesting.consume(character) && top_level && character == ','
    })
}

fn dart_is_constructor(declaration: &str, name: &str) -> bool {
    let mut declaration = declaration.trim_start();
    loop {
        let Some((modifier, rest)) = declaration.split_once(' ') else {
            break;
        };
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
    let mut member_block = false;

    for line in block.lines() {
        let line = line.trim();
        if line.is_empty() || (nesting.quote.is_none() && line.starts_with("//")) {
            continue;
        }
        if !member.is_empty() {
            member.push(' ');
        }
        for character in line.chars() {
            let top_level = nesting.is_top_level();
            let syntax = nesting.consume(character);
            if syntax && top_level && character == '=' && assignment_prefix.is_none() {
                assignment_prefix = Some(member.clone());
            }
            if syntax && top_level && character == '{' {
                member_block = assignment_prefix
                    .as_deref()
                    .and_then(|prefix| dart_instance_field_name(prefix, class_name))
                    .is_none();
            }
            member.push(character);
            if !syntax {
                continue;
            }
            if character == '}' && nesting.is_top_level() && member_block {
                member.clear();
                assignment_prefix = None;
                member_block = false;
            } else if character == ';' && nesting.is_top_level() {
                let declaration = std::mem::take(&mut member);
                if !declaration.trim_matches(';').trim().is_empty() {
                    members.push(declaration);
                }
                assignment_prefix = None;
                member_block = false;
            }
        }
    }
    assert!(
        member.trim().is_empty(),
        "Dart class {class_name} has an unterminated top-level member"
    );
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

fn assert_inventory(
    language: &str,
    model: &str,
    expected: &BTreeSet<String>,
    actual: &BTreeSet<String>,
) {
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
        "{language} {model} fields differ; missing: {missing:?}; extra: {extra:?}"
    );
}

fn dart_enum_values(source: &str, name: &str) -> BTreeSet<String> {
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
        "Python",
        "Example",
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
        assert_inventory("Python", &model, &rust_fields, &python_fields);

        let mut dart_expected = rust_fields;
        if matches!(model.as_str(), "UpbitMarketEvent" | "BithumbMarketAlert") {
            dart_expected.insert("market".to_owned());
        }
        let dart_fields = dart_public_model_fields(&model);
        assert_inventory("Dart", &model, &dart_expected, &dart_fields);
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
        .collect::<BTreeSet<_>>();
    let features = Feature::ALL
        .map(|value| value.id().to_owned())
        .into_iter()
        .collect::<BTreeSet<_>>();

    assert_eq!(python_enum_values(PYTHON_MODELS, "Exchange"), exchanges);
    assert_eq!(python_enum_values(PYTHON_MODELS, "Feature"), features);
    assert_eq!(dart_enum_values(DART_MODELS, "Exchange"), exchanges);
    assert_eq!(
        dart_enum_values(DART_MODELS, "Feature")
            .into_iter()
            .map(|value| snake_case(&value))
            .collect::<BTreeSet<_>>(),
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

    let mut client = rust_public_methods(CORE_CLIENT, "Client", false);
    client.remove("new");
    assert_eq!(python_class_methods(PYTHON_API, "Client", false), client);
    client.remove("into_adapter");
    assert_eq!(
        dart_class_members(DART_CLIENT, "final class Client", false),
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
    let providers = [
        (
            "upbit",
            "UpbitAdapter",
            include_str!("../../../src/adapters/upbit/mod.rs"),
        ),
        (
            "bithumb",
            "BithumbAdapter",
            include_str!("../../../src/adapters/bithumb/mod.rs"),
        ),
        (
            "binance",
            "BinanceAdapter",
            include_str!("../../../src/adapters/binance/mod.rs"),
        ),
        (
            "hyperliquid",
            "HyperliquidAdapter",
            include_str!("../../../src/adapters/hyperliquid/mod.rs"),
        ),
    ];
    assert_eq!(
        providers
            .iter()
            .map(|(exchange, _, _)| (*exchange).to_owned())
            .collect::<BTreeSet<_>>(),
        Exchange::ALL
            .map(|value| value.id().to_owned())
            .into_iter()
            .collect()
    );

    for (_, adapter, rust_source) in providers {
        let mut methods = rust_public_methods(rust_source, adapter, false);
        for constructor in [
            "new",
            "with_region",
            "with_credentials",
            "spot",
            "usd_m_futures",
            "testnet",
            "with_wallet",
        ] {
            methods.remove(constructor);
        }

        let mut python_methods = python_class_methods(PYTHON_ADAPTERS, adapter, false);
        for constructor in ["spot", "usd_m_futures", "testnet"] {
            python_methods.remove(constructor);
        }
        assert_eq!(
            python_methods, methods,
            "Python {adapter} provider methods differ"
        );

        let mut dart_methods =
            dart_class_members(DART_ADAPTERS, &format!("final class {adapter}"), false);
        dart_methods.remove("withCredentials");
        dart_methods.remove("withWallet");
        assert_eq!(
            dart_methods,
            methods.iter().map(|value| camel_case(value)).collect(),
            "Dart {adapter} provider methods differ"
        );
    }
}
