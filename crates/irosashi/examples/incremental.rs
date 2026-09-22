//! Tokenizes a buffer line by line with the `Session` API, then edits one line and
//! re-tokenizes only the lines whose state changed, as an editor would.
//! `cargo run -p irosashi --example incremental`

use irosashi::{Highlighter, Session, StateStack, TokenizeOptions, split_lines};

const CODE: &str = r#"fn main() {
    let items = vec![1, 2, 3];
    /* a comment that
       spans lines */
    for item in &items {
        println!("{item}");
    }
}
"#;

/// Tokenizes every line from `first` on, stopping early when a line's outgoing state
/// equals the one recorded before the edit. Returns how many lines were tokenized.
fn retokenize(
    session: &mut Session,
    lines: &[&str],
    states: &mut Vec<StateStack>,
    first: usize,
) -> usize {
    let mut state = if first == 0 {
        session.initial_state()
    } else {
        states[first - 1].clone()
    };
    let mut count = 0;
    for (index, line) in lines.iter().enumerate().skip(first) {
        let result = session.tokenize_line(line, &state, index == 0, TokenizeOptions::default());
        count += 1;
        let unchanged = states.get(index) == Some(&result.state);
        state = result.state;
        if index < states.len() {
            states[index] = state.clone();
        } else {
            states.push(state.clone());
        }
        if unchanged && index >= first {
            break;
        }
    }
    count
}

fn main() -> Result<(), irosashi::Error> {
    let highlighter = Highlighter::new()?;
    let mut session = highlighter.session("rust")?;

    let mut source = CODE.to_owned();
    let ranges = split_lines(&source);
    let lines: Vec<&str> = ranges.iter().map(|r| &source[r.clone()]).collect();
    let mut states = Vec::new();
    let total = retokenize(&mut session, &lines, &mut states, 0);
    println!("initial pass: {total} of {} lines", lines.len());

    // Edit line 2 (index 1). Its own state is unchanged afterwards, so the pass stops
    // right after it.
    source = source.replacen("vec![1, 2, 3]", "vec![1, 2, 3, 4]", 1);
    let ranges = split_lines(&source);
    let lines: Vec<&str> = ranges.iter().map(|r| &source[r.clone()]).collect();
    let done = retokenize(&mut session, &lines, &mut states, 1);
    println!(
        "after editing line 2: re-tokenized {done} of {} lines",
        lines.len()
    );

    // Open a block comment on line 3 (index 2): every later line changes state, so the
    // pass runs to the end.
    source = source.replacen("/* a comment that", "/* a comment that /*", 1);
    let ranges = split_lines(&source);
    let lines: Vec<&str> = ranges.iter().map(|r| &source[r.clone()]).collect();
    let done = retokenize(&mut session, &lines, &mut states, 2);
    println!(
        "after editing line 3: re-tokenized {done} of {} lines",
        lines.len()
    );

    let stats = session.stats();
    println!(
        "session: {} lines tokenized, {} pattern searches, {} cache hits",
        stats.lines, stats.pattern_searches, stats.pattern_cache_hits
    );
    Ok(())
}
