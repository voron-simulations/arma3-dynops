pub mod cluster;
pub mod geometry;
pub mod kdtree;
pub mod locations;
pub mod population;

use anyhow::Context as _;
use arma_rs::{Context, ContextState, Extension, Group, arma};
use geometry::obb_to_marker_dir;
use locations::{DetectParams, Location};
use nalgebra::Vector2;
use population::Assignment;
use std::result::Result;
use std::str::FromStr;
use std::sync::Mutex;
use uuid::Uuid;

/// A page of a `Detected` or `Planned` session's output never exceeds this many
/// bytes (except a single entry that alone is larger), well under
/// `callExtension`'s 10240-byte hard cap.
const PAGE_BYTE_BUDGET: usize = 9000;

/// State machine for the chunked `locations:*` and `population:*` protocols.
/// Held in extension-wide state (`Mutex<Option<Session>>`) so invalid call
/// sequences -- paging before detecting, adding after planning -- are rejected
/// rather than silently operating on stale data. There is one slot, so a
/// `begin` of either protocol replaces whatever session was there.
enum Session {
    /// `locations:begin` .. `locations:detect`: building positions so far.
    Collecting(Vec<Vector2<f64>>),
    /// After `locations:detect`: the result being paged out.
    Detected(Vec<Location>),
    /// `population:begin` .. `population:plan`: one weight per candidate
    /// location, one budget per faction and one `(faction, cost)` per
    /// candidate group, each in the order received.
    Planning {
        seed: u64,
        weights: Vec<f64>,
        budgets: Vec<u32>,
        groups: Vec<population::Group>,
    },
    /// After `population:plan`: the assignments being paged out.
    Planned(Vec<Assignment>),
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
                .command("end", end_session),
        )
        .group(
            "population",
            Group::new()
                .command("begin", population_begin)
                .command("addLocations", population_add_locations)
                .command("addFactions", population_add_factions)
                .command("addGroups", population_add_groups)
                .command("plan", population_plan)
                .command("page", population_page)
                .command("end", end_session),
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

/// Parses one value per line, skipping blank lines. The whole chunk is parsed
/// before the caller touches the session, so a bad line leaves it unchanged.
fn parse_lines<T>(data: &str, parse: impl Fn(&str) -> anyhow::Result<T>) -> anyhow::Result<Vec<T>> {
    data.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(parse)
        .collect()
}

fn parse_value<T>(text: &str, what: &str) -> anyhow::Result<T>
where
    T: FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    text.parse::<T>()
        .with_context(|| format!("bad {what} {text:?}"))
}

/// One `population:addGroups` line: `factionIdx,cost`.
fn parse_group(line: &str) -> anyhow::Result<population::Group> {
    let (faction, cost) = line
        .split_once(',')
        .with_context(|| format!("expected \"factionIdx,cost\", got {line:?}"))?;
    Ok(population::Group {
        faction: parse_value(faction.trim(), "group faction index")?,
        cost: parse_value(cost.trim(), "group cost")?,
    })
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
    // Validated before the session is touched: a rejected call must leave the
    // session in `Collecting` so SQF can retry with corrected parameters.
    let params = DetectParams::new(eps, max_span, split_factor, min_eps, min_buildings)
        .map_err(|e| format!("invalid locations:detect parameters: {e}"))?;
    let mut guard = lock(session_mutex(&context)?)?;
    // Borrowed, not taken: a misplaced call must not destroy a session of the
    // other protocol (or a finished one still being paged).
    let detected = match guard.as_ref() {
        Some(Session::Collecting(points)) => locations::detect(points, &params),
        _ => {
            return Err(
                "locations:detect called without an active locations:begin session".to_owned(),
            );
        }
    };
    let count = detected.len();
    *guard = Some(Session::Detected(detected));
    Ok(count.to_string())
}

