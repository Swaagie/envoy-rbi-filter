use crate::parser::{Handle, NodeData};

use std::collections::VecDeque;
use std::io;

use html5ever::serialize::TraversalScope::{ChildrenOnly, IncludeNode};
use html5ever::serialize::{Serialize, Serializer, TraversalScope};
use html5ever::QualName;

enum SerializeOp {
    Open(Handle),
    Close(QualName),
}

pub struct SerializableHandle(Handle);

impl From<Handle> for SerializableHandle {
    fn from(h: Handle) -> SerializableHandle {
        SerializableHandle(h)
    }
}

impl Serialize for SerializableHandle {
    fn serialize<S>(&self, serializer: &mut S, traversal_scope: TraversalScope) -> io::Result<()>
    where
        S: Serializer,
    {
        let mut ops = VecDeque::new();
        match traversal_scope {
            IncludeNode => ops.push_back(SerializeOp::Open(self.0.clone())),
            ChildrenOnly(_) => ops.extend(
                self.0
                    .children
                    .borrow()
                    .iter()
                    .map(|h| SerializeOp::Open(h.clone())),
            ),
        }

        while let Some(op) = ops.pop_front() {
            match op {
                SerializeOp::Open(handle) => match handle.data {
                    NodeData::Element {
                        ref name,
                        ref attrs,
                        ..
                    } => {
                        serializer.start_elem(
                            name.clone(),
                            attrs.borrow().iter().map(|at| (&at.name, &at.value[..])),
                        )?;

                        ops.reserve(1 + handle.children.borrow().len());
                        ops.push_front(SerializeOp::Close(name.clone()));

                        for child in handle.children.borrow().iter().rev() {
                            ops.push_front(SerializeOp::Open(child.clone()));
                        }
                    }

                    NodeData::Doctype { ref name, .. } => serializer.write_doctype(&name)?,

                    NodeData::Text { ref contents } => serializer.write_text(&contents.borrow())?,

                    NodeData::Comment { ref contents } => serializer.write_comment(&contents)?,

                    NodeData::ProcessingInstruction {
                        ref target,
                        ref contents,
                    } => serializer.write_processing_instruction(target, contents)?,

                    NodeData::Document => panic!("Can't serialize Document node itself"),
                },

                SerializeOp::Close(name) => {
                    serializer.end_elem(name)?;
                }
            }
        }

        Ok(())
    }
}
