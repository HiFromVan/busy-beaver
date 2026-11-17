//! 从全空白纸带模拟一台机器，同时运行两个判定器：
//!
//! - **循环机（cycler）**：某一时刻的完整格局（状态、读写头、整条纸带）与更早某时刻完全相同，
//!   则此后周期重复，永不停机。用 Brent 法：在第 1, 2, 4, 8, … 步存一份快照，之后每步与快照比较。
//! - **平移循环机（translated cycler）**：在 t₁ < t₂ 两个时刻，读写头都刚刚走到纸带的新边界（同一侧），
//!   状态相同，且以读写头为界、向内 L 格的纸带内容相同，其中 L 是 [t₁, t₂] 内读写头离开 t₁ 边界的最远距离。
//!   边界外全是空白，[t₁, t₂] 内读到的格子全在这段窗口里，所以 t₂ 之后会把 [t₁, t₂] 的过程平移重演，永不停机。
//!
//! 两个判定都只在"说不停机"时下结论，并且结论是可靠的：不会把会停的机器判成不停。

use crate::machine::{HALT, Machine};
use std::cell::RefCell;

pub const TAPE_LEN: usize = 1 << 16;

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    /// 最多模拟多少步。
    pub steps: u64,
    /// 判定器只在前这么多步里运行，之后只做纯模拟。
    pub deciders: u64,
    /// 平移循环机快照的窗口宽度（格）。
    pub tc_window: usize,
    /// 每一侧最多记录多少个边界事件。
    pub tc_events: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Limits { steps: 100_000, deciders: 100_000, tc_window: 512, tc_events: 4096 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// 走了 `steps` 步后遇到未定义的转移 (state, read)。
    Undefined { state: u8, read: u8, steps: u64 },
    /// 执行停机转移后停下；`steps` 含停机这一步，`ones` 为纸带上 1 的个数。
    Halt { steps: u64, ones: u64 },
    Cycler { steps: u64 },
    TranslatedCycler { steps: u64 },
    OutOfTime,
    OutOfSpace { steps: u64 },
}

struct Event {
    state: u8,
    pos: usize,
    /// 从本事件到下一个同侧事件之间，读写头离边界最远的位置（右侧事件记最小值，左侧记最大值）。
    seg: usize,
    snap_from: usize,
    snap_off: usize,
    snap_len: usize,
}

struct Buffers {
    tape: Vec<u8>,
    cyc: Vec<u8>,
    right: Vec<Event>,
    left: Vec<Event>,
    pool_r: Vec<u8>,
    pool_l: Vec<u8>,
}

thread_local! {
    static BUF: RefCell<Buffers> = RefCell::new(Buffers {
        tape: vec![0; TAPE_LEN],
        cyc: Vec::new(),
        right: Vec::new(),
        left: Vec::new(),
        pool_r: Vec::new(),
        pool_l: Vec::new(),
    });
}

pub fn run(m: &Machine, lim: &Limits) -> Outcome {
    BUF.with(|b| run_with(&mut b.borrow_mut(), m, lim))
}

#[inline]
fn snap_cell(pool: &[u8], ev: &Event, abs: usize) -> u8 {
    if abs >= ev.snap_from && abs < ev.snap_from + ev.snap_len {
        pool[ev.snap_off + abs - ev.snap_from]
    } else {
        0
    }
}

