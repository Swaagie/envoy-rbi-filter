 #[macro_use] extern crate html5ever;

mod parser;
mod serializer;

use parser::{Dom, Handle, NodeData};
use serializer::{SerializableHandle};

use std::collections::{VecDeque};
use std::default::Default;
use std::str;
use log::{trace};

use html5ever::{parse_document, parse_fragment, serialize, QualName, LocalName};
use html5ever::interface::tree_builder::{NodeOrText};
use html5ever::interface::TreeSink;
use html5ever::tendril::*;

use proxy_wasm::traits::*;
use proxy_wasm::types::*;

// #[no_mangle]
// Fails Linux compile, just let it mangle it for now
pub fn _start() {
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_http_context(|_, _| -> Box<dyn HttpContext> {
        Box::new(RBI {})
    });
}

struct RBI {}

impl Context for RBI {}

impl HttpContext for RBI {
    fn on_http_response_body(&mut self, _: usize, _: bool) -> Action {
        // Set proper max buffer
        // Remove unwrap
        let body = &self.get_http_response_body(0, 100000).unwrap();
        let body = str::from_utf8(&body).unwrap();

        // Read configuration and apply injection for each property:value
        trace!("In WASM: {}", body);
        let body = match inject(body, "html", "<h1>test</h1>") {
            Ok(result) => result,
            Err(error) => {
                panic!("There was a problem parsing the HTML: {:?}", error)
            }
        };

        // Use status code, headers from original
        self.send_http_response(
            200,
            vec![("Powered-By", "proxy-wasm")],
            Some(body.as_bytes()),
        );

        Action::Continue
    }
}

fn inject(src: &str, target: &str, fragment: &str) -> Result<String, std::string::FromUtf8Error> {
    let mut dom = parse_document(Dom::default(), Default::default()).one(src);
    let mut ops: VecDeque<Handle> = VecDeque::new();
    ops.push_back(dom.document.clone());

    // Unwrap parsed fragment, find better way! Namespace matters for result
    let fragment = parse_fragment(Dom::default(), Default::default(), QualName::new(None, ns!(xml), LocalName::from("html")), vec![]).one(fragment);
    let html = &fragment.document.children.borrow_mut()[0];
    let value = &html.children.borrow_mut()[0].clone();
    dom.remove_from_parent(&value);

    while let Some(op) = ops.pop_front() {
        match op {
            handle => {
                // Push any children to the front of the queue for iteration
                for child in handle.children.borrow().iter().rev() {
                    ops.push_front(child.clone());
                }

                match handle.data {
                    NodeData::Element { ref name, .. } => {
                        // Element local name matches the target, insert fragment.
                        if name.local == LocalName::from(target) {
                            dom.append(&handle, NodeOrText::AppendNode(value.clone()));
                        }
                    },
                    _ => {}
                }
            }
        }
    }

    let document: SerializableHandle = dom.document.into();
    let mut result = vec![];

    serialize(&mut result, &document, Default::default())
        .expect("failed to serialize to HTML");

    String::from_utf8(result)
}

#[test]
fn test_hello_world_injection() -> Result<(), std::string::FromUtf8Error> {
    let output = inject("<html><head></head><body></body></html>", "body", "hello world")?;
    assert_eq!(output, "<html><head></head><body>hello world</body></html>");

    Ok(())
}

#[test]
fn test_html_injection() -> Result<(), std::string::FromUtf8Error> {
    let output = inject("<html><head></head><body><main>Hi there!</main></body></html>", "body", "<script async src=\"https://www.google-analytics.com/analytics.js\"></script>")?;
    assert_eq!(output, "<html><head></head><body><main>Hi there!</main><script async=\"\" src=\"https://www.google-analytics.com/analytics.js\"></script></body></html>");

    Ok(())
}

#[test]
fn test_partial_head_injection() -> Result<(), std::string::FromUtf8Error> {
    let output = inject("<html><head></head><body></body></html>", "head", "<title>Test")?;
    assert_eq!(output, "<html><head><title>Test</title></head><body></body></html>");

    Ok(())
}