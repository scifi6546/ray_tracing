use generational_arena::{Arena, Index as ArenaIndex};
use std::{
    cmp::PartialEq,
    collections::{HashMap, HashSet},
};
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct RenderPassID {
    index: ArenaIndex,
}
trait RenderPass {
    type Base;
    type RenderPassOutputMarker: PartialEq + Clone;

    type RenderPassOutput;
    fn get_dependencies(&self) -> Vec<Self::RenderPassOutputMarker>;
    fn get_output(&self) -> Vec<Self::RenderPassOutputMarker>;
    fn process(
        &mut self,
        base: &mut Self::Base,
        input: Vec<&Self::RenderPassOutput>,
    ) -> Vec<Self::RenderPassOutput>;
}
struct RenderPassItem<T: RenderPass> {
    item: T,
    dependencies: Vec<RenderPassOutputMarker<T::RenderPassOutputMarker>>,
}

struct RenderPassOutputMarker<T: Clone> {
    ty: T,
    /// renderpass that creates the output
    parent_pass: RenderPassID,
}
struct RenderGraph<T: RenderPass> {
    graph_items: Arena<RenderPassItem<T>>,
    output_pass: Option<RenderPassID>,
}

impl<T: RenderPass> RenderGraph<T> {
    pub fn new() -> Self {
        Self {
            graph_items: Arena::new(),
            output_pass: None,
        }
    }
    /// Dont have to do cycle checking as items can only depend on already existing passes
    pub fn insert_pass(
        &mut self,
        pass: T,
        dependencies: Vec<RenderPassOutputMarker<T::RenderPassOutputMarker>>,
    ) -> (
        RenderPassID,
        Vec<RenderPassOutputMarker<T::RenderPassOutputMarker>>,
    ) {
        let mut pass_output = pass.get_output();
        for dep in dependencies.iter() {
            if let Some(src_pass) = self.graph_items.get(dep.parent_pass.index) {
                let mut src_pass_output = src_pass.item.get_output();

                let num_found = src_pass_output
                    .drain(..)
                    .filter(|o| o.clone() == dep.ty.clone())
                    .count();
                if num_found != 1 {
                    panic!("dependency not found")
                }
            } else {
                panic!("invalid parent")
            }
        }
        let index = RenderPassID {
            index: self.graph_items.insert(RenderPassItem {
                item: pass,
                dependencies,
            }),
        };

        (
            index,
            pass_output
                .drain(..)
                .map(|item| RenderPassOutputMarker {
                    ty: item,
                    parent_pass: index,
                })
                .collect(),
        )
    }
    pub fn insert_output_pass(
        &mut self,
        pass: T,
        dependencies: Vec<RenderPassOutputMarker<T::RenderPassOutputMarker>>,
    ) -> RenderPassID {
        let (pass_id, _) = self.insert_pass(pass, dependencies);
        self.output_pass = Some(pass_id);
        pass_id
    }
    pub fn run_graph(&mut self, base: &mut T::Base) {
        fn recurse<T: RenderPass>(
            graph: &RenderGraph<T>,
            current_pass: RenderPassID,
            run_list: &mut Vec<RenderPassID>,
        ) {
            if !run_list.contains(&current_pass) {
                run_list.push(current_pass);
                for dep in graph.graph_items[current_pass.index].dependencies.iter() {
                    recurse(graph, dep.parent_pass, run_list);
                }
            }
        }
        fn build_run_list<T: RenderPass>(graph: &RenderGraph<T>) -> Vec<RenderPassID> {
            if graph.output_pass.is_none() {
                return vec![];
            }
            let mut run_list = vec![graph.output_pass.unwrap()];
            let pass_deps = &graph.graph_items[run_list[0].index].dependencies;
            for dep in pass_deps.iter() {
                recurse(graph, dep.parent_pass, &mut run_list);
            }
            run_list.reverse();
            return run_list;
        }
        let run_list = build_run_list(self);

        let mut resources: HashMap<
            RenderPassID,
            Vec<(T::RenderPassOutputMarker, T::RenderPassOutput)>,
        > = HashMap::new();
        for pass_id in run_list.iter() {
            let dependencies = self.graph_items[pass_id.index]
                .dependencies
                .iter()
                .map(|dep| {
                    let dep_pass = dep.parent_pass;
                    let (pass_items, output) = resources
                        .get(&dep_pass)
                        .unwrap()
                        .iter()
                        .filter(|(marker, output)| marker.clone() == dep.ty)
                        .next()
                        .unwrap();
                    output
                })
                .collect();
            let mut output = self.graph_items[pass_id.index]
                .item
                .process(base, dependencies);
            let insert = self.graph_items[pass_id.index]
                .item
                .get_output()
                .drain(..)
                .zip(output.drain(..))
                .collect();
            resources.insert(*pass_id, insert);
        }
    }
}
#[cfg(test)]
mod test {
    use super::*;
    use std::{cell::RefCell, rc::Rc};
    struct TestPass {
        deps: Vec<String>,
        output: Vec<String>,
        name: String,
        mock_run: Rc<RefCell<Vec<String>>>,
    }
    impl RenderPass for TestPass {
        type Base = ();
        type RenderPassOutputMarker = String;
        type RenderPassOutput = ();

