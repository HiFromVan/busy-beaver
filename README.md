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

## 进度

- [ ] 枚举 $\mathcal{M}(5, 2)$ 并复现已知的判定结果
- [ ] 在 $\mathrm{BB}(2, 5)$ 或 $\mathrm{BB}(3, 3)$ 的未判定机器上试新判定器

## 参考

- The bbchallenge Collaboration, *Determination of the fifth Busy Beaver value*, [arXiv:2509.12337](https://arxiv.org/abs/2509.12337)
- [bbchallenge.org](https://bbchallenge.org)，[BB(6) 现状](https://wiki.bbchallenge.org/wiki/BB(6))
