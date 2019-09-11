use pandoc_types::definition::{Attr, Block, Inline, Pandoc, Target};

use std::collections::HashMap;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;

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

    let transforms: Vec<Box<dyn crate::walk_pandoc::Walk>> = vec![
        Box::new(RunPython { run_dir: root }),
        Box::new(Graphviz { run_dir: root }),
    ];

    let transformed = transforms.into_iter().fold(parsed, |p, mut t| {
        crate::walk_pandoc::walk_pandoc(&mut *t, p)
    });

    Ok(transformed)
}

pub fn to_pdf(pandoc: &Pandoc, root: &Path, pdf_path: &Path) -> io::Result<()> {
    let mut cmd = Command::new("pandoc")
        .args(&[
            "-f",
            "json",
            "-o",
            pdf_path.to_str().expect("failed to make str from path"),
        ])
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let src = serde_json::to_string(pandoc).unwrap();
    write!(cmd.stdin.as_mut().expect("failed to get stdin"), "{}", &src)?;
    let out = cmd.wait_with_output()?;
    let compiled = String::from_utf8_lossy(&out.stdout).to_string();

    Ok(())
}

struct RunPython<'a> {
    run_dir: &'a Path,
}

impl<'a> crate::walk_pandoc::Walk for RunPython<'a> {
    fn block(&mut self, block: Block) -> Vec<Block> {
        match block {
            Block::CodeBlock(attr, src) => {
                // TODO: OOB
                if attr.1.len() == 0 {
                    return vec![Block::CodeBlock(attr.clone(), src)];
                }
                let lang = &attr.1[0];
                if lang == "python" {
                    lazy_static::lazy_static! {
                        static ref PYTHON_CACHE: Mutex<HashMap<u64, String>> = Mutex::new(HashMap::new());
                    }

                    let src_hash = {
                        use std::hash::{Hash, Hasher};
                        let mut hasher = std::collections::hash_map::DefaultHasher::new();
                        src.hash(&mut hasher);
                        hasher.finish()
                    };

                    let mut cache = PYTHON_CACHE.lock().unwrap();

                    let compiled = if cache.contains_key(&src_hash) {
                        cache.get(&src_hash).unwrap().clone()
                    } else {
                        let mut cmd = Command::new("python")
                            .current_dir(self.run_dir)
                            .stdin(Stdio::piped())
                            .stdout(Stdio::piped())
                            .spawn()
                            .unwrap();
                        write!(cmd.stdin.as_mut().expect("failed to get stdin"), "{}", &src)
                            .unwrap();
                        let out = cmd.wait_with_output().unwrap();
                        let compiled = String::from_utf8_lossy(&out.stdout).to_string();
                        cache.insert(src_hash, compiled.clone());
                        compiled
                    };

                    vec![
                        Block::CodeBlock(attr.clone(), src),
                        Block::CodeBlock(
                            Attr("".to_string(), vec!["".to_string()], vec![]),
                            compiled,
                        ),
                    ]
                } else {
                    vec![Block::CodeBlock(attr, src)]
                }
            }
            _ => vec![block],
        }
    }
}

struct Graphviz<'a> {
    run_dir: &'a Path,
}

impl<'a> crate::walk_pandoc::Walk for Graphviz<'a> {
    fn block(&mut self, block: Block) -> Vec<Block> {
        match block {
            Block::CodeBlock(attr, src) => {
                // TODO: OOB
                if attr.1.len() == 0 {
                    return vec![Block::CodeBlock(attr.clone(), src)];
                }
                let lang = &attr.1[0];
                if lang == "graphviz" {
                    lazy_static::lazy_static! {
                        static ref GRAPHVIZ_CACHE: Mutex<HashMap<u64, (PathBuf, PathBuf)>> = Mutex::new(HashMap::new());
                    }

                    let src_hash = {
                        use std::hash::{Hash, Hasher};
                        let mut hasher = std::collections::hash_map::DefaultHasher::new();
                        src.hash(&mut hasher);
                        hasher.finish()
                    };

                    let mut cache = GRAPHVIZ_CACHE.lock().unwrap();

                    let (output_name, output_path) = if cache.contains_key(&src_hash) {
                        cache.get(&src_hash).unwrap().clone()
                    } else {
                        let output_name = PathBuf::from(format!("graph-{}.png", src_hash));
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
                        write!(cmd.stdin.as_mut().expect("failed to get stdin"), "{}", &src)
                            .unwrap();
                        let _ = cmd.wait_with_output().unwrap();

                        cache.insert(src_hash, (output_name.clone(), output_path.clone()));

                        (output_name, output_path)
                    };

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
