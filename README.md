# busy-beaver

判定小规模图灵机是否停机。

## 问题

记 $\mathcal{M}(n, m)$ 为所有 $n$ 状态、$m$ 符号的图灵机，从全空白纸带出发。
对停机的机器 $M$，记 $s(M)$ 为它停机前走的步数。定义

$$
\mathrm{BB}(n, m) = \max\{\, s(M) : M \in \mathcal{M}(n, m),\ M \text{ 停机} \,\}
$$

$\mathrm{BB}$ 不可计算：没有一个算法能对所有 $n$ 求出它的值。
但对固定的、较小的 $n, m$，只需对 $\mathcal{M}(n, m)$ 中的每台机器回答"停或不停"，就能求出确切值。

## 已知

| | 值 | |
|---|---|---|
| $\mathrm{BB}(4, 2)$ | 107 | 1983 |
| $\mathrm{BB}(5, 2)$ | 47,176,870 | 2024，bbchallenge，Coq 证明 |
| $\mathrm{BB}(6, 2)$ | $> 10 \uparrow\uparrow 11{,}010{,}000$ | 2025 年下界，未定 |
| $\mathrm{BB}(2, 5)$、$\mathrm{BB}(3, 3)$ | | 未定 |

## 方法

停机的机器，模拟到停下即可证明。难的是不停机的机器：必须给出证明，说明它永远不会停。

一个**判定器**是一个算法 $D$：输入一台机器 $M$，输出"不停机"及一份可检查的证据，或者输出"不知道"。
它必须是可靠的：只要 $D$ 说不停机，$M$ 就真的不停机。

目标是逐步缩小"不知道"的那部分机器，并让每份证据都能用 Coq 或 Lean 独立检查。

## 实现

枚举与判定用 Rust，证明用 Coq 或 Lean。

## 结果

按树范式（Tree Normal Form）枚举：在全空白纸带上模拟，遇到未定义的转移才分叉，
目标状态只取已出现的或下一个新状态，第一步只向右。计数不含 `1RZ` 停机子节点。

| $n$ | 树范式 | A0 = 1RB | 最长停机 | 与已知值 |
|---|---|---|---|---|
| 2 | 61 | 41 | 6 | 一致 |
| 3 | 5,417 | 4,057 | 21 | 一致 |
| 4 | 858,909 | 620,261 | 107 | 一致 |
| 5 | 181,385,789 | 126,891,605 | 47,176,870（冠军单独验证） | 一致 |

$n = 5$ 的枚举在 14 核上约 9 秒。两个判定器（每台机器前 $10^4$ 步）：

| 结论 | 台数 |
|---|---|
| 停机 | 48,379,793 |
| 循环机：格局完全重复 | 17,982,641 |
| 平移循环机：在边界处平移重复 | 109,324,839 |
| 未判定 | 5,698,516 |

判定器只在"不停机"时下结论。$n = 4$ 时，全部 588,656 台被判为不停机的机器，关掉判定器后跑 1000 步（$> \mathrm{BB}(4) = 107$）均未停机。

```
cargo build --release
target/release/bb run 1RB1LC_1RC1RB_1RD0LE_1LA1LD_1RZ0LA
target/release/bb enum 5 --steps 10000 --holdouts holdouts.txt
cargo test --release -- --include-ignored
```

## 进度

- [x] 枚举 $\mathcal{M}(n, 2)$，$n \le 5$，计数与已知值一致
- [x] 判定器：循环机、平移循环机
- [ ] 判定 $n = 5$ 剩下的 5,698,516 台
- [ ] 在 $\mathrm{BB}(2, 5)$ 或 $\mathrm{BB}(3, 3)$ 的未判定机器上试新判定器

## 参考

- The bbchallenge Collaboration, *Determination of the fifth Busy Beaver value*, [arXiv:2509.12337](https://arxiv.org/abs/2509.12337)
- [bbchallenge.org](https://bbchallenge.org)，[BB(6) 现状](https://wiki.bbchallenge.org/wiki/BB(6))
