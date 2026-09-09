mod bounding;
pub mod cluster;
pub mod geometry;
pub mod kdtree;
pub mod locations;
mod shape;

use anyhow::Context as _;
use arma_rs::{Context, ContextState, Extension, Group, arma};
use geometry::obb_to_marker_dir;
use locations::{DetectParams, Location};
use nalgebra::Vector2;
use std::result::Result;
use std::sync::Mutex;
use uuid::Uuid;

/// A page of a `Detected` session's output never exceeds this many bytes, well
/// under `callExtension`'s 10240-byte hard cap.
const PAGE_BYTE_BUDGET: usize = 9000;

/// State machine for the chunked `locations:*` protocol. Held in extension-wide
/// state (`Mutex<Option<Session>>`) so invalid call sequences -- paging before
/// detecting, adding after detecting -- are rejected rather than silently
/// operating on stale data.
enum Session {
    Collecting(Vec<Vector2<f64>>),
    Detected(Vec<Location>),
}

#[arma]
fn init() -> Extension {
    Extension::build()
        .state(Mutex::new(None::<Session>))
        .command("ok", || -> Result<String, String> { Ok("OK".to_owned()) })
        .command("err", || -> Result<String, String> {
            Err("ERR".to_owned())
        })
        .command("echo", echo)
        .command("hint", hint)
        .command("uuid", Uuid::new_v4)
        .group(
            "locations",
            Group::new()
                .command("begin", locations_begin)
                .command("add", locations_add)
                .command("detect", locations_detect)
                .command("page", locations_page)
                .command("end", locations_end),
        )
        .command("onMissionEnded", on_mission_ended)
        .finish()
}

fn echo(input: Vec<String>) -> String {
    format!("echo({})", input.join(", "))
}

fn hint(context: Context, input: String) -> Result<String, String> {
    match context.callback_data("dynops", "hint", input) {
        Ok(_) => Ok("OK".to_owned()),
        Err(e) => Err(e.to_string()),
    }
}

fn session_mutex(context: &Context) -> Result<&Mutex<Option<Session>>, String> {
    context
        .global()
        .get::<Mutex<Option<Session>>>()
        .ok_or_else(|| "extension state unavailable".to_string())
}

fn lock(
    mutex: &Mutex<Option<Session>>,
) -> Result<std::sync::MutexGuard<'_, Option<Session>>, String> {
    mutex
        .lock()
        .map_err(|_| "session lock poisoned".to_string())
}

fn parse_chunk(data: &str) -> anyhow::Result<Vec<Vector2<f64>>> {
    data.lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            let (x, y) = line
                .split_once(',')
                .with_context(|| format!("expected \"x,y\", got {line:?}"))?;
            Ok(Vector2::new(
                x.trim()
                    .parse::<f64>()
                    .with_context(|| format!("bad x coordinate in {line:?}"))?,
                y.trim()
                    .parse::<f64>()
                    .with_context(|| format!("bad y coordinate in {line:?}"))?,
            ))
        })
        .collect()
}

fn locations_begin(context: Context) -> Result<String, String> {
    let mut guard = lock(session_mutex(&context)?)?;
    *guard = Some(Session::Collecting(Vec::new()));
    Ok("OK".to_owned())
}

fn locations_add(context: Context, chunk: String) -> Result<String, String> {
    let points = parse_chunk(&chunk).map_err(|e| e.to_string())?;
    let mut guard = lock(session_mutex(&context)?)?;
    match guard.as_mut() {
        Some(Session::Collecting(accumulated)) => {
            accumulated.extend(points);
            Ok(accumulated.len().to_string())
        }
        _ => Err("locations:add called without an active locations:begin session".to_owned()),
    }
}

