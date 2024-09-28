use anyhow::{Context, Result};
use scraper::{Html, Selector};
use url::Url;

/// Extracts URLs from the given HTML content based on the provided base URL.
pub fn get_urls(url: Url, html: &str) -> Result<Vec<String>> {
        let fragment = Html::parse_document(html);
        let selector =
            Selector::parse("a").map_err(|e| anyhow::anyhow!("Failed to parse selector: {}", e))?;
        let mut urls = Vec::new();
        for element in fragment.select(&selector) {
            if let Some(path) = element.value().attr("href") {
                if path.starts_with('/') || path.starts_with("./") {
                    if let Some(base_url) = get_base_url(&url) {
                        let url = join_urls(&base_url, path).context("Failed to join URLs")?;
                        urls.push(url.to_string());
                    }
                } else {
                    match Url::parse(path) {
                        Ok(_) => {
                            urls.push(path.to_string());
                        }
                        Err(_) => {
                            if let Ok(url) = Url::parse(url.as_ref()) {
                                match join_urls(url.as_ref(), path) {
                                    Some(url) => urls.push(url),
                                    None => continue,
                                };
                            }
                        }
                    }
                }
            }
        }

    Ok(urls)
}

/// Creates a full URL from a base URL and a relative path.
fn create_full_url(base_url: &Url, path: &str) -> Result<String> {
    if path.starts_with('/') || path.starts_with("./") {
        // Handle relative paths
        let base = get_base_url(base_url).context("Failed to get base URL")?;
        return join_urls(&base, path).context("Failed to join URLs");
    }

    // Try to parse absolute URLs
    Url::parse(path)
        .map(|url| url.to_string())
        .or_else(|_| {
            let base_str = base_url.as_str();
            join_urls(base_str, path).ok_or_else(|| anyhow::anyhow!("Failed to join URLs"))
        })
}

/// Retrieves the base URL from a given URL.
fn get_base_url(url: &Url) -> Option<String> {
    url.domain().map(|domain| format!("{}://{}", url.scheme(), domain))
}

/// Joins a base URL and a relative URL to create a full URL.
fn join_urls(base_url_string: &str, relative_url: &str) -> Option<String> {
    let base_url = Url::parse(base_url_string).ok()?;
    base_url.join(relative_url).ok().map(|url| url.as_str().to_string())
}

/// Checks if a URL is from a specific domain.
fn is_url_from_domain(url: &str, domain: &str) -> bool {
    Url::parse(url)
        .map(|u| u.domain() == Some(domain))
        .unwrap_or(false)
}

/// Filters out URLs that are not from a specific domain.
fn filter_urls_by_domain(urls: Vec<String>, domain: &str) -> Vec<String> {
    urls.into_iter()
        .filter(|url| is_url_from_domain(url, domain))
        .collect()
}

// / Adds a query parameter to a URL.
fn add_query_parameter(url: &str, key: &str, value: &str) -> Result<String> {
    let mut parsed_url = Url::parse(url).context("Failed to parse URL")?;
    
    {
        // Mutable borrow to add the query parameter
        let mut query_pairs = parsed_url.query_pairs_mut();
        query_pairs.append_pair(key, value);
    } // Mutable borrow ends here

    // Now we can convert it to string as no mutable borrow exists
    Ok(parsed_url.into_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_get_urls() -> Result<()> {
        let url = "https://cats.com/cat-breeds";
        let html = r#"
            <html>
                <body>
                    <a href="/birman">Birman</a>
                    <a href="/cymric">Cymric</a>
                    <a href="https://cats.com/cat-breeds">Cat Breeds</a>
                </body>
            </html>
        "#;

        let urls = get_urls(Url::parse(url)?, html)?;
        assert_eq!(urls.len(), 3);
        assert_eq!(urls[0], "https://cats.com/cat-breeds/birman");
        assert_eq!(urls[1], "https://cats.com/cat-breeds/cymric");
        assert_eq!(urls[2], "https://cats.com/cat-breeds");
        Ok(())
    }

    #[test]
    fn test_get_urls2() -> Result<()> {
        let url = "https://cats.com/cat-breeds";
        let html = r#"
            <html>
                <body>
                    <a href="./birman">Birman</a>
                    <a href="/cymric">Cymric</a>
                    <a href="https://cats.com/cat-breeds">Cat Breeds</a>
                    <a href="https://cats.com/cat-breeds/birman">Cat Breeds</a>
                </body>
            </html>
        "#;

        let urls = get_urls(Url::parse(url)?, html).unwrap();
        assert_eq!(urls.len(), 4);
        assert_eq!(urls[0], "https://cats.com/cat-breeds/birman");
        assert_eq!(urls[1], "https://cats.com/cat-breeds/cymric");
        assert_eq!(urls[2], "https://cats.com/cat-breeds");
        assert_eq!(urls[3], "https://cats.com/cat-breeds/birman");
        Ok(())
    }

    #[test]
    fn test_combine_relative_url() {
        let a = "https://www.geeksforgeeks.org";
        let b = "/machine-learning/";

        let url = join_urls(a, b).unwrap();
        let expected = "https://www.geeksforgeeks.org/machine-learning/";
        assert_eq!(url, expected);
    }

    #[test]
    fn test_combine_relative_url2() {
        let a = "https://www.geeksforgeeks.org/";
        let b = "/machine-learning/";

        let url = join_urls(a, b).unwrap();
        let expected = "https://www.geeksforgeeks.org/machine-learning/";
        assert_eq!(url, expected);
    }

    #[test]
    fn test_combine_relative_url3() {
        let a = "https://www.linux.org/";
        let b = "./pages/download/";

        let url = join_urls(a, b).unwrap();
        let expected = "https://www.linux.org/pages/download/";
        assert_eq!(url, expected);
    }

    #[test]
    fn test_filter_urls_by_domain() {
        let urls = vec![
            "https://cats.com/cat-breeds/birman".to_string(),
            "https://cats.com/cat-breeds/cymric".to_string(),
            "https://cats.com/cat-breeds".to_string(),
        ];
        let filtered = filter_urls_by_domain(urls.clone(), "cats.com");
        assert_eq!(filtered.len(), 2);
        assert!(filtered.contains(&urls[0]));
        assert!(filtered.contains(&urls[2]));
        assert!(!filtered.contains(&urls[1]));
    }

    #[test]
    fn test_add_query_parameter() -> Result<()> {
        let url = "https://cats.com";
        let updated_url = add_query_parameter(url, "fact", "true")?;
        assert_eq!(updated_url, "https://cats.com?fact=true");
        Ok(())
    }
}
