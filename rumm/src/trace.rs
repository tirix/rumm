use std::{fs::File, io::{BufWriter, Write}};
use crate::{lang::Display, context::Context};
use serde_json::json;

pub struct Trace {
    pub label: String,
    children: Option<Vec<Trace>>,
    depth: usize,
}

impl Trace {
    pub fn new() -> Self {
        Trace {
            label: "Entry".to_string(),
            children: None,
            depth: 0,
        }
    }

    pub fn message(&mut self, message: &str) {
        //println!("{:indent$}{message}", "", indent = self.depth);
        self.children.get_or_insert(vec![]).push(Trace{label: message.to_string(), children: None, depth: self.depth + 1});
    }

    #[must_use]
    pub fn enter(&mut self, context: &Context, message: &str) -> Self {
        self.message(&format!("Proving {}", context.goal().to_string(&context.db)));
        self.message(&format!(">> {message}"));
        Trace {
            label: message.to_string(),
            children: None,
            depth: self.depth +1,
        }
    }

    pub fn exit(&mut self, trace: Trace, message: &str) {
        self.children.get_or_insert(vec![]).push(trace);
        self.message(&format!("<< {message}"));
    }

    pub fn dump(&self) {
        println!("{:indent$}{label}", "", indent = self.depth, label = self.label); //
        for sub_trace in  self.children.iter().flatten() {
            sub_trace.dump();
        }
    }

    pub fn write_json(&self, f: &mut impl Write) -> std::io::Result<()> {
        for trace in self.children.iter().flatten() {
            f.write((format!("{{ name: {}, children: [", json!(trace.label).to_string())).as_bytes())?;
            trace.write_json(f)?;
            f.write(format!("] }},\n").as_bytes())?;
        }
        Ok(())
    }

    pub fn export_js_tree(&self, theorem: &str) {
        let mut f = BufWriter::new(File::create(format!("{theorem}.html")).expect("Unable to create file"));
        f.write("
        <html>
            <head>
                <script src=\"../vendor/treeview/dist/treeview.min.js\"></script>
                <link rel=\"stylesheet\" type=\"text/css\" href=\"../vendor/treeview/dist/treeview.min.css\"/>
            <script>
            window.onload = (event) => { new TreeView([
            ".as_bytes())
        .and_then(|_| { self.write_json(&mut f) } )
        .and_then(|_| { f.write("
            ], 'tree');};
            </script>
            </head>
            <body><div id=\"tree\"></div></body>
        </html>
        ".as_bytes())})
        .expect("Error while writing file");
    }
}

