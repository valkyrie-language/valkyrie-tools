use anyhow::Result;
use clap::Parser;
use maud::{DOCTYPE, html};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};
use valkyrie_compiler::context::CompilationContext;
use valkyrie_compiler::transform::ast_to_hir::{Lowering, LoweringContext};
use valkyrie_parser::ValkyrieParser;
use valkyrie_types::{NamePath, SourceID, hir::*};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Workspace root to scan for legion.json/legions.json
    #[arg(short, long, default_value = ".")]
    workspace: PathBuf,

    /// Output directory for the static site
    #[arg(short, long, default_value = "dist-docs")]
    output: PathBuf,
}

#[derive(Debug)]
struct ValkyrieProject {
    name: String,
    version: String,
    modules: Vec<HirModule>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("Scanning for Valkyrie projects in {}...", args.workspace.display());
    let projects = find_projects(&args.workspace)?;

    if projects.is_empty() {
        println!("No Valkyrie projects found. Ensure legion.json or legions.json exists.");
        return Ok(());
    }

    if !args.output.exists() {
        fs::create_dir_all(&args.output)?;
    }

    let css_path = args.output.join("style.css");
    fs::write(css_path, include_str!("style.css"))?;

    for project in &projects {
        generate_project_docs(project, &args.output)?;
    }

    generate_index(&projects, &args.output)?;

    println!("Documentation generated in \"{}\"", args.output.display());
    Ok(())
}

fn find_projects(root: &Path) -> Result<Vec<ValkyrieProject>> {
    let mut projects = Vec::new();

    let legions_path = root.join("legions.json");
    if legions_path.exists() {
        let content = fs::read_to_string(&legions_path)?;
        let config: serde_json::Value = serde_json::from_str(&content)?;
        if let Some(members) = config.get("members").and_then(|m| m.as_array()) {
            for member in members {
                if let Some(member_path) = member.as_str() {
                    let project_root = root.join(member_path);
                    if let Some(project) = load_project(&project_root)? {
                        projects.push(project);
                    }
                }
            }
        }
    }
    else {
        // Search for legion.json recursively, but skip vendor and hidden dirs
        for entry in walkdir::WalkDir::new(root)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !name.starts_with('.') && name != "vendor" && name != "node_modules" && name != "target"
            })
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name() == "legion.json")
        {
            let project_root = entry.path().parent().unwrap();
            if let Some(project) = load_project(project_root)? {
                projects.push(project);
            }
        }
    }

    Ok(projects)
}

fn load_project(root: &Path) -> Result<Option<ValkyrieProject>> {
    let legion_path = root.join("legion.json");
    if !legion_path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&legion_path)?;
    let config: serde_json::Value = serde_json::from_str(&content)?;

    let name = config.get("name").and_then(|n| n.as_str()).unwrap_or("unnamed").to_string();
    let version = config.get("version").and_then(|v| v.as_str()).unwrap_or("0.0.0").to_string();

    let mut merged_modules: HashMap<NamePath, HirModule> = HashMap::new();
    let subdirs = ["library", "binary", "script", "test"];

    for subdir in subdirs {
        let dir_path = root.join(subdir);
        if dir_path.exists() {
            for entry in walkdir::WalkDir::new(dir_path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map_or(false, |ext| ext == "vk"))
            {
                if let Some(program) = process_module(entry.path())? {
                    for module in program.modules {
                        let target = merged_modules.entry(module.name.clone()).or_insert_with(|| HirModule {
                            name: module.name.clone(),
                            doc: HirDocumentation::default(),
                            functions: Vec::new(),
                            structs: Vec::new(),
                            enums: Vec::new(),
                            traits: Vec::new(),
                            impls: Vec::new(),
                            statements: Vec::new(),
                        });
                        target.doc.0.extend(module.doc.0);
                        target.functions.extend(module.functions);
                        target.structs.extend(module.structs);
                        target.enums.extend(module.enums);
                        target.traits.extend(module.traits);
                        target.impls.extend(module.impls);
                    }
                }
            }
        }
    }

    if merged_modules.is_empty() {
        println!("Warning: Project {} found at {} has no .vk files.", name, root.display());
        return Ok(None);
    }

    Ok(Some(ValkyrieProject { name, version, modules: merged_modules.into_values().collect() }))
}

fn process_module(file_path: &Path) -> Result<Option<HirProgram>> {
    let content = fs::read_to_string(file_path)?;
    let file_id = SourceID::default();

    match ValkyrieParser::new().parse(&content, file_id) {
        Ok(root) => {
            let compilation = CompilationContext::new();
            let mut ctx = LoweringContext::new(&compilation, file_id);
            Ok(Some(root.lower(&mut ctx)))
        }
        Err(errors) => {
            eprintln!("Error parsing \"{}\": {:?}", file_path.display(), errors);
            Ok(None)
        }
    }
}