#[allow(clippy::too_many_arguments)]
fn locations_detect(
    context: Context,
    eps: f64,
    max_span: f64,
    split_factor: f64,
    min_eps: f64,
    min_buildings: usize,
) -> Result<String, String> {
    let params = DetectParams {
        eps,
        max_span,
        split_factor,
        min_eps,
        min_buildings,
    };
    let mut guard = lock(session_mutex(&context)?)?;
    let points = match guard.take() {
        Some(Session::Collecting(points)) => points,
        _ => {
            return Err(
                "locations:detect called without an active locations:begin session".to_owned(),
            );
        }
    };
    let detected = locations::detect(&points, &params);
    let count = detected.len();
    *guard = Some(Session::Detected(detected));
    Ok(count.to_string())
}

fn format_entry(location: &Location) -> String {
    format!(
        "[[{},{}],{},{},{},{},{}]",
        location.obb.center.x,
        location.obb.center.y,
        location.obb.a,
        location.obb.b,
        obb_to_marker_dir(location.obb.angle),
        location.buildings,
        location.class.index(),
    )
}

/// Builds one page of `locations` starting at `offset`, stopping before the
/// returned string would exceed [`PAGE_BYTE_BUDGET`] (always including at
/// least one entry, to guarantee forward progress). Returns the offset the
/// next page should start at, and the page itself as `[nextOffset, [entry, ...]]`.
fn format_page(locations: &[Location], offset: usize) -> (usize, String) {
    let mut idx = offset;
    let mut body = String::new();
    while idx < locations.len() {
        let entry = format_entry(&locations[idx]);
        let mut candidate_body = body.clone();
        if !candidate_body.is_empty() {
            candidate_body.push(',');
        }
        candidate_body.push_str(&entry);
        let candidate = format!("[{},[{}]]", idx + 1, candidate_body);
        if candidate.len() > PAGE_BYTE_BUDGET && !body.is_empty() {
            break;
        }
        body = candidate_body;
        idx += 1;
    }
    (idx, format!("[{},[{}]]", idx, body))
}

fn locations_page(context: Context, offset: usize) -> Result<String, String> {
    let guard = lock(session_mutex(&context)?)?;
    match guard.as_ref() {
        Some(Session::Detected(detected)) => Ok(format_page(detected, offset).1),
        _ => Err("locations:page called without a completed locations:detect".to_owned()),
    }
}

fn locations_end(context: Context) -> Result<String, String> {
    let mut guard = lock(session_mutex(&context)?)?;
    *guard = None;
    Ok("OK".to_owned())
}

/// Registered for the `Ended` mission event handler (`addons/extension/XEH_preInit.sqf`):
/// clears any in-progress session so a stale one from the previous mission can't
/// be paged or added to after a mission restart.
fn on_mission_ended(context: Context) -> Result<String, String> {
    locations_end(context)
}

#[cfg(test)]
mod tests {
    use arma_rs::IntoArma;

    use super::init;

    #[test]
    fn ok() {
        let extension = init().testing();
        let (output, code) = extension.call("ok", None);
        assert_eq!(code, 0);
        assert_eq!(output, "OK");
    }

    #[test]
    fn err() {
        let extension = init().testing();
        let (output, code) = extension.call("err", None);
        assert_eq!(code, 9);
        assert_eq!(output, "ERR");
    }

    #[test]
    fn echo() {
        let extension = init().testing();
        let args = vec!["a".to_string(), "b".to_string()];
        let (output, code) = extension.call("echo", Some(vec![args.to_arma().to_string()]));
        assert_eq!(code, 0);
        assert_eq!(output, "echo(a, b)");
    }

