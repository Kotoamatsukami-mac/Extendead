use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ServiceDefinition {
    pub id: &'static str,
    pub display_name: &'static str,
    pub aliases: &'static [&'static str],
    pub url: &'static str,
    pub category: &'static str,
}

pub static SERVICES: &[ServiceDefinition] = &[
    // Service catalog: Empty by design. This shell is local-first.
    // The app prioritizes installed macOS apps and system commands.
    // Web services can be opened via browser + URL, but are not built-in suggestions.
];

pub fn all_services() -> &'static [ServiceDefinition] {
    SERVICES
}

pub fn service_by_id(id: &str) -> Option<&'static ServiceDefinition> {
    SERVICES.iter().find(|service| service.id == id)
}

pub fn find_service_by_query(query: &str) -> Option<&'static ServiceDefinition> {
    let normalized = normalize(query);
    SERVICES.iter().find(|service| {
        service
            .aliases
            .iter()
            .any(|alias| normalize(alias) == normalized)
    })
}

pub fn search_services(query: &str, limit: usize) -> Vec<&'static ServiceDefinition> {
    let normalized = normalize(query);
    if normalized.is_empty() {
        return vec![];
    }

    SERVICES
        .iter()
        .filter(|service| {
            normalize(service.display_name).contains(&normalized)
                || service
                    .aliases
                    .iter()
                    .any(|alias| normalize(alias).contains(&normalized))
        })
        .take(limit)
        .collect()
}

pub fn approved_service_hosts() -> Vec<String> {
    SERVICES
        .iter()
        .filter_map(|service| extract_host(service.url))
        .collect()
}

pub fn is_approved_service_host(host: &str) -> bool {
    let normalized = host.trim().to_lowercase();
    SERVICES.iter().any(|service| {
        extract_host(service.url)
            .map(|known| known == normalized)
            .unwrap_or(false)
    })
}

fn normalize(value: &str) -> String {
    value
        .trim()
        .to_lowercase()
        .replace('+', " plus ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn extract_host(url: &str) -> Option<String> {
    let without_scheme = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))?;
    Some(without_scheme.split('/').next()?.to_lowercase())
}
