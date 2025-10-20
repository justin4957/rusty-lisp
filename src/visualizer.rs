use crate::ast::LispExpr;
use std::fmt::Write as FmtWrite;

/// Generates DOT graph representation of AST for Graphviz
pub struct DotVisualizer {
    node_counter: usize,
}

impl DotVisualizer {
    pub fn new() -> Self {
        DotVisualizer { node_counter: 0 }
    }

    /// Generate DOT graph for an AST
    pub fn visualize(&mut self, exprs: &[LispExpr]) -> String {
        let mut output = String::new();
        output.push_str("digraph AST {\n");
        output.push_str("  node [shape=box, style=rounded];\n");
        output.push_str("  rankdir=TB;\n\n");

        for expr in exprs {
            self.visualize_expr(expr, &mut output, None);
        }

        output.push_str("}\n");
        output
    }

    fn next_node_id(&mut self) -> String {
        let id = format!("node{}", self.node_counter);
        self.node_counter += 1;
        id
    }

    fn visualize_expr(&mut self, expr: &LispExpr, output: &mut String, parent_id: Option<&str>) -> String {
        let node_id = self.next_node_id();

        match expr {
            LispExpr::Number(n) => {
                writeln!(output, "  {} [label=\"{}\", fillcolor=\"lightblue\", style=\"filled,rounded\"];", node_id, n).unwrap();
            }
            LispExpr::Symbol(s) => {
                writeln!(output, "  {} [label=\"{}\", fillcolor=\"lightgreen\", style=\"filled,rounded\"];", node_id, escape_dot(s)).unwrap();
            }
            LispExpr::String(s) => {
                writeln!(output, "  {} [label=\"\\\"{}\\\"\", fillcolor=\"lightyellow\", style=\"filled,rounded\"];", node_id, escape_dot(s)).unwrap();
            }
            LispExpr::Bool(b) => {
                writeln!(output, "  {} [label=\"{}\", fillcolor=\"lightcoral\", style=\"filled,rounded\"];", node_id, b).unwrap();
            }
            LispExpr::Nil => {
                writeln!(output, "  {} [label=\"nil\", fillcolor=\"lightgray\", style=\"filled,rounded\"];", node_id).unwrap();
            }
            LispExpr::List(items) => {
                writeln!(output, "  {} [label=\"List\", fillcolor=\"wheat\", style=\"filled,rounded\"];", node_id).unwrap();
                for (i, item) in items.iter().enumerate() {
                    let child_id = self.visualize_expr(item, output, Some(&node_id));
                    writeln!(output, "  {} -> {} [label=\"{}\"];", node_id, child_id, i).unwrap();
                }
            }
            LispExpr::Macro { name, parameters, body } => {
                writeln!(output, "  {} [label=\"Macro: {}\", fillcolor=\"plum\", style=\"filled,rounded\"];", node_id, escape_dot(name)).unwrap();

                let params_id = self.next_node_id();
                let params_label = format!("Parameters: {}", parameters.join(", "));
                writeln!(output, "  {} [label=\"{}\", fillcolor=\"thistle\", style=\"filled,rounded\"];", params_id, escape_dot(&params_label)).unwrap();
                writeln!(output, "  {} -> {} [label=\"params\"];", node_id, params_id).unwrap();

                let body_id = self.visualize_expr(body, output, Some(&node_id));
                writeln!(output, "  {} -> {} [label=\"body\"];", node_id, body_id).unwrap();
            }
            LispExpr::MacroCall { name, args } => {
                writeln!(output, "  {} [label=\"MacroCall: {}\", fillcolor=\"violet\", style=\"filled,rounded\"];", node_id, escape_dot(name)).unwrap();
                for (i, arg) in args.iter().enumerate() {
                    let child_id = self.visualize_expr(arg, output, Some(&node_id));
                    writeln!(output, "  {} -> {} [label=\"arg{}\"];", node_id, child_id, i).unwrap();
                }
            }
            LispExpr::Quote(inner) => {
                writeln!(output, "  {} [label=\"Quote\", fillcolor=\"lightpink\", style=\"filled,rounded\"];", node_id).unwrap();
                let child_id = self.visualize_expr(inner, output, Some(&node_id));
                writeln!(output, "  {} -> {};", node_id, child_id).unwrap();
            }
            LispExpr::Quasiquote(inner) => {
                writeln!(output, "  {} [label=\"Quasiquote\", fillcolor=\"lightsalmon\", style=\"filled,rounded\"];", node_id).unwrap();
                let child_id = self.visualize_expr(inner, output, Some(&node_id));
                writeln!(output, "  {} -> {};", node_id, child_id).unwrap();
            }
            LispExpr::Unquote(inner) => {
                writeln!(output, "  {} [label=\"Unquote\", fillcolor=\"lightseagreen\", style=\"filled,rounded\"];", node_id).unwrap();
                let child_id = self.visualize_expr(inner, output, Some(&node_id));
                writeln!(output, "  {} -> {};", node_id, child_id).unwrap();
            }
            LispExpr::Splice(inner) => {
                writeln!(output, "  {} [label=\"Splice\", fillcolor=\"lightsteelblue\", style=\"filled,rounded\"];", node_id).unwrap();
                let child_id = self.visualize_expr(inner, output, Some(&node_id));
                writeln!(output, "  {} -> {};", node_id, child_id).unwrap();
            }
            LispExpr::Gensym(name) => {
                writeln!(output, "  {} [label=\"Gensym: {}\", fillcolor=\"lavender\", style=\"filled,rounded\"];", node_id, escape_dot(name)).unwrap();
            }
        }

        if let Some(_parent) = parent_id {
            // Edge already created in parent
        }

        node_id
    }
}

