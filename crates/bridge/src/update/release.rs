//! The newest GitHub release, read from where `releases/latest` redirects (no JSON needed).

use std::io;

use win32_sys::http::{self, Redirects};

use crate::update::version::Version;

const REPOSITORY_URL: &str = "https://github.com/avi12/quest-touch-to-wii-emulation";
/// The asset every release carries: this program.
pub(crate) const ASSET_NAME: &str = "touch_dsu.exe";
const USER_AGENT: &str = "touch_dsu-updater";
const TAG_PATH: &str = "/releases/tag/";
const NOT_FOUND: u16 = 404;
const REDIRECT_STATUSES: [u16; 2] = [301, 302];

/// The newest release's version, or `None` when the repository has no release yet.
pub(crate) fn latest() -> io::Result<Option<Version>> {
    let response = http::get(
        &format!("{REPOSITORY_URL}/releases/latest"),
        USER_AGENT,
        Redirects::Manual,
    )?;
    if response.status == NOT_FOUND {
        return Ok(None);
    }
    let is_redirect = REDIRECT_STATUSES.contains(&response.status);
    if !is_redirect {
        return Err(io::Error::other(format!(
            "releases/latest answered HTTP {}",
            response.status
        )));
    }
    Ok(response.location.as_deref().and_then(tag_of))
}

/// The release a `Location` such as `.../releases/tag/v1.2.3` points at.
fn tag_of(location: &str) -> Option<Version> {
    let (_, tag) = location.rsplit_once(TAG_PATH)?;
    Version::parse(tag)
}

/// The bytes of `version`'s release asset.
pub(crate) fn download(version: Version) -> io::Result<Vec<u8>> {
    let url = format!(
        "{REPOSITORY_URL}/releases/download/{}/{ASSET_NAME}",
        version.tag()
    );
    let response = http::get(&url, USER_AGENT, Redirects::Follow)?;
    let is_ok = response.status == http::STATUS_OK;
    if !is_ok {
        return Err(io::Error::other(format!(
            "download answered HTTP {}",
            response.status
        )));
    }
    Ok(response.body)
}

#[cfg(test)]
mod tests {
    use crate::update::release::tag_of;
    use crate::update::version::Version;

    #[test]
    fn reads_the_tag_from_the_redirect() {
        let location = "https://github.com/avi12/quest-touch-to-wii-emulation/releases/tag/v1.2.3";
        assert_eq!(tag_of(location), Version::parse("1.2.3"));
        assert_eq!(
            tag_of("https://github.com/avi12/quest-touch-to-wii-emulation/releases"),
            None
        );
    }
}
