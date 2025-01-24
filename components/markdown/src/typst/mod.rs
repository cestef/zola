use std::{collections::HashMap, io::Write, path::PathBuf, sync::Mutex};

use typst::{
    diag::{eco_format, FileError, FileResult, PackageError, PackageResult},
    foundations::{Bytes, Datetime, Label},
    syntax::{package::PackageSpec, FileId, Source},
    text::{Font, FontBook},
    utils::LazyHash,
    Library, World,
};

fn fonts() -> Vec<Font> {
    typst_assets::fonts()
        .flat_map(|bytes| {
            let buffer = Bytes::from_static(bytes);
            let face_count = ttf_parser::fonts_in_collection(&buffer).unwrap_or(1);
            (0..face_count).map(move |face| {
                Font::new(buffer.clone(), face).expect("failed to load font from typst-assets")
            })
        })
        .collect()
}

mod format;
mod svgo;
pub use format::*;
pub use svgo::*;

/// Fake file
///
/// This is a fake file which wrap the real content takes from the md math block
pub struct File {
    bytes: Bytes,

    source: Option<Source>,
}

impl File {
    fn source(&mut self, id: FileId) -> FileResult<Source> {
        let source = match &self.source {
            Some(source) => source,
            None => {
                let contents =
                    std::str::from_utf8(&self.bytes).map_err(|_| FileError::InvalidUtf8)?;
                let source = Source::new(id, contents.into());
                self.source.insert(source)
            }
        };
        Ok(source.clone())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    Display,
    Inline,
    Raw,
}

/// Compiler
///
/// This is the compiler which has all the necessary fields except the source
pub struct Compiler {
    pub library: LazyHash<Library>,
    pub book: LazyHash<FontBook>,
    pub fonts: Vec<Font>,

    pub cache: PathBuf,
    pub files: Mutex<HashMap<FileId, File>>,
}

impl Compiler {
    pub fn new() -> Self {
        let fonts = fonts();

        Self {
            library: LazyHash::new(Library::default()),
            book: LazyHash::new(FontBook::from_fonts(&fonts)),
            fonts,
            cache: PathBuf::new(),
            files: Mutex::new(HashMap::new()),
        }
    }

    pub fn wrap_source(&self, source: impl Into<String>) -> WrapSource<'_> {
        WrapSource {
            compiler: self,
            source: Source::detached(source),
            time: time::OffsetDateTime::now_local().unwrap_or(time::OffsetDateTime::now_utc()),
        }
    }

    /// Get the package directory or download if not exists
    fn package(&self, package: &PackageSpec) -> PackageResult<PathBuf> {
        let package_subdir = format!("{}/{}/{}", package.namespace, package.name, package.version);
        let path = self.cache.join(package_subdir);

        if path.exists() {
            return Ok(path);
        }

        // Download the package
        let package_url = format!(
            "https://packages.typst.org/{}/{}-{}.tar.gz",
            package.namespace, package.name, package.version
        );

        let mut response = libs::reqwest::blocking::get(package_url).map_err(|e| {
            PackageError::NetworkFailed(Some(eco_format!(
                "Failed to download package {}: {}",
                package.name,
                e
            )))
        })?;

        let mut compressed = Vec::new();
        response.copy_to(&mut compressed).map_err(|e| {
            PackageError::NetworkFailed(Some(eco_format!(
                "Failed to save package {}: {}",
                package.name,
                e
            )))
        })?;

        let mut decompressed = Vec::new();
        let mut decoder = flate2::write::GzDecoder::new(decompressed);
        decoder.write_all(&compressed).map_err(|e| {
            PackageError::MalformedArchive(Some(eco_format!(
                "Failed to decompress package {}: {}",
                package.name,
                e
            )))
        })?;
        decoder.try_finish().map_err(|e| {
            PackageError::MalformedArchive(Some(eco_format!(
                "Failed to decompress package {}: {}",
                package.name,
                e
            )))
        })?;
        decompressed = decoder.finish().map_err(|e| {
            PackageError::MalformedArchive(Some(eco_format!(
                "Failed to decompress package {}: {}",
                package.name,
                e
            )))
        })?;

        let mut archive = tar::Archive::new(decompressed.as_slice());
        archive.unpack(&path).map_err(|e| {
            std::fs::remove_dir_all(&path).ok();
            PackageError::MalformedArchive(Some(eco_format!(
                "Failed to unpack package {}: {}",
                package.name,
                e
            )))
        })?;

        Ok(path)
    }

