mod parser;
mod serializer;

use std::collections::VecDeque;
use std::default::Default;
use std::str;

use log::trace;

use html5ever::interface::tree_builder::NodeOrText;
use html5ever::interface::TreeSink;
use html5ever::tendril::*;
use html5ever::{parse_document, parse_fragment, serialize, QualName};

use string_cache::Atom;

use proxy_wasm::traits::*;
use proxy_wasm::types::*;

use parser::{Dom, Handle, NodeData};
use serializer::SerializableHandle;

#[no_mangle]
pub fn _start() {
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_http_context(|_, _| -> Box<dyn HttpContext> { Box::new(ResponseBodyInjectionFilter {}) });
}

struct ResponseBodyInjectionFilter {}

impl Context for ResponseBodyInjectionFilter {}

impl HttpContext for ResponseBodyInjectionFilter {
    fn on_http_response_body(&mut self, body_size: usize, end_of_stream: bool) -> Action {
        if !end_of_stream {
            return Action::Pause;
        }

        // Set proper max buffer from content-length
        if let Some(body) = &self.get_http_response_body(0, body_size) {
            let body = str::from_utf8(body).expect("Failed to read body from response");

            // TODO: read configuration and apply injection for each property:value
            let body = match inject(body, "body", "<h1>Hello from WASM</h1>") {
                Ok(result) => result,
                Err(error) => {
                    trace!("There was a problem parsing the HTML: {error}");
                    String::from(body)
                }
            };

            self.set_http_response_body(
                0,
                body_size,
                body.as_bytes(),
            );
        }

        Action::Continue
    }

    fn on_http_response_headers(&mut self, _: usize) -> Action {
        self.set_http_response_header("Content-Length", None);
        self.set_http_response_header("Powered-By", Some("proxy-wasm"));

        Action::Continue
    }
}

// TODO broken in wasi build
fn inject(src: &str, target: &str, fragment: &str) -> Result<String, std::string::FromUtf8Error> {
    let mut dom = parse_document(Dom::default(), Default::default()).one(src);
    let mut ops: VecDeque<Handle> = VecDeque::new();
    ops.push_back(dom.document.clone());

    // Unwrap parsed fragment, find better way! Namespace matters for result
    let fragment = parse_fragment(
        Dom::default(),
        Default::default(),
        QualName::new(None, Atom::from("html"), Atom::from("html")),
        vec![],
    ).one(fragment);

    let html = &fragment.document.children.borrow_mut()[0];
    let value = &html.children.borrow_mut()[0].clone();
    dom.remove_from_parent(value);

    while let Some(handle) = ops.pop_front() {
        // Push any children to the front of the queue for iteration
        for child in handle.children.borrow().iter().rev() {
            ops.push_front(child.clone());
        }

        if let NodeData::Element { ref name, .. } = handle.data {
            // Element local name matches the target, insert fragment.
            if name.local == *target {
                dom.append(&handle, NodeOrText::AppendNode(value.clone()));
            }
        };
    }

    let document: SerializableHandle = dom.document.into();
    let mut result = vec![];

    serialize(&mut result, &document, Default::default()).expect("failed to serialize to HTML");

    String::from_utf8(result)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_hello_world_injection() -> Result<(), std::string::FromUtf8Error> {
        let output = inject(
            "<html><head></head><body></body></html>",
            "body",
            "hello world",
        )?;

        assert_eq!(output, "<html><head></head><body>hello world</body></html>");

        Ok(())
    }

    #[test]
    fn test_html_injection() -> Result<(), std::string::FromUtf8Error> {
        let output = inject(
            "<html><head></head><body><main>Hi there!</main></body></html>",
            "body",
            "<script async src=\"https://www.google-analytics.com/analytics.js\"></script>",
        )?;

        assert_eq!(output, "<html><head></head><body><main>Hi there!</main><script async=\"\" src=\"https://www.google-analytics.com/analytics.js\"></script></body></html>");

        Ok(())
    }

    #[test]
    fn test_partial_head_injection() -> Result<(), std::string::FromUtf8Error> {
        let output = inject(
            "<html><head></head><body></body></html>",
            "head",
            "<title>Test",
        )?;

        assert_eq!(
            output,
            "<html><head><title>Test</title></head><body></body></html>"
        );

        Ok(())
    }

    #[test]
    fn test_empty_body() -> Result<(), std::string::FromUtf8Error> {
        let output = inject(
            "",
            "head",
            "<title>Test</title>",
        )?;

        assert_eq!(
            output,
            "<html><head><title>Test</title></head><body></body></html>"
        );

        Ok(())
    }
}