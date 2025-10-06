use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use super::Buffer;
use super::context::Context;
use super::generate::generate_proto;
use super::sanitize::sanitize_filepath;
use crate::ast::FileDescriptor;
use crate::{Error, parse};

#[derive(Debug, Default)]
pub enum MapType {
    #[default]
    HashMap,
    BTreeMap,
}

pub struct Config {
    output: Option<PathBuf>,
    filename: Option<String>,

    pub(crate) build_server: bool,
    pub(crate) build_client: bool,
    pub(crate) no_std: bool,
    pub(crate) message_attributes: HashMap<String, Vec<String>>,
    pub(crate) enum_attributes: HashMap<String, Vec<String>>,
    pub(crate) oneof_attributes: HashMap<String, Vec<String>>,
    pub(crate) skip_serialize: HashSet<String>,
    pub(crate) skip_deserialize: HashSet<String>,
    pub(crate) tree_map: HashMap<String, MapType>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            output: None,
            filename: None,
            no_std: false,
            build_client: true,
            build_server: true,
            message_attributes: Default::default(),
            enum_attributes: Default::default(),
            oneof_attributes: Default::default(),
            skip_deserialize: Default::default(),
            skip_serialize: Default::default(),
            tree_map: Default::default(),
        }
    }
}

impl Config {
    // set output paths
    pub fn output(&mut self, path: impl Into<PathBuf>) -> &mut Self {
        self.output = Some(path.into());
        self
    }

    pub fn no_std(&mut self, no_std: bool) -> &mut Self {
        self.no_std = no_std;
        self
    }

    pub fn build_server(&mut self, build_server: bool) -> &mut Self {
        self.build_server = build_server;
        self
    }

    pub fn build_client(&mut self, build_client: bool) -> &mut Self {
        self.build_client = build_client;
        self
    }

    pub fn filename(&mut self, name: &str) -> &mut Self {
        self.filename = Some(name.to_string());
        self
    }

    /// set custom attributes for a message
    ///
    /// path could be something like `foo.bar`, `foo.bar.SomeMessage`
    pub fn message_attribute<P: ToString, A: ToString>(
        &mut self,
        path: P,
        attribute: A,
    ) -> &mut Self {
        self.message_attributes
            .entry(path.to_string())
            .and_modify(|attrs| {
                attrs.push(attribute.to_string());
                attrs.dedup();
            })
            .or_insert_with(|| vec![attribute.to_string()]);
        self
    }

    /// set custom attributes for an enum
    ///
    /// path could be something like `foo.bar`, `foo.bar.SomeMessage`
    pub fn enum_attribute<P: ToString, A: ToString>(&mut self, path: P, attribute: A) -> &mut Self {
        self.enum_attributes
            .entry(path.to_string())
            .and_modify(|attrs| {
                attrs.push(attribute.to_string());
                attrs.dedup();
            })
            .or_insert_with(|| vec![attribute.to_string()]);
        self
    }

    pub fn oneof_attribute<P: ToString, A: ToString>(
        &mut self,
        path: P,
        attribute: A,
    ) -> &mut Self {
        self.oneof_attributes
            .entry(path.to_string())
            .and_modify(|attrs| {
                attrs.push(attribute.to_string());
                attrs.dedup();
            })
            .or_insert_with(|| vec![attribute.to_string()]);
        self
    }

    pub fn btree_map<I, S>(&mut self, paths: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for path in paths {
            self.tree_map
                .insert(path.as_ref().to_string(), MapType::BTreeMap);
        }

        self
    }

    pub fn hashmap<I, S>(&mut self, paths: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for path in paths {
            self.tree_map
                .insert(path.as_ref().to_string(), MapType::HashMap);
        }

        self
    }

    /// This function prevent code generator to implement `Deserialize` for structs
    ///
    /// It's helpful for people to implement `Deserialize` manually
    pub fn skip_deserialize<T: ToString>(&mut self, paths: &[T]) -> &mut Self {
        for path in paths {
            self.skip_deserialize.insert(path.to_string());
        }
        self
    }

    /// This function prevent code generator to implement `Serialize` for structs
    ///
    /// It's helpful for people to implement `Serialize` manually
    pub fn skip_serialize<T: ToString>(&mut self, paths: &[T]) -> &mut Self {
        for path in paths {
            self.skip_serialize.insert(path.to_string());
        }
        self
    }

    pub fn compile<P: AsRef<Path>>(&mut self, includes: &[P], files: &[P]) -> Result<(), Error> {
        if files.is_empty() {
            return Ok(());
        }

        for path in files {
            // import -> fd
            let mut imports = HashMap::new();

            let mut fd = load_proto(path)?;
            for import in &fd.imports {
                load_imports(import, includes, &mut imports)?;
            }

            let mut buf = Buffer::default();
            buf.push("use prust::*;\n");

            let mut cx = Context {
                fd: &mut fd,
                config: &self,
                imports: &mut imports,
                messages: Vec::new(),
            };
            generate_proto(&mut buf, &mut cx)?;

            let filename = match &self.filename {
                Some(name) => name.to_string(),
                None => match fd.package.as_ref() {
                    Some(package) => sanitize_filepath(package),
                    None => path
                        .as_ref()
                        .file_stem()
                        .unwrap()
                        .to_string_lossy()
                        .to_string(),
                },
            };

            self.write(format!("{filename}.rs"), buf.into_inner())?;
        }

        Ok(())
    }

    fn write<P: AsRef<Path>>(&self, filename: P, content: String) -> Result<(), Error> {
        let path = match self.output.as_ref() {
            Some(path) => path.clone(),
            None => PathBuf::from(std::env::var_os("OUT_DIR").unwrap()),
        }
        .join(filename);

        std::fs::create_dir_all(path.parent().unwrap())?;

        if let Err(err) = std::fs::write(&path, content) {
            panic!("Error writing file {path:?}: {err}");
        }

        Ok(())
    }
}

fn load_imports<P: AsRef<Path>>(
    name: &str,
    includes: &[P],
    imports: &mut HashMap<String, FileDescriptor>,
) -> Result<(), Error> {
    if imports.len() > 128 {
        panic!("import too many times");
    }

    if imports.contains_key(name) {
        return Ok(());
    }

    for include in includes {
        let path = include.as_ref().join(name);
        if path.exists() {
            let fd = load_proto(path)?;

            for import in &fd.imports {
                load_imports(import, includes, imports)?;
            }

            imports.insert(name.to_string(), fd);
        }
    }

    Ok(())
}

fn load_proto<P: AsRef<Path>>(path: P) -> Result<FileDescriptor, Error> {
    let content = std::fs::read(&path)?;
    let mut fd = parse::parse(&content).map_err(Error::Parse)?;
    if fd.package.is_none() {
        fd.package = Some(
            path.as_ref()
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .to_string(),
        );
    }

    Ok(fd)
}
