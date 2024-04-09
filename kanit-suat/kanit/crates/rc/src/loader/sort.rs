use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use kanit_common::error::{Result, StaticError, WithError};
use kanit_unit::UnitInfo;

#[derive(Clone, Debug)]
struct Edge<T> {
    from: Rc<RefCell<Node<T>>>,
    deleted: bool,
}

#[derive(Debug)]
struct Node<T> {
    data: T,
    edges: Vec<Rc<RefCell<Edge<T>>>>,
    idx: usize,
}

pub fn obtain_load_order(units: Vec<UnitInfo>) -> Result<Vec<Vec<UnitInfo>>> {
    let mut nodes = vec![];
    let mut map = HashMap::new();

    for (idx, unit) in units.iter().enumerate() {
        let node = Rc::new(RefCell::new(Node {
            data: unit.clone(),
            edges: vec![],
            idx,
        }));

        nodes.push(node.clone());
        map.insert(unit.name.clone(), node);
    }

    for node in nodes.iter() {
        let mut node_b = node.borrow_mut();
        let dependencies = node_b.data.dependencies.clone();

        for dep in dependencies.needs.iter() {
            if let Some(unit) = map.get(&dep.clone()) {
                let edge = Rc::new(RefCell::new(Edge {
                    from: node.clone(),
                    deleted: false,
                }));

                node_b.edges.push(edge.clone());
                unit.borrow_mut().edges.push(edge);
            } else {
                let dep = dep.to_string();
                Err(WithError::with(move || {
                    format!("failed to find needed dependency `{}`", dep)
                }))?;
            }
        }

        for dep in dependencies.wants.iter().chain(dependencies.after.iter()) {
            if let Some(unit) = map.get(dep) {
                let edge = Rc::new(RefCell::new(Edge {
                    from: node.clone(),
                    deleted: false,
                }));

                node_b.edges.push(edge.clone());
                unit.borrow_mut().edges.push(edge);
            }
        }

        for before in dependencies.before.iter() {
            if let Some(unit) = map.get(before) {
                let mut unit_b = unit.borrow_mut();

                let edge = Rc::new(RefCell::new(Edge {
                    from: unit.clone(),
                    deleted: false,
                }));

                node_b.edges.push(edge.clone());
                unit_b.edges.push(edge);
            }
        }
    }

    let mut order = vec![];

    while !nodes.is_empty() {
        let starting_amount = nodes.len();
        let nodes_c = nodes.clone();

        let mut without_incoming: Vec<_> = nodes_c
            .iter()
            .filter(|n| {
                !n.borrow().edges.iter().any(|e| {
                    let e_b = e.borrow();
                    !e_b.deleted && e_b.from.borrow().idx == n.borrow().idx
                })
            })
            .collect();

        let mut round = vec![];

        while let Some(node) = without_incoming.pop() {
            round.push(node.clone());

            for edge in node.borrow().edges.iter() {
                let mut edge_b = edge.borrow_mut();
                edge_b.deleted = true;
            }

            if let Some(pos) = nodes
                .iter()
                .position(|n| n.borrow().idx == node.borrow().idx)
            {
                nodes.remove(pos);
            }
        }

        order.push(round);

        if starting_amount == nodes.len() {
            Err(StaticError("cyclic dependency detected"))?;
        }
    }

    Ok(order
        .iter()
        .map(|r| r.iter().map(|n| n.borrow().data.clone()).collect())
        .collect())
}

// kinda gotta manually verify these since theres many valid dependency resolution schemes
#[cfg(test)]
mod tests {
    use async_trait::async_trait;

    use kanit_unit::{wrap_unit, Dependencies, RcUnit, Unit, UnitName};

    use super::*;

    struct NullUnit(&'static str, Dependencies);

    #[async_trait]
    impl Unit for NullUnit {
        fn name(&self) -> UnitName {
            UnitName::from(self.0)
        }

        fn dependencies(&self) -> Dependencies {
            self.1.clone()
        }

        async fn start(&mut self) -> Result<()> {
            Ok(())
        }
    }

    fn print_plan(units: Vec<Vec<UnitInfo>>) {
        for (i, order) in units.iter().enumerate() {
            println!("group {}", i);
            order.iter().for_each(|s| println!("|> {}", s.name));
        }
    }

    fn to_unit_info(units: Vec<RcUnit>) -> Vec<UnitInfo> {
        units.iter().map(UnitInfo::new).collect()
    }

    #[test]
    fn simple_generate_order() {
        let c = NullUnit("c", Dependencies::new());
        let d = NullUnit("d", Dependencies::new());
        let b = NullUnit("b", Dependencies::new().need(d.name()).clone());
        let a = NullUnit(
            "a",
            Dependencies::new().need(b.name()).need(c.name()).clone(),
        );

        let units = vec![wrap_unit(a), wrap_unit(b), wrap_unit(c), wrap_unit(d)];

        if let Ok(order) = obtain_load_order(to_unit_info(units)) {
            print_plan(order);
        }
    }

    #[test]
    fn complex_generate_order() {
        let f = NullUnit("2", Dependencies::new());
        let g = NullUnit("9", Dependencies::new());
        let h = NullUnit("10", Dependencies::new());
        let e = NullUnit("8", Dependencies::new().need(g.name()).clone());

        let c = NullUnit(
            "3",
            Dependencies::new().need(e.name()).need(h.name()).clone(),
        );
        let d = NullUnit(
            "11",
            Dependencies::new()
                .need(f.name())
                .need(g.name())
                .need(h.name())
                .clone(),
        );
        let b = NullUnit(
            "7",
            Dependencies::new().need(d.name()).need(e.name()).clone(),
        );

        let a = NullUnit("5", Dependencies::new().need(d.name()).clone());

        let units = vec![
            wrap_unit(a),
            wrap_unit(b),
            wrap_unit(c),
            wrap_unit(d),
            wrap_unit(e),
            wrap_unit(f),
            wrap_unit(g),
            wrap_unit(h),
        ];

        if let Ok(order) = obtain_load_order(to_unit_info(units)) {
            print_plan(order);
        }
    }

    #[test]
    fn cyclic_chain() {
        let a = NullUnit("a", Dependencies::new().need(UnitName::from("b")).clone());
        let b = NullUnit("b", Dependencies::new().need(UnitName::from("a")).clone());

        let units = vec![wrap_unit(a), wrap_unit(b)];

        assert!(obtain_load_order(to_unit_info(units)).is_err());
    }
}
