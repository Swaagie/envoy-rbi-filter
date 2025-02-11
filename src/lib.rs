use std::collections::HashMap;
use std::fmt::Debug;
use std::str;

use log::debug;

use proxy_wasm::traits::*;
use proxy_wasm::types::*;

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> {
        Box::new(ResponseBodyInjectionConfig {
            config: HashMap::new(),
        })
    });
}}

const ECHO: &str = "<!--#echo";
const END: &str = "-->";

#[derive(Debug)]
struct ResponseBodyInjectionFilter {
    config: HashMap<String, String>,
    content_type: Option<String>,
}

impl Context for ResponseBodyInjectionFilter {}

impl HttpContext for ResponseBodyInjectionFilter {
    fn on_http_response_headers(&mut self, _num_headers: usize, _end_of_stream: bool) -> Action {
        self.content_type = self.get_http_response_header("content-type");

        // Only augment HTML content
        if self.content_type != Some("text/html".to_string()) {
            debug!(target: "RBI", "Skipping non-HTML content, type: {:?}", &self.content_type);
            return Action::Continue;
        }

        // Remove content-length since it's augmented and let clients decide
        self.set_http_response_header("content-length", None);
        self.set_http_response_header("Powered-By", Some("x-envoy-rbi-filter"));

        Action::Continue
    }

    fn on_http_response_body(&mut self, body_size: usize, end_of_stream: bool) -> Action {
        // Only augment HTML content
        if self.content_type != Some("text/html".to_string()) {
            debug!(target: "RBI", "Skipping non-HTML content, type: {:?}", &self.content_type);
            return Action::Continue;
        }

        if !end_of_stream {
            return Action::Pause;
        }

        // Read the response body and augment using RB
        debug!(target: "RBI", "Augment body content");
        if let Some(body) = &self.get_http_response_body(0, body_size) {
            let body = str::from_utf8(body).expect("Failed to read body from response");

            // Inject for each property:value from configuration
            let body = inject(body, &self.config);

            // Update the entire body with the new content
            let length = body.len();
            debug!(target: "RBI", "Updating response body, length {length}");

            // Update entire body with new content, length is inferred
            self.set_http_response_body(0, body_size, body.as_bytes());
        }

        Action::Continue
    }
}

struct ResponseBodyInjectionConfig {
    config: HashMap<String, String>,
}

impl Context for ResponseBodyInjectionConfig {}

impl RootContext for ResponseBodyInjectionConfig {
    fn on_configure(&mut self, _: usize) -> bool {
        let configuration: Vec<u8> = self.get_plugin_configuration().unwrap_or_default();

        match serde_json::from_slice(&configuration) {
            Ok(config) => self.config = config,
            Err(msg) => panic!("Invalid config {msg}"),
        };

        true
    }

    fn create_http_context(&self, _: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(ResponseBodyInjectionFilter {
            config: self.config.clone(),
            content_type: None,
        }))
    }

    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }
}

// Use configured content to replace SSI echo statements
fn inject(source: &str, content: &HashMap<String, String>) -> String {
    source
        .split(ECHO)
        .map(|s| {
            match s.contains(END) {
                true => {
                    let mut echo = s.split(END).collect::<Vec<&str>>();

                    // Extract variable name and read from content
                    if let Some(mut variable) = echo[0].trim().split('=').last() {
                        debug!(target: "RBI", "Found variable {variable} in SSI");

                        // Remove wrapping quotes
                        if variable.contains('"') {
                            variable = &variable[1..variable.chars().count() - 1]
                        };

                        // Read the variable as key, if the key does not exist remove the SSI comment
                        echo[0] = match content.get(variable) {
                            Some(new) => new,
                            None => "",
                        };
                    }

                    echo.join("")
                }
                false => s.to_string(),
            }
        })
        .collect::<String>()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_hello_world_injection() {
        let output = inject(
            "<html><head></head><body><!--#echo var=\"hello\" --></body></html>",
            &HashMap::from([(String::from("hello"), String::from("<h1>hello world</h1>"))]),
        );

        assert_eq!(
            output,
            "<html><head></head><body><h1>hello world</h1></body></html>"
        )
    }

    #[test]
    fn test_no_injection() {
        let output = inject(
            "<html><head></head><body><!--#echo var=hello --></body></html>",
            &HashMap::from([(
                String::from("different"),
                String::from("<h1>hello world</h1>"),
            )]),
        );

        assert_eq!(output, "<html><head></head><body></body></html>")
    }

    #[test]
    fn test_html_injection() {
        let output = inject(
            "<html><head></head><body><main>Hi there!</main><!--#echo var=\"script\" --></body></html>",
        &HashMap::from([
                (String::from("script"), String::from("<script async src=\"https://www.google-analytics.com/analytics.js\"></script>"))
            ]),
        );

        assert_eq!(output, "<html><head></head><body><main>Hi there!</main><script async src=\"https://www.google-analytics.com/analytics.js\"></script></body></html>");
    }

    #[test]
    fn test_without_quotes_injection() {
        let output = inject(
            "<html><head></head><body><main>Hi there!</main><!--#echo var=script --></body></html>",
            &HashMap::from([(
                String::from("script"),
                String::from(
                    "<script async src=\"https://www.google-analytics.com/analytics.js\"></script>",
                ),
            )]),
        );

        assert_eq!(output, "<html><head></head><body><main>Hi there!</main><script async src=\"https://www.google-analytics.com/analytics.js\"></script></body></html>");
    }

    #[test]
    fn test_head_injection() {
        let output = inject(
            "<html><head><!--#echo var=\"title\" --></head><body></body></html>",
            &HashMap::from([(String::from("title"), String::from("<title>Test</title>"))]),
        );

        assert_eq!(
            output,
            "<html><head><title>Test</title></head><body></body></html>"
        );
    }

    #[test]
    fn test_newline_multi_injection() {
        let output = inject(
            "
                <html>
                    <head><!--#echo var=\"title\" --></head>
                    <body>
                        <section><!--#echo var=\"header\" --></section>
                        <section><!--#echo var=\"header\" --></section>
                    </body>
                </html>
            ",
            &HashMap::from([
                (String::from("title"), String::from("<title>Test</title>")),
                (
                    String::from("header"),
                    String::from("<h2>Multi header</h2>"),
                ),
            ]),
        );

        assert_eq!(
            output,
            "
                <html>
                    <head><title>Test</title></head>
                    <body>
                        <section><h2>Multi header</h2></section>
                        <section><h2>Multi header</h2></section>
                    </body>
                </html>
            "
        );
    }
}
