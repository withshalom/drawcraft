//! Help → project links (the ArtCraft website, this app's page there and DRAWCraft's GitHub
//! repository, which takes the place of upstream's Discord). The commands return the URL; the
//! frontend opens it.

use serde_json::{Value, json};

use super::{CommandSpec, Result, always, cmd};

/// The app's id on getartcraft.com and GitHub.
pub const APP_ID: &str = "vectorcraft";
pub const WEBSITE_URL: &str = "https://getartcraft.com";

/// This app's page on the website.
pub fn app_page_url() -> String {
    format!("{WEBSITE_URL}/apps/{APP_ID}")
}

/// This app's source repository: DRAWCraft's (upstream VectorCraft's is
/// `https://github.com/storytold/vectorcraft`).
pub fn github_url() -> String {
    "https://github.com/withshalom/drawcraft".to_string()
}

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(query "help.website", "ArtCraft Website", ["Help"], None, "{} → {url}", always, |_, _| url(WEBSITE_URL.into())),
        cmd!(query "help.appPage", "VectorCraft on getartcraft.com", ["Help"], None, "{} → {url} this app's page", always, |_, _| url(app_page_url())),
        cmd!(query "help.github", "VectorCraft on GitHub", ["Help"], None, "{} → {url} source code, issues and releases", always, |_, _| url(github_url())),
        cmd!(query "help.links", "Links", [], None, "{} → {website, appPage, github}", always, |_, _| {
            Ok(json!({ "website": WEBSITE_URL, "appPage": app_page_url(), "github": github_url() }))
        }),
    ]
}

fn url(u: String) -> Result<Value> {
    Ok(json!({ "url": u }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Session;

    #[test]
    fn links() {
        let mut s = Session::new();
        assert!(s.execute("help.discord", &json!({})).is_err(), "no Discord: DRAWCraft links its repository instead");
        assert_eq!(s.execute("help.appPage", &json!({})).unwrap()["url"], "https://getartcraft.com/apps/vectorcraft");
        assert_eq!(s.execute("help.github", &json!({})).unwrap()["url"], "https://github.com/withshalom/drawcraft");
        assert_eq!(s.execute("help.links", &json!({})).unwrap()["website"], "https://getartcraft.com");
    }
}
