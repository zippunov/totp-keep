use std::collections::HashMap;
use std::iter::repeat;

pub trait TableSymbols {
    fn print_top(&self) -> bool;
    fn print_head_bottom(&self) -> bool;
    fn print_mid(&self) -> bool;
    fn print_bottom(&self) -> bool;
    // table top
    fn top(&self) -> &'static str;
    fn top_mid(&self) -> &'static str;
    fn top_left(&self) -> &'static str;
    fn top_right(&self) -> &'static str;
    // table bottom
    fn bottom(&self) -> &'static str;
    fn bottom_mid(&self) -> &'static str;
    fn bottom_left(&self) -> &'static str;
    fn bottom_right(&self) -> &'static str;
    // table mid divider
    fn mid(&self) -> &'static str;
    fn mid_mid(&self) -> &'static str;
    fn mid_left(&self) -> &'static str;
    fn mid_right(&self) -> &'static str;
    // table data cells elements
    fn left(&self) -> &'static str;
    fn right(&self) -> &'static str;
    fn middle(&self) -> &'static str;
    // progress
    fn progress_left(&self) -> &'static str;
    fn progress_right(&self) -> &'static str;
    fn progress_middle(&self) -> &'static str;
}

pub struct UnicodeTableSymbols;

impl TableSymbols for UnicodeTableSymbols {
    fn print_top(&self) -> bool { true }
    fn print_head_bottom(&self) -> bool { true }
    fn print_mid(&self) -> bool  { true }
    fn print_bottom(&self) -> bool  { true }
    // table top
    fn top(&self) -> &'static str { "─" }
    fn top_mid(&self) -> &'static str { "┬" }
    fn top_left(&self) -> &'static str { "┌" }
    fn top_right(&self) -> &'static str { "┐" }
    // table bottom
    fn bottom(&self) -> &'static str { "─" }
    fn bottom_mid(&self) -> &'static str { "┴" }
    fn bottom_left(&self) -> &'static str { "└" }
    fn bottom_right(&self) -> &'static str { "┘" }
    // table mid divider
    fn mid(&self) -> &'static str { "─" }
    fn mid_mid(&self) -> &'static str { "┼" }
    fn mid_left(&self) -> &'static str { "├" }
    fn mid_right(&self) -> &'static str { "┤" }
    // table data cells elements
    fn left(&self) -> &'static str { "│" }
    fn right(&self) -> &'static str { "│" }
    fn middle(&self) -> &'static str { "│" }
    // progress
    fn progress_left(&self) -> &'static str {"│"}
    fn progress_middle(&self) -> &'static str {"░"}
    fn progress_right(&self) -> &'static str {"│"}

}

pub struct AsciiTableSymbols;

impl TableSymbols for AsciiTableSymbols {
    fn print_top(&self) -> bool { false }
    fn print_head_bottom(&self) -> bool { true }
    fn print_mid(&self) -> bool  { false }
    fn print_bottom(&self) -> bool  { false }
    // table top
    fn top(&self) -> &'static str { "" }
    fn top_mid(&self) -> &'static str { "" }
    fn top_left(&self) -> &'static str { "" }
    fn top_right(&self) -> &'static str { "" }
    // table bottom
    fn bottom(&self) -> &'static str { "" }
    fn bottom_mid(&self) -> &'static str { "" }
    fn bottom_left(&self) -> &'static str { "" }
    fn bottom_right(&self) -> &'static str { "" }
    // table mid divider
    fn mid(&self) -> &'static str { "-" }
    fn mid_mid(&self) -> &'static str { "+" }
    fn mid_left(&self) -> &'static str { "" }
    fn mid_right(&self) -> &'static str { "" }
    // table data cells elements
    fn left(&self) -> &'static str { "" }
    fn right(&self) -> &'static str { "" }
    fn middle(&self) -> &'static str { "|" }
    // progress
    fn progress_left(&self) -> &'static str {"["}
    fn progress_middle(&self) -> &'static str {"="}
    fn progress_right(&self) -> &'static str {"]"}

}

pub enum Alignment {
    Left,
    Right,
    Middle
}

pub struct RowFormat {
    pub padding: u8,
    pub alignment: Alignment,
    pub row_numbers: bool
}

impl RowFormat {
    pub fn default() -> Self {
        RowFormat{padding: 1, alignment: Alignment::Middle, row_numbers: false}
    }
}

pub struct StringTableFormatter<'a> {
    table_symbols: &'a TableSymbols,
    pub rows: Vec<RowFormat>
}

impl<'a> StringTableFormatter<'a>{
    pub fn new(symbols: &'a TableSymbols) -> Self {
        StringTableFormatter {table_symbols: symbols, rows: Vec::new()}
    }

    pub fn format(&self, table: &Table) -> String {
        let mut result = String::new();
        result.push_str(&self.format_header(table));
        result.push_str(&self.format_body(table));
        result.push_str(&self.format_footer(table));
        result
    }

