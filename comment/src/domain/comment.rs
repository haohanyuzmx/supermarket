use crate::repo::comment::Comment;
use anyhow::{anyhow, Result};
use async_recursion::async_recursion;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use serde_json::{Map, Number, Value};
use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub struct CommentNode {
    pub comment_id: u64,
    pub comment: String,
    pub item_id: u64,
    pub user: String,
    pub user_id: u64,
    pub parents: RefCell<Weak<CommentNode>>,
    pub children: RefCell<Vec<Rc<CommentNode>>>,
}

unsafe impl Send for CommentNode {}

impl From<Comment> for CommentNode {
    fn from(value: Comment) -> Self {
        Self {
            comment_id: value.id.unwrap(),
            comment: value.comment,
            item_id: value.item_id,
            user: value.user_name,
            user_id: value.user_id,
            parents: RefCell::new(Weak::new()),
            children: RefCell::new(vec![]),
        }
    }
}

impl CommentNode {
    fn new(comment_id: u64, comment: String, item_id: u64, user: String, user_id: u64) -> Self {
        Self {
            comment_id,
            comment,
            item_id,
            user,
            user_id,
            parents: RefCell::new(Weak::new()),
            children: RefCell::new(vec![]),
        }
    }

    fn add_child(this: Rc<CommentNode>, child: Rc<CommentNode>) {
        *child.parents.borrow_mut() = Rc::downgrade(&this);
        this.children.borrow_mut().push(child.clone())
    }

    #[async_recursion::async_recursion(?Send)]
    async fn find_root(father_id: u64, this: Rc<CommentNode>) -> Rc<CommentNode> {
        if father_id == 0 {
            return this;
        }
        let (grandpa_id, father) = match Comment::select_by_id(father_id).await {
            Some(comment) => (comment.father_comment, Rc::new(CommentNode::from(comment))),
            None => return this,
        };
        CommentNode::add_child(father.clone(), this.clone());
        return CommentNode::find_root(grandpa_id, father).await;
    }
}

impl Drop for CommentNode {
    fn drop(&mut self) {
        let mut children = self.children.replace(vec![]);
        let mut next: Vec<Rc<CommentNode>> = vec![];
        while children.len() != 0 {
            for child in children {
                let mut node = child;
                next.append(&mut node.children.replace(vec![]))
            }
            children = next;
            next = vec![];
        }
    }
}

impl CommentNode {
    // 可以改循环
    fn serialize_value(&self) -> Value {
        let mut map = Map::new();
        map.insert(
            "comment_id".to_string(),
            Value::Number(Number::from(self.comment_id)),
        );
        map.insert("comment".to_string(), Value::String(self.comment.clone()));
        map.insert(
            "item_id".to_string(),
            Value::Number(Number::from(self.item_id)),
        );
        map.insert("user".to_string(), Value::String(self.user.clone()));
        map.insert(
            "user_id".to_string(),
            Value::Number(Number::from(self.user_id)),
        );
        let children: Vec<Value> = self
            .children
            .borrow()
            .iter()
            .map(|child| child.serialize_value())
            .collect();
        map.insert("reply".to_string(), Value::Array(children));
        Value::Object(map)
    }
}

impl Serialize for CommentNode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let v = self.serialize_value();
        v.serialize(serializer)
    }
}

pub async fn get_item_comment(item_id: u64) -> Result<Vec<CommentNode>> {
    let mut result = vec![];
    let mut father_id_comments = Comment::get_by_item_id(item_id).await?;
    let root_comments = father_id_comments.remove(&0u64).unwrap();
    for root_comment in root_comments {
        let i = root_comment.id.unwrap();
        let root = Rc::new(CommentNode::from(root_comment));
        let mut nodes = vec![root.clone()];
        let mut next = vec![];
        while nodes.len() != 0 {
            for node in nodes {
                let children = match father_id_comments.remove(&node.comment_id) {
                    None => continue,
                    Some(children) => children,
                };
                children.into_iter().for_each(|child| {
                    let child_node = Rc::new(CommentNode::from(child));
                    CommentNode::add_child(node.clone(), child_node.clone());
                    next.push(child_node)
                })
            }
            nodes = next;
            next = vec![];
        }
        result.push(
            Rc::try_unwrap(root)
                .map_err(|_| "strong rc only one")
                .unwrap(),
        )
    }
    Ok(result)
}

pub async fn comment_to_item(
    comment: String,
    item_id: u64,
    user_name: String,
    user_id: u64,
    father_id: u64,
) -> Result<CommentNode> {
    let mut comment = Comment::new(comment, user_name, user_id, item_id, Some(father_id));
    comment.create().await?;
    Ok(comment.into())
}

pub async fn change_comment(
    comment_str: String,
    comment_id: u64,
    user_id: u64,
) -> Result<CommentNode> {
    let mut comment = Comment::select_by_id(comment_id)
        .await
        .ok_or(anyhow!("not found"))?;
    if comment.user_id != user_id {
        return Err(anyhow!("not user"));
    }
    comment.change(comment_str).await?;
    Ok(comment.into())
}

pub async fn delete_comment(comment_id: u64, user_id: u64) -> Result<CommentNode> {
    change_comment("".to_string(), comment_id, user_id).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::comment::CommentNode;
    use psutil::process::Process;
    use std::mem::size_of;
    use std::ops::Deref;
    use std::rc::Rc;
    use std::sync::Arc;

    fn build_test_tree(num: i32) -> Rc<CommentNode> {
        let root = Rc::new(CommentNode::new(0, 0.to_string(), 0, 0.to_string(), 0));
        let mut father = root.clone();
        for i in 1..num {
            let index = i as u64;
            let node = Rc::new(CommentNode::new(
                index,
                index.to_string(),
                index,
                index.to_string(),
                index,
            ));
            CommentNode::add_child(father.clone(), node.clone());
            father = node;
        }
        root
    }

    #[test]
    fn test_drop() {
        let process = Process::new(std::process::id()).unwrap();
        let print_mem = || {
            let info = process.memory_info().unwrap();
            dbg!(info.rss());
            //dbg!(info.vms());
        };
        print_mem();

        let root1 = build_test_tree(10000);
        print_mem();

        drop(root1);
        print_mem();

        let root2 = build_test_tree(10000);
        print_mem();

        drop(root2);
        print_mem();

        let root3 = build_test_tree(10000);
        print_mem();
    }

    #[test]
    fn test_debug() {
        let rc = Rc::new(4);
        let item = rc.clone();
        let node = Rc::new(CommentNode::new(0, 0.to_string(), 0, 0.to_string(), 0));
        let rc_node = Arc::new(node);
        dbg!(1);
    }

    #[test]
    fn test_print() {
        let root = build_test_tree(5);
        dbg!(serde_json::to_string(Rc::deref(&root)).unwrap());
    }

    #[tokio::test]
    async fn get_comment() {
        crate::repo::init().await;
        let comments = get_item_comment(1).await.unwrap();
        dbg!(serde_json::to_string(&comments).unwrap());
    }
}
