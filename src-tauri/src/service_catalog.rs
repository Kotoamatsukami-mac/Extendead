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
    ServiceDefinition {
        id: "youtube",
        display_name: "YouTube",
        aliases: &["youtube", "yt"],
        url: "https://www.youtube.com",
        category: "streaming_video",
    },
    ServiceDefinition {
        id: "netflix",
        display_name: "Netflix",
        aliases: &["netflix"],
        url: "https://www.netflix.com",
        category: "streaming_video",
    },
    ServiceDefinition {
        id: "disney_plus",
        display_name: "Disney+",
        aliases: &["disney plus", "disney+"],
        url: "https://www.disneyplus.com",
        category: "streaming_video",
    },
    ServiceDefinition {
        id: "hulu",
        display_name: "Hulu",
        aliases: &["hulu"],
        url: "https://www.hulu.com",
        category: "streaming_video",
    },
    ServiceDefinition {
        id: "pluto_tv",
        display_name: "Pluto TV",
        aliases: &["pluto tv", "pluto"],
        url: "https://pluto.tv",
        category: "streaming_video",
    },
    ServiceDefinition {
        id: "tubi",
        display_name: "Tubi",
        aliases: &["tubi"],
        url: "https://tubitv.com",
        category: "streaming_video",
    },
    ServiceDefinition {
        id: "dailymotion",
        display_name: "Dailymotion",
        aliases: &["dailymotion"],
        url: "https://www.dailymotion.com",
        category: "streaming_video",
    },
    ServiceDefinition {
        id: "vimeo",
        display_name: "Vimeo",
        aliases: &["vimeo"],
        url: "https://vimeo.com",
        category: "streaming_video",
    },
    ServiceDefinition {
        id: "twitch",
        display_name: "Twitch",
        aliases: &["twitch"],
        url: "https://www.twitch.tv",
        category: "streaming_video",
    },
    ServiceDefinition {
        id: "reddit",
        display_name: "Reddit",
        aliases: &["reddit"],
        url: "https://www.reddit.com",
        category: "community",
    },
    ServiceDefinition {
        id: "walmart",
        display_name: "Walmart",
        aliases: &["walmart"],
        url: "https://www.walmart.com",
        category: "shopping",
    },
    ServiceDefinition {
        id: "target",
        display_name: "Target",
        aliases: &["target"],
        url: "https://www.target.com",
        category: "shopping",
    },
    ServiceDefinition {
        id: "ebay",
        display_name: "eBay",
        aliases: &["ebay", "e bay"],
        url: "https://www.ebay.com",
        category: "shopping",
    },
    ServiceDefinition {
        id: "amazon",
        display_name: "Amazon",
        aliases: &["amazon"],
        url: "https://www.amazon.com",
        category: "shopping",
    },
    ServiceDefinition {
        id: "stan",
        display_name: "Stan",
        aliases: &["stan"],
        url: "https://www.stan.com.au",
        category: "streaming_video",
    },
    ServiceDefinition {
        id: "nebula",
        display_name: "Nebula",
        aliases: &["nebula"],
        url: "https://nebula.tv",
        category: "streaming_video",
    },
    ServiceDefinition {
        id: "rumble",
        display_name: "Rumble",
        aliases: &["rumble"],
        url: "https://rumble.com",
        category: "streaming_video",
    },
    ServiceDefinition {
        id: "odysee",
        display_name: "Odysee",
        aliases: &["odysee"],
        url: "https://odysee.com",
        category: "streaming_video",
    },
    ServiceDefinition {
        id: "bitchute",
        display_name: "BitChute",
        aliases: &["bitchute", "bit chute"],
        url: "https://www.bitchute.com",
        category: "streaming_video",
    },
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
        service.aliases.iter().any(|alias| normalize(alias) == normalized)
    })
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