fn run_with(b: &mut Buffers, m: &Machine, lim: &Limits) -> Outcome {
    // 转移表压成一维：bit0 写入，bit1 向右，bit8..15 下一状态；u32::MAX 表示未定义。
    let mut tbl = [u32::MAX; 12];
    for s in 0..m.n as usize {
        for r in 0..2 {
            if let Some(t) = m.t[s][r] {
                tbl[s * 2 + r] = (t.write as u32) | ((t.right as u32) << 1) | ((t.next as u32) << 8);
            }
        }
    }

    let Buffers { tape, cyc, right, left, pool_r, pool_l } = b;
    let start = TAPE_LEN / 2;
    let (mut head, mut lo, mut hi) = (start, start, start);
    let mut state: u8 = 0;
    let mut step: u64 = 0;

    // 循环机快照
    let mut next_snap: u64 = 1;
    let (mut sn_state, mut sn_head, mut sn_lo, mut sn_hi) = (0u8, start, start, start);
    cyc.clear();
    cyc.push(0);

    // 平移循环机
    right.clear();
    left.clear();
    pool_r.clear();
    pool_l.clear();
    let (mut cur_min, mut cur_max) = (start, start);

    let outcome = loop {
        let r = tape[head];
        let e = tbl[state as usize * 2 + r as usize];
        if e == u32::MAX {
            break Outcome::Undefined { state, read: r, steps: step };
        }
        tape[head] = (e & 1) as u8;
        step += 1;
        let next = (e >> 8) as u8;
        if next == HALT {
            let ones = tape[lo..=hi].iter().filter(|&&c| c == 1).count() as u64;
            break Outcome::Halt { steps: step, ones };
        }
        if e & 2 != 0 {
            head += 1
        } else {
            head -= 1
        }
        state = next;
        if head == 0 || head == TAPE_LEN - 1 {
            break Outcome::OutOfSpace { steps: step };
        }
        let new_right = head > hi;
        let new_left = head < lo;
        if new_right {
            hi = head;
        }
        if new_left {
            lo = head;
        }
        if step >= lim.steps {
            break Outcome::OutOfTime;
        }
        if step > lim.deciders {
            continue;
        }

        // —— 循环机 ——
        if state == sn_state && head == sn_head {
            let same = (lo..=hi).all(|i| {
                let old = if i >= sn_lo && i <= sn_hi { cyc[i - sn_lo] } else { 0 };
                tape[i] == old
            });
            if same {
                break Outcome::Cycler { steps: step };
            }
        }
        if step == next_snap {
            next_snap *= 2;
            sn_state = state;
            sn_head = head;
            sn_lo = lo;
            sn_hi = hi;
            cyc.clear();
            cyc.extend_from_slice(&tape[lo..=hi]);
        }

        // —— 平移循环机 ——
        cur_min = cur_min.min(head);
        cur_max = cur_max.max(head);
        if new_right {
            if let Some(last) = right.last_mut() {
                last.seg = cur_min;
            }
            cur_min = head;
            let mut far = usize::MAX;
            let mut found = false;
            for ev in right.iter().rev() {
                far = far.min(ev.seg);
                if ev.state != state {
                    continue;
                }
                let l = ev.pos - far;
                if l > lim.tc_window || l > head {
                    continue;
                }
                if (0..=l).all(|k| tape[head - k] == snap_cell(pool_r, ev, ev.pos - k)) {
                    found = true;
                    break;
                }
            }
            if found {
                break Outcome::TranslatedCycler { steps: step };
            }
            if right.len() < lim.tc_events {
                let from = lo.max(head.saturating_sub(lim.tc_window));
                let off = pool_r.len();
                pool_r.extend_from_slice(&tape[from..=head]);
                right.push(Event { state, pos: head, seg: head, snap_from: from, snap_off: off, snap_len: head - from + 1 });
            }
        }
        if new_left {
            if let Some(last) = left.last_mut() {
                last.seg = cur_max;
            }
            cur_max = head;
            let mut far = 0usize;
            let mut found = false;
            for ev in left.iter().rev() {
                far = far.max(ev.seg);
                if ev.state != state {
                    continue;
                }
                let l = far - ev.pos;
                if l > lim.tc_window || head + l >= TAPE_LEN {
                    continue;
                }
                if (0..=l).all(|k| tape[head + k] == snap_cell(pool_l, ev, ev.pos + k)) {
                    found = true;
                    break;
                }
            }
            if found {
                break Outcome::TranslatedCycler { steps: step };
            }
            if left.len() < lim.tc_events {
                let to = hi.min(head + lim.tc_window);
                let off = pool_l.len();
                pool_l.extend_from_slice(&tape[head..=to]);
                left.push(Event { state, pos: head, seg: head, snap_from: head, snap_off: off, snap_len: to - head + 1 });
            }
        }
    };

    for c in &mut tape[lo..=hi] {
        *c = 0;
    }
    outcome
}
