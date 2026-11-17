//! 图灵机：n 个状态（A, B, …），2 个符号（0 为空白）。
//! 文本格式与 bbchallenge 一致：`1RB1LC_1RC1RB_...`，每个状态两条转移（读到 0、读到 1），
//! `---` 表示未定义，目标状态 `Z`（或任何超出 n 的字母）表示停机。

use std::fmt;

pub const MAX_STATES: usize = 6;
/// 目标状态为 HALT 表示停机转移。
pub const HALT: u8 = 255;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Trans {
    pub write: u8,
    pub right: bool,
    pub next: u8,
}

impl Trans {
    pub const HALT_1RZ: Trans = Trans { write: 1, right: true, next: HALT };
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Machine {
    pub n: u8,
    pub t: [[Option<Trans>; 2]; MAX_STATES],
}

impl Machine {
    pub fn empty(n: u8) -> Machine {
        assert!((1..=MAX_STATES as u8).contains(&n));
        Machine { n, t: [[None; 2]; MAX_STATES] }
    }

    pub fn with(&self, state: u8, read: u8, tr: Trans) -> Machine {
        let mut m = *self;
        m.t[state as usize][read as usize] = Some(tr);
        m
    }

    /// 未定义转移的个数。
    pub fn undefined(&self) -> usize {
        (0..self.n as usize).map(|s| self.t[s].iter().filter(|x| x.is_none()).count()).sum()
    }

    /// 已出现过的最大状态编号（A 总算出现过）。
    pub fn max_used_state(&self) -> u8 {
        let mut mx = 0;
        for s in 0..self.n as usize {
            for tr in self.t[s].iter().flatten() {
                if tr.next != HALT && tr.next > mx {
                    mx = tr.next;
                }
            }
        }
        mx
    }

    pub fn parse(text: &str) -> Result<Machine, String> {
        let parts: Vec<&str> = text.trim().split('_').collect();
        let n = parts.len();
        if n == 0 || n > MAX_STATES {
            return Err(format!("状态数 {n} 不在 1..={MAX_STATES}"));
        }
        let mut m = Machine::empty(n as u8);
        for (s, p) in parts.iter().enumerate() {
            let b = p.as_bytes();
            if b.len() != 6 {
                return Err(format!("第 {} 个状态 `{p}` 应为 6 个字符", s + 1));
            }
            for r in 0..2 {
                let c = &b[r * 3..r * 3 + 3];
                if c == b"---" {
                    continue;
                }
                let write = match c[0] {
                    b'0' => 0,
                    b'1' => 1,
                    x => return Err(format!("非法写入符号 `{}`", x as char)),
                };
                let right = match c[1] {
                    b'R' => true,
                    b'L' => false,
                    x => return Err(format!("非法方向 `{}`", x as char)),
                };
                let next = match c[2] {
                    x @ b'A'..=b'Z' if ((x - b'A') as usize) < n => x - b'A',
                    b'A'..=b'Z' => HALT,
                    x => return Err(format!("非法状态 `{}`", x as char)),
                };
                m.t[s][r] = Some(Trans { write, right, next });
            }
        }
        Ok(m)
    }
}

impl fmt::Display for Machine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for s in 0..self.n as usize {
            if s > 0 {
                write!(f, "_")?;
            }
            for r in 0..2 {
                match self.t[s][r] {
                    None => write!(f, "---")?,
                    Some(tr) => {
                        let st = if tr.next == HALT { 'Z' } else { (b'A' + tr.next) as char };
                        write!(f, "{}{}{}", tr.write, if tr.right { 'R' } else { 'L' }, st)?
                    }
                }
            }
        }
        Ok(())
    }
}