    #[test]
    fn locations_full_round_trip() {
        let extension = init().testing();

        let (output, code) = extension.call("locations:begin", None);
        assert_eq!(code, 0, "begin failed: {output}");

        let chunk = "0,0\n10,0\n1000,0\n1010,0".to_string();
        let (output, code) = extension.call("locations:add", Some(vec![chunk]));
        assert_eq!(code, 0, "add failed: {output}");
        assert_eq!(output, "4");

        let (output, code) = extension.call(
            "locations:detect",
            Some(vec![
                "100.0".to_string(),
                "700.0".to_string(),
                "0.7".to_string(),
                "35.0".to_string(),
                "2".to_string(),
            ]),
        );
        assert_eq!(code, 0, "detect failed: {output}");
        assert_eq!(output, "2");

        let (output, code) = extension.call("locations:page", Some(vec!["0".to_string()]));
        assert_eq!(code, 0, "page failed: {output}");
        assert!(
            output.starts_with("[2,[["),
            "unexpected page shape: {output}"
        );

        let (output, code) = extension.call("locations:end", None);
        assert_eq!(code, 0, "end failed: {output}");
    }

    #[test]
    fn page_before_detect_errors() {
        let extension = init().testing();
        let (_, code) = extension.call("locations:begin", None);
        assert_eq!(code, 0);

        let (output, code) = extension.call("locations:page", Some(vec!["0".to_string()]));
        assert_ne!(code, 0, "page before detect should fail, got: {output}");
    }

    #[test]
    fn add_after_detect_errors() {
        let extension = init().testing();
        let _ = extension.call("locations:begin", None);
        let _ = extension.call(
            "locations:detect",
            Some(vec![
                "100.0".to_string(),
                "700.0".to_string(),
                "0.7".to_string(),
                "35.0".to_string(),
                "2".to_string(),
            ]),
        );

        let (output, code) = extension.call("locations:add", Some(vec!["0,0".to_string()]));
        assert_ne!(code, 0, "add after detect should fail, got: {output}");
    }

    #[test]
    fn detect_without_begin_errors() {
        let extension = init().testing();
        let (output, code) = extension.call(
            "locations:detect",
            Some(vec![
                "100.0".to_string(),
                "700.0".to_string(),
                "0.7".to_string(),
                "35.0".to_string(),
                "2".to_string(),
            ]),
        );
        assert_ne!(code, 0, "detect without begin should fail, got: {output}");
    }

    #[test]
    fn paging_to_exhaustion_covers_every_location_exactly_once() {
        let extension = init().testing();
        let _ = extension.call("locations:begin", None);

        // 1000 well-separated 2-building farms: entries are ~25-30 bytes each,
        // so this comfortably exceeds the 9000-byte page budget and forces
        // several pages.
        let mut chunk = String::new();
        for i in 0..1000 {
            let base = i as f64 * 1000.0;
            chunk.push_str(&format!("{base},0\n{},0\n", base + 10.0));
        }
        let (output, code) = extension.call("locations:add", Some(vec![chunk]));
        assert_eq!(code, 0, "add failed: {output}");

        let (output, code) = extension.call(
            "locations:detect",
            Some(vec![
                "100.0".to_string(),
                "700.0".to_string(),
                "0.7".to_string(),
                "35.0".to_string(),
                "2".to_string(),
            ]),
        );
        assert_eq!(code, 0, "detect failed: {output}");
        let total: usize = output.parse().expect("detect must return a count");
        assert_eq!(total, 1000);

        let mut offset = 0usize;
        let mut seen = 0usize;
        let mut pages = 0usize;
        loop {
            let (output, code) = extension.call("locations:page", Some(vec![offset.to_string()]));
            assert_eq!(code, 0, "page failed: {output}");
            assert!(
                output.len() <= 9000,
                "page exceeded byte budget: {} bytes",
                output.len()
            );

            let next_offset: usize = output[1..]
                .split_once(',')
                .and_then(|(n, _)| n.parse().ok())
                .expect("page must start with [nextOffset,...]");
            let entries_on_page = next_offset - offset;
            seen += entries_on_page;
            pages += 1;
            offset = next_offset;

            if entries_on_page == 0 || offset >= total {
                break;
            }
        }

        assert_eq!(seen, total, "paging must cover every location exactly once");
        assert!(pages > 1, "1000 locations should not fit on a single page");
    }
}
