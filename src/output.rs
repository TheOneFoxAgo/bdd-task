use std::mem;

use bitvec::vec::BitVec;
use comfy_table::{Cell, Row, Table, presets};
use oxidd::{BooleanFunction, bdd::BDDFunction};

use crate::{M, M_LENGTH, N, RESULT_TABLE_WIDTH};

pub fn all_interpretations(solution: BDDFunction) -> Table {
    let mut res = Table::new();
    res.load_preset(presets::NOTHING);
    let mut current_row = Row::new();
    let mut idx = 0;
    for_all_true_interpretations(solution, BitVec::new(), &mut |interpretation| {
        if idx < RESULT_TABLE_WIDTH {
            current_row.add_cell(Cell::new(interpretation_to_table(interpretation)));
            idx += 1;
        } else {
            res.add_row(mem::replace(&mut current_row, Row::new()));
            idx = 0;
        }
    });
    if idx != 0 {
        res.add_row(current_row);
    }
    res
}

fn interpretation_to_table(interpretation: BitVec) -> Table {
    let mut res = Table::new();
    res.load_preset(presets::ASCII_BORDERS_ONLY_CONDENSED);
    let mut object_properties = interpretation.chunks(M_LENGTH).map(|i| {
        i.iter().by_vals().fold(0, |mut acc, b| {
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

fn for_all_true_interpretations(
    solution: BDDFunction,
    mut accumulator: BitVec,
    action: &mut impl FnMut(BitVec),
) {
    if let Some((t, f)) = solution.cofactors() {
        match (t.satisfiable(), f.satisfiable()) {
            (true, true) => {
                let mut another = accumulator.clone();
                accumulator.push(true);
                another.push(false);
                for_all_true_interpretations(t, accumulator, action);
                for_all_true_interpretations(f, another, action);
            }
            (true, false) => {
                accumulator.push(true);
                for_all_true_interpretations(t, accumulator, action);
            }
            (false, true) => {
                accumulator.push(false);
                for_all_true_interpretations(f, accumulator, action);
            }
            (false, false) => (),
        }
    } else {
        action(accumulator)
    }
}
