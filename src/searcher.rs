use std::sync::mpsc;

use grep::{
    matcher::LineTerminator,
    regex::RegexMatcher,
    searcher::{Searcher as GrepSearcher, SearcherBuilder as GrepSearcherBuilder, Sink, SinkMatch},
};
use ignore::WalkBuilder;

//use crate::entries::{FileEntry, Match};
//use crate::ig::Event;

pub struct Searcher {
    tx: mpsc::Sender<String>,
}

impl Searcher {
    pub fn new(tx: mpsc::Sender<String>) -> Self {
        Self { tx }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let matcher = RegexMatcher::new_line_matcher("kernel")?; // handle this unwrap with if let here
        let builder = WalkBuilder::new("/");

        let walk_parallel = builder.build_parallel();
        walk_parallel.run(move || {
            let tx = self.tx.clone();
            let matcher = matcher.clone();
            let mut grep_searcher = GrepSearcherBuilder::new()
                //.binary_detection(BinaryDetection::quit(b'\x00')) // from simplegrep - check it
                .line_terminator(LineTerminator::byte(b'\n'))
                .line_number(true)
                .multi_line(false)
                .build();

            Box::new(move |result| {
                let dir_entry = match result {
                    Err(err) => {
                        //eprintln!("{}", err);
                        return ignore::WalkState::Continue;
                    }
                    Ok(entry) => {
                        if !entry.file_type().map_or(false, |ft| ft.is_file()) {
                            return ignore::WalkState::Continue;
                        }
                        entry
                    }
                };

                let mut matches_in_entry = Vec::new();

                // handle error like in simplegrep
                let _ = grep_searcher.search_path(
                    &matcher,
                    dir_entry.path(),
                    MatchesSink(|_, sink_match| {
                        let line_number = sink_match.line_number().unwrap();
                        let text = std::str::from_utf8(sink_match.bytes()).unwrap_or("Not UTF-8");
                        //let m = Match::new(line_number, text);
                        //matches_in_entry.push(m);
                        matches_in_entry.push(String::from("asd"));
                        Ok(true)
                    }),
                );

                if !matches_in_entry.is_empty() {
                    /*tx.send(Event::NewEntry(FileEntry {
                        name: String::from(dir_entry.path().to_str().unwrap()),
                        matches: matches_in_entry,
                    }))
                    .unwrap();*/
                    tx.send(String::from("asd")).unwrap();
                }

                ignore::WalkState::Continue
            })
        });

        Ok(())
    }
}

struct MatchesSink<F>(pub F)
where
    F: FnMut(&GrepSearcher, &SinkMatch) -> Result<bool, std::io::Error>;

impl<F> Sink for MatchesSink<F>
where
    F: FnMut(&GrepSearcher, &SinkMatch) -> Result<bool, std::io::Error>,
{
    type Error = std::io::Error;

    fn matched(
        &mut self,
        searcher: &GrepSearcher,
        sink_match: &SinkMatch,
    ) -> Result<bool, std::io::Error> {
        (self.0)(searcher, sink_match)
    }
}
