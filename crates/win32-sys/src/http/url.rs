//! The parts of an `https://` URL that `WinHTTP` takes separately.

/// The default HTTPS port.
const HTTPS_PORT: u16 = 443;
const SCHEME: &str = "https://";

/// An `https://host[:port]/path` URL, split for `WinHTTP`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HttpsUrl {
    /// Host name, without the port.
    pub host: String,
    /// TCP port (443 unless the URL names one).
    pub port: u16,
    /// Path and query, starting with `/`.
    pub path: String,
}

impl HttpsUrl {
    /// Splits `url`; `None` when it is not a well-formed `https://` URL.
    pub fn parse(url: &str) -> Option<Self> {
        let rest = url.strip_prefix(SCHEME)?;
        let (authority, path) = rest
            .find('/')
            .map_or((rest, "/"), |slash| rest.split_at(slash));
        let (host, port) = match authority.split_once(':') {
            Some((host, port)) => (host, port.parse().ok()?),
            None => (authority, HTTPS_PORT),
        };
        if host.is_empty() {
            return None;
        }
        Some(Self {
            host: host.to_owned(),
            port,
            path: path.to_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::http::HttpsUrl;

    const CUSTOM_PORT: u16 = 8443;

    #[test]
    fn splits_host_port_and_path() {
        let url = HttpsUrl::parse("https://github.com/owner/repo/releases/latest");
        assert_eq!(
            url,
            Some(HttpsUrl {
                host: "github.com".to_owned(),
                port: 443,
                path: "/owner/repo/releases/latest".to_owned(),
            })
        );
        let custom = HttpsUrl::parse("https://example.org:8443").map(|url| (url.port, url.path));
        assert_eq!(custom, Some((CUSTOM_PORT, "/".to_owned())));
    }

    #[test]
    fn rejects_other_schemes_and_bad_ports() {
        assert_eq!(HttpsUrl::parse("http://github.com/"), None);
        assert_eq!(HttpsUrl::parse("https://:443/"), None);
        assert_eq!(HttpsUrl::parse("https://host:port/"), None);
    }
}