fn generate_project_docs(project: &ValkyrieProject, output_dir: &Path) -> Result<()> {
    let project_slug = slug::slugify(&project.name);
    let project_dir = output_dir.join("projects");
    if !project_dir.exists() {
        fs::create_dir_all(&project_dir)?;
    }

    let file_path = project_dir.join(format!("{}.html", project_slug));

    let markup = html! {
        (DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                title { (project.name) " - Valkyrie Documentation" }
                link rel="stylesheet" href="../style.css";
            }
            body {
                nav {
                    a href="../index.html" { "← Back to Workspace" }
                }
                header {
                    h1 { (project.name) }
                    p { "Version: " (project.version) }
                }
                main {
                    @for module in &project.modules {
                        section.module {
                            h2 { (module.name) }
                            div.module-docs {
                                @for doc in &module.doc.0 {
                                    p { (doc.as_str()) }
                                }
                            }

                            @if !module.traits.is_empty() {
                                h3 { "Traits" }
                                @for t in &module.traits {
                                    (render_trait(t))
                                }
                            }

                            @if !module.structs.is_empty() {
                                h3 { "Structs" }
                                @for s in &module.structs {
                                    (render_struct(s))
                                }
                            }

                            @if !module.enums.is_empty() {
                                h3 { "Enums" }
                                @for e in &module.enums {
                                    (render_enum(e))
                                }
                            }

                            @if !module.functions.is_empty() {
                                h3 { "Functions" }
                                @for f in &module.functions {
                                    (render_function(f))
                                }
                            }
                        }
                    }
                }
            }
        }
    };

    fs::write(file_path, markup.into_string())?;
    Ok(())
}

fn render_function(f: &HirFunction) -> maud::Markup {
    html! {
        div.entry {
            div.entry-header {
                span.kind { "function" }
                span.name { (f.name) }
            }
            div.entry-docs {
                @if f.doc.0.is_empty() {
                    p.no-docs { "No documentation provided." }
                } @else {
                    @for doc in &f.doc.0 {
                        p { (doc.as_str()) }
                    }
                }
            }
        }
    }
}

fn render_struct(s: &HirStruct) -> maud::Markup {
    html! {
        div.entry {
            div.entry-header {
                span.kind { "struct" }
                span.name { (s.name) }
            }
            div.entry-docs {
                @if s.doc.0.is_empty() {
                    p.no-docs { "No documentation provided." }
                } @else {
                    @for doc in &s.doc.0 {
                        p { (doc.as_str()) }
                    }
                }
            }
            @if !s.fields.is_empty() {
                div.sub-entries {
                    h4 { "Fields" }
                    @for field in &s.fields {
                        div.sub-entry {
                            span.name { (field.name) }
                            span.type { ": " (format!("{:?}", field.ty)) }
                            div.sub-entry-docs {
                                @for doc in &field.doc.0 {
                                    p { (doc.as_str()) }
                                }
                            }
                        }
                    }
                }
            }
            @if !s.methods.is_empty() {
                div.sub-entries {
                    h4 { "Methods" }
                    @for method in &s.methods {
                        (render_function(method))
                    }
                }
            }
        }
    }
}

fn render_trait(t: &HirTrait) -> maud::Markup {
    html! {
        div.entry {
            div.entry-header {
                span.kind { "trait" }
                span.name { (t.name) }
            }
            div.entry-docs {
                @if t.doc.0.is_empty() {
                    p.no-docs { "No documentation provided." }
                } @else {
                    @for doc in &t.doc.0 {
                        p { (doc.as_str()) }
                    }
                }
            }
            @if !t.methods.is_empty() {
                div.sub-entries {
                    h4 { "Methods" }
                    @for method in &t.methods {
                        (render_function(method))
                    }
                }
            }
        }
    }
}

fn render_enum(e: &HirEnum) -> maud::Markup {
    html! {
        div.entry {
            div.entry-header {
                span.kind { "enum" }
                span.name { (e.name) }
            }
            div.entry-docs {
                @if e.doc.0.is_empty() {
                    p.no-docs { "No documentation provided." }
                } @else {
                    @for doc in &e.doc.0 {
                        p { (doc.as_str()) }
                    }
                }
            }
            @if !e.variants.is_empty() {
                div.sub-entries {
                    h4 { "Variants" }
                    @for variant in &e.variants {
                        div.sub-entry {
                            span.name { (variant.name) }
                            div.sub-entry-docs {
                                @for doc in &variant.doc.0 {
                                    p { (doc.as_str()) }
                                }
                            }
                        }
                    }
                }
            }
            @if !e.methods.is_empty() {
                div.sub-entries {
                    h4 { "Methods" }
                    @for method in &e.methods {
                        (render_function(method))
                    }
                }
            }
        }
    }
}

fn generate_index(projects: &[ValkyrieProject], output_dir: &Path) -> Result<()> {
    let file_path = output_dir.join("index.html");

    let markup = html! {
        (DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                title { "Valkyrie Workspace Documentation" }
                link rel="stylesheet" href="style.css";
            }
            body {
                header {
                    h1 { "Valkyrie Workspace Documentation" }
                    p { "Generated on " (chrono::Local::now().format("%Y-%m-%d %H:%M:%S")) }
                }
                main {
                    @if projects.is_empty() {
                        p { "No projects found." }
                    } @else {
                        ul.project-list {
                            @for project in projects {
                                li {
                                    a href=(format!("projects/{}.html", slug::slugify(&project.name))) {
                                        div.project-card {
                                            strong { (project.name) }
                                            span.version { (project.version) }
                                            span.module-count { (project.modules.len()) " modules" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    };

    fs::write(file_path, markup.into_string())?;
    Ok(())
}
