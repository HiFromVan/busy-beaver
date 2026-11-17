//! 树范式（Tree Normal Form）枚举。
//!
//! 从根机器出发（全部转移未定义；或固定 A0 = 1RB），在全空白纸带上模拟。
//! 遇到未定义的转移 (s, r) 时分叉：
//! - 总可以填停机转移 `1RZ`；
//! - 若此处之外还有未定义的转移，还可以填任意 (写 w, 方向 d, 目标 q)，其中 q 只能是已出现过的状态
//!   或下一个新状态（消去状态编号的置换对称）；整台机器的第一条转移只能向右（消去左右镜像对称）。
//! 判为不停机或超出限制的机器是叶子。

use crate::machine::{Machine, Trans};
use crate::sim::{Limits, Outcome, run};
use rayon::prelude::*;

#[derive(Default, Debug, Clone)]
pub struct Stats {
    /// 树上的全部节点（含根）。
    pub nodes: u64,
    /// 遇到未定义转移、因而被展开的节点。
    pub expanded: u64,
    /// 含 `1RZ` 并在其上停机的节点。
    pub halt: u64,
    pub cycler: u64,
    pub translated: u64,
    pub out_time: u64,
    pub out_space: u64,
    /// 停机机器中最长的步数及其 1 的个数。
    pub max_steps: u64,
    pub max_ones: u64,
    pub champion: Option<Machine>,
    /// 没判定出来的机器。
    pub holdouts: Vec<Machine>,
}

impl Stats {
    fn merge(mut self, o: Stats) -> Stats {
        self.nodes += o.nodes;
        self.expanded += o.expanded;
        self.halt += o.halt;
        self.cycler += o.cycler;
        self.translated += o.translated;
        self.out_time += o.out_time;
        self.out_space += o.out_space;
        if o.max_steps > self.max_steps {
            self.max_steps = o.max_steps;
            self.champion = o.champion;
        }
        self.max_ones = self.max_ones.max(o.max_ones);
        self.holdouts.extend(o.holdouts);
        self
    }
}

pub fn root(n: u8, one_rb: bool) -> Machine {
    let m = Machine::empty(n);
    if one_rb { m.with(0, 0, Trans { write: 1, right: true, next: 1 }) } else { m }
}

pub fn children(m: &Machine, state: u8, read: u8) -> Vec<Machine> {
    let mut out = vec![m.with(state, read, Trans::HALT_1RZ)];
    if m.undefined() > 1 {
        let first = m.undefined() == 2 * m.n as usize;
        let top = (m.max_used_state() + 1).min(m.n - 1);
        for next in 0..=top {
            for write in 0..2 {
                for right in [true, false] {
                    if first && !right {
                        continue;
                    }
                    out.push(m.with(state, read, Trans { write, right, next }));
                }
            }
        }
    }
    out
}

pub fn enumerate(n: u8, one_rb: bool, lim: &Limits) -> Stats {
    explore(root(n, one_rb), lim, 0)
}

fn explore(m: Machine, lim: &Limits, depth: u32) -> Stats {
    let mut st = Stats { nodes: 1, ..Default::default() };
    match run(&m, lim) {
        Outcome::Undefined { state, read, .. } => {
            st.expanded = 1;
            let kids = children(&m, state, read);
            let sub = if depth < 8 {
                kids.into_par_iter().map(|c| explore(c, lim, depth + 1)).reduce(Stats::default, Stats::merge)
            } else {
                kids.into_iter().map(|c| explore(c, lim, depth + 1)).fold(Stats::default(), Stats::merge)
            };
            st = st.merge(sub);
        }
        Outcome::Halt { steps, ones } => {
            st.halt = 1;
            st.max_steps = steps;
            st.max_ones = ones;
            st.champion = Some(m);
        }
        Outcome::Cycler { .. } => st.cycler = 1,
        Outcome::TranslatedCycler { .. } => st.translated = 1,
        Outcome::OutOfTime => {
            st.out_time = 1;
            st.holdouts.push(m);
        }
        Outcome::OutOfSpace { .. } => {
            st.out_space = 1;
            st.holdouts.push(m);
        }
    }
    st
}
