use std::mem;

use comfy_table::{Cell, Row, Table, presets};
use oxidd::{BooleanFunction, bdd::BDDFunction};

use crate::{M, M_LENGTH, N};

pub fn all_interpretations(solution: BDDFunction, result_table_width: usize) -> Table {
    let mut res = Table::new();
    res.load_preset(presets::NOTHING);
    let mut current_row = Row::new();
    let mut added = 0;
    for_all_true_interpretations(solution, |interpretation| {
        current_row.add_cell(Cell::new(interpretation_to_table(interpretation)));
        added += 1;
        if added >= result_table_width {
            res.add_row(mem::replace(&mut current_row, Row::new()));
            added = 0;
        }
    });
    if added != 0 {
        res.add_row(current_row);
    }
    res
}

fn interpretation_to_table(interpretation: &[bool]) -> Table {
    let mut res = Table::new();
    res.load_preset(presets::ASCII_BORDERS_ONLY_CONDENSED);
    let mut object_properties = interpretation.chunks(M_LENGTH).map(|i| {
        i.iter().copied().fold(0, |mut acc, b| {
            acc <<= 1;
            if b {
                acc |= 1;
            }
            acc
        })
    });
    for object in 0..N {
        let mut row = Row::new();
        row.add_cell(
            Cell::new(format!("{object}:")).set_alignment(comfy_table::CellAlignment::Right),
        );
        (&mut object_properties)
            .take(M)
            .for_each(|property: usize| {
                row.add_cell(Cell::new(property));
            });
        res.add_row(row);
    }
    res.column_mut(0).unwrap().set_padding((1, 1));
    res.column_iter_mut().skip(1).for_each(|c| {
        c.set_padding((0, 1));
    });
    res
}

fn for_all_true_interpretations(mut fun: BDDFunction, mut action: impl FnMut(&[bool])) {
    if !fun.satisfiable() {
        return;
    }
    let mut stack = vec![];
    let mut interpretation = vec![];
    let mut len = 0;
    loop {
        if let Some((t, f)) = fun.cofactors() {
            match (t.satisfiable(), f.satisfiable()) {
                (true, true) => {
                    stack.push((t, len));
                    interpretation.push(false);
                    fun = f;
                }
                (true, false) => {
                    interpretation.push(true);
                    fun = t;
                }
                (false, true) => {
                    interpretation.push(false);
                    fun = f;
                }
                (false, false) => unreachable!(),
            }
            len += 1;
        } else {
            action(&interpretation);
            if let Some(saved) = stack.pop() {
                (fun, len) = saved;
                interpretation.truncate(len);
                interpretation.push(true);
                len += 1;
            } else {
                break;
            }
        }
    }
}
