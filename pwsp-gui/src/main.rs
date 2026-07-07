mod gui;

use anyhow::{Context, Result};
use pwsp_lib::utils::gui::ensure_pwsp_audio_dir;
use rust_i18n::i18n;
use std::{env, path::PathBuf};

i18n!("locales", fallback = "en");

#[tokio::main]
async fn main() -> Result<()> {
    let locale = sys_locale::get_locale().unwrap_or(String::from("en-US"));
    rust_i18n::set_locale(&locale);

    let args = env::args().skip(1).collect::<Vec<String>>();

    if let Some(uri) = args.first() {
        match download_audio_from_url(uri).await {
            Ok(path) => println!("Successfully downloaded to: {:?}", path),
            Err(e) => eprintln!("Error downloading file: {}", e),
        }
    } else {
        gui::run().await?;
    }

    Ok(())
}

async fn download_audio_from_url(uri: &str) -> Result<PathBuf> {
    let prefix = "soundpad://sound/url/";

    let target_url = uri
        .strip_prefix(prefix)
        .ok_or_else(|| anyhow::anyhow!("URI does not containt an expected prefix: {}", prefix))?;

    let file_name_encoded = target_url
        .split('/')
        .next_back()
        .unwrap_or("downloaded_audio.mp3");

    let file_name = percent_encoding::percent_decode_str(file_name_encoded)
        .decode_utf8()
        .unwrap_or_else(|_| file_name_encoded.into())
        .into_owned();

    let save_path = ensure_pwsp_audio_dir().join(file_name);

    let parsed_url = reqwest::Url::parse(target_url).context("Failed to parse target URL")?;

    // Validate host using DNS resolution to catch DNS rebinding/custom domains
    let host_str = parsed_url
        .host_str()
        .ok_or_else(|| anyhow::anyhow!("SSRF validation failed: no host in URL"))?;

    if host_str == "localhost" {
        return Err(anyhow::anyhow!("SSRF validation failed: host is localhost"));
    }

    // Try to resolve the host using tokio to avoid blocking the async executor
    let port = parsed_url.port_or_known_default().unwrap_or(80);
    let host_with_port = format!("{}:{}", host_str, port);

    if let Ok(addrs) = tokio::net::lookup_host(&host_with_port).await {
        for addr in addrs {
            let ip = addr.ip();
            let is_internal = ip.is_loopback()
                || ip.is_unspecified()
                || match ip {
                    std::net::IpAddr::V4(ipv4) => {
                        ipv4.is_private()
                            || ipv4.is_link_local()
                            || ipv4.is_broadcast()
                            || ipv4.is_documentation()
                    }
                    std::net::IpAddr::V6(ipv6) => ipv6.is_multicast(),
                };
            if is_internal {
                return Err(anyhow::anyhow!(
                    "SSRF validation failed: resolved to private or internal IP"
                ));
            }
        }
    } else {
        // If it doesn't resolve, it could be an invalid host or no internet.
        // Also fallback to parsing the URL host directly if it's an IP literal like IPv6 with brackets.
        if let Some(host) = parsed_url.host() {
            let ip_opt = match host {
                url::Host::Ipv4(ipv4) => Some(std::net::IpAddr::V4(ipv4)),
                url::Host::Ipv6(ipv6) => Some(std::net::IpAddr::V6(ipv6)),
                _ => None,
            };
            if let Some(ip) = ip_opt {
                let is_internal = ip.is_loopback()
                    || ip.is_unspecified()
                    || match ip {
                        std::net::IpAddr::V4(ipv4) => {
                            ipv4.is_private()
                                || ipv4.is_link_local()
                                || ipv4.is_broadcast()
                                || ipv4.is_documentation()
                        }
                        std::net::IpAddr::V6(ipv6) => ipv6.is_multicast(),
                    };
                if is_internal {
                    return Err(anyhow::anyhow!(
                        "SSRF validation failed: IP literal is private or internal"
                    ));
                }
            }
        }
    }

    // Use a custom client that handles redirects by validating them or rejecting them.
    // We reject redirects entirely to prevent TOCTOU and bypasses via 302 redirects to localhost.
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .context("Failed to build HTTP client")?;

    let response = client
        .get(target_url)
        .send()
        .await?
        .error_for_status()
        .context("Failed to fetch file")?;

    let bytes = response.bytes().await?;

    tokio::fs::write(&save_path, bytes)
        .await
        .context("Failed to save file to disk")?;

    Ok(save_path)
}
