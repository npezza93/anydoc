//! Ruby bindings for anydoc.

use magnus::{
    Error, Exception, RString, Ruby, Value, exception::ExceptionClass, function, prelude::*,
};
use std::{ffi::c_void, panic::AssertUnwindSafe, ptr};

mod document;

const FORMATS: [(&str, anydoc_core::Format); 12] = [
    ("doc", anydoc_core::Format::Doc),
    ("docx", anydoc_core::Format::Docx),
    ("odt", anydoc_core::Format::Odt),
    ("pdf", anydoc_core::Format::Pdf),
    ("ppt", anydoc_core::Format::Ppt),
    ("pptx", anydoc_core::Format::Pptx),
    ("rtf", anydoc_core::Format::Rtf),
    ("epub", anydoc_core::Format::Epub),
    ("xlsx", anydoc_core::Format::Excel),
    ("ods", anydoc_core::Format::Ods),
    ("odp", anydoc_core::Format::Odp),
    ("csv", anydoc_core::Format::Csv),
];

fn parse_format(ruby: &Ruby, name: &str) -> Result<anydoc_core::Format, Error> {
    FORMATS.iter().find(|(candidate, _)| *candidate == name).map(|(_, format)| *format).ok_or_else(
        || {
            let names = FORMATS.iter().map(|(name, _)| *name).collect::<Vec<_>>().join(", ");
            Error::new(
                ruby.exception_arg_error(),
                format!("unknown format {name:?}; expected one of {names}"),
            )
        },
    )
}

fn format_name(format: anydoc_core::Format) -> &'static str {
    FORMATS
        .iter()
        .find(|(_, candidate)| *candidate == format)
        .map(|(name, _)| *name)
        .expect("every format is named")
}

fn convert_error(ruby: &Ruby, error: anydoc_core::ConvertError) -> Error {
    match error {
        anydoc_core::ConvertError::Io(error) => match error.raw_os_error() {
            Some(errno) => ruby
                .exception_system_call_error()
                .funcall::<_, _, Exception>("new", (error.to_string(), errno))
                .map(Error::from)
                .unwrap_or_else(|error| error),
            None => Error::new(ruby.exception_io_error(), error.to_string()),
        },
        other => {
            let module = ruby.define_module("Anydoc").expect("Anydoc module is defined");
            let class: ExceptionClass =
                module.const_get("ConvertError").expect("Anydoc::ConvertError is defined");
            Error::new(class, other.to_string())
        }
    }
}

fn bytes(data: RString) -> Vec<u8> {
    // Copy before conversion so Rust never retains a pointer into Ruby's heap.
    unsafe { data.as_slice() }.to_vec()
}

/// Run CPU- and I/O-heavy parsing without holding Ruby's global VM lock.
fn without_gvl<F, T>(function: F) -> T
where
    F: FnOnce() -> T + Send,
    T: Send,
{
    struct Call<F, T> {
        function: Option<F>,
        result: Option<std::thread::Result<T>>,
    }

    unsafe extern "C" fn call<F, T>(data: *mut c_void) -> *mut c_void
    where
        F: FnOnce() -> T,
    {
        let call = unsafe { &mut *data.cast::<Call<F, T>>() };
        let function = call.function.take().expect("without_gvl callback runs once");
        call.result = Some(std::panic::catch_unwind(AssertUnwindSafe(function)));
        ptr::null_mut()
    }

    let mut state = Call { function: Some(function), result: None };
    unsafe {
        rb_sys::rb_thread_call_without_gvl(
            Some(call::<F, T>),
            ptr::from_mut(&mut state).cast(),
            None,
            ptr::null_mut(),
        );
    }
    match state.result.expect("without_gvl callback completed") {
        Ok(value) => value,
        Err(panic) => std::panic::resume_unwind(panic),
    }
}

fn format_from_bytes(data: RString) -> Option<&'static str> {
    anydoc_core::Format::from_bytes(&bytes(data)).map(format_name)
}

fn format_from_extension(extension: String) -> Option<&'static str> {
    anydoc_core::Format::from_extension(extension.trim_start_matches('.')).map(format_name)
}

fn format_from_path(path: String) -> Option<&'static str> {
    anydoc_core::Format::from_path(std::path::Path::new(&path)).map(format_name)
}

fn to_markdown(ruby: &Ruby, path: String) -> Result<String, Error> {
    without_gvl(move || anydoc_core::to_markdown(path)).map_err(|error| convert_error(ruby, error))
}

fn to_markdown_bytes(ruby: &Ruby, data: RString, format: Option<String>) -> Result<String, Error> {
    let format = format.as_deref().map(|name| parse_format(ruby, name)).transpose()?;
    let data = bytes(data);
    without_gvl(move || anydoc_core::to_markdown_bytes(&data, format))
        .map_err(|error| convert_error(ruby, error))
}

fn to_document(ruby: &Ruby, data: RString, format: Option<String>) -> Result<Value, Error> {
    let format = format.as_deref().map(|name| parse_format(ruby, name)).transpose()?;
    let data = bytes(data);
    let parsed = without_gvl(move || anydoc_core::to_document(&data, format))
        .map_err(|error| convert_error(ruby, error))?;
    document::document(ruby, parsed)
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("Anydoc")?;
    module.define_singleton_method("_format_from_bytes", function!(format_from_bytes, 1))?;
    module
        .define_singleton_method("_format_from_extension", function!(format_from_extension, 1))?;
    module.define_singleton_method("_format_from_path", function!(format_from_path, 1))?;
    module.define_singleton_method("_to_markdown", function!(to_markdown, 1))?;
    module.define_singleton_method("_to_markdown_bytes", function!(to_markdown_bytes, 2))?;
    module.define_singleton_method("_to_document", function!(to_document, 2))?;
    Ok(())
}
