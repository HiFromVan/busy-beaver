//! 与已知结果逐一对照。数字来源：bbchallenge wiki「Tree Normal Form」页、arXiv:2509.12337。

use bb::enumerate::{children, enumerate, root};
use bb::machine::Machine;
use bb::sim::{Limits, Outcome, run};

fn tnf_count(n: u8, one_rb: bool, steps: u64) -> (u64, u64) {
    let st = enumerate(n, one_rb, &Limits { steps, deciders: steps, ..Default::default() });
    (st.nodes - st.halt, st.max_steps)
}

#[test]
fn champions() {
    let cases = [
        ("1RB1LB_1LA1RZ", 6, 4),
        ("1RB1RZ_1LB0RC_1LC1LA", 21, 5),
        ("1RB1LB_1LA0LC_1RZ1LD_1RD0RA", 107, 13),
        ("1RB1LC_1RC1RB_1RD0LE_1LA1LD_1RZ0LA", 47_176_870, 4098),
    ];
    for (text, steps, ones) in cases {
        let m = Machine::parse(text).unwrap();
        assert_eq!(m.to_string(), text);
        let lim = Limits { steps: 100_000_000, deciders: 0, ..Default::default() };
        assert_eq!(run(&m, &lim), Outcome::Halt { steps, ones }, "{text}");
    }
}

#[test]
fn tnf_counts_and_bb_up_to_4() {
    // (n, TNF, TNF-1RB, BB(n))
    for (n, tnf, tnf_1rb, bb) in [(2, 61, 41, 6), (3, 5_417, 4_057, 21), (4, 858_909, 620_261, 107)] {
        assert_eq!(tnf_count(n, false, 1000), (tnf, bb), "n = {n}");
        assert_eq!(tnf_count(n, true, 1000), (tnf_1rb, bb), "n = {n}, 1RB");
    }
}

/// 约 15 秒：cargo test --release -- --ignored
#[test]
#[ignore]
fn tnf_counts_5() {
    assert_eq!(tnf_count(5, false, 1000).0, 181_385_789);
    assert_eq!(tnf_count(5, true, 1000).0, 126_891_605);
}

/// 判定器可靠性：4 状态里凡被判为不停机的机器，关掉判定器跑 1000 步（> BB(4) = 107）都不能停。
#[test]
fn deciders_are_sound_on_4() {
    let with = Limits { steps: 1000, deciders: 1000, ..Default::default() };
    let without = Limits { steps: 1000, deciders: 0, ..Default::default() };
    let mut stack = vec![root(4, false)];
    let mut checked = 0;
    while let Some(m) = stack.pop() {
        match run(&m, &with) {
            Outcome::Undefined { state, read, .. } => stack.extend(children(&m, state, read)),
            Outcome::Cycler { .. } | Outcome::TranslatedCycler { .. } => {
                assert_eq!(run(&m, &without), Outcome::OutOfTime, "误判：{m}");
                checked += 1;
            }
            _ => {}
        }
    }
    assert_eq!(checked, 87_754 + 500_902);
}