impl Default for DotVisualizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Generates HTML visualization of AST
pub struct HtmlVisualizer;

impl HtmlVisualizer {
    pub fn new() -> Self {
        HtmlVisualizer
    }

    /// Generate interactive HTML visualization
    pub fn visualize(&self, exprs: &[LispExpr]) -> String {
        let mut output = String::new();

        output.push_str("<!DOCTYPE html>\n");
        output.push_str("<html lang=\"en\">\n");
        output.push_str("<head>\n");
        output.push_str("  <meta charset=\"UTF-8\">\n");
        output.push_str("  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
        output.push_str("  <title>AST Visualization</title>\n");
        output.push_str("  <style>\n");
        output.push_str(include_str!("visualizer_style.css"));
        output.push_str("  </style>\n");
        output.push_str("</head>\n");
        output.push_str("<body>\n");
        output.push_str("  <div class=\"container\">\n");
        output.push_str("    <h1>AST Visualization</h1>\n");
        output.push_str("    <div class=\"ast-tree\">\n");

        for expr in exprs {
            self.visualize_expr(expr, &mut output, 0);
        }

        output.push_str("    </div>\n");
        output.push_str("  </div>\n");
        output.push_str("  <script>\n");
        output.push_str(include_str!("visualizer_script.js"));
        output.push_str("  </script>\n");
        output.push_str("</body>\n");
        output.push_str("</html>\n");

        output
    }

    fn visualize_expr(&self, expr: &LispExpr, output: &mut String, depth: usize) {
        let indent = "  ".repeat(depth + 3);

        match expr {
            LispExpr::Number(n) => {
                writeln!(output, "{}<div class=\"ast-node ast-number\">", indent).unwrap();
                writeln!(output, "{}  <span class=\"node-type\">Number</span>", indent).unwrap();
                writeln!(output, "{}  <span class=\"node-value\">{}</span>", indent, n).unwrap();
                writeln!(output, "{}</div>", indent).unwrap();
            }
            LispExpr::Symbol(s) => {
                writeln!(output, "{}<div class=\"ast-node ast-symbol\">", indent).unwrap();
                writeln!(output, "{}  <span class=\"node-type\">Symbol</span>", indent).unwrap();
                writeln!(output, "{}  <span class=\"node-value\">{}</span>", indent, escape_html(s)).unwrap();
                writeln!(output, "{}</div>", indent).unwrap();
            }
            LispExpr::String(s) => {
                writeln!(output, "{}<div class=\"ast-node ast-string\">", indent).unwrap();
                writeln!(output, "{}  <span class=\"node-type\">String</span>", indent).unwrap();
                writeln!(output, "{}  <span class=\"node-value\">\"{}\"</span>", indent, escape_html(s)).unwrap();
                writeln!(output, "{}</div>", indent).unwrap();
            }
            LispExpr::Bool(b) => {
                writeln!(output, "{}<div class=\"ast-node ast-bool\">", indent).unwrap();
                writeln!(output, "{}  <span class=\"node-type\">Bool</span>", indent).unwrap();
                writeln!(output, "{}  <span class=\"node-value\">{}</span>", indent, b).unwrap();
                writeln!(output, "{}</div>", indent).unwrap();
            }
            LispExpr::Nil => {
                writeln!(output, "{}<div class=\"ast-node ast-nil\">", indent).unwrap();
                writeln!(output, "{}  <span class=\"node-type\">Nil</span>", indent).unwrap();
                writeln!(output, "{}</div>", indent).unwrap();
            }
            LispExpr::List(items) => {
                writeln!(output, "{}<div class=\"ast-node ast-list\">", indent).unwrap();
                writeln!(output, "{}  <div class=\"node-header\">", indent).unwrap();
                writeln!(output, "{}    <span class=\"node-type\">List</span>", indent).unwrap();
                writeln!(output, "{}    <span class=\"toggle\">▼</span>", indent).unwrap();
                writeln!(output, "{}  </div>", indent).unwrap();
                writeln!(output, "{}  <div class=\"node-children\">", indent).unwrap();
                for item in items {
                    self.visualize_expr(item, output, depth + 1);
                }
                writeln!(output, "{}  </div>", indent).unwrap();
                writeln!(output, "{}</div>", indent).unwrap();
            }
            LispExpr::Macro { name, parameters, body } => {
                writeln!(output, "{}<div class=\"ast-node ast-macro\">", indent).unwrap();
                writeln!(output, "{}  <div class=\"node-header\">", indent).unwrap();
                writeln!(output, "{}    <span class=\"node-type\">Macro</span>", indent).unwrap();
                writeln!(output, "{}    <span class=\"node-value\">{}</span>", indent, escape_html(name)).unwrap();
                writeln!(output, "{}    <span class=\"toggle\">▼</span>", indent).unwrap();
                writeln!(output, "{}  </div>", indent).unwrap();
                writeln!(output, "{}  <div class=\"node-children\">", indent).unwrap();
                writeln!(output, "{}    <div class=\"param-list\">", indent).unwrap();
                writeln!(output, "{}      <span class=\"label\">Parameters: {}</span>", indent, parameters.join(", ")).unwrap();
                writeln!(output, "{}    </div>", indent).unwrap();
                writeln!(output, "{}    <div class=\"macro-body\">", indent).unwrap();
                writeln!(output, "{}      <span class=\"label\">Body:</span>", indent).unwrap();
                self.visualize_expr(body, output, depth + 2);
                writeln!(output, "{}    </div>", indent).unwrap();
                writeln!(output, "{}  </div>", indent).unwrap();
                writeln!(output, "{}</div>", indent).unwrap();
            }
            LispExpr::MacroCall { name, args } => {
                writeln!(output, "{}<div class=\"ast-node ast-macro-call\">", indent).unwrap();
                writeln!(output, "{}  <div class=\"node-header\">", indent).unwrap();
                writeln!(output, "{}    <span class=\"node-type\">MacroCall</span>", indent).unwrap();
                writeln!(output, "{}    <span class=\"node-value\">{}</span>", indent, escape_html(name)).unwrap();
                writeln!(output, "{}    <span class=\"toggle\">▼</span>", indent).unwrap();
                writeln!(output, "{}  </div>", indent).unwrap();
                writeln!(output, "{}  <div class=\"node-children\">", indent).unwrap();
                for arg in args {
                    self.visualize_expr(arg, output, depth + 1);
                }
                writeln!(output, "{}  </div>", indent).unwrap();
                writeln!(output, "{}</div>", indent).unwrap();
            }
            LispExpr::Quote(inner) => {
                writeln!(output, "{}<div class=\"ast-node ast-quote\">", indent).unwrap();
                writeln!(output, "{}  <div class=\"node-header\">", indent).unwrap();
                writeln!(output, "{}    <span class=\"node-type\">Quote</span>", indent).unwrap();
                writeln!(output, "{}    <span class=\"toggle\">▼</span>", indent).unwrap();
                writeln!(output, "{}  </div>", indent).unwrap();
                writeln!(output, "{}  <div class=\"node-children\">", indent).unwrap();
                self.visualize_expr(inner, output, depth + 1);
                writeln!(output, "{}  </div>", indent).unwrap();
                writeln!(output, "{}</div>", indent).unwrap();
            }
            LispExpr::Quasiquote(inner) => {
                writeln!(output, "{}<div class=\"ast-node ast-quasiquote\">", indent).unwrap();
                writeln!(output, "{}  <div class=\"node-header\">", indent).unwrap();
                writeln!(output, "{}    <span class=\"node-type\">Quasiquote</span>", indent).unwrap();
                writeln!(output, "{}    <span class=\"toggle\">▼</span>", indent).unwrap();
                writeln!(output, "{}  </div>", indent).unwrap();
                writeln!(output, "{}  <div class=\"node-children\">", indent).unwrap();
                self.visualize_expr(inner, output, depth + 1);
                writeln!(output, "{}  </div>", indent).unwrap();
                writeln!(output, "{}</div>", indent).unwrap();
            }
            LispExpr::Unquote(inner) => {
                writeln!(output, "{}<div class=\"ast-node ast-unquote\">", indent).unwrap();
                writeln!(output, "{}  <div class=\"node-header\">", indent).unwrap();
                writeln!(output, "{}    <span class=\"node-type\">Unquote</span>", indent).unwrap();
                writeln!(output, "{}    <span class=\"toggle\">▼</span>", indent).unwrap();
                writeln!(output, "{}  </div>", indent).unwrap();
                writeln!(output, "{}  <div class=\"node-children\">", indent).unwrap();
                self.visualize_expr(inner, output, depth + 1);
                writeln!(output, "{}  </div>", indent).unwrap();
                writeln!(output, "{}</div>", indent).unwrap();
            }
            LispExpr::Splice(inner) => {
                writeln!(output, "{}<div class=\"ast-node ast-splice\">", indent).unwrap();
                writeln!(output, "{}  <div class=\"node-header\">", indent).unwrap();
                writeln!(output, "{}    <span class=\"node-type\">Splice</span>", indent).unwrap();
                writeln!(output, "{}    <span class=\"toggle\">▼</span>", indent).unwrap();
                writeln!(output, "{}  </div>", indent).unwrap();
                writeln!(output, "{}  <div class=\"node-children\">", indent).unwrap();
                self.visualize_expr(inner, output, depth + 1);
                writeln!(output, "{}  </div>", indent).unwrap();
                writeln!(output, "{}</div>", indent).unwrap();
            }
            LispExpr::Gensym(name) => {
                writeln!(output, "{}<div class=\"ast-node ast-gensym\">", indent).unwrap();
                writeln!(output, "{}  <span class=\"node-type\">Gensym</span>", indent).unwrap();
                writeln!(output, "{}  <span class=\"node-value\">{}</span>", indent, escape_html(name)).unwrap();
                writeln!(output, "{}</div>", indent).unwrap();
            }
        }
    }
}

impl Default for HtmlVisualizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Escape special characters for DOT format
fn escape_dot(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Escape special characters for HTML
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dot_visualizer_number() {
        let mut viz = DotVisualizer::new();
        let exprs = vec![LispExpr::Number(42.0)];
        let output = viz.visualize(&exprs);

        assert!(output.contains("digraph AST"));
        assert!(output.contains("42"));
        assert!(output.contains("lightblue"));
    }

    #[test]
    fn test_dot_visualizer_symbol() {
        let mut viz = DotVisualizer::new();
        let exprs = vec![LispExpr::Symbol("+".to_string())];
        let output = viz.visualize(&exprs);

        assert!(output.contains("+"));
        assert!(output.contains("lightgreen"));
    }

    #[test]
    fn test_dot_visualizer_list() {
        let mut viz = DotVisualizer::new();
        let exprs = vec![LispExpr::List(vec![
            LispExpr::Symbol("+".to_string()),
            LispExpr::Number(1.0),
            LispExpr::Number(2.0),
        ])];
        let output = viz.visualize(&exprs);

        assert!(output.contains("List"));
        assert!(output.contains("+"));
        assert!(output.contains("1"));
        assert!(output.contains("2"));
    }

    #[test]
    fn test_html_visualizer_number() {
        let viz = HtmlVisualizer::new();
        let exprs = vec![LispExpr::Number(42.0)];
        let output = viz.visualize(&exprs);

        assert!(output.contains("<!DOCTYPE html>"));
        assert!(output.contains("ast-number"));
        assert!(output.contains("42"));
    }

    #[test]
    fn test_html_visualizer_list() {
        let viz = HtmlVisualizer::new();
        let exprs = vec![LispExpr::List(vec![
            LispExpr::Symbol("+".to_string()),
            LispExpr::Number(1.0),
        ])];
        let output = viz.visualize(&exprs);

        assert!(output.contains("ast-list"));
        assert!(output.contains("ast-symbol"));
        assert!(output.contains("+"));
    }

    #[test]
    fn test_escape_dot() {
        assert_eq!(escape_dot("hello\"world"), "hello\\\"world");
        assert_eq!(escape_dot("line1\nline2"), "line1\\nline2");
    }

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("<div>"), "&lt;div&gt;");
        assert_eq!(escape_html("a & b"), "a &amp; b");
    }

    #[test]
    fn test_dot_visualizer_macro() {
        let mut viz = DotVisualizer::new();
        let exprs = vec![LispExpr::Macro {
            name: "double".to_string(),
            parameters: vec!["x".to_string()],
            body: Box::new(LispExpr::List(vec![
                LispExpr::Symbol("*".to_string()),
                LispExpr::Symbol("x".to_string()),
                LispExpr::Number(2.0),
            ])),
        }];
        let output = viz.visualize(&exprs);

        assert!(output.contains("Macro: double"));
        assert!(output.contains("Parameters"));
        assert!(output.contains("body"));
    }

    #[test]
    fn test_html_visualizer_macro() {
        let viz = HtmlVisualizer::new();
        let exprs = vec![LispExpr::Macro {
            name: "double".to_string(),
            parameters: vec!["x".to_string()],
            body: Box::new(LispExpr::Number(2.0)),
        }];
        let output = viz.visualize(&exprs);

        assert!(output.contains("ast-macro"));
        assert!(output.contains("double"));
        assert!(output.contains("Parameters"));
    }
}