fn format_location(location: &Location) -> String {
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

fn format_assignment(assignment: &Assignment) -> String {
    format!("[{},{}]", assignment.location, assignment.group)
}

/// Builds one page of `items` starting at `offset`, stopping before the
/// returned string would exceed [`PAGE_BYTE_BUDGET`] (always including at
/// least one entry, to guarantee forward progress). Returns the offset the
/// next page should start at, and the page itself as `[nextOffset, [entry, ...]]`.
fn format_page<T>(
    items: &[T],
    offset: usize,
    format_entry: impl Fn(&T) -> String,
) -> (usize, String) {
    let mut idx = offset;
    let mut body = String::new();
    while idx < items.len() {
        let entry = format_entry(&items[idx]);
        let separator = usize::from(!body.is_empty());
        // "[" + nextOffset + ",[" + body + "]]"
        let next_offset_digits = (idx + 1).ilog10() as usize + 1;
        let framed_len = 5 + next_offset_digits + body.len() + separator + entry.len();
        if framed_len > PAGE_BYTE_BUDGET && !body.is_empty() {
            break;
        }
        if separator == 1 {
            body.push(',');
        }
        body.push_str(&entry);
        idx += 1;
    }
    (idx, format!("[{idx},[{body}]]"))
}

fn locations_page(context: Context, offset: usize) -> Result<String, String> {
    let guard = lock(session_mutex(&context)?)?;
    match guard.as_ref() {
        Some(Session::Detected(detected)) => Ok(format_page(detected, offset, format_location).1),
        _ => Err("locations:page called without a completed locations:detect".to_owned()),
    }
}

fn population_begin(context: Context, seed: u64) -> Result<String, String> {
    let mut guard = lock(session_mutex(&context)?)?;
    *guard = Some(Session::Planning {
        seed,
        weights: Vec::new(),
        budgets: Vec::new(),
        groups: Vec::new(),
    });
    Ok("OK".to_owned())
}

fn population_add_locations(context: Context, chunk: String) -> Result<String, String> {
    let weights = parse_lines(&chunk, |line| parse_value::<f64>(line, "location weight"))
        .map_err(|e| format!("{e:#}"))?;
    let mut guard = lock(session_mutex(&context)?)?;
    match guard.as_mut() {
        Some(Session::Planning {
            weights: accumulated,
            ..
        }) => {
            accumulated.extend(weights);
            Ok(accumulated.len().to_string())
        }
        _ => Err(
            "population:addLocations called without an active population:begin session".to_owned(),
        ),
    }
}

fn population_add_factions(context: Context, chunk: String) -> Result<String, String> {
    let budgets = parse_lines(&chunk, |line| parse_value::<u32>(line, "faction budget"))
        .map_err(|e| format!("{e:#}"))?;
    let mut guard = lock(session_mutex(&context)?)?;
    match guard.as_mut() {
        Some(Session::Planning {
            budgets: accumulated,
            ..
        }) => {
            accumulated.extend(budgets);
            Ok(accumulated.len().to_string())
        }
        _ => Err(
            "population:addFactions called without an active population:begin session".to_owned(),
        ),
    }
}

fn population_add_groups(context: Context, chunk: String) -> Result<String, String> {
    let groups = parse_lines(&chunk, parse_group).map_err(|e| format!("{e:#}"))?;
    let mut guard = lock(session_mutex(&context)?)?;
    match guard.as_mut() {
        Some(Session::Planning {
            groups: accumulated,
            ..
        }) => {
            accumulated.extend(groups);
            Ok(accumulated.len().to_string())
        }
        _ => {
            Err("population:addGroups called without an active population:begin session".to_owned())
        }
    }
}

fn population_plan(context: Context) -> Result<String, String> {
    let mut guard = lock(session_mutex(&context)?)?;
    // Borrowed, not taken: invalid input must leave the session in `Planning`
    // so SQF can add or begin again instead of losing what it already sent.
    let assignments = match guard.as_ref() {
        Some(Session::Planning {
            seed,
            weights,
            budgets,
            groups,
        }) => population::plan(weights, budgets, groups, *seed)
            .map_err(|e| format!("invalid population:plan input: {e}"))?,
        _ => {
            return Err(
                "population:plan called without an active population:begin session".to_owned(),
            );
        }
    };
    let count = assignments.len();
    *guard = Some(Session::Planned(assignments));
    Ok(count.to_string())
}

fn population_page(context: Context, offset: usize) -> Result<String, String> {
    let guard = lock(session_mutex(&context)?)?;
    match guard.as_ref() {
        Some(Session::Planned(assignments)) => {
            Ok(format_page(assignments, offset, format_assignment).1)
        }
        _ => Err("population:page called without a completed population:plan".to_owned()),
    }
}

/// Backs both `locations:end` and `population:end`: there is one session slot,
/// so ending either protocol clears it whatever it held.
fn end_session(context: Context) -> Result<String, String> {
    let mut guard = lock(session_mutex(&context)?)?;
    *guard = None;
    Ok("OK".to_owned())
}

/// Registered for the `Ended` mission event handler (`addons/extension/XEH_preInit.sqf`):
/// clears any in-progress session (locations or population) so a stale one from
/// the previous mission can't be paged or added to after a mission restart.
fn on_mission_ended(context: Context) -> Result<String, String> {
    end_session(context)
}

#[cfg(test)]
mod tests {
    use arma_rs::IntoArma;

    use super::{PAGE_BYTE_BUDGET, format_page, init};
    use crate::population::Group;

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
        let (output, code) =
            extension.call("locations:add", Some(vec![chunk.to_arma().to_string()]));
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

        let (output, code) =
            extension.call("locations:add", Some(vec!["0,0".to_arma().to_string()]));
        assert_ne!(code, 0, "add after detect should fail, got: {output}");
        // Pins the failure to the session-state check; an unquoted or malformed
        // chunk would also return non-zero, just for the wrong reason.
        assert!(
            output.contains("without an active locations:begin"),
            "add after detect failed for the wrong reason: {output}"
        );
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
    fn detect_rejects_invalid_params() {
        let extension = init().testing();
        let (_, code) = extension.call("locations:begin", None);
        assert_eq!(code, 0);
        let (output, code) = extension.call(
            "locations:add",
            Some(vec!["0,0\n10,0".to_string().to_arma().to_string()]),
        );
        assert_eq!(code, 0, "add failed: {output}");
        assert_eq!(output, "2");

        // split_factor 1 never shrinks eps; unvalidated it recursed until the
        // stack overflowed inside Arma.
        let (output, code) = extension.call(
            "locations:detect",
            Some(vec![
                "100.0".to_string(),
                "700.0".to_string(),
                "1".to_string(),
                "35.0".to_string(),
                "2".to_string(),
            ]),
        );
        assert_ne!(code, 0, "invalid params should fail, got: {output}");
        assert!(
            output.contains("split_factor"),
            "error should name the offending parameter: {output}"
        );

        // Still `Collecting`: adding works and the count continues from before.
        let (output, code) =
            extension.call("locations:add", Some(vec!["20,0".to_arma().to_string()]));
        assert_eq!(code, 0, "add after rejected detect failed: {output}");
        assert_eq!(output, "3");

        // And a corrected detect succeeds over the points collected so far.
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
        assert_eq!(code, 0, "valid detect failed: {output}");
        assert_eq!(output, "1");
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
        let (output, code) =
            extension.call("locations:add", Some(vec![chunk.to_arma().to_string()]));
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

    // --- population:* ---

    type Ext = arma_rs::testing::Extension;

    fn call(ext: &Ext, function: &str, args: Vec<String>) -> (String, i32) {
        let args = if args.is_empty() { None } else { Some(args) };
        ext.call(function, args)
    }

    fn call_ok(ext: &Ext, function: &str, args: Vec<String>) -> String {
        let (output, code) = call(ext, function, args);
        assert_eq!(code, 0, "{function} failed: {output}");
        output
    }

    fn call_err(ext: &Ext, function: &str, args: Vec<String>) -> String {
        let (output, code) = call(ext, function, args);
        assert_ne!(code, 0, "{function} should have failed, got: {output}");
        output
    }

    /// A string argument, quoted the way SQF would send it.
    fn text(s: &str) -> Vec<String> {
        vec![s.to_owned().to_arma().to_string()]
    }

    fn num(n: impl ToString) -> Vec<String> {
        vec![n.to_string()]
    }

    /// Parses `[nextOffset,[[loc,grp],...]]` without depending on a JSON crate.
    fn parse_page(output: &str) -> (usize, Vec<(usize, usize)>) {
        let inner = output
            .strip_prefix('[')
            .and_then(|s| s.strip_suffix(']'))
            .expect("page must be a bracketed array");
        let (next, entries) = inner
            .split_once(',')
            .expect("page must start with [nextOffset,...]");
        let digits: String = entries
            .chars()
            .map(|c| if c.is_ascii_digit() { c } else { ' ' })
            .collect();
        let numbers: Vec<usize> = digits
            .split_whitespace()
            .map(|n| n.parse().expect("entries are unsigned integers"))
            .collect();
        assert_eq!(numbers.len() % 2, 0, "entries come in pairs: {output}");
        (
            next.parse().expect("nextOffset is an unsigned integer"),
            numbers.chunks(2).map(|p| (p[0], p[1])).collect(),
        )
    }

    fn page_to_exhaustion(ext: &Ext, total: usize) -> (Vec<(usize, usize)>, usize) {
        let (mut offset, mut pages, mut all) = (0usize, 0usize, Vec::new());
        loop {
            let output = call_ok(ext, "population:page", num(offset));
            assert!(
                output.len() <= PAGE_BYTE_BUDGET,
                "page exceeded byte budget: {} bytes",
                output.len()
            );
            let (next, entries) = parse_page(&output);
            assert_eq!(
                next - offset,
                entries.len(),
                "offset must advance per entry"
            );
            all.extend(entries);
            pages += 1;
            if next == offset || next >= total {
                return (all, pages);
            }
            offset = next;
        }
    }

    fn add_in_chunks(ext: &Ext, function: &str, lines: &[String]) {
        let mut sent = 0;
        for chunk in lines.chunks(400) {
            sent += chunk.len();
            let output = call_ok(ext, function, text(&chunk.join("\n")));
            assert_eq!(output, sent.to_string());
        }
    }

    /// Sends the three inputs of one plan: weights, faction budgets, groups.
    fn load(ext: &Ext, weights: &str, budgets: &str, groups: &str) {
        call_ok(ext, "population:addLocations", text(weights));
        call_ok(ext, "population:addFactions", text(budgets));
        call_ok(ext, "population:addGroups", text(groups));
    }

    #[test]
    fn population_full_round_trip() {
        let extension = init().testing();

        assert_eq!(call_ok(&extension, "population:begin", num(42)), "OK");
        // Blank lines are ignored and the count is the running total.
        assert_eq!(
            call_ok(&extension, "population:addLocations", text("1\n\n3\n")),
            "2"
        );
        assert_eq!(
            call_ok(&extension, "population:addLocations", text("0")),
            "3"
        );
        assert_eq!(
            call_ok(&extension, "population:addFactions", text("8")),
            "1"
        );
        assert_eq!(
            call_ok(&extension, "population:addGroups", text("0,1")),
            "1"
        );

        // Weights 1, 3, 0 over cost-1 groups: 2 and 6 groups, none at the
        // zero-weight location.
        assert_eq!(call_ok(&extension, "population:plan", vec![]), "8");
        assert_eq!(
            call_ok(&extension, "population:page", num(0)),
            "[8,[[0,0],[0,0],[1,0],[1,0],[1,0],[1,0],[1,0],[1,0]]]"
        );
        // Paging past the end is a valid, empty page.
        assert_eq!(call_ok(&extension, "population:page", num(8)), "[8,[]]");

        assert_eq!(call_ok(&extension, "population:end", vec![]), "OK");
        call_err(&extension, "population:page", num(0));
    }

    #[test]
    fn population_factions_share_one_plan_in_proportion_to_their_budgets() {
        let extension = init().testing();
        call_ok(&extension, "population:begin", num(5));
        assert_eq!(
            call_ok(&extension, "population:addFactions", text("6\n2")),
            "2"
        );
        assert_eq!(
            call_ok(&extension, "population:addLocations", text("1\n3\n0")),
            "3"
        );
        assert_eq!(
            call_ok(&extension, "population:addGroups", text("0,1\n1,1")),
            "2"
        );

        assert_eq!(call_ok(&extension, "population:plan", vec![]), "8");
        let (entries, _) = page_to_exhaustion(&extension, 8);
        let faction_zero = entries.iter().filter(|e| e.1 == 0).count();
        let faction_one = entries.iter().filter(|e| e.1 == 1).count();
        assert_eq!((faction_zero, faction_one), (6, 2));
        let at = |loc: usize| entries.iter().filter(|e| e.0 == loc).count();
        assert_eq!((at(0), at(1), at(2)), (2, 6, 0));
    }

    #[test]
    fn population_agrees_with_the_pure_planner() {
        let extension = init().testing();
        call_ok(&extension, "population:begin", num(1234));
        // Whitespace around the comma is tolerated.
        load(
            &extension,
            "1\n2\n4\n8\n16",
            "100\n60\n40",
            "0,3\n0,5\n1, 8\n2 ,13\n1,2",
        );
        let count: usize = call_ok(&extension, "population:plan", vec![])
            .parse()
            .expect("plan returns a count");

        let groups = [
            Group {
                faction: 0,
                cost: 3,
            },
            Group {
                faction: 0,
                cost: 5,
            },
            Group {
                faction: 1,
                cost: 8,
            },
            Group {
                faction: 2,
                cost: 13,
            },
            Group {
                faction: 1,
                cost: 2,
            },
        ];
        let expected =
            crate::population::plan(&[1.0, 2.0, 4.0, 8.0, 16.0], &[100, 60, 40], &groups, 1234)
                .expect("valid input");
        assert_eq!(count, expected.len());

        let (entries, _) = page_to_exhaustion(&extension, count);
        let expected: Vec<_> = expected.iter().map(|a| (a.location, a.group)).collect();
        assert_eq!(entries, expected);
    }

    #[test]
    fn population_paging_covers_every_assignment_exactly_once() {
        let extension = init().testing();
        call_ok(&extension, "population:begin", num(7));

        // 5000 entries of 7-11 bytes each is far over the 9000-byte page
        // budget, so this forces many pages.
        let weights: Vec<String> = (0..1100).map(|_| "1".to_owned()).collect();
        add_in_chunks(&extension, "population:addLocations", &weights);
        call_ok(&extension, "population:addFactions", text("3000\n2000"));
        call_ok(&extension, "population:addGroups", text("0,1\n1,1"));
        let count: usize = call_ok(&extension, "population:plan", vec![])
            .parse()
            .expect("plan returns a count");
        assert_eq!(count, 5000);

        let (entries, pages) = page_to_exhaustion(&extension, count);
        assert!(pages > 1, "5000 assignments should not fit on one page");

        let groups = [
            Group {
                faction: 0,
                cost: 1,
            },
            Group {
                faction: 1,
                cost: 1,
            },
        ];
        let expected =
            crate::population::plan(&vec![1.0; 1100], &[3000, 2000], &groups, 7).expect("valid");
        let expected: Vec<_> = expected.iter().map(|a| (a.location, a.group)).collect();
        assert_eq!(entries, expected);
    }

    #[test]
    fn population_calls_out_of_order_error() {
        let extension = init().testing();
        let misuse = |extension: &Ext, state: &str| {
            for (function, args) in [
                ("population:addLocations", text("1")),
                ("population:addFactions", text("1")),
                ("population:addGroups", text("0,1")),
                ("population:plan", vec![]),
                ("population:page", num(0)),
            ] {
                let output = call_err(extension, function, args);
                assert!(
                    output.contains(function),
                    "{state}: error should name {function}: {output}"
                );
            }
        };

        // No session at all.
        misuse(&extension, "empty");

        // Planning: only the page call is out of order.
        call_ok(&extension, "population:begin", num(1));
        let output = call_err(&extension, "population:page", num(0));
        assert!(output.contains("population:plan"), "{output}");

        // Planned: nothing but page and end are allowed.
        load(&extension, "1", "1", "0,1");
        call_ok(&extension, "population:plan", vec![]);
        for (function, args) in [
            ("population:addLocations", text("1")),
            ("population:addFactions", text("1")),
            ("population:addGroups", text("0,1")),
            ("population:plan", vec![]),
        ] {
            let output = call_err(&extension, function, args);
            assert!(output.contains("population:begin"), "{output}");
        }
        assert_eq!(
            call_ok(&extension, "population:page", num(0)),
            "[1,[[0,0]]]"
        );

        // After end.
        call_ok(&extension, "population:end", vec![]);
        misuse(&extension, "ended");

        // A locations session is not a population session.
        call_ok(&extension, "locations:begin", vec![]);
        misuse(&extension, "locations:Collecting");
    }

    #[test]
    fn population_end_is_valid_in_any_state() {
        let extension = init().testing();
        assert_eq!(call_ok(&extension, "population:end", vec![]), "OK");
        call_ok(&extension, "population:begin", num(1));
        assert_eq!(call_ok(&extension, "population:end", vec![]), "OK");
        assert_eq!(call_ok(&extension, "population:end", vec![]), "OK");
    }

    #[test]
    fn population_plan_with_zero_cost_errors_and_stays_planning() {
        let extension = init().testing();
        call_ok(&extension, "population:begin", num(1));
        call_ok(&extension, "population:addLocations", text("1"));
        call_ok(&extension, "population:addFactions", text("10"));
        assert_eq!(
            call_ok(&extension, "population:addGroups", text("0,5\n0,0")),
            "2"
        );

        let output = call_err(&extension, "population:plan", vec![]);
        assert!(
            output.contains("cost"),
            "error should name the cost: {output}"
        );
        assert!(
            output.contains("index 1"),
            "error should name the index: {output}"
        );

        // Still `Planning`: more groups can be added. The zero-cost entry is
        // still in the catalogue, so a retry fails again until `begin`.
        assert_eq!(
            call_ok(&extension, "population:addGroups", text("0,2")),
            "3"
        );
        call_err(&extension, "population:plan", vec![]);
        call_err(&extension, "population:page", num(0));

        call_ok(&extension, "population:begin", num(1));
        load(&extension, "1", "10", "0,5");
        assert_eq!(call_ok(&extension, "population:plan", vec![]), "2");
    }

    #[test]
    fn population_plan_with_unknown_faction_errors_and_stays_planning() {
        let extension = init().testing();
        call_ok(&extension, "population:begin", num(1));
        load(&extension, "1", "10", "0,5\n1,5");

        let output = call_err(&extension, "population:plan", vec![]);
        assert!(
            output.contains("faction 1"),
            "error should name the faction: {output}"
        );
        assert!(
            output.contains("index 1"),
            "error should name the group: {output}"
        );
        call_err(&extension, "population:page", num(0));

        // Still `Planning`, and supplying the missing budget fixes it: two
        // factions with 10 points each buy two cost-5 groups apiece.
        assert_eq!(
            call_ok(&extension, "population:addFactions", text("10")),
            "2"
        );
        assert_eq!(call_ok(&extension, "population:plan", vec![]), "4");
    }

    #[test]
    fn population_plan_over_the_assignment_cap_errors_and_stays_planning() {
        let extension = init().testing();
        call_ok(&extension, "population:begin", num(1));
        load(&extension, "1", "100001", "0,1");

        let output = call_err(&extension, "population:plan", vec![]);
        assert!(
            output.contains("100000"),
            "error should state the limit: {output}"
        );
        call_err(&extension, "population:page", num(0));

        // Still `Planning`.
        assert_eq!(
            call_ok(&extension, "population:addLocations", text("2")),
            "2"
        );

        // One point less is within the cap and plans in full.
        call_ok(&extension, "population:begin", num(1));
        load(&extension, "1", "100000", "0,1");
        assert_eq!(call_ok(&extension, "population:plan", vec![]), "100000");
        let (entries, pages) = page_to_exhaustion(&extension, 100_000);
        assert_eq!(entries.len(), 100_000);
        assert!(pages > 50, "{pages}");
    }

    #[test]
    fn population_plan_with_invalid_weight_errors_and_stays_planning() {
        let extension = init().testing();
        call_ok(&extension, "population:begin", num(1));
        load(&extension, "1\n-2", "10", "0,1");

        let output = call_err(&extension, "population:plan", vec![]);
        assert!(
            output.contains("weight"),
            "error should name the weight: {output}"
        );
        assert!(
            output.contains("index 1"),
            "error should name the index: {output}"
        );

        // Still `Planning`.
        assert_eq!(
            call_ok(&extension, "population:addLocations", text("3")),
            "3"
        );
    }

    #[test]
    fn population_rejects_unparseable_chunks_without_partial_effect() {
        let extension = init().testing();
        call_ok(&extension, "population:begin", num(1));
        load(&extension, "1", "1", "0,1");

        let output = call_err(&extension, "population:addLocations", text("2\nbanana"));
        assert!(output.contains("banana"), "{output}");
        let output = call_err(&extension, "population:addFactions", text("2\n-3"));
        assert!(output.contains("-3"), "{output}");
        let output = call_err(&extension, "population:addFactions", text("2\n1.5"));
        assert!(output.contains("1.5"), "{output}");

        // Group lines are `factionIdx,cost` and nothing else.
        for (bad, needle) in [
            ("0,1\nbanana", "banana"),
            ("0,1\n1", "factionIdx,cost"),
            ("0,1\n,5", "faction index"),
            ("0,1\n-1,5", "-1"),
            ("0,1\n0,", "group cost"),
            ("0,1\n0,-2", "-2"),
            ("0,1\n0,1.5", "1.5"),
            ("0,1\n0,1,2", "1,2"),
        ] {
            let output = call_err(&extension, "population:addGroups", text(bad));
            assert!(output.contains(needle), "{bad:?} -> {output}");
        }

        // None of the failed chunks added their valid first line.
        assert_eq!(
            call_ok(&extension, "population:addLocations", text("2")),
            "2"
        );
        assert_eq!(
            call_ok(&extension, "population:addFactions", text("2")),
            "2"
        );
        assert_eq!(
            call_ok(&extension, "population:addGroups", text("0,2")),
            "2"
        );
    }

    #[test]
    fn population_begin_replaces_a_locations_session() {
        let extension = init().testing();
        call_ok(&extension, "locations:begin", vec![]);
        call_ok(&extension, "locations:add", text("0,0"));

        call_ok(&extension, "population:begin", num(1));
        let output = call_err(&extension, "locations:add", text("0,0"));
        assert!(output.contains("locations:begin"), "{output}");
        assert_eq!(
            call_ok(&extension, "population:addLocations", text("1")),
            "1"
        );
    }

    #[test]
    fn locations_begin_replaces_a_population_session() {
        let extension = init().testing();
        call_ok(&extension, "population:begin", num(1));
        call_ok(&extension, "population:addLocations", text("1"));

        call_ok(&extension, "locations:begin", vec![]);
        let output = call_err(&extension, "population:addLocations", text("1"));
        assert!(output.contains("population:begin"), "{output}");
        assert_eq!(call_ok(&extension, "locations:add", text("0,0")), "1");
    }

    #[test]
    fn population_begin_discards_the_previous_population() {
        let extension = init().testing();
        call_ok(&extension, "population:begin", num(1));
        load(&extension, "1\n1\n1", "5\n5", "0,1\n1,1");
        call_ok(&extension, "population:begin", num(2));
        assert_eq!(
            call_ok(&extension, "population:addLocations", text("1")),
            "1"
        );
        assert_eq!(
            call_ok(&extension, "population:addFactions", text("1")),
            "1"
        );
        assert_eq!(
            call_ok(&extension, "population:addGroups", text("0,1")),
            "1"
        );
    }

    #[test]
    fn misused_locations_calls_leave_a_population_session_intact() {
        let extension = init().testing();
        call_ok(&extension, "population:begin", num(1));
        load(&extension, "1", "3", "0,1");

        // A stray locations call must not destroy the session it is not for.
        call_err(&extension, "locations:add", text("0,0"));
        call_err(
            &extension,
            "locations:detect",
            ["100.0", "700.0", "0.7", "35.0", "2"]
                .map(str::to_owned)
                .to_vec(),
        );
        call_err(&extension, "locations:page", num(0));

        assert_eq!(call_ok(&extension, "population:plan", vec![]), "3");

        // Same for a Planned session.
        call_err(&extension, "locations:add", text("0,0"));
        call_err(&extension, "locations:page", num(0));
        assert_eq!(
            call_ok(&extension, "population:page", num(0)),
            "[3,[[0,0],[0,0],[0,0]]]"
        );
    }

    #[test]
    fn mission_end_clears_population_sessions() {
        let extension = init().testing();

        call_ok(&extension, "population:begin", num(1));
        call_ok(&extension, "population:addLocations", text("1"));
        assert_eq!(call_ok(&extension, "onMissionEnded", vec![]), "OK");
        call_err(&extension, "population:addLocations", text("1"));

        call_ok(&extension, "population:begin", num(1));
        load(&extension, "1", "1", "0,1");
        call_ok(&extension, "population:plan", vec![]);
        assert_eq!(call_ok(&extension, "onMissionEnded", vec![]), "OK");
        call_err(&extension, "population:page", num(0));
    }

    #[test]
    fn format_page_always_makes_progress() {
        // Each entry alone is bigger than the whole budget.
        let items = vec!["x".repeat(PAGE_BYTE_BUDGET + 1); 3];
        let (next, page) = format_page(&items, 0, |s| s.clone());
        assert_eq!(next, 1);
        assert!(page.starts_with("[1,["), "{}", &page[..10]);

        let (next, _) = format_page(&items, 1, |s| s.clone());
        assert_eq!(next, 2);
    }

    #[test]
    fn format_page_past_the_end_is_empty() {
        let items = vec!["a".to_owned(); 2];
        assert_eq!(
            format_page(&items, 2, |s| s.clone()),
            (2, "[2,[]]".to_owned())
        );
        assert_eq!(
            format_page(&items, 9, |s| s.clone()),
            (9, "[9,[]]".to_owned())
        );
        assert_eq!(
            format_page::<String>(&[], 0, |s| s.clone()),
            (0, "[0,[]]".to_owned())
        );
    }

    #[test]
    fn format_page_never_exceeds_the_budget_when_entries_fit() {
        let items: Vec<String> = (0..2000).map(|i| format!("[{i},{}]", i * 3)).collect();
        let mut offset = 0;
        while offset < items.len() {
            let (next, page) = format_page(&items, offset, |s| s.clone());
            assert!(next > offset);
            assert!(page.len() <= PAGE_BYTE_BUDGET, "{} bytes", page.len());
            offset = next;
        }
    }
}
