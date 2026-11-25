use url::Url;

/// Returns true if the URL belongs to Antigravity or Google's authentication/account
/// infrastructure and should stay inside the embedded client WebView.
pub fn is_in_app_url(url: &Url) -> bool {
    match url.scheme() {
        "about" | "blob" | "data" | "tauri" => return true,
        "http" | "https" => {}
        _ => return false,
    }

    let Some(host) = url.host_str() else {
        return false;
    };
    let host = host.to_ascii_lowercase();

    // Tauri internal local origin
    if host == "tauri.localhost" || host == "localhost" {
        return true;
    }

    // Antigravity domains
    if host == "antigravity.google.com"
        || host.ends_with(".antigravity.google.com")
        || host == "antigravity.google"
        || host.ends_with(".antigravity.google")
    {
        return true;
    }

    // Google authentication, sign-in redirects (including regional accounts.google.*),
    // session cookies, and supporting static/API endpoints.
    if host.starts_with("accounts.google.")
        || host == "accounts.youtube.com"
        || host == "myaccount.google.com"
        || host == "gds.google.com"
        || host == "consent.google.com"
        || host == "ogs.google.com"
        || host == "id.google.com"
        || host == "apis.google.com"
        || host == "clients6.google.com"
        || host.ends_with(".clients6.google.com")
        || host == "gstatic.com"
        || host.ends_with(".gstatic.com")
        || host == "googleusercontent.com"
        || host.ends_with(".googleusercontent.com")
        || host == "googleapis.com"
        || host.ends_with(".googleapis.com")
    {
        return true;
    }

    false
}

/// Extracts an Antigravity URL if passed on the command line (e.g. desktop shortcut or deep link).
pub fn extract_cli_url(args: &[String]) -> Option<Url> {
    args.iter().skip(1).find_map(|arg| {
        let parsed = Url::parse(arg).ok()?;
        let host = parsed.host_str()?.to_ascii_lowercase();
        if (parsed.scheme() == "https" || parsed.scheme() == "http")
            && (host == "antigravity.google.com"
                || host.ends_with(".antigravity.google.com")
                || host == "antigravity.google")
        {
            Some(parsed)
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_antigravity_and_google_auth_domains() {
        assert!(is_in_app_url(&Url::parse("https://antigravity.google.com").unwrap()));
        assert!(is_in_app_url(&Url::parse("https://antigravity.google.com/r/abc123").unwrap()));
        assert!(is_in_app_url(&Url::parse("https://accounts.google.com/ServiceLogin").unwrap()));
        assert!(is_in_app_url(&Url::parse("https://accounts.google.co.uk/CheckCookie").unwrap()));
        assert!(is_in_app_url(&Url::parse("https://accounts.youtube.com/accounts/SetSID").unwrap()));
        assert!(is_in_app_url(&Url::parse("https://myaccount.google.com").unwrap()));
    }

    #[test]
    fn sends_external_sites_out_to_browser() {
        assert!(!is_in_app_url(&Url::parse("https://github.com/google").unwrap()));
        assert!(!is_in_app_url(&Url::parse("https://stackoverflow.com/questions/1").unwrap()));
        assert!(!is_in_app_url(&Url::parse("https://example.com").unwrap()));
    }

    #[test]
    fn extracts_cli_antigravity_url() {
        let args = vec![
            "antigravity-client.exe".to_string(),
            "https://antigravity.google.com/r/machine-1".to_string(),
        ];
        assert_eq!(
            extract_cli_url(&args).unwrap().as_str(),
            "https://antigravity.google.com/r/machine-1"
        );
    }
}
