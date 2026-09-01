fn test_map_data(data: &str) {
    let result = dynops::cluster::entrypoint(data);
    assert!(result.is_ok(), "{:?}", result.err());
    let Ok(output) = result else {
        return;
    };

    let ellipses: Vec<&str> = output
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(",\n")
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    assert!(!ellipses.is_empty(), "expected at least one cluster");

    for ellipse in ellipses {
        let parsed: Result<Vec<f64>, _> = ellipse
            .trim_start_matches('[')
            .trim_end_matches(']')
            .split(',')
            .map(|v| v.parse::<f64>())
            .collect();
        let Ok(params) = parsed else {
            panic!(
                "failed to parse ellipse params from {}: {:?}",
                ellipse, parsed
            );
        };
        assert_eq!(params.len(), 5, "expected [x,y,a,b,r], got {}", ellipse);
        for param in params {
            assert!(
                param.is_finite(),
                "non-finite ellipse parameter in {}",
                ellipse
            );
        }
    }
}

#[test]
fn test_map_altis() {
    test_map_data(include_str!("../data/objects.Altis.txt"));
}

#[test]
fn test_map_stratis() {
    test_map_data(include_str!("../data/objects.Stratis.txt"));
}

#[test]
fn test_map_livonia() {
    test_map_data(include_str!("../data/objects.Livonia.txt"));
}

#[test]
fn test_map_tanoa() {
    test_map_data(include_str!("../data/objects.Tanoa.txt"));
}

#[test]
fn test_map_malden() {
    test_map_data(include_str!("../data/objects.Malden.txt"));
}

#[test]
fn test_map_chernarus() {
    test_map_data(include_str!("../data/objects.Chernarus2020.txt"));
}
