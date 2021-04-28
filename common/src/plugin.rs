use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use runestick::Vm;
use rune::EmitDiagnostics;
use std::sync::Arc;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub name: String,
    pub description: String,
    icon: String,
    pub authors: Vec<String>,
    pub items: Vec<crate::item::Item>,
}

pub struct Plugin {
    pub vm: Vm,
    pub manifest: Manifest,
}

impl Plugin {
    pub fn read(path: PathBuf) -> Plugin {
        let manifest = std::fs::read_to_string(path.join("plugin.toml")).unwrap();
        let manifest = toml::from_str(&manifest).unwrap();

        let ctx = runestick::Context::with_default_modules().unwrap();
        let mut sources = rune::Sources::new();
        for file in std::fs::read_dir(&path).unwrap() {
            let path = file.unwrap().path();
            if path.extension().unwrap().to_str().unwrap() == "rn" {
                sources.insert(runestick::Source::from_path(&path).unwrap());
            }
        }
        
        let mut diags = rune::Diagnostics::new();
        let result = rune::load_sources(&ctx, &rune::Options::default(), &mut sources, &mut diags);

        if !diags.is_empty() {
            let mut writer = rune::termcolor::StandardStream::stdout(rune::termcolor::ColorChoice::Auto);
            diags.emit_diagnostics(&mut writer, &sources).unwrap();
        }

        let unit = result.unwrap();
        let vm = Vm::new(Arc::new(ctx.runtime()), Arc::new(unit));

        Plugin {
            vm,
            manifest,
        }
    }
}

pub struct Plugins(pub Vec<Plugin>);

impl Plugins {
    pub fn init() -> Plugins {
        Plugins(
            std::fs::read_dir("plugins").unwrap().filter_map(|entry| {
                let entry = entry.unwrap();
                if entry.file_type().unwrap().is_dir() {
                    Some(Plugin::read(entry.path()))
                } else {
                    None
                }
            }).collect()
        )
    }
}
