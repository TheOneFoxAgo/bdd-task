mod conditions;
mod config;
mod neighbours;
mod output;
use std::io::Read;

use anyhow::{Context, Result, bail};
use oxidd::{
    BooleanFunction, Manager, ManagerRef,
    bdd::{BDDFunction, BDDManagerRef, new_manager},
    util::{AllocResult, num::F64},
};

use crate::config::Config;

pub const N: usize = 9;
pub const M: usize = 4;
pub const M_LENGTH: usize = N.ilog2() as usize + if N.is_power_of_two() { 0 } else { 1 };

const INNER_NODES: usize = 1024 * 1024 * 64;
const CACHE: usize = 1024 * 32;
const THREADS: u32 = 16;

const VARIABLES_AMOUNT: u32 = (M * M_LENGTH * N) as u32;

const RESULT_TABLE_WIDTH: usize = 5;

fn main() -> Result<()> {
    let manager_ref = new_manager(INNER_NODES, CACHE, THREADS);
    let variables: Vec<BDDFunction> = manager_ref.with_manager_exclusive(|manager| {
        AllocResult::from_iter(
            manager
                .add_vars(VARIABLES_AMOUNT)
                .map(|i| BDDFunction::var(manager, i)),
        )
    })?;
    let config = read_config(&std::env::args().nth(1).context("Config missing")?)?;
    println!("Приступаем к решению");
    println!("Без склейки");
    let mut solution_without_wrap =
        solve_einstein(manager_ref.clone(), &variables, &config, false)?;

    // Печатаем решения, если требуется
    if config.print_solutions {
        println!(
            "Все решения:\n{}",
            output::all_interpretations(solution_without_wrap.clone())
        );
    } else {
        // Дальше решение не пригодится. Стираем его.
        manager_ref.with_manager_shared(|m| {
            solution_without_wrap = BDDFunction::f(m);
            m.gc();
        })
    }

    println!("\nСо склейкой");
    let solution_with_wrap = solve_einstein(manager_ref.clone(), &variables, &config, true)?;

    // Печатаем новые решения, которые добавились в резульате склейки, если требуется
    if config.print_solutions {
        // "Вычитаем" решения без склейки, чтобы получить только те,
        // в которых склейка есть
        println!(
            "Все новые решения:\n{}",
            output::all_interpretations(
                solution_without_wrap
                    .not_owned()?
                    .and(&solution_with_wrap)?,
            )
        );
    }
    Ok(())
}
fn read_config(filename: &str) -> Result<Config> {
    let mut buf: Vec<u8> = vec![];
    std::io::BufReader::new(std::fs::File::open(filename)?).read_to_end(&mut buf)?;
    let result: Config = toml::from_slice(&buf)?;
    Ok(result)
}

fn solve_einstein(
    manager: BDDManagerRef,
    variables: &[BDDFunction],
    config: &Config,
    wrapping: bool,
) -> Result<BDDFunction> {
    let mut b = conditions::ConditionBuilder::new(manager.clone(), variables, wrapping)?;
    for condition in config.order.iter() {
        match condition {
            1 => {
                for c in config.first.iter() {
                    b.add_first_type_condition(c.0, c.1, c.2)?;
                }
                println!("Условия первого типа учтены");
            }
            2 => {
                for c in config.second.iter() {
                    b.add_second_type_condition(c.0, c.1, c.2, c.3)?;
                }
                println!("Условия второго типа учтены");
            }
            3 => {
                for c in config.third.iter() {
                    b.add_third_type_condition(c.0, c.1, c.2, c.3, c.4)?;
                }
                println!("Условия третьего типа учтены");
            }
            4 => {
                for c in config.forth.iter() {
                    b.add_forth_type_condition(c.0, c.1, c.2, c.3)?;
                }
                println!("Условия четвёртого типа учтены");
            }
            5 => {
                b.add_fifth_type_condition()?;
                println!("Условия пятого типа учтены");
            }
            6 => {
                b.add_sixth_type_condition()?;
                println!("Условия шестого типа учтены");
            }
            _ => {
                bail!("incorrect number in order")
            }
        }
    }
    let model = b.model();
    let solutions_amount = model
        .sat_count::<F64, std::hash::RandomState>(VARIABLES_AMOUNT, &mut Default::default())
        .0;
    println!("Количество решений: {solutions_amount}");

    Ok(model)
}
