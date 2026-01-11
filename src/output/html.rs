//! HTML report output

use std::io::Write;
use std::path::Path;

use anyhow::Result;

use crate::diff::DiffResult;
use crate::model::Table;

use super::OutputFormatter;

/// HTML report output
pub struct HtmlOutput;

impl HtmlOutput {
    pub fn new() -> Self {
        Self
    }
}

impl Default for HtmlOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputFormatter for HtmlOutput {
    fn render(
        &self,
        diff: &DiffResult,
        old_table: &Table,
        new_table: &Table,
        old_path: &Path,
        new_path: &Path,
        writer: &mut dyn Write,
    ) -> Result<()> {
        // HTML header
        writeln!(writer, "<!DOCTYPE html>")?;
        writeln!(writer, "<html lang=\"en\">")?;
        writeln!(writer, "<head>")?;
        writeln!(writer, "  <meta charset=\"UTF-8\">")?;
        writeln!(writer, "  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">")?;
        writeln!(writer, "  <title>datadiff: {} → {}</title>", 
            html_escape(old_path.display().to_string()),
            html_escape(new_path.display().to_string())
        )?;
        writeln!(writer, "  <style>")?;
        writeln!(writer, "{}", CSS_STYLES)?;
        writeln!(writer, "  </style>")?;
        writeln!(writer, "</head>")?;
        writeln!(writer, "<body>")?;

        // Header
        writeln!(writer, "  <div class=\"header\">")?;
        writeln!(writer, "    <h1>datadiff</h1>")?;
        writeln!(writer, "    <p class=\"files\">{} → {}</p>",
            html_escape(old_path.display().to_string()),
            html_escape(new_path.display().to_string())
        )?;
        writeln!(writer, "  </div>")?;

        // Summary
        writeln!(writer, "  <div class=\"summary\">")?;
        writeln!(writer, "    <div class=\"stat added\"><span class=\"num\">+{}</span><span class=\"label\">added</span></div>",
            diff.stats.rows_added)?;
        writeln!(writer, "    <div class=\"stat removed\"><span class=\"num\">-{}</span><span class=\"label\">removed</span></div>",
            diff.stats.rows_removed)?;
        writeln!(writer, "    <div class=\"stat modified\"><span class=\"num\">~{}</span><span class=\"label\">modified</span></div>",
            diff.stats.rows_modified)?;
        writeln!(writer, "    <div class=\"stat total\"><span class=\"num\">{} → {}</span><span class=\"label\">rows</span></div>",
            diff.stats.old_row_count, diff.stats.new_row_count)?;
        writeln!(writer, "  </div>")?;

        // Schema changes
        if !diff.schema_changes.is_empty() {
            writeln!(writer, "  <div class=\"section\">")?;
            writeln!(writer, "    <h2>Schema Changes</h2>")?;
            writeln!(writer, "    <ul>")?;
            for change in &diff.schema_changes {
                writeln!(writer, "      <li>{}</li>", html_escape(change.to_string()))?;
            }
            writeln!(writer, "    </ul>")?;
            writeln!(writer, "  </div>")?;
        }

        // Added rows
        let added: Vec<_> = diff.added_rows().collect();
        if !added.is_empty() {
            writeln!(writer, "  <div class=\"section\">")?;
            writeln!(writer, "    <h2>Added Rows</h2>")?;
            write_rows_table(writer, &added, new_table, "added")?;
            writeln!(writer, "  </div>")?;
        }

        // Removed rows
        let removed: Vec<_> = diff.removed_rows().collect();
        if !removed.is_empty() {
            writeln!(writer, "  <div class=\"section\">")?;
            writeln!(writer, "    <h2>Removed Rows</h2>")?;
            write_rows_table(writer, &removed, old_table, "removed")?;
            writeln!(writer, "  </div>")?;
        }

        // Modified rows
        let modified: Vec<_> = diff.modified_rows().collect();
        if !modified.is_empty() {
            writeln!(writer, "  <div class=\"section\">")?;
            writeln!(writer, "    <h2>Modified Rows</h2>")?;
            for (old_row, _new_row, changes) in modified {
                writeln!(writer, "    <div class=\"modified-row\">")?;
                writeln!(writer, "      <h3>{}</h3>", html_escape(&old_row.key))?;
                writeln!(writer, "      <table class=\"changes\">")?;
                writeln!(writer, "        <tr><th>Column</th><th>Old Value</th><th>New Value</th></tr>")?;
                for change in changes {
                    writeln!(writer, "        <tr>")?;
                    writeln!(writer, "          <td>{}</td>", html_escape(&change.column))?;
                    writeln!(writer, "          <td class=\"old\">{}</td>", html_escape(change.old_value.display()))?;
                    writeln!(writer, "          <td class=\"new\">{}</td>", html_escape(change.new_value.display()))?;
                    writeln!(writer, "        </tr>")?;
                }
                writeln!(writer, "      </table>")?;
                writeln!(writer, "    </div>")?;
            }
            writeln!(writer, "  </div>")?;
        }

        // Footer
        writeln!(writer, "  <div class=\"footer\">")?;
        writeln!(writer, "    <p>Generated by <a href=\"https://github.com/example/datadiff\">datadiff</a></p>")?;
        writeln!(writer, "  </div>")?;

        writeln!(writer, "</body>")?;
        writeln!(writer, "</html>")?;

        Ok(())
    }
}

fn write_rows_table(
    writer: &mut dyn Write,
    rows: &[&crate::model::Row],
    table: &Table,
    class: &str,
) -> Result<()> {
    writeln!(writer, "    <table class=\"{}\">", class)?;
    
    // Header
    writeln!(writer, "      <tr>")?;
    for col in &table.columns {
        writeln!(writer, "        <th>{}</th>", html_escape(&col.name))?;
    }
    writeln!(writer, "      </tr>")?;
    
    // Rows
    for row in rows {
        writeln!(writer, "      <tr>")?;
        for cell in &row.cells {
            writeln!(writer, "        <td>{}</td>", html_escape(cell.display()))?;
        }
        writeln!(writer, "      </tr>")?;
    }
    
    writeln!(writer, "    </table>")?;
    Ok(())
}

fn html_escape(s: impl AsRef<str>) -> String {
    s.as_ref()
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

const CSS_STYLES: &str = r#"
    :root {
      --bg: #1a1b26;
      --fg: #a9b1d6;
      --accent: #7aa2f7;
      --green: #9ece6a;
      --red: #f7768e;
      --yellow: #e0af68;
      --border: #414868;
    }
    
    * { box-sizing: border-box; margin: 0; padding: 0; }
    
    body {
      font-family: 'JetBrains Mono', 'Fira Code', monospace;
      background: var(--bg);
      color: var(--fg);
      padding: 2rem;
      line-height: 1.6;
    }
    
    .header {
      border-bottom: 2px solid var(--border);
      padding-bottom: 1rem;
      margin-bottom: 2rem;
    }
    
    .header h1 {
      color: var(--accent);
      font-size: 2rem;
      font-weight: 600;
    }
    
    .header .files {
      color: var(--fg);
      opacity: 0.8;
      margin-top: 0.5rem;
    }
    
    .summary {
      display: flex;
      gap: 2rem;
      margin-bottom: 2rem;
    }
    
    .stat {
      display: flex;
      flex-direction: column;
      padding: 1rem;
      border-radius: 8px;
      background: rgba(255,255,255,0.05);
    }
    
    .stat .num {
      font-size: 1.5rem;
      font-weight: 600;
    }
    
    .stat.added .num { color: var(--green); }
    .stat.removed .num { color: var(--red); }
    .stat.modified .num { color: var(--yellow); }
    
    .section {
      margin-bottom: 2rem;
    }
    
    .section h2 {
      color: var(--accent);
      font-size: 1.25rem;
      margin-bottom: 1rem;
      padding-bottom: 0.5rem;
      border-bottom: 1px solid var(--border);
    }
    
    table {
      width: 100%;
      border-collapse: collapse;
      margin-bottom: 1rem;
    }
    
    th, td {
      text-align: left;
      padding: 0.75rem;
      border: 1px solid var(--border);
    }
    
    th {
      background: rgba(255,255,255,0.05);
      font-weight: 600;
    }
    
    table.added tr:not(:first-child) {
      background: rgba(158, 206, 106, 0.1);
    }
    
    table.removed tr:not(:first-child) {
      background: rgba(247, 118, 142, 0.1);
    }
    
    .modified-row {
      margin-bottom: 1.5rem;
      padding: 1rem;
      background: rgba(255,255,255,0.02);
      border-radius: 8px;
    }
    
    .modified-row h3 {
      color: var(--yellow);
      margin-bottom: 0.5rem;
    }
    
    .changes td.old {
      background: rgba(247, 118, 142, 0.15);
      color: var(--red);
    }
    
    .changes td.new {
      background: rgba(158, 206, 106, 0.15);
      color: var(--green);
    }
    
    .footer {
      margin-top: 3rem;
      padding-top: 1rem;
      border-top: 1px solid var(--border);
      opacity: 0.6;
      font-size: 0.875rem;
    }
    
    .footer a {
      color: var(--accent);
      text-decoration: none;
    }
    
    ul {
      list-style: none;
      padding-left: 1rem;
    }
    
    ul li::before {
      content: "→";
      margin-right: 0.5rem;
      color: var(--accent);
    }
"#;
