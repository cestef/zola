use typst::comemo::{Track, Tracked};
use typst::diag::{FileError, FileResult, SourceDiagnostic, SourceResult};
use typst::ecow::{EcoString, EcoVec};
use typst::engine::{Route, Sink, Traced};
use typst::foundations::{
    Bytes, Datetime, Module, NativeElement, Packed, Selector, StyledElem, Styles,
};
use typst::math::EquationElem;
use typst::syntax::{FileId, Source};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, World, ROUTINES};

#[derive(Debug)]
pub enum EvalMathResult {
    CompilerInnerError(EcoVec<SourceDiagnostic>),
    NoEquationError,
}

impl ToString for EvalMathResult {
    fn to_string(&self) -> String {
        match self {
            EvalMathResult::CompilerInnerError(e) => {
                format!("Compiler inner error: {:?}", e)
            }
            EvalMathResult::NoEquationError => "No equation found".to_string(),
        }
    }
}

pub fn eval(content: &str) -> Result<(Packed<EquationElem>, Option<Styles>), EvalMathResult> {
    let traced = Traced::default();
    let world = TypstWrapperWorld::new(content.to_string());

    let eval_res = match eval_impl(&world, traced.track()) {
        Ok(res) => res,
        Err(e) => return Err(EvalMathResult::CompilerInnerError(e)),
    };

    let typst_content = eval_res.content();
    let styles = if typst_content.elem().name() == "styled" {
        let coerced = typst_content
            .to_packed::<StyledElem>()
            .expect("Type conversion to `StyledElem` must be successful.");
        Some(coerced.styles.to_owned())
    } else {
        None
    };

    let typst_elem = match typst_content.query_first(&Selector::Elem(EquationElem::elem(), None)) {
        Some(content) => content,
        None => return Err(EvalMathResult::NoEquationError),
    };

    Ok((
        typst_elem.to_packed::<EquationElem>().expect("Conversion must be successful.").to_owned(),
        styles,
    ))
}

fn eval_impl(world: &dyn World, traced: Tracked<Traced>) -> SourceResult<Module> {
    let mut sink = Sink::new();

    let main = world.main();
    let main = world.source(main).unwrap();

    typst_eval::eval(
        &ROUTINES,
        world.track(),
        traced,
        sink.track_mut(),
        Route::default().track(),
        &main,
    )
}

/// Main interface that determines the environment for Typst.
pub struct TypstWrapperWorld {
    /// The content of a source.
    source: Source,

    /// The standard library.
    library: LazyHash<Library>,

    /// Metadata about all known fonts.
    book: LazyHash<FontBook>,

    /// Metadata about all known fonts.
    fonts: Vec<Font>,
}

impl TypstWrapperWorld {
    pub fn new(source: String) -> Self {
        let fonts = Vec::with_capacity(0);

        Self {
            library: LazyHash::new(Library::default()),
            book: LazyHash::new(FontBook::from_fonts(&fonts)),
            fonts,
            source: Source::detached(source),
        }
    }
}

impl typst::World for TypstWrapperWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    fn main(&self) -> FileId {
        self.source.id()
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.source.id() {
            Ok(self.source.clone())
        } else {
            Err(FileError::Other(Some(EcoString::inline("sourceddd"))))
        }
    }

    fn file(&self, _id: FileId) -> FileResult<Bytes> {
        Err(FileError::Other(Some(EcoString::inline("fileddd"))))
    }

    fn font(&self, id: usize) -> Option<Font> {
        self.fonts.get(id).cloned()
    }

    /// In math equation, datatime may be useless.
    fn today(&self, _offset: Option<i64>) -> Option<Datetime> {
        None
    }
}
