#[cfg(test)]
mod test {
    #[tokio::test]
    async fn test_e2e_request() -> std::result::Result<(), reqwest::Error> {
        let response = reqwest::get("http://localhost:10000/").await?;

        assert_eq!(response.status(), 200);
        assert_eq!(
            response.text().await?,
            "<html><body><h1>Hello WASM</h1></body></html>"
        );

        Ok(())
    }
}
