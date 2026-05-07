use rustyline::{
    completion::{Completer, Pair},
    highlight::{CmdKind, Highlighter, MatchingBracketHighlighter},
    hint::HistoryHinter,
    validate::MatchingBracketValidator,
    Cmd, CompletionType, Config, EditMode, Editor, Helper, Hinter, KeyEvent, Validator,
};
use std::borrow::Cow;
use std::io::Result;
use trie_rs::{Trie, TrieBuilder};

use crate::{
    builtins::{list_path, BUILTINS},
    history::MyHistory,
};

pub type MyShell = Editor<MyHelper, MyHistory>;

#[derive(Helper, Hinter, Validator)]
pub struct MyHelper {
    trie: Trie<u8>,
    highlighter: MatchingBracketHighlighter,
    #[rustyline(Validator)]
    validator: MatchingBracketValidator,
    #[rustyline(Hinter)]
    hinter: HistoryHinter,
}

impl Highlighter for MyHelper {
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Owned("\x1b[1m".to_owned() + hint + "\x1b[m")
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize, kind: CmdKind) -> bool {
        self.highlighter.highlight_char(line, pos, kind)
    }
}

impl Completer for MyHelper {
    type Candidate = Pair;

    fn complete(
        &self, // FIXME should be `&mut self`
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let prefix = &line[..pos];
        let targets = self
            .trie
            .predictive_search(prefix)
            .map(|s: String| Pair {
                display: s.clone(),
                replacement: s + " ",
            })
            .collect();
        Ok((0, targets))
    }
}

fn gather_executables() -> Result<Trie<u8>> {
    // Trie for completion search
    let mut builder = TrieBuilder::new();
    // Gather builtins command name
    for builtin in BUILTINS {
        builder.push(builtin.name);
    }
    // Gather names of executables on path
    for path in list_path()? {
        let name = path.file_name().unwrap().to_str().unwrap();
        builder.push(name);
    }
    Ok(builder.build())
}

fn create_helper() -> Result<MyHelper> {
    let trie = gather_executables()?;
    Ok(MyHelper {
        trie,
        highlighter: MatchingBracketHighlighter::new(),
        validator: MatchingBracketValidator::new(),
        hinter: HistoryHinter {},
    })
}

pub fn create_readline() -> rustyline::Result<MyShell> {
    let config = Config::builder()
        .auto_add_history(true)
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Vi)
        .bell_style(rustyline::config::BellStyle::Audible)
        .build();
    let h = create_helper()?;

    let history = MyHistory::new();

    let mut rl = Editor::with_history(config, history)?;
    rl.set_helper(Some(h));
    rl.bind_sequence(KeyEvent::alt('n'), Cmd::HistorySearchForward);
    rl.bind_sequence(KeyEvent::alt('p'), Cmd::HistorySearchBackward);
    Ok(rl)
}