    fn format_header(&self, table: &Table) -> String {
        let mut result = String::new();
        // Top header line
        if self.table_symbols.print_top() {
            result.push_str(
                &(self.format_div_row(
                    table,
                    self.table_symbols.top_left(),
                    self.table_symbols.top_mid(),
                    self.table_symbols.top_right(),
                    self.table_symbols.top()
                )[..]));
            result.push_str("\n");
        }
        // Header columns
        result.push_str(self.table_symbols.left());
        let max_column = table.fields.len();
        for column in 0..max_column {
            let name = &table.fields[column][..];
            let header = match table.headers.get(name) {
                Some(s) => &s[..],
                None => ""
            };
            result.push_str(&self.format_data_cell(column, table, header));
            if max_column - column > 1 {
                result.push_str(self.table_symbols.middle());
            }
        }
        result.push_str(self.table_symbols.right());
        result.push_str("\n");

        // Bottom header line
        if self.table_symbols.print_head_bottom() {
            result.push_str(
                &self.format_div_row(
                    table,
                    self.table_symbols.mid_left(),
                    self.table_symbols.mid_mid(),
                    self.table_symbols.mid_right(),
                    self.table_symbols.mid()
                ));
            result.push_str("\n");
        }
        result
    }

    fn format_body(&self, table: &Table) -> String {
        let mut result = String::new();
        let rows_num = table.rows.len();
        for row in 0..rows_num {
            result.push_str(&self.format_data_row(
                table,
                self.table_symbols.left(),
                self.table_symbols.middle(),
                self.table_symbols.right(),
                row
            ));
            result.push_str("\n");
            if rows_num - row > 1 && self.table_symbols.print_mid() {
                result.push_str(&self.format_div_row(
                    table,
                    self.table_symbols.mid_left(),
                    self.table_symbols.mid_mid(),
                    self.table_symbols.mid_right(),
                    self.table_symbols.mid()
                ));
                result.push_str("\n");
            }
        }
        result
    }

    fn format_footer(&self, table: &Table) -> String {
        let mut result = String::new();
        if self.table_symbols.print_bottom() {
            result.push_str(
                &self.format_div_row(
                    table,
                    self.table_symbols.bottom_left(),
                    self.table_symbols.bottom_mid(),
                    self.table_symbols.bottom_right(),
                    self.table_symbols.bottom()
                ));
            result.push_str("\n");
        }
        result
    }

    fn format_div_row(&self, table: &Table, left: &'static str, middle: &'static str,
                      right: &'static str, fill_char: &'static str) -> String {
        let mut line = String::from(left);
        let max_index = table.fields.len();
        for index in 0..max_index {
            line.push_str(&self.format_div_cell(index, table, fill_char));
            if max_index - index > 1 {
                line.push_str(middle);
            }
        }
        line.push_str(right);
        line
    }

    fn format_div_cell(&self, index: usize, table: &Table, symbol: &'static str) -> String {
        let length = self.get_field_formatted_len(index, table);
        let mut line = String::with_capacity(length);
        repeat(symbol).take(length).for_each(|ch| line.push_str(ch));
        line
    }

    fn format_data_row(&self, table: &Table, left: &'static str, middle: &'static str,
                       right: &'static str, row: usize) -> String {
        let mut line = String::from(left);
        let max_column = table.fields.len();
        for column in 0..max_column {
            let cell_data = match table.get_cell_data(row, column) {
                Some(s) => &s[..],
                None => ""
            };
            line.push_str(&self.format_data_cell(column, table, cell_data));
            if max_column - column > 1 {
                line.push_str(middle);
            }
        }
        line.push_str(right);
        line
    }

    fn format_data_cell(&self, index: usize, table: &Table, data: &str) -> String {
        let padding = self.rows[index].padding as usize;
        let field_len = self.get_field_data_max_len(index, table);
        let p = format!("{p:^padding$}", p="", padding=padding);
        match self.rows[index].alignment {
            ref Left => format!("{p}{d:<w$}{p}", p=p, w=field_len, d=data),
            ref Middle => format!("{p}{d:^w$}{p}", p=p, w=field_len, d=data),
            ref Right => format!("{p}{d:>w$}{p}", p=p, w=field_len, d=data)
        }
    }

    fn get_field_data_max_len(&self, index: usize, table: &Table) -> usize {
        let field_name = &table.fields[index][..];
        match table.max_len.get(field_name) {
            Some(&l) => l,
            None => 0usize
        }
    }

    fn get_field_formatted_len(&self, index: usize, table: &Table) -> usize {
        let padding = self.rows[index].padding as usize;
        let field_len = self.get_field_data_max_len(index, table);
        padding * 2 + field_len
    }
}

type Row = HashMap<String, String>;

#[derive(Debug)]
pub struct Table {
    fields: Vec<String>,
    headers: Row,
    rows: Vec<Row>,
    max_len: HashMap<String, usize>,
}

impl  Table {
    pub fn new() -> Self {
        Table {
            fields: Vec::new(),
            headers: Row::new(),
            rows: Vec::new(),
            max_len: HashMap::new(),
        }
    }

    pub fn add_field(&mut self, name: &str) {
        self.fields.push(name.to_string());
    }

    pub fn append_header(&mut self, name: &str, value: &str) {
        self.headers.insert(name.to_string(), value.to_string());
        self.update_max_len(name, value.len() as usize);
    }

    pub fn get_cell_data(&self, row: usize, column: usize) -> Option<&String> {
        let field_name = &self.fields[column][..];
        self.rows[row].get(field_name)
    }

    pub fn add_row(&mut self,  names: &[&str], values: &[&str]) {
        let mut row = Row::new();
        (0..names.len()).for_each(|index| {
            let k = names[index as usize];
            let v = values[index as usize];
            row.insert(k.to_string(), v.to_string());
            self.update_max_len(k, v.len() as usize);
        });
        self.rows.push(row);
    }

    fn update_max_len(&mut self, name: &str, len: usize) {
        let ms = match self.max_len.get(name) {
            Some(&number) => number,
            _ => 0
        };
        if len > ms {
            self.max_len.insert(name.to_string(), len);
        }
    }
}
