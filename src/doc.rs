use pandoc_types::definition::{Attr, Block, Inline, Target};

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub fn compile(src: &str, root: &Path) -> io::Result<pandoc_types::definition::Pandoc> {
    let mut cmd = Command::new("pandoc")
        .args(&["-f", "markdown", "-t", "json"])
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    write!(cmd.stdin.as_mut().expect("failed to get stdin"), "{}", &src)?;
    let out = cmd.wait_with_output()?;
    let compiled = String::from_utf8_lossy(&out.stdout).to_string();

    let parsed: pandoc_types::definition::Pandoc =
        serde_json::from_str(&compiled).expect("failed to parse pandoc output");

    let transformed = crate::walk_pandoc::walk_pandoc(&mut RunPython { run_dir: root }, parsed);

    Ok(transformed)
}

struct RunPython<'a> {
    run_dir: &'a Path,
}

impl<'a> crate::walk_pandoc::Walk for RunPython<'a> {
    fn block(&mut self, block: Block) -> Vec<Block> {
        match block {
            Block::CodeBlock(attr, src) => {
                let lang = &attr.1[0];
                if lang == "python" {
                    let mut cmd = Command::new("python")
                        .current_dir(self.run_dir)
                        .stdin(Stdio::piped())
                        .stdout(Stdio::piped())
                        .spawn()
                        .unwrap();
                    write!(cmd.stdin.as_mut().expect("failed to get stdin"), "{}", &src).unwrap();
                    let out = cmd.wait_with_output().unwrap();
                    let compiled = String::from_utf8_lossy(&out.stdout).to_string();

                    vec![
                        Block::CodeBlock(attr.clone(), src),
                        Block::CodeBlock(
                            Attr("".to_string(), vec!["".to_string()], vec![]),
                            compiled,
                        ),
                    ]
                } else if lang == "graphviz" {
                    let output_name = PathBuf::from("graph.png");
                    let output_path = self.run_dir.join(&output_name);

                    let mut cmd = Command::new("dot")
                        .args(&[
                            "-Tpng",
                            &format!("-o{}", output_path.to_string_lossy().to_string()),
                        ])
                        .current_dir(self.run_dir)
                        .stdin(Stdio::piped())
                        .spawn()
                        .unwrap();
                    write!(cmd.stdin.as_mut().expect("failed to get stdin"), "{}", &src).unwrap();
                    let _ = cmd.wait_with_output().unwrap();

                    vec![Block::Para(vec![Inline::Image(
                        Attr::null(),
                        vec![],
                        Target(
                            output_name.to_string_lossy().to_string(),
                            output_name.to_string_lossy().to_string(),
                        ),
                    )])]
                } else {
                    vec![Block::CodeBlock(attr, src)]
                }
            }
            _ => vec![block],
        }
    }
}
