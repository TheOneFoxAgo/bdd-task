use std::array;

use super::neighbours;
use super::{M, M_LENGTH, N};
use oxidd::{
    BooleanFunction, Manager, ManagerRef,
    bdd::{BDDFunction, BDDManagerRef},
    util::AllocResult,
};

pub struct ConditionBuilder<'b> {
    manager: BDDManagerRef,
    model: BDDFunction,
    properties: [[[BDDFunction; N]; M]; N],
    variables: &'b [BDDFunction],
    wrapping: bool,
}

impl<'b> ConditionBuilder<'b> {
    pub fn new(
        manager: BDDManagerRef,
        variables: &'b [BDDFunction],
        wrapping: bool,
    ) -> AllocResult<Self> {
        let model = manager.with_manager_shared(|manager| BDDFunction::t(manager));
        let mut properties: [[[BDDFunction; N]; M]; N] =
            array::repeat(array::repeat(array::repeat(model.clone())));
        #[allow(clippy::needless_range_loop)]
        for position in 0..N {
            for kind in 0..M {
                for value in 0..N {
                    // Кодируем таблицу состояний свойств.
                    // Нам она понадобится в полном объёме в 5 типе.
                    Self::encode(
                        &mut properties[position][kind][value],
                        variables,
                        position,
                        kind,
                        value,
                    )?;
                }
            }
        }
        Ok(Self {
            model,
            properties,
            manager,
            variables,
            wrapping,
        })
    }

    pub fn model(self) -> BDDFunction {
        self.model
    }

    pub fn add_first_type_condition(
        &mut self,
        position: usize,
        kind: usize,
        value: usize,
    ) -> AllocResult<()> {
        self.model = self.model.and(&self.properties[position][kind][value])?;
        Ok(())
    }

    pub fn add_second_type_condition(
        &mut self,
        first_kind: usize,
        first_value: usize,
        second_kind: usize,
        second_value: usize,
    ) -> AllocResult<()> {
        for position in 0..N {
            let first = &self.properties[position][first_kind][first_value];
            let second = &self.properties[position][second_kind][second_value];
            self.model = self.model.and(&first.equiv(second)?)?;
        }
        Ok(())
    }

    pub fn add_third_type_condition(
        &mut self,
        origin_kind: usize,
        origin_value: usize,
        direction: neighbours::Direction,
        neighbour_kind: usize,
        neighbour_value: usize,
    ) -> AllocResult<()> {
        for position in 0..N {
            if !neighbours::validate(position, direction, self.wrapping) {
                self.model = self
                    .model
                    .and(&self.properties[position][neighbour_kind][neighbour_value].not()?)?;
            }
            match neighbours::neighbour(position, direction, self.wrapping) {
                Some(neighbour_position) => {
                    self.model = self.model.and(
                        &self.properties[position][origin_kind][origin_value].equiv(
                            &self.properties[neighbour_position][neighbour_kind][neighbour_value],
                        )?,
                    )?
                }
                None => {
                    self.model = self
                        .model
                        .and(&self.properties[position][origin_kind][origin_value].not()?)?
                }
            }
        }
        self.manager.with_manager_shared(|manager| manager.gc());
        Ok(())
    }
    pub fn add_forth_type_condition(
        &mut self,
        origin_kind: usize,
        origin_value: usize,
        neighbour_kind: usize,
        neighbour_value: usize,
    ) -> AllocResult<()> {
        use neighbours::Direction::{Left, Right};
        for position in 0..N {
            // Проверяем, могут ли соседи находится в данных местах. Запрещаем, только если оба не могут
            if !neighbours::validate(position, Left, self.wrapping)
                && !neighbours::validate(position, Right, self.wrapping)
            {
                self.model = self
                    .model
                    .and(&self.properties[position][neighbour_kind][neighbour_value].not()?)?;
            }
            let right_position = neighbours::neighbour(position, Right, self.wrapping);
            let left_position = neighbours::neighbour(position, Left, self.wrapping);
            let origin_property = &self.properties[position][origin_kind][origin_value];
            if let (None, None) = (right_position, left_position) {
                self.model = self.model.and(&origin_property.not()?)?;
            } else {
                let no_neighbour = self.manager.with_manager_shared(|m| BDDFunction::f(m));
                let right_neighbour;
                if let Some(right_position) = right_position {
                    right_neighbour =
                        &self.properties[right_position][neighbour_kind][neighbour_value];
                } else {
                    // Справа соседа нету
                    right_neighbour = &no_neighbour;
                }
                let left_neighbour;
                if let Some(left_position) = left_position {
                    left_neighbour =
                        &self.properties[left_position][neighbour_kind][neighbour_value];
                } else {
                    // Слева соседа нету
                    left_neighbour = &no_neighbour;
                }
                let right_condition = origin_property.equiv(right_neighbour)?;
                let left_condition = origin_property.equiv(left_neighbour)?;
                self.model = self.model.and(&right_condition.or(&left_condition)?)?;
            }
        }
        Ok(())
    }
    pub fn add_fifth_type_condition(&mut self) -> AllocResult<()> {
        for first_position in 0..(N - 1) {
            for second_position in (first_position + 1)..N {
                for kind in 0..M {
                    for value in 0..N {
                        let first = &self.properties[first_position][kind][value];
                        let second = &self.properties[second_position][kind][value];
                        self.model = self.model.and(&first.imp(&second.not()?)?)?;
                    }
                }
            }
        }
        Ok(())
    }
    pub fn add_sixth_type_condition(&mut self) -> AllocResult<()> {
        for position in 0..N {
            for kind in 0..M {
                for value in N..(1 << M_LENGTH) {
                    let mut incorrect_state =
                        self.manager.with_manager_shared(|m| BDDFunction::t(m));
                    Self::encode(&mut incorrect_state, self.variables, position, kind, value)?;
                    self.model = self.model.and(&incorrect_state.not()?)?;
                }
            }
        }
        Ok(())
    }
    /// Записывает в t конъюнкт, истинный только если
    /// Место, свойство и значение свойства имеют
    /// определённые значения.
    fn encode(
        t: &mut BDDFunction,
        variables: &[BDDFunction],
        position: usize,
        kind: usize,
        mut value: usize,
    ) -> AllocResult<()> {
        let variables = {
            let start = position * M * M_LENGTH + kind * M_LENGTH;
            let end = start + M_LENGTH;
            &variables[start..end]
        };
        for var in variables.iter().rev() {
            if value & 1 == 1 {
                *t = t.and(var)?;
            } else {
                *t = t.and(&var.not()?)?;
            }
            value >>= 1;
        }
        Ok(())
    }
}
