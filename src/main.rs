use bb::enumerate::enumerate;
use bb::machine::Machine;
use bb::sim::{Limits, run};
use std::io::Write;
use std::time::Instant;

const USAGE: &str = "\
用法：
  bb run <机器> [步数上限]          模拟一台机器，例：bb run 1RB1LB_1LA1RZ
  bb enum <n> [选项]                按树范式枚举 n 状态 2 符号的全部机器
      --1rb                         固定 A0 = 1RB
      --steps <N>                   每台机器最多模拟 N 步（默认 100000）
      --deciders <N>                判定器只在前 N 步运行（默认同 --steps）
      --holdouts <文件>             把没判定出来的机器写到文件，一行一台";

fn arg_u64(args: &[String], name: &str) -> Option<u64> {
    args.iter().position(|a| a == name).map(|i| args[i + 1].replace('_', "").parse().expect(name))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("run") if args.len() >= 3 => {
            let m = Machine::parse(&args[2]).unwrap_or_else(|e| panic!("{e}"));
            let steps = args.get(3).map(|s| s.replace('_', "").parse().unwrap()).unwrap_or(100_000_000);
            let lim = Limits { steps, deciders: steps.min(1_000_000), ..Default::default() };
            let t = Instant::now();
            let out = run(&m, &lim);
            println!("{m}\n{out:?}\n用时 {:.3?}", t.elapsed());
        }
        Some("enum") if args.len() >= 3 => {
            let n: u8 = args[2].parse().expect("n");
            let one_rb = args.iter().any(|a| a == "--1rb");
            let steps = arg_u64(&args, "--steps").unwrap_or(100_000);
            let deciders = arg_u64(&args, "--deciders").unwrap_or(steps);
            let lim = Limits { steps, deciders, ..Default::default() };
            let t = Instant::now();
            let st = enumerate(n, one_rb, &lim);
            let el = t.elapsed();
            println!("n = {n}{}，步数上限 {steps}，判定器前 {deciders} 步", if one_rb { "（A0 = 1RB）" } else { "" });
            println!("  树范式机器数      {:>13}   （不含 1RZ 停机子节点，与 bbchallenge 口径一致）", st.nodes - st.halt);
            println!("  节点总数          {:>13}", st.nodes);
            println!("    被展开          {:>13}", st.expanded);
            println!("    停机（1RZ）     {:>13}", st.halt);
            println!("    循环机          {:>13}", st.cycler);
            println!("    平移循环机      {:>13}", st.translated);
            println!("    超时            {:>13}", st.out_time);
            println!("    超出纸带        {:>13}", st.out_space);
            println!("  最长停机          {} 步，{} 个 1", st.max_steps, st.max_ones);
            if let Some(c) = st.champion {
                println!("  冠军              {c}");
            }
            println!("  用时              {el:.2?}");
            if let Some(i) = args.iter().position(|a| a == "--holdouts") {
                let mut f = std::io::BufWriter::new(std::fs::File::create(&args[i + 1]).unwrap());
                for m in &st.holdouts {
                    writeln!(f, "{m}").unwrap();
                }
                println!("  未判定的 {} 台写入 {}", st.holdouts.len(), args[i + 1]);
            }
        }
        _ => eprintln!("{USAGE}"),
    }
}
