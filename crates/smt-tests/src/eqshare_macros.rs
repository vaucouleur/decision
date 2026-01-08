#![forbid(unsafe_code)]

use smt_core::TermId;
use smt_engine::atoms::TheoryId;
use smt_engine::eqshare_trace::EqShareEvent;

fn has_pair(events: &[EqShareEvent], src: TheoryId, dst: TheoryId, a: TermId, b: TermId) -> bool {
    events.iter().any(|e| e.src == src && e.dst == dst && ((e.a == a && e.b == b) || (e.a == b && e.b == a)))
}

fn has_dir(events: &[EqShareEvent], src: TheoryId, dst: TheoryId) -> bool {
    events.iter().any(|e| e.src == src && e.dst == dst)
}

fn dump(events: &[EqShareEvent]) -> String {
    events.iter().take(40)
        .map(|e| format!("epoch={} {:?}->{:?} a={:?} b={:?} reason={:?}", e.epoch, e.src, e.dst, e.a, e.b, e.explain))
        .collect::<Vec<_>>()
        .join("\n")
}

pub const UF: TheoryId = TheoryId(0);
pub const DL: TheoryId = TheoryId(1);

#[macro_export]
macro_rules! assert_eqshare_events_empty {
    ($events:expr $(,)?) => {{
        let events = $events;
        if !events.is_empty() {
            panic!(
                "expected no eqshare events, but got {}.\nfirst events:\n{}",
                events.len(),
                $crate::eqshare_macros::dump(events)
            );
        }
    }};
}

#[macro_export]
macro_rules! assert_eqshare_dir {
    ($events:expr, UF => DL $(,)?) => {{
        let events = $events;
        if !$crate::eqshare_macros::has_dir(events, $crate::eqshare_macros::UF, $crate::eqshare_macros::DL) {
            panic!("missing eqshare direction UF=>DL\nfirst events:\n{}", $crate::eqshare_macros::dump(events));
        }
    }};
    ($events:expr, DL => UF $(,)?) => {{
        let events = $events;
        if !$crate::eqshare_macros::has_dir(events, $crate::eqshare_macros::DL, $crate::eqshare_macros::UF) {
            panic!("missing eqshare direction DL=>UF\nfirst events:\n{}", $crate::eqshare_macros::dump(events));
        }
    }};
}

#[macro_export]
macro_rules! assert_eqshare_dir_none {
    ($events:expr, UF => DL $(,)?) => {{
        let events = $events;
        if $crate::eqshare_macros::has_dir(events, $crate::eqshare_macros::UF, $crate::eqshare_macros::DL) {
            panic!("unexpected eqshare direction UF=>DL\nfirst events:\n{}", $crate::eqshare_macros::dump(events));
        }
    }};
    ($events:expr, DL => UF $(,)?) => {{
        let events = $events;
        if $crate::eqshare_macros::has_dir(events, $crate::eqshare_macros::DL, $crate::eqshare_macros::UF) {
            panic!("unexpected eqshare direction DL=>UF\nfirst events:\n{}", $crate::eqshare_macros::dump(events));
        }
    }};
}

#[macro_export]
macro_rules! assert_eqshare_hop {
    ($events:expr, UF => DL, $a:expr, $b:expr $(,)?) => {{
        let events = $events;
        let a = $a;
        let b = $b;
        if !$crate::eqshare_macros::has_pair(events, $crate::eqshare_macros::UF, $crate::eqshare_macros::DL, a, b) {
            panic!(
                "missing eqshare hop UF=>DL for pair {:?} = {:?}\nfirst events:\n{}",
                a, b, $crate::eqshare_macros::dump(events)
            );
        }
    }};
    ($events:expr, DL => UF, $a:expr, $b:expr $(,)?) => {{
        let events = $events;
        let a = $a;
        let b = $b;
        if !$crate::eqshare_macros::has_pair(events, $crate::eqshare_macros::DL, $crate::eqshare_macros::UF, a, b) {
            panic!(
                "missing eqshare hop DL=>UF for pair {:?} = {:?}\nfirst events:\n{}",
                a, b, $crate::eqshare_macros::dump(events)
            );
        }
    }};
}

#[macro_export]
macro_rules! assert_eqshare_hop_any {
    ($events:expr, UF => DL, [ $(($a:expr, $b:expr)),+ $(,)? ] $(,)?) => {{
        let candidates = [ $(($a, $b)),+ ];
        $crate::assert_eqshare_hop_any!($events, UF => DL, &candidates);
    }};
    ($events:expr, DL => UF, [ $(($a:expr, $b:expr)),+ $(,)? ] $(,)?) => {{
        let candidates = [ $(($a, $b)),+ ];
        $crate::assert_eqshare_hop_any!($events, DL => UF, &candidates);
    }};
    ($events:expr, UF => DL, $candidates:expr $(,)?) => {{
        let events = $events;
        let cands = $candidates;
        let ok = cands.iter().any(|(a, b)| $crate::eqshare_macros::has_pair(events, $crate::eqshare_macros::UF, $crate::eqshare_macros::DL, *a, *b));
        if !ok {
            panic!(
                "missing eqshare hop UF=>DL for any candidate pair.\ncandidates={:?}\nfirst events:\n{}",
                cands, $crate::eqshare_macros::dump(events)
            );
        }
    }};
    ($events:expr, DL => UF, $candidates:expr $(,)?) => {{
        let events = $events;
        let cands = $candidates;
        let ok = cands.iter().any(|(a, b)| $crate::eqshare_macros::has_pair(events, $crate::eqshare_macros::DL, $crate::eqshare_macros::UF, *a, *b));
        if !ok {
            panic!(
                "missing eqshare hop DL=>UF for any candidate pair.\ncandidates={:?}\nfirst events:\n{}",
                cands, $crate::eqshare_macros::dump(events)
            );
        }
    }};
}

#[macro_export]
macro_rules! assert_eqshare_hop_none {
    ($events:expr, UF => DL, [ $(($a:expr, $b:expr)),+ $(,)? ] $(,)?) => {{
        let candidates = [ $(($a, $b)),+ ];
        $crate::assert_eqshare_hop_none!($events, UF => DL, &candidates);
    }};
    ($events:expr, DL => UF, [ $(($a:expr, $b:expr)),+ $(,)? ] $(,)?) => {{
        let candidates = [ $(($a, $b)),+ ];
        $crate::assert_eqshare_hop_none!($events, DL => UF, &candidates);
    }};
    ($events:expr, UF => DL, $candidates:expr $(,)?) => {{
        let events = $events;
        let cands = $candidates;
        let found = cands.iter().find(|(a, b)| $crate::eqshare_macros::has_pair(events, $crate::eqshare_macros::UF, $crate::eqshare_macros::DL, **a, **b));
        if let Some((a, b)) = found {
            panic!(
                "unexpected eqshare hop UF=>DL for pair {:?} = {:?}\ncandidates={:?}\nfirst events:\n{}",
                a, b, cands, $crate::eqshare_macros::dump(events)
            );
        }
    }};
    ($events:expr, DL => UF, $candidates:expr $(,)?) => {{
        let events = $events;
        let cands = $candidates;
        let found = cands.iter().find(|(a, b)| $crate::eqshare_macros::has_pair(events, $crate::eqshare_macros::DL, $crate::eqshare_macros::UF, **a, **b));
        if let Some((a, b)) = found {
            panic!(
                "unexpected eqshare hop DL=>UF for pair {:?} = {:?}\ncandidates={:?}\nfirst events:\n{}",
                a, b, cands, $crate::eqshare_macros::dump(events)
            );
        }
    }};
}
