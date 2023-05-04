use crate::{Type, Var};
use displaydoc::Display;
use petgraph::graph::{DiGraph, NodeIndex};

/// A node on the inference graph.
#[derive(Copy, Clone, Debug)]
pub enum Node<'hir, 'input> {
    // #[displaydoc("{0} ≡ {1}")]
    Equiv(Type<'hir, 'input>, Type<'hir, 'input>),
    // #[displaydoc("{0} ≢ {1}")]
    NotEquiv(Type<'hir, 'input>, Type<'hir, 'input>),
    // #[displaydoc("{var} := {definition}")]
    Binding {
        var: Var,
        definition: Type<'hir, 'input>,
    },
}

/// An edge on the inference graph i.e. the reason why a proof (node) leads to
/// a conclusion (another node).
#[derive(Display)]
pub enum Edge {
    #[displaydoc("lhs")]
    FunctionLhs,
    #[displaydoc("rhs")]
    FunctionRhs,
    #[displaydoc("output")]
    FunctionOutput,
    #[displaydoc("transitivity")]
    Transitivity,
    #[displaydoc("tuple_{0}")]
    Tuple(usize),
}

#[derive(Default)]
pub struct Equations<'hir, 'input> {
    pub graph: DiGraph<Node<'hir, 'input>, Edge>,
}

impl<'hir, 'input> Equations<'hir, 'input> {
    pub fn new() -> Self {
        Equations {
            graph: DiGraph::new(),
        }
    }

    pub fn add_rule(&mut self, rule: Node<'hir, 'input>) -> NodeIndex {
        self.graph.add_node(rule)
    }

    pub fn add_proof(&mut self, proof: NodeIndex, conclusion: NodeIndex, edge: Edge) {
        self.graph.add_edge(proof, conclusion, edge);
    }
}

// impl fmt::Display for Equations<'_> {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         fn get_edge_attributes(
//             graph: &DiGraph<Node<'_>, Edge>,
//             edge: EdgeReference<Edge>,
//         ) -> String {
//             if let Node::NotEquiv(_, _) = &graph[edge.source()] {
//                 "color = red".to_string()
//             } else {
//                 String::new()
//             }
//         }

//         fn get_node_attributes(
//             _graph: &DiGraph<Node<'_>, Edge>,
//             (_ix, rule): (NodeIndex, &Node<'_>),
//         ) -> String {
//             if let Node::NotEquiv(_, _) = rule {
//                 "color = red".to_string()
//             } else {
//                 String::new()
//             }
//         }

//         Dot::with_attr_getters(&self.graph, &[], &get_edge_attributes, &get_node_attributes).fmt(f)
//     }
// }