    // Weird pattern because mapping a MutexGuard is not stable yet.
    fn file<T>(&self, id: FileId, map: impl FnOnce(&mut File) -> T) -> FileResult<T> {
        let mut files = self.files.lock().unwrap();
        if let Some(entry) = files.get_mut(&id) {
            return Ok(map(entry));
        }
        // `files` must stay locked here so we don't download the same package multiple times.
        // TODO proper multithreading, maybe with typst-kit.

        'x: {
            if let Some(package) = id.package() {
                let package_dir = self.package(package)?;
                let Some(path) = id.vpath().resolve(&package_dir) else {
                    break 'x;
                };
                let contents =
                    std::fs::read(&path).map_err(|error| FileError::from_io(error, &path))?;
                let entry =
                    files.entry(id).or_insert(File { bytes: contents.into(), source: None });
                return Ok(map(entry));
            }
        }

        Err(FileError::NotFound(id.vpath().as_rootless_path().into()))
    }

    pub fn render_math(&self, source: &str, mode: RenderMode) -> Result<(String, f64), String> {
        let source = match mode {
            RenderMode::Display => display_math_template(source),
            RenderMode::Inline => inline_math_template(source),
            RenderMode::Raw => panic!("raw mode should be handled by render_raw"),
        };

        // println!("{}", source);
        let world = self.wrap_source(source);

        let document = typst::compile(&world);
        let warnings = document.warnings;

        if !warnings.is_empty() {
            return Err(format!("{:?}", warnings));
        }

        let document = document.output.map_err(|diags| format!("{:?}", diags))?;
        let query = document.introspector.query_label(Label::construct("label".into()));
        let align = query
            .map(|it| {
                let field = it.clone().field_by_name("value").unwrap();
                if let typst::foundations::Value::Length(value) = field {
                    value.abs.to_pt()
                } else {
                    0.0
                }
            })
            .unwrap_or(0.0);

        let page = document.pages.first().ok_or("no pages")?;
        let image = typst_svg::svg(page);

        Ok((image, align))
    }

    pub fn render_raw(&self, source: impl Into<String>) -> Result<String, String> {
        let source = source.into();
        let source = raw_template(&source);
        let world = self.wrap_source(source);

        let document = typst::compile(&world);
        let warnings = document.warnings;

        if !warnings.is_empty() {
            return Err(format!("{:?}", warnings));
        }

        let document = document.output.map_err(|diags| format!("{:?}", diags))?;
        let page = document.pages.first().ok_or("no pages")?;
        let image = typst_svg::svg(page);

        Ok(image)
    }
}

/// Wrap source
///
/// This is a wrapper for the source which provides ref to the compiler
pub struct WrapSource<'a> {
    compiler: &'a Compiler,
    source: Source,
    time: time::OffsetDateTime,
}

impl World for WrapSource<'_> {
    fn library(&self) -> &LazyHash<Library> {
        &self.compiler.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.compiler.book
    }

    fn main(&self) -> FileId {
        self.source.id()
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.source.id() {
            Ok(self.source.clone())
        } else {
            self.compiler.file(id, |file| file.source(id))?
        }
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.compiler.file(id, |file| file.bytes.clone())
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.compiler.fonts.get(index).cloned()
    }

    fn today(&self, _offset: Option<i64>) -> Option<Datetime> {
        Some(Datetime::Date(self.time.date()))
    }
}

const TYPST_HELPER_FUNCTIONS: &str = include_str!("./helpers.typ");

fn display_math_template(code: &str) -> String {
    format!(
        r#"
#set page(height: auto, width: auto, margin: 0pt, fill: none)
#set text(14pt)
{TYPST_HELPER_FUNCTIONS}
$ {code} $
"#,
    )
}

fn inline_math_template(code: &str) -> String {
    format!(
        r#"
#set page(height: auto, width: auto, margin: 0pt, fill: none)
#set text(13pt)
#let s = state("t", (:))
{TYPST_HELPER_FUNCTIONS}

#let pin(t) = context {{
    let computed = measure(
        line(length: here().position().y)
    )
    s.update(it => it.insert(t, computed.width) + it)
    }}

#show math.equation: it => {{
    box(it, inset: (top: 0.5em, bottom: 0.5em))
    }}

$pin("l1"){code}$

#context [
    #metadata(s.final().at("l1")) <label>
]
"#,
    )
}

fn raw_template(code: &str) -> String {
    format!(
        r#"
#set page(height: auto, width: auto, margin: 0pt, fill: none)
#set text(16pt)
{TYPST_HELPER_FUNCTIONS}
{code}
"#,
    )
}