        fn get_dependencies(&self) -> Vec<Self::RenderPassOutputMarker> {
            self.deps.clone()
        }

        fn get_output(&self) -> Vec<Self::RenderPassOutputMarker> {
            self.output.clone()
        }

        fn process(
            &mut self,
            base: &mut Self::Base,
            input: Vec<&Self::RenderPassOutput>,
        ) -> Vec<Self::RenderPassOutput> {
            self.mock_run.borrow_mut().push(self.name.clone());
            self.output.iter().map(|_| ()).collect()
        }
    }
    #[test]
    fn build_rendergraph() {
        let _: RenderGraph<TestPass> = RenderGraph::new();
    }
    #[test]
    fn insert_one_pass() {
        let mock_run = Rc::new(RefCell::new(Vec::new()));
        let mut pass: RenderGraph<TestPass> = RenderGraph::new();
        let _ = pass.insert_pass(
            TestPass {
                deps: vec![],
                output: vec![],
                mock_run,
                name: "p0".to_string(),
            },
            vec![],
        );
    }
    #[test]
    fn insert_passes() {
        let mock_run = Rc::new(RefCell::new(Vec::new()));
        let mut graph: RenderGraph<TestPass> = RenderGraph::new();
        let (pass, pass_outputs) = graph.insert_pass(
            TestPass {
                deps: vec![],
                output: vec!["output".to_string()],
                mock_run: mock_run.clone(),
                name: "p0".to_string(),
            },
            vec![],
        );
        graph.insert_pass(
            TestPass {
                deps: vec!["output".to_string()],
                output: vec![],
                mock_run,
                name: "p0".to_string(),
            },
            pass_outputs,
        );
    }
    #[test]
    fn process() {
        let mut graph: RenderGraph<TestPass> = RenderGraph::new();
        let mock_run = Rc::new(RefCell::new(Vec::new()));
        let (pass, pass_outputs) = graph.insert_pass(
            TestPass {
                deps: vec![],
                output: vec!["output".to_string()],
                mock_run: mock_run.clone(),
                name: "p0".to_string(),
            },
            vec![],
        );
        graph.insert_output_pass(
            TestPass {
                deps: vec!["prev output".to_string()],
                output: vec![],
                mock_run: mock_run.clone(),
                name: "p1".to_string(),
            },
            pass_outputs,
        );
        graph.run_graph(&mut ());
        let output: Vec<String> = vec![];
        for (run_output, ground_truth) in mock_run.borrow_mut().iter().zip(output.iter()) {
            assert_eq!(run_output, ground_truth)
        }
    }
}